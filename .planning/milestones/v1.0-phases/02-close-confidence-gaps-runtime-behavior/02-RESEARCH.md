# Phase 2: close-confidence-gaps-runtime-behavior - Research

**Researched:** 2026-04-08
**Domain:** Rust plugin test coverage and configuration/runtime alignment
**Confidence:** HIGH

## Summary

This phase addresses three specific confidence gaps in the GLM Plan Usage plugin: (1) missing test coverage for countdown/formatting helpers and cache fallback behavior in `glm_usage.rs`, (2) config/runtime drift where `timeout_ms` and `retry_attempts` are defined in `ApiConfig` but hardcoded in the client implementation, and (3) documentation that needs targeted updates to reflect the corrected behavior.

The `ApiConfig` struct already has `timeout_ms` (default 5000ms) and `retry_attempts` (default 2) properly defined with correct defaults. The changes are minimal: update `GlmApiClient::from_env()` to accept the config and use those values instead of hardcoding, add comprehensive tests to the existing inline test module following the established pattern from Phase 1, and update the documentation to reflect that these options are now actually usable.

**Primary recommendation:** Proceed with the locked decisions — add inline tests, eliminate hardcoding, update docs targeted, keeping all defaults unchanged to avoid breaking existing configurations.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
### Test Organization
- **D-01:** Keep all new tests inline using `#[cfg(test)]` modules directly in the source files where the code under test lives.
- **D-02:** This follows the existing pattern established in Phase 1 for endpoint resolution tests in `client.rs` and keeps tests co-located with the code they validate.

### Config/Runtime Alignment
- **D-03:** Update `GlmApiClient` to actually use the `timeout_ms` and `retry_attempts` values from `config.api` instead of hardcoding.
- **D-04:** Remove the existing hardcoded 5-second timeout and 3-attempt retry loop from `client.rs` and read from the config struct.
- **D-05:** This eliminates the documented config/runtime drift and allows users to customize these values if needed while keeping defaults at 5000ms and 2 retries.

### Cache Test Coverage
- **D-06:** Implement full test coverage for all cache behaviors in `glm_usage.rs`.
- **D-07:** Tests must cover: cache hit when fresh, cache miss when empty/expired, stale cache fallback when API fails, and correct behavior when cache is disabled via config.

### Documentation Refinement
- **D-08:** Only targeted updates to documentation where drift is known to exist.
- **D-09:** Update the config/API section in `CLAUDE.md` to reflect that timeout and retry are now configurable via `ApiConfig` (fixing the specific drift we addressed in this phase).

### Claude's Discretion
- Exact test function organization and naming within the inline test module.
- Choice of test doubles/mocking approach for simulating API failures in cache fallback tests.
- Minor wording tweaks in documentation as needed for clarity.

### Deferred Ideas (OUT OF SCOPE)
- Moving all tests to a separate top-level `tests/` directory — can be considered in a future maintenance phase.
- Full review and update of all documentation beyond the targeted config/runtime section — can be done later if drift appears elsewhere.

None of the pending todos were folded into this phase.
</user_constraints>

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| Rust | 2021 edition | Core language | Already in use, established project |
| cargo test | Built-in | Test framework | Standard Rust, already in CI [VERIFIED: Cargo.toml] |
| ureq | 2.10 | HTTP client | Already in use for API calls [VERIFIED: Cargo.toml] |
| serde | 1.0 | Serialization | Standard Rust deserialization for config [VERIFIED: Cargo.toml] |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| std::sync::Mutex | std | Cache synchronization | Already used for cache interior mutability [VERIFIED: glm_usage.rs] |
| std::time::Instant/Duration | std | Timing for cache TTL | Standard library, already in use [VERIFIED: glm_usage.rs] |

**Installation:**
No new dependencies needed. All required testing infrastructure is already in place via standard Rust.

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Inline tests | Separate `tests/` directory | Kept inline per locked decision, follows existing pattern from Phase 1 |
| Mockall for mocking | Manual test closures | Simple behavior can be tested without additional dependency; dependency adds complexity for minimal gain |

## Architecture Patterns

