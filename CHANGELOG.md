<p align="center">
  <strong>Fondue</strong>
</p>

# Changelog

All notable changes to this project will be documented in this file.

## [0.1.0] - 2025-08-16

### Added
- Initial release of the Fondue cache library.
- Core cache implementation featuring TTL, LRU, and unified LRU+TTL eviction policies.
- User-friendly cache macros supporting TTL, limits, and combined constraints.
- Global and per-namespace statistics tracking with both detailed and table formats.
- Support for fixed and sliding TTL expiration strategies.
- Human-readable TTL string parsing with support for fractional durations (e.g., "1.5h", "500ms").
- Thread-safe cache operations leveraging DashMap and Arc for high performance.
- Comprehensive example demonstrating cache macros, TTL usage, capacity limits, manual invalidation, and stats reporting.
- Extensive test coverage for cache statistics and duration parsing.