# Phase 2: close-confidence-gaps-runtime-behavior - Context

**Gathered:** 2026-04-08
**Status:** Ready for planning

<domain>
## Phase Boundary

Reduce regression risk in the plugin's highest-value logic by adding missing test coverage, aligning configuration with actual runtime behavior, and tightening contributor documentation to match verified implementation. This phase specifically covers: adding tests for countdown/formatting helpers and cache fallback behavior; reviewing and fixing config/runtime drift especially around API timeout/retry options; and tightening documentation where it overstates or under-specifies current behavior.

</domain>

<decisions>
## Implementation Decisions

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

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Project planning artifacts
- `.planning/PROJECT.md` — Project context, fork constraints, and low-drift objectives.
- `.planning/REQUIREMENTS.md` — Global functional, quality, and compatibility requirements.
- `.planning/ROADMAP.md` — Phase 2 goal, tasks, and exit criteria.
- `.planning/STATE.md` — Current project state and risk framing.
- `.planning/codebase/TESTING.md` — Existing test infrastructure status and recommended test additions.

### Core implementation files
- `src/core/segments/glm_usage.rs` — Countdown/formatting helpers and cache implementation to test.
- `src/api/client.rs` — API client where timeout/retry hardcoding needs to be updated.
- `src/config/types.rs` — `ApiConfig` struct with timeout_ms and retry_attempts already defined.
- `CLAUDE.md` — Contributor-oriented architecture documentation that needs targeted updates.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `src/core/segments/glm_usage.rs`: Already contains `format_tokens` and `format_countdown` helpers that need tests.
- `src/api/client.rs`: Already has `normalize_base_url` and `build_quota_url` exported for testing (pattern to follow).
- `cargo test` is already running in CI — no new infrastructure needed.

### Established Patterns
- Inline tests in the same source file as the code under test.
- Export helper functions for testing while keeping them internal to the crate.
- Graceful degradation — the plugin must never cause Claude Code to fail.
- CI enforces `cargo fmt` and `cargo clippy` — all new code must pass.

### Integration Points
- `GlmApiClient::from_env()` needs to accept the config values instead of using hardcoded timeout.
- `GlmApiClient::fetch_usage_stats()` needs to use the configured retry count instead of fixed 0..=2.
- Cache fall-back logic in `GlmUsageSegment::get_usage_stats()` needs comprehensive test coverage.

</code_context>

<specifics>
## Specific Ideas

- Keep the changes focused and minimal — only address the confidence gaps called out in the roadmap.
- Don't move existing inline tests — just add new ones following the established pattern.
- Preserve the current defaults (5000ms timeout, 2 retries) so existing user configurations don't change behavior.

</specifics>

<deferred>
## Deferred Ideas

- Moving all tests to a separate top-level `tests/` directory — can be considered in a future maintenance phase.
- Full review and update of all documentation beyond the targeted config/runtime section — can be done later if drift appears elsewhere.

None of the pending todos were folded into this phase.

</deferred>

---

*Phase: 02-close-confidence-gaps-runtime-behavior*
*Context gathered: 2026-04-08*
