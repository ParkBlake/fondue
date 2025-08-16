use crate::cache::{Cache, EvictionPolicy, TtlType};
use crate::stats::CacheStats;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// A named cache context that groups related cache operations
pub struct CacheContext {
    name: String,
    caches: Arc<Mutex<HashMap<String, Cache<String, String>>>>,
}

impl CacheContext {
    /// Creates a new cache context with the specified name
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            caches: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Retrieves a cached value or computes and caches it if missing
    ///
    /// Parses the cached string result into type V.
    pub fn get<F, V>(&self, key: impl Into<String>, compute: F) -> V
    where
        F: FnOnce() -> V,
        V: Clone + ToString + std::str::FromStr,
        V::Err: std::fmt::Debug,
    {
        let key = key.into();
        let cache_key = format!("{}::{}", self.name, key);
        let cache = {
            let mut caches = self.caches.lock().unwrap();
            caches.entry(key.clone()).or_default().clone()
        };
        let result = cache.get(&cache_key, || compute().to_string());
        result.parse::<V>().expect("Failed to parse cached value")
    }

    /// Retrieves a cached value with TTL (defaults to Fixed TTL), or computes and caches it
    pub fn get_with_ttl<F, V>(&self, key: impl Into<String>, ttl: Duration, compute: F) -> V
    where
        F: FnOnce() -> V,
        V: Clone + ToString + std::str::FromStr,
        V::Err: std::fmt::Debug,
    {
        self.get_with_ttl_type(key, ttl, TtlType::Fixed, compute)
    }

    /// Retrieves a cached value with TTL and specified TTL type, or computes and caches it
    pub fn get_with_ttl_type<F, V>(
        &self,
        key: impl Into<String>,
        ttl: Duration,
        ttl_type: TtlType,
        compute: F,
    ) -> V
    where
        F: FnOnce() -> V,
        V: Clone + ToString + std::str::FromStr,
        V::Err: std::fmt::Debug,
    {
        let key = key.into();
        let cache_key = format!("{}::{}", self.name, key);
        let cache = {
            let mut caches = self.caches.lock().unwrap();
            caches
                .entry(key.clone())
                .or_insert_with(|| {
                    Cache::with_policy(EvictionPolicy::Ttl {
                        duration: ttl,
                        ttl_type,
                    })
                })
                .clone()
        };
        let result = cache.get(&cache_key, || compute().to_string());
        result.parse::<V>().expect("Failed to parse cached value")
    }

    /// Gets a cached value if it exists without computing
    pub fn get_if_cached<V>(&self, key: impl Into<String>) -> Option<V>
    where
        V: Clone + std::str::FromStr,
        V::Err: std::fmt::Debug,
    {
        let key = key.into();
        let cache_key = format!("{}::{}", self.name, key);
        let caches = self.caches.lock().unwrap();
        let cache = caches.get(&key)?;
        let cached_value = cache.get_if_cached(&cache_key)?;
        Some(
            cached_value
                .parse::<V>()
                .expect("Failed to parse cached value"),
        )
    }

    /// Inserts a value manually into the cache
    pub fn insert<V>(&self, key: impl Into<String>, value: V)
    where
        V: ToString,
    {
        let key = key.into();
        let cache_key = format!("{}::{}", self.name, key);
        let cache = {
            let mut caches = self.caches.lock().unwrap();
            caches.entry(key.clone()).or_default().clone()
        };
        cache.insert(cache_key, value.to_string());
    }

    /// Invalidates a specific cached key in this context, returning if it was removed
    pub fn invalidate(&self, key: impl Into<String>) -> bool {
        let key = key.into();
        let cache_key = format!("{}::{}", self.name, key);
        let caches = self.caches.lock().unwrap();
        if let Some(cache) = caches.get(&key) {
            cache.invalidate(&cache_key)
        } else {
            false
        }
    }

    /// Clears all caches in this context
    pub fn clear(&self) {
        let mut caches = self.caches.lock().unwrap();
        for cache in caches.values() {
            cache.clear();
        }
        caches.clear();
    }

    /// Returns the name of this context
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns aggregated statistics for this context across all its caches
    pub fn stats(&self) -> CacheStats {
        let caches = self.caches.lock().unwrap();
        let mut total_hits = 0;
        let mut total_misses = 0;
        let mut total_entries = 0;
        for cache in caches.values() {
            total_hits += cache.hit_count();
            total_misses += cache.miss_count();
            total_entries += cache.len();
        }
        CacheStats {
            name: self.name.clone(),
            hits: total_hits,
            misses: total_misses,
            entries: total_entries as u64,
            hit_rate: if total_hits + total_misses > 0 {
                total_hits as f64 / (total_hits + total_misses) as f64
            } else {
                0.0
            },
        }
    }

    /// Returns the number of sub-caches in this context
    pub fn cache_count(&self) -> usize {
        let caches = self.caches.lock().unwrap();
        caches.len()
    }

    /// Returns total count of all cached entries across sub-caches
    pub fn total_entries(&self) -> usize {
        let caches = self.caches.lock().unwrap();
        caches.values().map(|cache| cache.len()).sum()
    }
}

impl Clone for CacheContext {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            caches: Arc::clone(&self.caches),
        }
    }
}
