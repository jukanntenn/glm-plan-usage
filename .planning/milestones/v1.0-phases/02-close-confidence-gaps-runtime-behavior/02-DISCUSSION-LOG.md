# Phase 2: close-confidence-gaps-runtime-behavior - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-08
**Phase:** 02-close-confidence-gaps-runtime-behavior
**Areas discussed:** Test Organization, Config/Runtime Alignment, Cache Test Coverage, Documentation Refinement

---

## Test Organization

| Option | Description | Selected |
|--------|-------------|----------|
| Keep inline tests | Continue using inline #[cfg(test)] in source files (consistent with existing endpoint tests) | ✓ |
| Move to separate tests/ | Create top-level tests/ directory (standard Rust convention) | |

**User's choice:** Keep inline tests
**Notes:** Consistent with existing pattern established in Phase 1 for endpoint resolution tests.

---

## Config/Runtime Alignment

| Option | Description | Selected |
|--------|-------------|----------|
| Make config effective | Update client to actually use config.api.timeout_ms and config.api.retry_attempts | ✓ |
| Keep hardcoding | Keep hardcoded values for simplicity, no user customization | |

**User's choice:** Make config effective
**Notes:** Eliminates documented config/runtime drift, allows user customization while keeping existing defaults.

---

## Cache Test Coverage

| Option | Description | Selected |
|--------|-------------|----------|
| Full coverage | Test all behaviors: hit/miss, TTL expiration, stale fallback on API error, disabled cache | ✓ |
| Basic coverage only | Only test basic hit/miss and enabled/disabled | |

**User's choice:** Full coverage
**Notes:** Covers all scenarios called out in the roadmap phase description.

---

## Documentation Refinement

| Option | Description | Selected |
|--------|-------------|----------|
| Targeted update only | Only update CLAUDE.md config/API section to fix known drift | ✓ |
| Full review and update | Review all docs (CLAUDE.md, README, README_en) for accuracy | |

**User's choice:** Targeted update only
**Notes:** Focuses on the specific drift we identified (timeout/retry configuration) rather than broad changes.

---

## Claude's Discretion

- Exact test function organization and naming
- Choice of test doubles/mocking approach for API failure simulation

## Deferred Ideas

- Moving tests to separate `tests/` directory
- Full review of all project documentation

