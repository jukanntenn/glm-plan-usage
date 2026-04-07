# TESTING

## Current automated checks
- `cargo test --verbose` in CI
- `cargo fmt -- --check` in CI
- `cargo clippy -- -D warnings` in CI
- `cargo build --release` on Linux, Windows, and macOS in CI

## Test infrastructure status
- CI is present and active
- Build validation is stronger than behavioral validation
- No `tests/` directory exists
- No visible unit tests were found in the current Rust source files read during mapping

## What is covered well
- Basic compile/build correctness across multiple platforms
- Formatting and lint discipline
- Release build matrix exercises packaging targets

## What appears under-covered
- Platform detection behavior
- URL transformation rules for ZHIPU vs ZAI
- Cache TTL and stale-fallback behavior
- Status text formatting and countdown formatting
- npm launcher resolution logic across npm/pnpm/libc combinations
- release packaging script behavior and version synchronization

## Recommended verification commands
- `cargo test`
- `cargo fmt -- --check`
- `cargo clippy -- -D warnings`
- `cargo build --release`

## Suggested next test additions
1. Rust unit tests for `Platform::detect_from_url`
2. Rust unit tests for countdown/token formatting helpers
3. Rust tests for cache hit/miss behavior around `GlmUsageSegment`
4. Rust tests for config load/default/init behavior using temp directories
5. Node tests or smoke checks for npm binary resolution and postinstall path logic
