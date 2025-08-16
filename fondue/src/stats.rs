use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Statistics for a single cache or context
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub name: String,
    pub hits: u64,
    pub misses: u64,
    pub entries: u64,
    pub hit_rate: f64,
}

impl CacheStats {
    /// Creates a new CacheStats instance with zeroed values
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            hits: 0,
            misses: 0,
            entries: 0,
            hit_rate: 0.0,
        }
    }

    /// Returns the total number of requests (hits plus misses)
    pub fn total_requests(&self) -> u64 {
        self.hits + self.misses
    }

    /// Prints human-readable cache statistics
    pub fn print(&self) {
        println!("Cache Stats: {}", self.name);
        println!("  Entries:     {}", self.entries);
        println!("  Hits:        {}", self.hits);
        println!("  Misses:      {}", self.misses);
        println!("  Hit Rate:    {:.2}%", self.hit_rate * 100.0);
        println!("  Total Reqs:  {}", self.total_requests());
    }

    /// Serializes the cache statistics to a JSON string
    pub fn to_json(&self) -> String {
        format!(
            r#"{{
  "name": "{}",
  "hits": {},
  "misses": {},
  "entries": {},
  "hit_rate": {:.4},
  "total_requests": {}
}}"#,
            self.name,
            self.hits,
            self.misses,
            self.entries,
            self.hit_rate,
            self.total_requests()
        )
    }
}

/// Global statistics manager to track multiple caches
pub struct GlobalStats {
    stats: Arc<Mutex<HashMap<String, CacheStats>>>,
}

