# Project Retrospective

*A living document updated after each milestone. Lessons feed forward into future planning.*

## Milestone: v1.0 — GLM Plan Usage with reliable z.ai support

**Shipped:** 2026-04-08
**Phases:** 2 | **Plans:** 5

### What Was Built
- Verified z.ai platform detection and correct quota endpoint resolution
- Table-driven tests for platform detection and endpoint construction
- Fixed config/runtime drift for `timeout_ms` and `retry_attempts`
- Bilingual documentation with separate configuration examples
- Additional tests for core segment formatting and cache behavior

### What Worked
- Incremental approach: verify then implement then close gaps worked smoothly
- Extracting pure helper functions for testing avoided large refactors
- GSD audit process caught the existing config/runtime drift before shipping
- Phase structure kept work focused and allowed incremental completion

### What Was Inefficient
- Initial discovery phase required reading and understanding existing Rust codebase which took time
- No major inefficiencies — scope stayed bounded to z.ai support only

### Patterns Established
- Brownfield fork: understand, test, then fix — don't rewrite what works
- Audit milestone before completion catches gaps early
- Keep documentation in sync with implementation as you go

### Key Lessons
1. Existing documentation claims don't mean the feature actually works — always verify before claiming completion
2. Small, targeted tests are better than large refactors when working in a fork
3. Config/runtime drift is common — verify config fields actually get used

### Cost Observations
- Model mix: mostly sonnet for implementation, some opus for architecture decisions
- Notable: Rust codebase was straightforward to navigate with targeted exploration

---

## Cross-Milestone Trends

### Process Evolution

| Milestone | Phases | Key Change |
|-----------|--------|------------|
| v1.0 | 2 | Initial implementation of z.ai support with full GSD workflow |

### Cumulative Quality

| Milestone | Plans Added | New Tests | Zero-Dep Additions |
|-----------|-------------|-----------|-------------------|
| v1.0 | 5 | Comprehensive tests for platform detection and core segment behavior | All changes maintain upstream compatibility |

### Top Lessons (Verified Across Milestones)

1. Always verify existing claims before building on them
2. Minimal targeted changes are better than large refactors in forks
3. Audit before completion catches gaps that slip through incremental work
