# Phase 1: verify-and-implement-z-ai-support - Research

**Researched:** 2026-04-07
**Domain:** Rust API client platform routing and verification for ZHIPU CN + z.ai
**Confidence:** MEDIUM

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
### Platform routing
- **D-01:** Keep the existing `Platform` enum plus single `GlmApiClient` architecture. Do not introduce a separate client layer or broad config redesign for Phase 1.
- **D-02:** Platform-specific behavior may be made more explicit inside endpoint/base URL resolution, but changes should stay minimal and localized to current API client/platform logic.

### Compatibility scope
- **D-03:** Phase 1 should guarantee correct `z.ai` routing and verified compatibility with the current quota-fetching flow before considering any broader refactor.
- **D-04:** A light normalization layer is acceptable only if needed to map small `z.ai` response differences into the existing `UsageStats` model without changing user-visible status line output.
- **D-05:** Existing status line format and CLI behavior remain unchanged unless a change is strictly required for verified `z.ai` support.

### Test strategy
- **D-06:** Phase 1 tests must prioritize platform detection, endpoint/base URL resolution, and explicit regression coverage for existing ZHIPU CN behavior.
- **D-07:** If response-shape handling or cache fallback logic is touched while implementing `z.ai` support, add targeted tests for those paths in the same phase rather than leaving them implicit.

### Documentation posture
- **D-08:** Documentation should only claim support paths that are verified by code and tests in this phase.
- **D-09:** README and troubleshooting guidance should clearly distinguish CN and overseas configuration examples, rather than relying on broad support claims.

### Claude's Discretion
- Exact internal helper shape for endpoint resolution.
- Exact test placement and fixture style, as long as it follows normal Rust project patterns.
- Exact doc wording, as long as it stays conservative and verification-backed.

### Deferred Ideas (OUT OF SCOPE)
- Large API/client refactors beyond what is needed for verified `z.ai` support.
- New status line segments or new output formats unrelated to GLM usage verification.
- Packaging/release redesign unless Phase 1 uncovers a direct blocker.
</user_constraints>

## Summary

Phase 1 should stay narrowly focused on making platform routing explicit and testable inside the existing `Platform` enum plus `GlmApiClient` flow, because that is already where platform detection and base URL transformation live in this fork. The current code detects `api.z.ai` as `Platform::Zai`, rewrites only the ZHIPU CN URL, and always appends `/monitor/usage/quota/limit` when fetching quota data. That keeps the likely fix surface very small: platform detection, normalized base URL resolution, and response parsing behavior that feeds the existing `UsageStats` model. [VERIFIED: /d/00_Coding/glm-plan-usage/src/api/types.rs] [VERIFIED: /d/00_Coding/glm-plan-usage/src/api/client.rs] [VERIFIED: /d/00_Coding/glm-plan-usage/src/core/segments/glm_usage.rs]

