# Project Context

## Summary
This repository is a fork of `glm-plan-usage`, a Rust-based Claude Code status line plugin distributed via native binaries and npm. The fork remains broadly compatible with upstream, but adds dependable support for the overseas Zhipu platform `z.ai`.

## Project Type
- Brownfield fork
- Internal customization of an existing open-source plugin
- CLI/status-line integration tool

## What This Is
A Claude Code status line plugin that tracks GLM model token usage for both the Chinese mainland Zhipu platform (bigmodel.cn) and the overseas Zhipu platform (z.ai).

## Core Value
Reliable token usage tracking for GLM models on both Zhipu platforms with zero configuration changes for existing users.

## Requirements

### Validated

- ✓ Recognize z.ai base URLs as supported platform — v1.0
- ✓ Correct quota endpoint resolution for z.ai — v1.0
- ✓ Preserve graceful degradation on failures — v1.0
- ✓ Documentation describes configuration for both platforms — v1.0
- ✓ Bilingual documentation (Chinese + English) — v1.0
- ✓ Add automated tests for platform detection logic — v1.0
- ✓ Add automated tests for endpoint resolution — v1.0
- ✓ Avoid increasing drift between config schema and runtime behavior — v1.0 (fixed existing drift)
- ✓ Keep public CLI behavior unchanged — v1.0
- ✓ Keep repo structure compatible with upstream — v1.0
- ✓ Avoid breaking config changes for existing users — v1.0

### Active

- [ ] Improve maintainability by removing stale/legacy API types
- [ ] Refine internal module boundaries where change pressure appears

### Out of Scope

- Large architectural refactors unrelated to platform support
- New status line segments unrelated to GLM usage
- Major npm/release redesign
- Fork rebranding or package renaming

## Current State

Shipped **v1.0** with:
- 4478 lines of Rust code
- Verified z.ai platform detection and quota endpoint routing
- 5 new/updated plans with comprehensive test coverage
- Fixed existing config/runtime drift for timeout_ms and retry_attempts
- CI/build/release automation remains intact

## Constraints
- Prefer compatibility with upstream architecture and user-facing behavior
- Avoid unnecessary divergence in packaging and release structure
- Preserve graceful degradation behavior expected by a Claude Code status line plugin

## Key Decisions

| Decision | Rationale | Outcome | Status |
|----------|-----------|----------|--------|
| Keep single GlmApiClient architecture | Avoids breaking changes, minimizes drift | One client handles both platforms ✓ | Good |
| Extract pure helpers for detection/endpoint resolution | Improves testability without refactor | Table-driven tests cover all cases ✓ | Good |
| z.ai doesn't need URL normalization | Base URL already works correctly | Simplifies implementation ✓ | Good |
| Separate explicit examples for CN/z.ai in docs | User clarity | No confusion about configuration ✓ | Good |
| Token-safe troubleshooting guidance | Avoids encouraging secret exposure | Safer for users ✓ | Good |
| Tests directly manipulate cache state | No mocking/refactor needed | Tests added without architecture change ✓ | Good |

## Known Risks & Technical Debt
- **WR-01:** `retry_attempts` naming mismatch — field controls total attempts (including initial), but name implies retries after failure. Non-critical, works as documented.

## Immediate Objective (Original)
Implement and validate support for the overseas Zhipu platform (`z.ai`) while keeping fork drift low enough that future upstream sync remains practical.

---
*Last updated: 2026-04-08 after v1.0 milestone*
