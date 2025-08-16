use crate::stats::{register_stats, CacheStats};
use dashmap::DashMap;
use std::{
    hash::Hash,
    sync::atomic::Ordering,
    sync::{Arc, OnceLock},
    time::{Duration, Instant},
};

/// TTL (time-to-live) types for cache entries
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TtlType {
    Fixed,   // TTL counted from creation time
    Sliding, // TTL counted from last accessed time
}

/// Eviction policies supported by the cache
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EvictionPolicy {
    None,       // No eviction
    Lru(usize), // Least Recently Used with capacity limit
    Ttl {
        duration: Duration,
        ttl_type: TtlType,
    }, // TTL-based eviction only
    LruTtl {
        limit: usize,
        duration: Duration,
        ttl_type: TtlType,
    }, // Combined LRU + TTL eviction
}

/// Represents a cached entry with timing and access metadata
#[derive(Debug, Clone)]
pub struct CacheEntry<V> {
    pub value: V,
    pub created_at: Instant,
    pub last_accessed: Instant,
    pub access_count: u64,
    pub ttl: Option<Duration>,
    pub ttl_type: Option<TtlType>,
}

impl<V> CacheEntry<V> {
    /// Creates a new cache entry with current timestamps
    pub fn new(value: V, ttl: Option<Duration>, ttl_type: Option<TtlType>) -> Self {
        let now = Instant::now();
        Self {
            value,
            created_at: now,
            last_accessed: now,
            access_count: 0,
            ttl,
            ttl_type,
        }
    }

    /// Checks if the entry is expired based on TTL and TTL type
    pub fn is_expired(&self) -> bool {
        match (self.ttl, self.ttl_type.as_ref()) {
            (Some(ttl), Some(ttl_type)) => match ttl_type {
                TtlType::Sliding => self.last_accessed.elapsed() >= ttl,
                TtlType::Fixed => self.created_at.elapsed() >= ttl,
            },
            _ => false,
        }
    }

    /// Updates last accessed time and increments access count
    pub fn touch(&mut self) {
        self.access_count += 1;
        self.last_accessed = Instant::now();
    }
}

/// Generic cache supporting configurable eviction policies and TTL
pub struct Cache<K, V>
where
    K: Hash + Eq + Clone,
    V: Clone,
{
    storage: Arc<DashMap<K, CacheEntry<V>>>,
    policy: EvictionPolicy,
    hits: Arc<std::sync::atomic::AtomicU64>,
    misses: Arc<std::sync::atomic::AtomicU64>,
}

