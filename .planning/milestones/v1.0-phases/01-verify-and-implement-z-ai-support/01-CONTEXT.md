# Phase 1: verify-and-implement-z-ai-support - Context

**Gathered:** 2026-04-07
**Status:** Ready for planning

<domain>
## Phase Boundary

Verify and implement reliable `z.ai` platform support for the existing GLM usage status-line plugin while preserving current ZHIPU CN behavior, graceful degradation, and low fork drift. This phase covers platform detection, endpoint/base URL handling, targeted automated tests, and documentation alignment for verified dual-platform support.

</domain>

<decisions>
## Implementation Decisions

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

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Project planning artifacts
- `.planning/PROJECT.md` — Project type, fork constraints, and low-drift objective for `z.ai` support.
- `.planning/REQUIREMENTS.md` — Functional, quality, and compatibility requirements for verified `z.ai` support.
- `.planning/ROADMAP.md` — Phase 1 goal, ordered tasks, and exit criteria.
- `.planning/STATE.md` — Current focus and risk framing for Phase 1.
- `.planning/research/z-ai-support.md` — Lightweight prior research on current `z.ai` support assumptions and likely gaps.

### Contributor and architecture docs
- `CLAUDE.md` — Architecture, graceful-degradation expectations, and operational guidance for this repo.
- `README.md` — Current Chinese user-facing support/configuration claims to validate and tighten.
- `README_en.md` — Current English user-facing support/configuration claims to validate and tighten.

### Core implementation files
- `src/api/types.rs` — Platform detection and quota response model.
- `src/api/client.rs` — Base URL handling, endpoint construction, and quota fetch behavior.
- `src/core/segments/glm_usage.rs` — Cache fallback and user-visible usage formatting behavior.
- `src/config/types.rs` — Cache-related config and current config surface.
- `src/main.rs` — CLI override behavior and top-level runtime flow.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `src/api/types.rs`: existing `Platform::detect_from_url` already recognizes `api.z.ai` and `bigmodel.cn` style URLs.
- `src/api/client.rs`: current client already centralizes platform detection, base URL transformation, retry behavior, auth header setup, and quota fetch logic.
- `src/core/segments/glm_usage.rs`: current segment already handles stale-cache fallback and stable output formatting.

### Established Patterns
- Graceful degradation is preferred over hard failure when env/config/API behavior is imperfect.
- Runtime behavior is intentionally compact and centralized rather than split across many abstractions.
- Rust source is cohesive, with most business logic concentrated in `src/api/client.rs` and `src/core/segments/glm_usage.rs`.
- Documentation currently risks drifting ahead of verified behavior, so code/test truth should drive docs updates.

### Integration Points
- `ANTHROPIC_BASE_URL` platform detection starts in `src/api/types.rs` and is consumed in `src/api/client.rs`.
- Any routing fix for `z.ai` flows through `GlmApiClient::from_env()` and `try_fetch_usage_stats()`.
- Any parsing or fallback adjustment affects the rendered segment through `GlmUsageSegment::get_usage_stats()`.
- User-visible verification will also require README updates after code/tests are settled.

</code_context>

<specifics>
## Specific Ideas

- Favor the smallest possible patch set that makes `z.ai` behavior explicit and testable.
- Prefer conservative docs that say what is verified, not what is merely intended.
- Preserve current output contract for the Claude Code status line.

</specifics>

<deferred>
## Deferred Ideas

- Large API/client refactors beyond what is needed for verified `z.ai` support.
- New status line segments or new output formats unrelated to GLM usage verification.
- Packaging/release redesign unless Phase 1 uncovers a direct blocker.

</deferred>

---

*Phase: 01-verify-and-implement-z-ai-support*
*Context gathered: 2026-04-07*
