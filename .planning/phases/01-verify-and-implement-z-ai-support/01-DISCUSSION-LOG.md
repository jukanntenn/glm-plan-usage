# Phase 1: verify-and-implement-z-ai-support - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-07
**Phase:** 01-verify-and-implement-z-ai-support
**Areas discussed:** URL strategy, Compatibility scope, Test focus, Documentation posture

---

## URL strategy

| Option | Description | Selected |
|--------|-------------|----------|
| Keep current Platform + single client | Reuse `Platform` enum and `GlmApiClient`, make only minimal platform-specific routing changes | ✓ |
| Add explicit platform-specific routing helpers | Still low-risk, but introduces more structure in client code | |
| Split clients by platform | Higher divergence and unnecessary for Phase 1 unless proven required | |

**User's choice:** Claude decided — keep the current `Platform` + single client structure and make only minimal routing changes.
**Notes:** User explicitly delegated decision-making. Decision optimized for low fork drift and localized change.

---

## Compatibility scope

| Option | Description | Selected |
|--------|-------------|----------|
| Route correctly and keep current response/output contract | Prefer smallest fix that preserves current rendering behavior | ✓ |
| Allow light normalization | Accept small parsing/normalization shim if needed to map `z.ai` response differences into current model | ✓ |
| Redesign response model or output | Broader change, out of scope for Phase 1 | |

**User's choice:** Claude decided — guarantee correct routing first, allow only light normalization if required, and keep status line output unchanged.
**Notes:** This keeps verification tight and avoids broad refactors.

---

## Test focus

| Option | Description | Selected |
|--------|-------------|----------|
| Platform detection + endpoint resolution + CN regression | Highest-value confidence for Phase 1 | ✓ |
| Also cover parsing/cache if touched | Add targeted tests when implementation changes those paths | ✓ |
| Defer tests until after code changes | Higher regression risk, not preferred | |

**User's choice:** Claude decided — prioritize routing/regression tests, and include parsing/cache tests if the implementation touches those paths.
**Notes:** Aligns with roadmap ordering and requirements coverage.

---

## Documentation posture

| Option | Description | Selected |
|--------|-------------|----------|
| Conservative verified docs | Only document behavior proven by code/tests | ✓ |
| Broad support claims | Keep claiming support without narrowing examples or wording | |
| Speculative future-facing docs | Document intended support before verification | |

**User's choice:** Claude decided — docs should only claim verified support and should distinguish CN vs overseas setup clearly.
**Notes:** Current README support language is broader than current evidence, so Phase 1 should tighten that.

---

## Claude's Discretion

- Exact helper/API shape used to express endpoint resolution
- Exact unit/integration test placement
- Exact README/troubleshooting wording

## Deferred Ideas

- Large client architecture refactor
- New status line output formats
- Packaging/release redesign unrelated to verified `z.ai` support