The main planning risk is not architecture; it is verification depth. Official z.ai docs clearly document `ANTHROPIC_BASE_URL=https://api.z.ai/api/anthropic` for Claude Code-style usage and document 5-hour plus 7-day quota windows, but I could not find official docs that explicitly publish the quota-monitor endpoint path used by this plugin. The codebase already assumes `/monitor/usage/quota/limit` for both platforms, so the phase must treat live endpoint verification or captured-response verification as a first-class deliverable rather than silently assuming parity. That gap is now assigned to execution: Plan 01 must produce `.planning/phases/01-verify-and-implement-z-ai-support/01-z-ai-verification.md`, and Plan 02 must derive documentation wording from that artifact instead of from this unresolved research note. [CITED: https://docs.z.ai/devpack/faq] [CITED: https://docs.z.ai/devpack/overview] [VERIFIED: official-doc search on docs.z.ai/docs.bigmodel.cn did not surface a published `/monitor/usage/quota/limit` page in this session]

The repo currently has CI gates for `cargo test`, `cargo fmt -- --check`, and `cargo clippy -- -D warnings`, but no Rust test files or in-source `#[test]` modules were found. That means the phase plan should include Wave 0-style test scaffolding in the touched modules before or alongside the minimal runtime patch. [VERIFIED: /d/00_Coding/glm-plan-usage/.github/workflows/ci.yml] [VERIFIED: repo grep for `#[test]`, `#[cfg(test)]`, and `mod tests` returned no matches] [VERIFIED: project glob for `tests/**/*` found no Rust test tree]

**Primary recommendation:** Add explicit platform/base-URL resolution helpers with unit tests first, then make the smallest runtime patch needed for z.ai, then produce a concrete verification artifact closing the endpoint/parser question before updating docs. [VERIFIED: /d/00_Coding/glm-plan-usage/src/api/client.rs] [CITED: https://docs.z.ai/devpack/faq]

## Project Constraints (from CLAUDE.md)

- Keep compatibility with upstream architecture and user-facing behavior. [VERIFIED: /d/00_Coding/glm-plan-usage/CLAUDE.md]
- Prefer graceful degradation; API errors should return cached data or `None`, missing config should use defaults, invalid stdin should not crash the plugin, and missing env vars should return `None` from the segment path. [VERIFIED: /d/00_Coding/glm-plan-usage/CLAUDE.md] [VERIFIED: /d/00_Coding/glm-plan-usage/src/core/segments/glm_usage.rs] [VERIFIED: /d/00_Coding/glm-plan-usage/src/main.rs]
- Keep changes localized to the current module structure: `src/api/*` for platform/API behavior, `src/core/segments/glm_usage.rs` for caching/rendering behavior, `src/config/*` for config schema, and `src/main.rs` for CLI/runtime flow. [VERIFIED: /d/00_Coding/glm-plan-usage/CLAUDE.md]
- If config surface changes are required, update all three places: `src/config/types.rs`, the `Default` implementation, and the default config template/docs. [VERIFIED: /d/00_Coding/glm-plan-usage/CLAUDE.md]
- Preserve the existing status-line output contract unless a verified z.ai compatibility issue forces a change. [VERIFIED: /d/00_Coding/glm-plan-usage/CLAUDE.md] [VERIFIED: /d/00_Coding/glm-plan-usage/src/core/segments/glm_usage.rs]
- This repo currently expects tests to run with `cargo test`; CLAUDE.md explicitly calls out platform-detection tests, color-calculation tests, and mock-server integration tests as useful additions. [VERIFIED: /d/00_Coding/glm-plan-usage/CLAUDE.md]
- Release/distribution coupling matters: version bumps touch both `Cargo.toml` and `npm/main/package.json`, and packaging stays aligned with the current cargo + npm release flow. Phase 1 should avoid unnecessary changes here. [VERIFIED: /d/00_Coding/glm-plan-usage/CLAUDE.md]

## Standard Stack

### Core
| Library / Tool | Version in repo | Purpose | Why Standard |
|---------|---------|---------|--------------|
| Rust | edition 2021 | Implementation language | Existing codebase and CI are built around Rust crates and cargo workflows. [VERIFIED: /d/00_Coding/glm-plan-usage/Cargo.toml] |
| `ureq` | `2.10` | HTTP client for quota calls | Already used by `GlmApiClient`; keeps Phase 1 low-drift by avoiding an HTTP stack change. [VERIFIED: /d/00_Coding/glm-plan-usage/Cargo.toml] [VERIFIED: /d/00_Coding/glm-plan-usage/src/api/client.rs] |
| `serde` + `serde_json` | `1.0` | Response and stdin parsing | Existing API and stdin parsing are already modeled with serde derives. [VERIFIED: /d/00_Coding/glm-plan-usage/Cargo.toml] [VERIFIED: /d/00_Coding/glm-plan-usage/src/api/types.rs] [VERIFIED: /d/00_Coding/glm-plan-usage/src/main.rs] |
| `thiserror` + `anyhow` | `1.0` | Error typing and propagation | Current API layer uses typed errors and `anyhow::Result`, so Phase 1 should extend that pattern instead of changing error architecture. [VERIFIED: /d/00_Coding/glm-plan-usage/Cargo.toml] [VERIFIED: /d/00_Coding/glm-plan-usage/src/api/types.rs] [VERIFIED: /d/00_Coding/glm-plan-usage/src/api/client.rs] |

### Supporting
| Library / Tool | Version in repo | Purpose | When to Use |
|---------|---------|---------|-------------|
| `clap` | `4.5` | CLI flags | Leave unchanged unless z.ai support truly requires CLI surface changes, which current requirements do not. [VERIFIED: /d/00_Coding/glm-plan-usage/Cargo.toml] [VERIFIED: /d/00_Coding/glm-plan-usage/src/main.rs] |
| `cargo test` | toolchain `1.94.1` present | Unit/integration test runner | Use for all new Phase 1 automated coverage. [VERIFIED: local toolchain probe `cargo --version`] |
| `rustfmt` | `1.8.0-stable` present | Formatting gate | Required because CI runs `cargo fmt -- --check`. [VERIFIED: local toolchain probe `rustfmt --version`] [VERIFIED: /d/00_Coding/glm-plan-usage/.github/workflows/ci.yml] |
| `clippy` | `0.1.94` present | Lint gate | Required because CI runs `cargo clippy -- -D warnings`. [VERIFIED: local toolchain probe `cargo clippy -V`] [VERIFIED: /d/00_Coding/glm-plan-usage/.github/workflows/ci.yml] |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Localized helper(s) in `src/api/client.rs` | New client abstraction per platform | Rejected for Phase 1 because CONTEXT.md locks the single-client architecture and low-drift objective. [VERIFIED: /d/00_Coding/glm-plan-usage/.planning/phases/01-verify-and-implement-z-ai-support/01-CONTEXT.md] |
| Inline string replacement logic only | Dedicated URL normalization/resolution helper functions | Prefer helper functions because they are easier to test directly without broad refactor. [VERIFIED: /d/00_Coding/glm-plan-usage/src/api/client.rs] |
| Live-endpoint-only verification | Captured fixture or mock-response verification plus optional manual live check | Prefer both if possible; official docs do not publish the exact monitor endpoint path, so testable fixtures plus a verification artifact reduce uncertainty while keeping CI deterministic. [CITED: https://docs.z.ai/devpack/faq] [VERIFIED: official-doc search on docs.z.ai/docs.bigmodel.cn did not surface a published `/monitor/usage/quota/limit` page in this session] |

**Installation / execution for this phase:**
```bash
cargo test
cargo fmt -- --check
cargo clippy -- -D warnings
```

**Version verification note:** The table above reflects versions verified in the repo/toolchain during this session, not “latest available on the internet.” That is sufficient for Phase 1 planning because the objective is minimal-drift implementation in an existing brownfield fork. [VERIFIED: /d/00_Coding/glm-plan-usage/Cargo.toml] [VERIFIED: local toolchain probe]

## Architecture Patterns

### Recommended Project Structure
```text
src/
├── api/
│   ├── types.rs      # Platform enum, API DTOs, typed errors
│   └── client.rs     # env loading, base URL normalization, endpoint resolution, fetch logic
├── core/
│   └── segments/
│       └── glm_usage.rs   # cache fallback + user-visible formatting
└── main.rs           # CLI overrides and stdin/runtime orchestration
```

### Pattern 1: Explicit platform detection plus URL normalization helper
**What:** Keep `Platform::detect_from_url` as the single classifier, then normalize the base URL and compose the quota endpoint in explicit helper(s). [VERIFIED: /d/00_Coding/glm-plan-usage/src/api/types.rs] [VERIFIED: /d/00_Coding/glm-plan-usage/src/api/client.rs]

**When to use:** For Phase 1 routing fixes and regression coverage around CN vs z.ai URL handling. [VERIFIED: /d/00_Coding/glm-plan-usage/.planning/REQUIREMENTS.md]

**Example:**
```rust
// Source: current repo pattern in src/api/client.rs and src/api/types.rs
let platform = Platform::detect_from_url(&base_url)
    .ok_or(ApiError::PlatformDetectionFailed)?;

let normalized_base_url = match platform {
    Platform::Zhipu => base_url
        .replace("/api/anthropic", "/api")
        .replace("/anthropic", ""),
    Platform::Zai => base_url,
};

let quota_url = format!("{}/monitor/usage/quota/limit", normalized_base_url);
```

### Pattern 2: Unit tests in the touched Rust files first
**What:** Add `#[cfg(test)]` modules close to platform detection and URL resolution logic, because the logic is pure and currently untested. [VERIFIED: repo grep for `#[test]`, `#[cfg(test)]`, and `mod tests` returned no matches] [CITED: C:\Users\he_al\.claude\rules\rust\testing.md]

**When to use:** For `Platform::detect_from_url`, any new normalization helper, and any small response-normalization helper introduced in Phase 1. [VERIFIED: /d/00_Coding/glm-plan-usage/.planning/REQUIREMENTS.md]

**Example:**
```rust
// Source: Rust testing pattern guidance + this repo's pure helper shape
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_zai_from_api_z_ai_base_url() {
        assert_eq!(
            Platform::detect_from_url("https://api.z.ai/api/anthropic"),
            Some(Platform::Zai)
        );
    }
}
```

### Pattern 3: Preserve graceful degradation at segment boundary
**What:** Keep API errors from bubbling into Claude Code by preserving the current `Some(cached)` or `None` fallback behavior. [VERIFIED: /d/00_Coding/glm-plan-usage/src/core/segments/glm_usage.rs] [VERIFIED: /d/00_Coding/glm-plan-usage/CLAUDE.md]

**When to use:** If Phase 1 touches response parsing, endpoint selection, or temporary API incompatibilities. [VERIFIED: /d/00_Coding/glm-plan-usage/.planning/PROJECT.md]

**Example:**
```rust
// Source: src/core/segments/glm_usage.rs
Err(_) => {
    self.cache.lock().unwrap().as_ref().map(|e| e.stats.clone())
}
```

### Anti-Patterns to Avoid
- **Do not add a second API client abstraction:** It violates the locked single-client architecture for this phase. [VERIFIED: /d/00_Coding/glm-plan-usage/.planning/phases/01-verify-and-implement-z-ai-support/01-CONTEXT.md]
- **Do not broaden config schema unless required:** `ApiConfig` already exists but `GlmApiClient` currently hardcodes timeout/retry behavior, so extra config work would deepen config/runtime drift unless the phase explicitly includes wiring it through. [VERIFIED: /d/00_Coding/glm-plan-usage/src/config/types.rs] [VERIFIED: /d/00_Coding/glm-plan-usage/src/api/client.rs]
- **Do not claim verified z.ai support in docs before tests or runtime verification exist:** Both READMEs currently claim ZAI support more broadly than the current evidence supports. [VERIFIED: /d/00_Coding/glm-plan-usage/README.md] [VERIFIED: /d/00_Coding/glm-plan-usage/README_en.md]

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Platform routing | A new routing subsystem | Small pure helpers around `Platform::detect_from_url` and base URL normalization | The existing code already centralizes this behavior; more abstraction increases fork drift without solving the Phase 1 problem. [VERIFIED: /d/00_Coding/glm-plan-usage/src/api/types.rs] [VERIFIED: /d/00_Coding/glm-plan-usage/src/api/client.rs] |
| Test harness | A custom external test runner | Native Rust `#[test]` modules and `cargo test` | CI and local tooling already use cargo; no extra framework is required for Phase 1 unit coverage. [VERIFIED: /d/00_Coding/glm-plan-usage/.github/workflows/ci.yml] [CITED: C:\Users\he_al\.claude\rules\rust\testing.md] |
| Response parsing | A parallel response model for z.ai unless proven necessary | Keep `QuotaLimitResponse` / `UsageStats`, with a thin normalization helper only if live/captured responses demand it | CONTEXT.md allows only a light normalization layer, and broader DTO forks increase maintenance cost. [VERIFIED: /d/00_Coding/glm-plan-usage/.planning/phases/01-verify-and-implement-z-ai-support/01-CONTEXT.md] [VERIFIED: /d/00_Coding/glm-plan-usage/src/api/types.rs] |
| Documentation validation | Broad marketing-style support claims | Conservative config + troubleshooting examples for CN and z.ai | Docs are already ahead of verification; Phase 1 should reduce, not amplify, that drift. [VERIFIED: /d/00_Coding/glm-plan-usage/README.md] [VERIFIED: /d/00_Coding/glm-plan-usage/README_en.md] |

**Key insight:** The dangerous complexity in this phase is not code volume; it is silent assumption drift between base URL detection, quota endpoint composition, API response shape, and user-facing docs. Use narrow helpers plus tests plus a concrete verification artifact to make those assumptions explicit. [VERIFIED: /d/00_Coding/glm-plan-usage/src/api/client.rs] [VERIFIED: /d/00_Coding/glm-plan-usage/.planning/PROJECT.md]

## Common Pitfalls

### Pitfall 1: Treating Anthropic-compatible base URL docs as proof of quota-monitor endpoint parity
**What goes wrong:** The planner assumes that because z.ai documents `ANTHROPIC_BASE_URL=https://api.z.ai/api/anthropic`, the plugin’s monitor endpoint path must also be officially guaranteed. [CITED: https://docs.z.ai/devpack/faq]

**Why it happens:** The plugin uses an Anthropic-compatible base URL for Claude Code but a separate monitor endpoint path for quota data; those are related but not the same contract. [VERIFIED: /d/00_Coding/glm-plan-usage/src/api/client.rs]

**How to avoid:** Make “verify exact quota endpoint behavior” an explicit execution task and require `.planning/phases/01-verify-and-implement-z-ai-support/01-z-ai-verification.md` before docs claim verified support. [VERIFIED: /d/00_Coding/glm-plan-usage/.planning/REQUIREMENTS.md]

**Warning signs:** Tests only assert string concatenation, while no fixture/live check proves the endpoint actually returns the expected shape for z.ai. [ASSUMED]

### Pitfall 2: Regressing ZHIPU CN while fixing z.ai
**What goes wrong:** A normalization change fixes z.ai but accidentally breaks the current ZHIPU path that removes `/anthropic` before appending `/monitor/usage/quota/limit`. [VERIFIED: /d/00_Coding/glm-plan-usage/src/api/client.rs]

**Why it happens:** The current CN path is handled by string replacement, so a seemingly harmless refactor can alter slash behavior or duplicate path fragments. [VERIFIED: /d/00_Coding/glm-plan-usage/src/api/client.rs]

**How to avoid:** Add table-driven tests covering representative CN and z.ai inputs and exact resolved output URLs before changing runtime logic. [VERIFIED: /d/00_Coding/glm-plan-usage/.planning/REQUIREMENTS.md]

**Warning signs:** Helper tests cover only one platform, or expected URLs are reconstructed in multiple places. [ASSUMED]

### Pitfall 3: Deepening config/runtime drift
**What goes wrong:** The phase adds config knobs for retries/timeouts/platform behavior without wiring them consistently into runtime behavior. [VERIFIED: /d/00_Coding/glm-plan-usage/src/config/types.rs] [VERIFIED: /d/00_Coding/glm-plan-usage/src/api/client.rs]

**Why it happens:** `ApiConfig` already exists, but `GlmApiClient::from_env()` currently hardcodes 5-second timeout and three tries via loop logic. [VERIFIED: /d/00_Coding/glm-plan-usage/src/config/types.rs] [VERIFIED: /d/00_Coding/glm-plan-usage/src/api/client.rs]

**How to avoid:** Keep Phase 1 scoped to routing/tests/docs unless a config mismatch blocks verified z.ai support. [VERIFIED: /d/00_Coding/glm-plan-usage/.planning/PROJECT.md]

**Warning signs:** New config fields appear in types/docs with no matching runtime reads or tests. [ASSUMED]

### Pitfall 4: Updating docs before the verification story is complete
**What goes wrong:** README examples imply equal support certainty for CN and z.ai before code/tests prove it. [VERIFIED: /d/00_Coding/glm-plan-usage/README.md] [VERIFIED: /d/00_Coding/glm-plan-usage/README_en.md]

**Why it happens:** Docs already advertise dual-platform support while behavioral coverage is currently sparse. [VERIFIED: /d/00_Coding/glm-plan-usage/.planning/STATE.md]

**How to avoid:** Draft docs from tested behavior and `.planning/phases/01-verify-and-implement-z-ai-support/01-z-ai-verification.md`, using explicit wording such as “verified with tested routing” rather than broad “supports everything” phrasing. [VERIFIED: /d/00_Coding/glm-plan-usage/.planning/phases/01-verify-and-implement-z-ai-support/01-CONTEXT.md]

**Warning signs:** README changes land without new tests or without a verification artifact/manual verification note. [ASSUMED]

## Code Examples

Verified patterns from current repo and official docs:

### Platform detection from base URL
```rust
// Source: /d/00_Coding/glm-plan-usage/src/api/types.rs
pub fn detect_from_url(base_url: &str) -> Option<Self> {
    if base_url.contains("api.z.ai") {
        Some(Platform::Zai)
    } else if base_url.contains("bigmodel.cn") || base_url.contains("zhipu") {
        Some(Platform::Zhipu)
    } else {
        None
    }
}
```

### Current quota endpoint composition behavior
```rust
// Source: /d/00_Coding/glm-plan-usage/src/api/client.rs
let base_url = if platform == Platform::Zhipu {
    base_url
        .replace("/api/anthropic", "/api")
        .replace("/anthropic", "")
} else {
    base_url
};

let url = format!("{}/monitor/usage/quota/limit", self.base_url);
```

### Official z.ai Claude-compatible environment example
```json
// Source: https://docs.z.ai/devpack/faq
{
  "ANTHROPIC_AUTH_TOKEN": "your_Z.ai_api_key",
  "ANTHROPIC_BASE_URL": "https://api.z.ai/api/anthropic"
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Untested implicit platform behavior | Explicitly tested platform detection and URL resolution | Needed now in Phase 1 because docs already claim dual-platform support but coverage is sparse. [VERIFIED: /d/00_Coding/glm-plan-usage/.planning/STATE.md] | Lowers regression risk and makes support claims credible. |
| CN-centered docs/examples | Separate CN and z.ai config/troubleshooting examples derived from the verification artifact | Needed now because official z.ai docs provide a different Anthropic-compatible base URL and the docs must be bounded by verified runtime evidence. [CITED: https://docs.z.ai/devpack/faq] [VERIFIED: /d/00_Coding/glm-plan-usage/README.md] | Reduces user confusion and support drift. |
| String replacement embedded directly in constructor | Small tested helper(s) for normalization/resolution | Recommended now for testability, not architecture overhaul. [VERIFIED: /d/00_Coding/glm-plan-usage/src/api/client.rs] | Enables precise regression tests with minimal fork drift. |

**Deprecated/outdated:**
- Broad README claim that the plugin “supports ZAI and ZHIPU platforms” without verification detail is outdated for this fork’s current evidence level. [VERIFIED: /d/00_Coding/glm-plan-usage/README.md] [VERIFIED: /d/00_Coding/glm-plan-usage/README_en.md]
- CN-only environment variable examples are outdated for a fork whose immediate goal is verified z.ai support. [VERIFIED: /d/00_Coding/glm-plan-usage/README.md] [VERIFIED: /d/00_Coding/glm-plan-usage/README_en.md] [CITED: https://docs.z.ai/devpack/faq]

## Assumptions Log

> List all claims tagged `[ASSUMED]` in this research. The planner and discuss-phase use this
> section to identify decisions that need user confirmation before execution.

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Tests that only assert URL composition will be insufficient without at least one live or captured-response verification for z.ai. | Common Pitfalls | Could overstate confidence in z.ai support even if the endpoint exists but returns a different schema. |
| A2 | Helper tests that omit one platform are likely to miss slash/path regressions. | Common Pitfalls | CN or z.ai routing could regress despite “green” tests. |
| A3 | README updates without tests or a verification artifact/manual verification note are likely to recreate docs/runtime drift. | Common Pitfalls | Users may trust unsupported paths and report failures later. |

## Open Questions (RESOLVED)

These questions were unresolved during research, but they are resolved for planning. Their closure mechanism is now explicitly delegated to execution through `.planning/phases/01-verify-and-implement-z-ai-support/01-z-ai-verification.md`, and Plan 02 documentation cannot proceed unless that artifact closes the questions with an allowed conclusion.

1. **Resolved for planning: What is the officially supported z.ai quota-monitor endpoint path for this plugin’s usage query?**
   - What we know: Official docs document `ANTHROPIC_BASE_URL=https://api.z.ai/api/anthropic` for Claude Code use and document quota windows/reset cycles. [CITED: https://docs.z.ai/devpack/faq] [CITED: https://docs.z.ai/devpack/overview]
   - What remained unclear at research time: I did not find official docs in this session that explicitly publish `/monitor/usage/quota/limit` for z.ai. [VERIFIED: official-doc search on docs.z.ai/docs.bigmodel.cn did not surface a published `/monitor/usage/quota/limit` page in this session]
   - Planning resolution: Plan 01 Task 3 must record the exact resolved quota URL and the verification method in `01-z-ai-verification.md`, and Phase 1 cannot complete without that artifact reaching a closed conclusion.

2. **Resolved for planning: Does z.ai return the same `QuotaLimitResponse` shape as the current ZHIPU CN parser expects?**
   - What we know: Current code expects `success`, `msg`, `data.limits[]`, `type`, `usage`, `currentValue`, `percentage`, and optional `nextResetTime`. [VERIFIED: /d/00_Coding/glm-plan-usage/src/api/types.rs]
   - What remained unclear at research time: No official z.ai response schema for this monitor endpoint was found in this session. [VERIFIED: official-doc search on docs.z.ai/docs.bigmodel.cn did not surface a published `/monitor/usage/quota/limit` page in this session]
   - Planning resolution: Plan 01 Task 3 must record observed response-shape fields or a redacted captured response and conclude `compatible` or `requires normalization` in `01-z-ai-verification.md`. `unresolved` is not a valid success outcome.

3. **Resolved for Phase 1: Should Phase 1 wire existing `ApiConfig.timeout_ms` and `retry_attempts` into runtime?**
   - What we know: The config schema has those fields, but `GlmApiClient` currently hardcodes 5-second timeout and three tries via loop logic. [VERIFIED: /d/00_Coding/glm-plan-usage/src/config/types.rs] [VERIFIED: /d/00_Coding/glm-plan-usage/src/api/client.rs]
   - Resolution: Out of scope for Phase 1 because current requirements focus on routing, compatibility verification, tests, and documentation, not config/runtime parity work. [VERIFIED: /d/00_Coding/glm-plan-usage/.planning/REQUIREMENTS.md]
   - Conditional reopen rule: Reopen only if Plan 01 verification proves z.ai compatibility is blocked specifically by the current hardcoded timeout/retry behavior.
   - Planning effect: No dedicated Phase 1 task is required for timeout/retry wiring unless that blocker is observed during verification.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| cargo | Rust tests/build/lint | ✓ | 1.94.1 | — |
| rustc | Compile changes | ✓ | 1.94.1 | — |
| rustfmt | CI formatting gate | ✓ | 1.8.0-stable | — |
| clippy | CI lint gate | ✓ | 0.1.94 | — |
| gh | GitHub code search required by user workflow guidance | ✓ | 2.89.0 | WebSearch if GitHub search quality is insufficient |

**Missing dependencies with no fallback:**
- None found in the local phase-planning environment. [VERIFIED: local toolchain probe]

**Missing dependencies with fallback:**
- None found. [VERIFIED: local toolchain probe]

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Native Rust `#[test]` unit tests via cargo [CITED: C:\Users\he_al\.claude\rules\rust\testing.md] |
| Config file | none — cargo defaults plus repo CI workflow [VERIFIED: /d/00_Coding/glm-plan-usage/.github/workflows/ci.yml] |
| Quick run command | `cargo test platform` or `cargo test client` once tests are named accordingly [ASSUMED] |
| Full suite command | `cargo test && cargo fmt -- --check && cargo clippy -- -D warnings` [VERIFIED: /d/00_Coding/glm-plan-usage/.github/workflows/ci.yml] |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| FR-1 | Recognize `z.ai` base URLs as supported platform | unit | `cargo test detect_from_url` [ASSUMED] | ❌ Wave 0 |
| FR-2 | Resolve correct quota endpoint for z.ai and preserve CN path | unit | `cargo test quota_endpoint_resolution` [ASSUMED] | ❌ Wave 0 |
| FR-3 | Graceful degradation on auth/network/parsing failure | unit | `cargo test graceful_degradation` if touched [ASSUMED] | ❌ Wave 0 |
| QR-1 | Regression coverage for platform routing | unit | `cargo test platform` [ASSUMED] | ❌ Wave 0 |
| QR-2 | Regression coverage for base URL transformation | unit | `cargo test client` [ASSUMED] | ❌ Wave 0 |
| QR-3 | CI quality gates remain green | build/lint | `cargo test && cargo fmt -- --check && cargo clippy -- -D warnings` | ✅ |

### Sampling Rate
- **Per task commit:** `cargo test`
- **Per wave merge:** `cargo test && cargo fmt -- --check && cargo clippy -- -D warnings`
- **Phase gate:** Full suite green plus `01-z-ai-verification.md` closing the z.ai endpoint/response behavior question

### Wave 0 Gaps
- [ ] Add `#[cfg(test)]` module in `src/api/types.rs` for platform detection coverage.
- [ ] Add `#[cfg(test)]` module in `src/api/client.rs` for base URL normalization and quota endpoint resolution.
- [ ] If response parsing or cache fallback changes, add parser/fallback tests in the touched file(s), likely `src/api/client.rs` and/or `src/core/segments/glm_usage.rs`.
- [ ] Capture one verified z.ai response sample or manual verification note in `.planning/phases/01-verify-and-implement-z-ai-support/01-z-ai-verification.md` if official endpoint docs remain unavailable.

## Security Domain

### Applicable ASVS Categories
| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | yes | Bearer token from `ANTHROPIC_AUTH_TOKEN`; no hardcoded secrets. [VERIFIED: /d/00_Coding/glm-plan-usage/src/api/client.rs] [CITED: C:\Users\he_al\.claude\rules\rust\security.md] |
| V3 Session Management | no | Not a session-based app; this is a short-lived CLI process. [VERIFIED: /d/00_Coding/glm-plan-usage/src/main.rs] |
| V4 Access Control | no | No internal role/access layer in this plugin. [VERIFIED: /d/00_Coding/glm-plan-usage/src/main.rs] |
| V5 Input Validation | yes | `serde` parsing with fallback/default behavior at stdin and API boundaries. [VERIFIED: /d/00_Coding/glm-plan-usage/src/main.rs] [VERIFIED: /d/00_Coding/glm-plan-usage/src/api/types.rs] |
| V6 Cryptography | no | TLS is delegated to the HTTP client stack; no custom cryptography is implemented here. [VERIFIED: /d/00_Coding/glm-plan-usage/src/api/client.rs] |

### Known Threat Patterns for this stack
| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Token leakage in docs/tests/artifacts | Information Disclosure | Keep `ANTHROPIC_AUTH_TOKEN` in env vars only; never commit live credentials or captured responses containing secrets. [CITED: C:\Users\he_al\.claude\rules\common\security.md] [CITED: C:\Users\he_al\.claude\rules\rust\security.md] |
| Malformed or changed API response shape | Tampering | Parse into typed structs and preserve graceful fallback to cache/`None` rather than crashing Claude Code. [VERIFIED: /d/00_Coding/glm-plan-usage/src/api/types.rs] [VERIFIED: /d/00_Coding/glm-plan-usage/src/core/segments/glm_usage.rs] |
| Misconfigured base URL causing requests to wrong host/path | Spoofing/Tampering | Keep platform detection explicit, fail closed on unknown platform, and cover known base URL patterns with tests. [VERIFIED: /d/00_Coding/glm-plan-usage/src/api/types.rs] [VERIFIED: /d/00_Coding/glm-plan-usage/src/api/client.rs] |

## Sources

### Primary (HIGH confidence)
- `/d/00_Coding/glm-plan-usage/.planning/phases/01-verify-and-implement-z-ai-support/01-CONTEXT.md` - locked decisions, scope, and constraints
- `/d/00_Coding/glm-plan-usage/.planning/REQUIREMENTS.md` - functional, quality, and compatibility requirements
- `/d/00_Coding/glm-plan-usage/.planning/PROJECT.md` - fork goals and low-drift constraints
- `/d/00_Coding/glm-plan-usage/.planning/ROADMAP.md` - phase tasks and exit criteria
- `/d/00_Coding/glm-plan-usage/.planning/STATE.md` - current focus and risk framing
- `/d/00_Coding/glm-plan-usage/CLAUDE.md` - project architecture and graceful-degradation expectations
- `/d/00_Coding/glm-plan-usage/src/api/types.rs` - platform detection and API response model
- `/d/00_Coding/glm-plan-usage/src/api/client.rs` - current URL normalization and fetch path
- `/d/00_Coding/glm-plan-usage/src/core/segments/glm_usage.rs` - cache fallback and output contract
- `/d/00_Coding/glm-plan-usage/src/config/types.rs` - config schema and existing drift surface
- `/d/00_Coding/glm-plan-usage/src/main.rs` - CLI/runtime behavior
- `/d/00_Coding/glm-plan-usage/.github/workflows/ci.yml` - test/format/lint quality gates
- `/d/00_Coding/glm-plan-usage/README.md` and `/d/00_Coding/glm-plan-usage/README_en.md` - current support/config claims
- `https://docs.z.ai/devpack/faq` - official z.ai Anthropic-compatible env examples and quota/reset details
- `https://docs.z.ai/devpack/overview` - official z.ai plan quota and reset-cycle details

### Secondary (MEDIUM confidence)
- `/d/00_Coding/glm-plan-usage/.planning/research/z-ai-support.md` - prior lightweight repo research
- `https://docs.z.ai/devpack/extension/usage-query-plugin` - official z.ai plugin/extension context confirming usage-query capability

### Tertiary (LOW confidence)
- Official-domain searches on `docs.z.ai`, `docs.bigmodel.cn`, and `open.bigmodel.cn` did not surface a published page for `/monitor/usage/quota/limit`; this supports a “not found in docs” conclusion but does not prove the endpoint is unofficial or unsupported. [VERIFIED: WebSearch/WebFetch session artifacts]

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - determined directly from repo code, Cargo manifest, and CI workflow.
- Architecture: HIGH - current code centralizes routing/fetch/render flow clearly, and CONTEXT.md locks the architecture.
- Pitfalls: MEDIUM - repo evidence is strong, but live z.ai endpoint/schema verification is intentionally deferred to a concrete execution artifact in Phase 1.

**Research date:** 2026-04-07
**Valid until:** 2026-05-07
