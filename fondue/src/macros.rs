/// Cache macros
///
/// Cache lookup and insertion macro.
/// Usage: `cache!("namespace", "key", || compute_value())`
///
/// Expands to call `cache_get` with the provided namespace, key, and compute closure.
#[macro_export]
macro_rules! cache {
    ($ns:expr, $key:expr, $compute:expr) => {
        $crate::cache_get($ns, $key, $compute)
    };
}

/// Cache macro with TTL support.
/// Usage: `cache_with_ttl!("namespace", "key", "200ms", TtlType::Fixed, || compute_value())`
///
/// Parses the TTL string using `parse_duration` and calls `cache_get_with_ttl`.
#[macro_export]
macro_rules! cache_with_ttl {
    ($ns:expr, $key:expr, $ttl:expr, $ttl_type:expr, $compute:expr) => {
        $crate::cache_get_with_ttl(
            $ns,
            $key,
            $crate::parse_duration($ttl).expect("Invalid TTL"),
            $ttl_type,
            $compute,
        )
    };
}

/// Cache macro with limit support specifying maximum entries.
/// Usage: `cache_with_limit!("namespace", "key", 10, || compute_value())`
///
/// Calls `cache_get_with_limit` with provided parameters.
#[macro_export]
macro_rules! cache_with_limit {
    ($ns:expr, $key:expr, $limit:expr, $compute:expr) => {
        $crate::cache_get_with_limit($ns, $key, $limit, $compute)
    };
}

/// Cache macro with both TTL and limit support.
/// Usage: `cache_with_ttl_and_limit!("namespace", "key", "500ms", 5, TtlType::Sliding, || compute_value())`
///
/// Parses TTL string and calls `cache_get_with_ttl_and_limit`.
#[macro_export]
macro_rules! cache_with_ttl_and_limit {
    ($ns:expr, $key:expr, $ttl:expr, $limit:expr, $ttl_type:expr, $compute:expr) => {
        $crate::cache_get_with_ttl_and_limit(
            $ns,
            $key,
            $crate::parse_duration($ttl).expect("Invalid TTL"),
            $limit,
            $ttl_type,
            $compute,
        )
    };
}
