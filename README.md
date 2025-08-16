# Fondue

Fondue is a flexible and efficient caching library in Rust that supports multiple eviction policies, TTL mechanisms, cache namespaces, and usage statistics. It provides macros and APIs for easy caching of expensive computations with fine-grained control over cache behaviour.

---

## Features

- **Cache types:** In-memory caching with optional LRU and TTL eviction policies, including sliding and fixed TTL.
- **Namespace support:** Isolate caches into namespaces to avoid collisions.
- **Macros:** Simple macros for cache access with TTL, capacity limits, and combined policies.
- **Statistics:** Detailed cache usage statistics with global aggregation, JSON export, and formatted output.
- **Thread safe:** Uses `DashMap` and `Arc` for high concurrency and safety.
- **Flexible TTL parsing:** Accepts human-readable TTL strings with fractional support (e.g., "500ms", "1.5h").

---

## Getting Started

Add Fondue to your `Cargo.toml`:

```toml
[dependencies]
fondue = "0.1"
```


### Basic usage with macros

```rust
use fondue::cache;
use fondue_core::{cache_clear_all, TtlType};
use std::thread;
use std::time::Duration;

fn expensive_calculation(x: i32) -> i32 {
thread::sleep(Duration::from_millis(100));
x * x
}

fn main() {
// Cache without TTL
let result = cache!("default", "my_key", || expensive_calculation(42));
println!("Result: {}", result);

text
// Cache with TTL of 500ms
let ttl_result = cache_with_ttl!("default", "ttl_key", "500ms", TtlType::Fixed, || {
    expensive_calculation(10)
});
println!("TTL cached result: {}", ttl_result);

// Clear all caches
cache_clear_all();
}
```

## Statistics

Fondue collects cache hits, misses, entries, and hit rates which you can print or export.

---

## Eviction Policies

- `None`: Unlimited cache size, no eviction.
- `Lru(limit)`: Least Recently Used with specified capacity.
- `Ttl { duration, ttl_type }`: Evict entries after TTL expiration; fixed or sliding.
- `LruTtl { limit, duration, ttl_type }`: Combined LRU and TTL eviction.

---

## TTL Parsing

Fondue supports human-friendly TTL strings with fractional values:

- Examples: `"200ms"`, `"1.5h"`, `"30s"`.
- Supported units: ns, us, ms, s, m, h, d.

Use `parse_duration` for manual parsing if needed.

---

## License

MIT OR Apache-2.0