impl<K, V> Cache<K, V>
where
    K: Hash + Eq + Clone,
    V: Clone,
{
    /// Creates a new cache without eviction policy
    pub fn new() -> Self {
        Self::with_policy(EvictionPolicy::None)
    }

    /// Creates a new cache with specified eviction policy
    pub fn with_policy(policy: EvictionPolicy) -> Self {
        Self {
            storage: Arc::new(DashMap::new()),
            policy,
            hits: Arc::new(std::sync::atomic::AtomicU64::new(0)),
            misses: Arc::new(std::sync::atomic::AtomicU64::new(0)),
        }
    }

    /// Retrieves cached value or computes and caches it
    ///
    /// Updates statistics after access and maintains eviction as needed.
    pub fn get<F>(&self, key: &K, compute: F) -> V
    where
        F: FnOnce() -> V,
    {
        if let Some(entry) = self.storage.get(key) {
            if !entry.is_expired() {
                drop(entry);
                if let Some(mut entry_mut) = self.storage.get_mut(key) {
                    entry_mut.touch();
                    self.hits.fetch_add(1, Ordering::Relaxed);
                    self.update_cache_stats();
                    return entry_mut.value.clone();
                }
            } else {
                drop(entry);
                self.storage.remove(key);
                self.update_cache_stats();
            }
        }
        self.misses.fetch_add(1, Ordering::Relaxed);
        let (ttl, ttl_type) = match &self.policy {
            EvictionPolicy::Ttl { duration, ttl_type } => (Some(*duration), Some(ttl_type.clone())),
            EvictionPolicy::LruTtl {
                duration, ttl_type, ..
            } => (Some(*duration), Some(ttl_type.clone())),
            _ => (None, None),
        };
        let value = compute();
        let entry = CacheEntry::new(value.clone(), ttl, ttl_type);
        self.storage.insert(key.clone(), entry);
        self.maybe_evict();
        self.update_cache_stats();
        value
    }

    /// Attempts to retrieve cached value without computing
    pub fn get_if_cached(&self, key: &K) -> Option<V> {
        if let Some(mut entry) = self.storage.get_mut(key) {
            if !entry.is_expired() {
                entry.touch();
                self.hits.fetch_add(1, Ordering::Relaxed);
                self.update_cache_stats();
                return Some(entry.value.clone());
            } else {
                drop(entry);
                self.storage.remove(key);
                self.update_cache_stats();
            }
        }
        None
    }

    /// Inserts a value directly into the cache
    pub fn insert(&self, key: K, value: V) {
        let (ttl, ttl_type) = match &self.policy {
            EvictionPolicy::Ttl { duration, ttl_type } => (Some(*duration), Some(ttl_type.clone())),
            EvictionPolicy::LruTtl {
                duration, ttl_type, ..
            } => (Some(*duration), Some(ttl_type.clone())),
            _ => (None, None),
        };
        let entry = CacheEntry::new(value, ttl, ttl_type);
        self.storage.insert(key, entry);
        self.maybe_evict();
        self.update_cache_stats();
    }

    /// Removes an entry by key, returns true if found and removed
    pub fn invalidate(&self, key: &K) -> bool {
        let removed = self.storage.remove(key).is_some();
        if removed {
            self.update_cache_stats();
        }
        removed
    }

    /// Clears all entries in the cache
    pub fn clear(&self) {
        self.storage.clear();
        self.update_cache_stats();
    }

    /// Returns current number of entries in the cache
    pub fn len(&self) -> usize {
        self.storage.len()
    }

    /// Checks whether the cache is empty
    pub fn is_empty(&self) -> bool {
        self.storage.is_empty()
    }

    /// Returns number of cache hits
    pub fn hit_count(&self) -> u64 {
        self.hits.load(Ordering::Relaxed)
    }

    /// Returns number of cache misses
    pub fn miss_count(&self) -> u64 {
        self.misses.load(Ordering::Relaxed)
    }

    /// Calculates current hit rate as fraction in [0.0, 1.0]
    pub fn hit_rate(&self) -> f64 {
        let hits = self.hit_count();
        let total = hits + self.miss_count();
        if total == 0 {
            0.0
        } else {
            hits as f64 / total as f64
        }
    }

    /// Removes expired entries and evicts based on policy limits if needed
    fn maybe_evict(&self) {
        let keys_to_remove: Vec<_> = self
            .storage
            .iter()
            .filter_map(|entry| {
                if entry.value().is_expired() {
                    Some(entry.key().clone())
                } else {
                    None
                }
            })
            .collect();
        for key in keys_to_remove {
            self.storage.remove(&key);
        }
        match &self.policy {
            EvictionPolicy::Lru(limit) | EvictionPolicy::LruTtl { limit, .. } => {
                if self.storage.len() > *limit {
                    self.evict_lru(self.storage.len() - limit);
                }
            }
            _ => {}
        }
    }

    /// Evicts least recently used entries equal to `count`
    fn evict_lru(&self, count: usize) {
        let mut entries: Vec<_> = self
            .storage
            .iter()
            .map(|entry| {
                (
                    entry.key().clone(),
                    entry.value().last_accessed,
                    entry.value().created_at,
                )
            })
            .collect();
        entries.sort_by(|a, b| a.1.cmp(&b.1).then_with(|| a.2.cmp(&b.2)));
        for (key, _, _) in entries.into_iter().take(count) {
            self.storage.remove(&key);
        }
    }

    /// Updates global cache statistics after cache state changes
    fn update_cache_stats(&self) {
        let name = format!("Cache@{:p}", self);
        let stats = CacheStats {
            name: name.clone(),
            hits: self.hit_count(),
            misses: self.miss_count(),
            entries: self.len() as u64,
            hit_rate: self.hit_rate(),
        };
        register_stats(name, stats);
    }
}

impl<K, V> Default for Cache<K, V>
where
    K: Hash + Eq + Clone,
    V: Clone,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<K, V> Clone for Cache<K, V>
where
    K: Hash + Eq + Clone,
    V: Clone,
{
    fn clone(&self) -> Self {
        Self {
            storage: Arc::clone(&self.storage),
            policy: self.policy.clone(),
            hits: Arc::clone(&self.hits),
            misses: Arc::clone(&self.misses),
        }
    }
}

// --- GLOBAL CACHE STORAGE ---

