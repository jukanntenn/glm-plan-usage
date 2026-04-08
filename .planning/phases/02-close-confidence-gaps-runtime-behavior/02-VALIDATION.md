---
phase: 2
slug: close-confidence-gaps-runtime-behavior
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-04-08
---

# Phase 2 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (built-in Rust testing) |
| **Config file** | none — Cargo handles it |
| **Quick run command** | `cargo test` |
| **Full suite command** | `cargo test && cargo fmt -- --check && cargo clippy -- -D warnings` |
| **Estimated runtime** | ~10 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test`
- **After every plan wave:** Run `cargo test && cargo fmt -- --check && cargo clippy -- -D warnings`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 60 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 02-01-01 | 01 | 1 | Add tests for formatting helpers | — | N/A | unit | `cargo test` | ✅ W0 | ⬜ pending |
| 02-01-02 | 01 | 1 | Add tests for cache behavior | — | N/A | unit | `cargo test` | ✅ W0 | ⬜ pending |
| 02-01-03 | 01 | 1 | Make config.timeout_ms and retry_attempts effective | — | N/A | integration | `cargo test` | ✅ W0 | ⬜ pending |
| 02-01-04 | 01 | 1 | Update documentation | — | N/A | manual | grep on CLAUDE.md | ✅ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

Existing infrastructure covers all phase requirements — cargo test already works.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Documentation update matches new config behavior | Doc tightening | Documentation is human-read | Verify that CLAUDE.md config section correctly describes configurable timeout and retry |

*All other phase behaviors have automated verification.*

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 60s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
