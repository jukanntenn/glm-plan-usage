# Roadmap

## Phase 1 — Verify and implement z.ai support
**Goal:** Make overseas `z.ai` platform support reliable with minimal fork drift.
**Requirements:** [FR-1, FR-2, FR-3, FR-4, FR-5, QR-1, QR-2, QR-3, CR-1, CR-2, CR-3]
**Plans:** 2 plans

Plans:
- [x] 01-01-PLAN.md — Add routing/regression tests and minimal z.ai normalization in the existing client flow
- [x] 01-02-PLAN.md — Align bilingual docs with verified CN and z.ai behavior

### Tasks
1. Inspect current `Platform` detection and API base URL handling.
2. Add tests for platform detection and endpoint/base URL resolution.
3. Implement the minimal code changes required for verified `z.ai` compatibility.
4. Validate existing ZHIPU CN behavior remains intact.
5. Update README and troubleshooting guidance for dual-platform support.

### Exit criteria
- `z.ai` routing behavior is explicit and tested.
- Existing CN path is not regressed.
- Docs are updated to match verified behavior.

## Phase 2 — Close confidence gaps around runtime behavior
**Goal:** Reduce regression risk in the plugin's highest-value logic.
**Plans:** 3 plans

Plans:
- [ ] 02-01-PLAN.md — Add comprehensive inline test coverage for formatting and cache behavior in glm_usage.rs
- [ ] 02-02-PLAN.md — Align GlmApiClient with config: use timeout_ms and retry_attempts from ApiConfig
- [ ] 02-03-PLAN.md — Targeted documentation update in CLAUDE.md for config/runtime alignment

### Tasks
1. Add tests for countdown/formatting helpers and cache fallback behavior.
2. Review config/runtime drift, especially around API timeout/retry options.
3. Tighten contributor docs where they currently overstate or under-specify behavior.

### Exit criteria
- Core behavior has targeted automated coverage.
- Config surface more closely reflects actual runtime behavior.

## Phase 3 — Improve maintainability without unnecessary divergence
**Goal:** Make future upstream sync and ongoing fork maintenance cheaper.

### Tasks
1. Remove or isolate stale/legacy API types if confirmed unused.
2. Refine internal module boundaries only where repeated change pressure appears.
3. Reassess release/packaging complexity only if it blocks platform support or maintenance.

### Exit criteria
- Maintenance burden is lower without a large refactor.
- Fork remains reasonably upstream-compatible.

## Recommended next command
Run `/gsd-execute-phase 2` to create the executable implementation.
