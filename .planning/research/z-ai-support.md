# Lightweight Research: z.ai support

## Research goal
Clarify the repo's current `z.ai` support status and identify the smallest safe scope for the fork's first planning cycle.

## Evidence from current repository
- README and README_en both claim support for ZAI and ZHIPU.
- `src/api/types.rs` already detects `api.z.ai` as `Platform::Zai`.
- `src/api/client.rs` only applies URL normalization for `Platform::Zhipu` and otherwise reuses the provided base URL.
- Existing API research notes are explicitly centered on the ZHIPU CN endpoint family (`open.bigmodel.cn`).
- Current codebase map identified weak behavioral test coverage around platform detection and URL transformation.

## What is likely already true
- The fork does not need a net-new platform abstraction; platform detection is already present.
- The highest-probability gap is not UI rendering but correctness/verification of endpoint resolution, request behavior, and docs for `z.ai`.
- The most compatible implementation path is to extend and verify the existing API client instead of restructuring the plugin.

## Likely gaps
1. `z.ai` support appears documented but not strongly validated by tests.
2. Research artifacts are skewed toward ZHIPU CN, so `z.ai` endpoint assumptions may be under-specified.
3. Troubleshooting and config examples still default to CN endpoints and may under-serve overseas users.

## Planning implications
- Phase 1 should focus on verified `z.ai` compatibility, not broad refactoring.
- Tests should be added around platform detection, base URL normalization, and usage parsing behavior.
- Documentation should be tightened only after behavior is verified, to avoid reinforcing drift.

## Recommended first implementation slice
- Add/expand tests for `Platform::detect_from_url` and API base URL handling.
- Confirm the quota endpoint path for `z.ai` in code assumptions and docs.
- Make the minimal code changes needed for reliable `z.ai` support.
- Update README/config guidance to show both CN and overseas endpoint options.
