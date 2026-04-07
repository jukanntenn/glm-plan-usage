---
phase: 01
slug: verify-and-implement-z-ai-support
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-04-08
---

# Phase 01 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Native Rust `#[test]` via cargo |
| **Config file** | none — cargo defaults plus `.github/workflows/ci.yml` |
| **Quick run command** | `cargo test` |
| **Full suite command** | `cargo test && cargo fmt -- --check && cargo clippy -- -D warnings` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test`
- **After every plan wave:** Run `cargo test && cargo fmt -- --check && cargo clippy -- -D warnings`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 01-01-01 | 01 | 1 | Platform detection and endpoint resolution | T-01-01 / Misconfigured base URL | Unknown platform fails closed; known CN/z.ai URLs resolve predictably | unit | `cargo test` | ❌ W0 | ⬜ pending |
| 01-01-02 | 01 | 1 | z.ai compatibility with current quota flow | T-01-02 / Response shape drift | Parser tolerates expected optional fields without crashing | unit | `cargo test` | ❌ W0 | ⬜ pending |
| 01-02-01 | 02 | 2 | Documentation alignment | T-01-03 / Docs-runtime drift | Docs only claim verified support paths | build/doc review | `cargo test && cargo fmt -- --check && cargo clippy -- -D warnings` | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `src/api/types.rs` — add platform-detection tests
- [ ] `src/api/client.rs` — add base URL normalization and quota endpoint resolution tests
- [ ] `src/api/client.rs` or `src/core/segments/glm_usage.rs` — add parser/fallback tests if implementation touches those paths

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Confirm the actual working z.ai quota-monitor endpoint and/or response sample | Verified z.ai support | Official docs found in research do not explicitly publish the quota-monitor endpoint path used by this plugin | Run the plugin with valid `ANTHROPIC_AUTH_TOKEN` and `ANTHROPIC_BASE_URL=https://api.z.ai/api/anthropic`, capture a successful request/response or verification note, and confirm output remains non-empty |
| Confirm existing ZHIPU CN setup still works with a real env configuration | No regression for CN users | Unit tests cannot prove live environment parity alone | Run the plugin with a known-good CN config and confirm the status line still renders expected token/MCP data |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