/// Global thread-safe registry of caches by namespace and policy
static GLOBAL_CACHE_STORAGE: OnceLock<Arc<DashMap<String, Cache<String, String>>>> =
    OnceLock::new();

/// Returns global cache storage singleton
fn get_global_cache_storage() -> &'static Arc<DashMap<String, Cache<String, String>>> {
    GLOBAL_CACHE_STORAGE.get_or_init(|| Arc::new(DashMap::new()))
}

/// Creates or retrieves a cache instance by namespace and eviction policy
fn get_or_create_cache(namespace: &str, policy: EvictionPolicy) -> Cache<String, String> {
    let caches = get_global_cache_storage();
    // Compose key by combining namespace and policy description
    let policy_key = match &policy {
        EvictionPolicy::None => "none".to_string(),
        EvictionPolicy::Lru(limit) => format!("lru({})", limit),
        EvictionPolicy::Ttl { duration, ttl_type } => format!("ttl({:?},{:?})", duration, ttl_type),
        EvictionPolicy::LruTtl {
            limit,
            duration,
            ttl_type,
        } => format!("lru_ttl({}, {:?},{:?})", limit, duration, ttl_type),
    };
    let cache_key = format!("{}::{}", namespace, policy_key);
    caches
        .entry(cache_key)
        .or_insert_with(|| Cache::with_policy(policy))
        .clone()
}

// --- Cache API functions ---

pub fn cache_get<F, V>(namespace: &str, key: &str, compute: F) -> V
where
    F: FnOnce() -> V,
    V: Clone + ToString + std::str::FromStr,
    V::Err: std::fmt::Debug,
{
    let cache = get_or_create_cache(namespace, EvictionPolicy::None);
    let cached_value = cache.get(&key.to_string(), || compute().to_string());
    cached_value
        .parse::<V>()
        .expect("Failed to parse cached value")
}

pub fn cache_get_with_ttl<F, V>(
    namespace: &str,
    key: &str,
    ttl: Duration,
    ttl_type: TtlType,
    compute: F,
) -> V
where
    F: FnOnce() -> V,
    V: Clone + ToString + std::str::FromStr,
    V::Err: std::fmt::Debug,
{
    let cache = get_or_create_cache(
        namespace,
        EvictionPolicy::Ttl {
            duration: ttl,
            ttl_type,
        },
    );
    let cached_value = cache.get(&key.to_string(), || compute().to_string());
    cached_value
        .parse::<V>()
        .expect("Failed to parse cached value")
}

pub fn cache_get_with_limit<F, V>(namespace: &str, key: &str, limit: usize, compute: F) -> V
where
    F: FnOnce() -> V,
    V: Clone + ToString + std::str::FromStr,
    V::Err: std::fmt::Debug,
{
    let cache = get_or_create_cache(namespace, EvictionPolicy::Lru(limit));
    let cached_value = cache.get(&key.to_string(), || compute().to_string());
    cached_value
        .parse::<V>()
        .expect("Failed to parse cached value")
}

pub fn cache_get_with_ttl_and_limit<F, V>(
    namespace: &str,
    key: &str,
    ttl: Duration,
    limit: usize,
    ttl_type: TtlType,
    compute: F,
) -> V
where
    F: FnOnce() -> V,
    V: Clone + ToString + std::str::FromStr,
    V::Err: std::fmt::Debug,
{
    let cache = get_or_create_cache(
        namespace,
        EvictionPolicy::LruTtl {
            limit,
            duration: ttl,
            ttl_type,
        },
    );
    let cached_value = cache.get(&key.to_string(), || compute().to_string());
    cached_value
        .parse::<V>()
        .expect("Failed to parse cached value")
}

/// Invalidate entry by key in all caches with the given namespace
pub fn cache_invalidate(namespace: &str, key: &str) -> bool {
    let caches = get_global_cache_storage();
    let mut invalidated = false;
    for cache in caches.iter() {
        if cache.key().starts_with(namespace) && cache.value().invalidate(&key.to_string()) {
            invalidated = true;
        }
    }
    invalidated
}

/// Clear all caches globally
pub fn cache_clear_all() {
    let caches = get_global_cache_storage();
    for cache in caches.iter() {
        cache.value().clear();
    }
}

/// Clear all caches within a given namespace
pub fn cache_clear_namespace(namespace: &str) {
    let caches = get_global_cache_storage();
    for cache in caches.iter() {
        if cache.key().starts_with(namespace) {
            cache.value().clear();
        }
    }
}