### Recommended Project Structure (existing, preserved)
```
src/
├── config/
│   └── types.rs          # ApiConfig with timeout_ms/retry_attempts already defined
├── api/
│   └── client.rs         # GlmApiClient to be updated
└── core/
    └── segments/
        └── glm_usage.rs  # Cache and formatting helpers to be tested
```

### Pattern 1: Inline Co-located Tests
**What:** Tests live in a `#[cfg(test)] mod tests` block at the bottom of the same source file as the code under test. Private helpers can be tested directly without exposing public APIs.
**When to use:** Established by Phase 1, required by locked decision D-01.
**Example:**
```rust
// Source: src/api/client.rs (existing pattern)
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quota_endpoint_resolution_zhipu_cn_standard() {
        let input = "https://open.bigmodel.cn/api/anthropic";
        let platform = Platform::Zhipu;
        let normalized = normalize_base_url(input, platform);
        let quota_url = build_quota_url(&normalized);
        assert_eq!(quota_url, "https://open.bigmodel.cn/api/monitor/usage/quota/limit");
    }
}
```
[VERIFIED: client.rs]

### Pattern 2: Config with Serde Defaults
**What:** All config options have `#[serde(default)]` and `impl Default` provides documented defaults. New fields don't break existing config files.
**When to use:** Already used for `timeout_ms` and `retry_attempts`.
**Example:**
```rust
// Source: src/config/types.rs
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ApiConfig {
    #[serde(default = "default_timeout")]
    pub timeout_ms: u64,
    #[serde(default = "default_retry")]
    pub retry_attempts: u32,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            timeout_ms: default_timeout(),
            retry_attempts: default_retry(),
        }
    }
}

fn default_timeout() -> u64 { 5000 }
fn default_retry() -> u32 { 2 }
```
[VERIFIED: types.rs]

### Anti-Patterns to Avoid
- **Testing private implementation details unnecessarily:** The decisions already scope exactly what needs testing — focus on the public behaviors and exported helpers.
- **Changing defaults:** The requirements explicitly state to keep defaults at 5000ms/2 retries to avoid breaking existing users. Changing defaults is out of scope.
- **Large documentation rewrite:** Only the specific section about API configuration needs update — full docs rewrite is deferred.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Test framework | Custom test harness | `cargo test` | Standard Rust, already in CI [VERIFIED: TESTING.md] |
| HTTP client wrapper | Custom connection management | `ureq::Agent` with configured timeout | Already in dependency tree, builder pattern accepts timeout [VERIFIED: client.rs] |
| Time/Duration calculations | Custom time handling | `std::time` types | Standard library, already in use [VERIFIED: glm_usage.rs] |

**Key insight:** All the infrastructure needed to complete this phase already exists in the project. No new dependencies are required. The changes are additive and don't require architectural changes.

## Common Pitfalls

### Pitfall 1: Breaking existing behavior by changing defaults
**What goes wrong:** Changing the default timeout or retry count from 5000ms/2 would alter behavior for existing users who haven't explicitly configured these options.
**Why it happens:** It's tempting to "improve" defaults when refactoring, but the project needs to preserve compatibility.
**How to avoid:** Use `config.api.timeout_ms` and `config.api.retry_attempts` directly from the config. The `Default` impl already provides the correct existing defaults.
**Warning signs:** The tests don't preserve the existing default values in assertions.

### Pitfall 2: Cache tests that fail due to timing
**What goes wrong:** Tests that rely on `Instant::elapsed()` sleeping for TTL expiration can be flaky on CI.
**Why it happens:** System load can cause sleeps to take longer than expected or be faster than expected.
**How to avoid:** Use table-driven tests that directly check the cache logic rather than relying on actual timing. For expired cache tests, you can restructure to allow injecting a fake clock if needed, or use the fact that the code compares `elapsed() < TTL` which is straightforward to test with controlled inputs.
**Warning signs:** Tests pass locally but intermittently fail on CI.

