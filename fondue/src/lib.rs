pub mod cache;
pub mod context;
pub mod duration;
pub mod stats;

#[macro_use]
mod macros;

// Re-export cache types, functions, macros at the crate root for easy access and macro resolution
pub use cache::{
    cache_clear_all, cache_get, cache_get_with_limit, cache_get_with_ttl,
    cache_get_with_ttl_and_limit, cache_invalidate, Cache, CacheEntry, EvictionPolicy, TtlType,
};

// Re-export context and duration utilities explicitly
pub use context::CacheContext;

// Only expose parse_duration function from duration module
pub use duration::parse_duration;

// Re-export statistics utilities explicitly
pub use stats::{
    aggregate_stats, clear_stats, export_json, get_stats, print_stats, print_stats_table,
    register_stats, update_stats, CacheStats, GlobalStats,
};