impl GlobalStats {
    /// Creates a new empty GlobalStats
    pub fn new() -> Self {
        Self {
            stats: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Registers new stats under a given name
    pub fn register(&self, name: impl Into<String>, stats: CacheStats) {
        let mut global_stats = self.stats.lock().unwrap();
        global_stats.insert(name.into(), stats);
    }

    /// Updates existing stats under the given name
    pub fn update(&self, name: &str, stats: CacheStats) {
        let mut global_stats = self.stats.lock().unwrap();
        global_stats.insert(name.to_string(), stats);
    }

    /// Retrieves stats by name, if present
    pub fn get(&self, name: &str) -> Option<CacheStats> {
        let stats = self.stats.lock().unwrap();
        stats.get(name).cloned()
    }

    /// Returns all stored statistics as a HashMap
    pub fn all(&self) -> HashMap<String, CacheStats> {
        let stats = self.stats.lock().unwrap();
        stats.clone()
    }

    /// Prints detailed stats for all caches, or a message if none available
    pub fn print_all(&self) {
        let stats = self.stats.lock().unwrap();
        if stats.is_empty() {
            println!("No cache statistics available");
            return;
        }
        println!("= Fondue Cache Statistics =");
        for stat in stats.values() {
            stat.print();
            println!();
        }
    }

    /// Prints a formatted table summary of all cache stats, or a message if none available
    pub fn print_table(&self) {
        let stats = self.stats.lock().unwrap();
        if stats.is_empty() {
            println!("No cache statistics available");
            return;
        }
        println!("┌─────────────────────────┬─────────┬──────┬────────┬──────────┬───────────┐");
        println!("│ Cache Name              │ Entries │ Hits │ Misses │ Hit Rate │ Total Req │");
        println!("├─────────────────────────┼─────────┼──────┼────────┼──────────┼───────────┤");
        for stat in stats.values() {
            println!(
                "│ {:<23} │ {:>7} │ {:>4} │ {:>6} │ {:>7.2}% │ {:>9} │",
                truncate_string(&stat.name, 23),
                stat.entries,
                stat.hits,
                stat.misses,
                stat.hit_rate * 100.0,
                stat.total_requests()
            );
        }
        println!("└─────────────────────────┴─────────┴──────┴────────┴──────────┴───────────┘");
    }

    /// Serializes all stats to a JSON array string
    pub fn to_json(&self) -> String {
        let stats = self.stats.lock().unwrap();
        let mut json_parts = Vec::new();
        for stat in stats.values() {
            json_parts.push(stat.to_json());
        }
        format!("[\n{}\n]", json_parts.join(",\n"))
    }

    /// Aggregates stats from all caches into a combined CacheStats
    pub fn aggregate(&self) -> CacheStats {
        let stats = self.stats.lock().unwrap();
        let mut total_hits = 0;
        let mut total_misses = 0;
        let mut total_entries = 0;
        for stat in stats.values() {
            total_hits += stat.hits;
            total_misses += stat.misses;
            total_entries += stat.entries;
        }
        let total_requests = total_hits + total_misses;
        let hit_rate = if total_requests > 0 {
            total_hits as f64 / total_requests as f64
        } else {
            0.0
        };
        CacheStats {
            name: "AGGREGATE".to_string(),
            hits: total_hits,
            misses: total_misses,
            entries: total_entries,
            hit_rate,
        }
    }

    /// Clears all stored cache statistics
    pub fn clear(&self) {
        let mut stats = self.stats.lock().unwrap();
        stats.clear();
    }

    /// Removes stats for a specific cache by name
    pub fn remove(&self, name: &str) -> Option<CacheStats> {
        let mut stats = self.stats.lock().unwrap();
        stats.remove(name)
    }
}

impl Default for GlobalStats {
    fn default() -> Self {
        Self::new()
    }
}

use std::sync::OnceLock;

/// Global singleton for all cache stats
static GLOBAL_STATS: OnceLock<GlobalStats> = OnceLock::new();

/// Access the global stats instance, initializing if needed
pub fn get_global_stats() -> &'static GlobalStats {
    GLOBAL_STATS.get_or_init(GlobalStats::new)
}

/// Prints all stats in detailed format
pub fn print_stats() {
    get_global_stats().print_all();
}

/// Prints all stats in table format
pub fn print_stats_table() {
    get_global_stats().print_table();
}

/// Gets stats for a specific cache by name, if available
pub fn get_stats(name: &str) -> Option<CacheStats> {
    get_global_stats().get(name)
}

/// Exports all stats in JSON format
pub fn export_json() -> String {
    get_global_stats().to_json()
}

/// Aggregates stats from all caches into one summary
pub fn aggregate_stats() -> CacheStats {
    get_global_stats().aggregate()
}

/// Clears all cached statistics globally
pub fn clear_stats() {
    get_global_stats().clear();
}

/// Registers new stats globally with given name
pub fn register_stats(name: impl Into<String>, stats: CacheStats) {
    get_global_stats().register(name, stats);
}

/// Updates existing stats globally for given name
pub fn update_stats(name: &str, stats: CacheStats) {
    get_global_stats().update(name, stats);
}

/// Utility to truncate strings with "…" suffix if over max length
fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_stats() {
        let mut stats = CacheStats::new("test_cache");
        stats.hits = 80;
        stats.misses = 20;
        stats.entries = 50;
        stats.hit_rate = 0.8;
        assert_eq!(stats.total_requests(), 100);
        assert_eq!(stats.hit_rate, 0.8);
        let json = stats.to_json();
        assert!(json.contains("\"name\": \"test_cache\""));
        assert!(json.contains("\"hits\": 80"));
    }

    #[test]
    fn test_global_stats() {
        let global = GlobalStats::new();
        let stats1 = CacheStats {
            name: "cache1".to_string(),
            hits: 50,
            misses: 10,
            entries: 30,
            hit_rate: 0.833,
        };
        let stats2 = CacheStats {
            name: "cache2".to_string(),
            hits: 30,
            misses: 20,
            entries: 25,
            hit_rate: 0.6,
        };
        global.register("cache1", stats1);
        global.register("cache2", stats2);
        let aggregate = global.aggregate();
        assert_eq!(aggregate.hits, 80);
        assert_eq!(aggregate.misses, 30);
        assert_eq!(aggregate.entries, 55);
        let retrieved = global.get("cache1").unwrap();
        assert_eq!(retrieved.name, "cache1");
        assert_eq!(retrieved.hits, 50);
    }

    #[test]
    fn test_truncate_string() {
        assert_eq!(truncate_string("short", 10), "short");
        assert_eq!(
            truncate_string("this_is_a_very_long_string", 10),
            "this_is..."
        );
    }
}