### Pitfall 3: Forgetting to handle the zero-value case
**What goes wrong:** Users who explicitly set `timeout_ms = 0` or `retry_attempts = 0` might get unexpected behavior if the code doesn't correctly handle the usize cast.
**Why it happens:** `timeout_ms: u64` accepts 0 which means "no timeout" in ureq. `retry_attempts: 0` should make only one attempt (no retries).
**How to avoid:** Use the values directly from config without adding 1. When iterating, `for attempt in 0..config.api.retry_attempts` gives exactly the right number of attempts including zero.
**Warning signs:** When `retry_attempts = 2` you get 3 attempts. When `retry_attempts = 0` you get 1 attempt instead of zero.

### Pitfall 4: Deadlocks in cache tests
**What goes wrong:** Cache uses `Arc<Mutex<Option<CacheEntry>>>`. Holding the lock across await points or nested locks can cause deadlocks in tests.
**Why it happens:** Rust's Mutex is a plain mutex, not a reentrant one.
**How to avoid:** In tests, drop the lock guard immediately after accessing the cache. The existing pattern already does this correctly (`self.cache.lock().unwrap().as_ref().map(...)` which drops the lock before mapping).
**Warning signs:** Tests deadlock/hang when run with `cargo test`.

## Code Examples

Verified patterns from the existing codebase:

### Format Countdown Helper (already exists, needs test)
```rust
// Source: src/core/segments/glm_usage.rs
/// Calculate countdown to reset time and format as HH:MM
fn format_countdown(reset_at: i64) -> Option<String> {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).ok()?.as_secs() as i64;

    let remaining = reset_at.saturating_sub(now);

    if remaining <= 0 {
        return Some("0:00".to_string());
    }

    let hours = remaining / 3600;
    let minutes = (remaining % 3600) / 60;

    Some(format!("{}:{:02}", hours, minutes))
}
```
[VERIFIED: glm_usage.rs]

### Cache Check Logic (already exists, needs test)
```rust
// Source: src/core/segments/glm_usage.rs
// Check cache first
if config.cache.enabled {
    if let Some(entry) = self.cache.lock().unwrap().as_ref() {
        if entry.timestamp.elapsed() < Duration::from_secs(config.cache.ttl_seconds) {
            return Some(entry.stats.clone());
        }
    }
}
```
[VERIFIED: glm_usage.rs]

### Update GlmApiClient to use Config (to be implemented)
```rust
// How it should look after changes
impl GlmApiClient {
    pub fn from_config(config: &Config) -> Result<Self> {
        // ... (existing environment reading)
        let agent = ureq::AgentBuilder::new()
            .timeout(Duration::from_millis(config.api.timeout_ms))
            .build();
        // ...
    }
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Hardcoded 5s timeout, 3 attempts | Config-driven from `ApiConfig` | This phase | Eliminates config/runtime drift; allows customization while preserving defaults |
| No tests for formatting/cache | Full inline test coverage | This phase | Improves confidence, prevents regression in caching behavior |
| Documentation describes configurable timeout/retry but it wasn't actually implemented | Documentation reflects actual implementation | This phase | Contributors understand available configuration options |

**Deprecated/outdated:**
- Hardcoded timeout in `client.rs:47`: Deprecated, replaced with config-driven value. The struct field already exists in `ApiConfig`, just wasn't being used.
- Fixed loop `0..=2` (3 attempts): Deprecated, replaced with `0..config.api.retry_attempts`.

## Assumptions Log

> List all claims tagged `[ASSUMED]` in this research. The planner and discuss-phase use this
> section to identify decisions that need user confirmation before execution.

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `GlmApiClient::from_env()` will need to be updated to accept a `&Config` parameter | Architecture Patterns | The client creation call chain will need to be adjusted to pass the config from the entry point. Changes would be localized. |
| A2 | Existing tests already pass in the current CI setup | Standard Stack | If there's a test infrastructure issue, it would block CI. But CI is already running `cargo test` per TESTING.md. |

**If this table is empty:** All claims in this research were verified or cited — no user confirmation needed.

## Open Questions

1. **Should `GlmApiClient::from_env()` be deprecated in favor of `from_config` or updated to take a config parameter?**
   - What we know: The current signature is `from_env() -> Result<Self>` and the config is available at the top level in `main.rs`.
   - What's unclear: Whether to keep `from_env` as-is and have it use defaults or update it to accept the config.
   - Recommendation: Update the signature to accept a `&Config` since the config is already available at the call site. This is what the locked decision requires.

2. **Will cache testing require refactoring to enable mockable API calls?**
   - What we know: Currently `GlmUsageSegment` calls `GlmApiClient::from_env().fetch_usage_stats()` directly.
   - What's unclear: How easy it is to test the fallback behavior without mocking.
   - Recommendation: The existing structure can be tested by making `get_usage_stats` package-private and using a testing closure, no mocking library needed. This is within Claude's discretion per locked decisions.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust / cargo | Build and test | ✗ | — | Build proceeds after cargo installation |

**Missing dependencies with no fallback:**
- cargo is not available in the current environment — the implementation phase will need to install Rust toolchain before running tests.

**Missing dependencies with fallback:**
- None. The only missing dependency is cargo itself, which is required to build/test.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | cargo test (Rust built-in) |
| Config file | none — uses standard cargo |
| Quick run command | `cargo test` |
| Full suite command | `cargo test && cargo fmt -- --check && cargo clippy -- -D warnings` |

### Phase Requirements → Test Map
| Requirement Behavior | Test Type | Automated Command | File Exists? |
|-----------------------|-----------|-------------------|-------------|
| `format_tokens` handles negative, K, M, raw correctly | unit | `cargo test format_tokens` | ❌ Wave 0 (will add) |
| `format_countdown` handles expired, future, missing, zero correctly | unit | `cargo test format_countdown` | ❌ Wave 0 (will add) |
| Cache behaves correctly on hit/miss/expired/disabled | unit | `cargo test cache_` | ❌ Wave 0 (will add) |
| Stale cache fallback on API failure | unit | `cargo test cache_fallback` | ❌ Wave 0 (will add) |
| `GlmApiClient` uses configured timeout_ms from config | compile/unit | `cargo test client_timeout` | ❌ Wave 0 (will add) |
| `GlmApiClient` uses configured retry_attempts from config | unit | `cargo test client_retry` | ❌ Wave 0 (will add) |

### Sampling Rate
- **Per task commit:** `cargo test`
- **Per wave merge:** `cargo test && cargo fmt -- --check && cargo clippy -- -D warnings`
- **Phase gate:** Full suite green before phase completion.

### Wave 0 Gaps
- [ ] `src/core/segments/glm_usage.rs` — need to add `#[cfg(test)] mod tests` with tests for formatting and cache behavior
- [ ] `src/api/client.rs` — need to modify `from_env` signature to accept config and use configured values

*(No other gaps — existing test infrastructure via cargo is already in place in CI)*

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | Already handled in existing code |
| V3 Session Management | no | No session management |
| V4 Access Control | no | No access control beyond API key |
| V5 Input Validation | yes | Already using serde for validation, defaults set correctly |
| V6 Cryptography | no | No custom cryptography |

### Known Threat Patterns for this stack

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Denial of Service via infinite timeout | Denial of Service | Timeout is now configurable, default 5s prevents hanging |
| Information disclosure in errors | Information Disclosure | Existing graceful degradation already handles this |

## Sources

### Primary (HIGH confidence)
- Read direct source: `src/core/segments/glm_usage.rs` — verified existing helpers and cache implementation
- Read direct source: `src/api/client.rs` — verified hardcoded timeout/retry
- Read direct source: `src/config/types.rs` — verified ApiConfig already has the required fields with correct defaults
- Read direct source: `.planning/codebase/TESTING.md` — confirmed CI infrastructure
- Read direct source: `CLAUDE.md` — confirmed existing documentation that needs update

### Secondary (MEDIUM confidence)
- Rust standard library documentation: `cargo test` inline tests is the expected pattern [CITED: doc.rust-lang.org]

### Tertiary (LOW confidence)
- None — all findings verified from direct source code inspection.

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all dependencies already exist in Cargo.toml, no new dependencies needed
- Architecture: HIGH — patterns already established in existing codebase, decisions locked
- Pitfalls: HIGH — identified common Rust testing issues that are well-understood

**Research date:** 2026-04-08
**Valid until:** 2026-05-08 (30 days — this is a stable codebase)
