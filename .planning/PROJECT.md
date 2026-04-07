# Project Context

## Summary
This repository is a fork of `glm-plan-usage`, a Rust-based Claude Code status line plugin distributed via native binaries and npm. The fork remains broadly compatible with upstream, but its immediate goal is to support the overseas Zhipu platform `z.ai` more reliably as an internal-customized version.

## Project Type
- Brownfield fork
- Internal customization of an existing open-source plugin
- CLI/status-line integration tool

## Current State
- Existing Rust implementation with modular API/config/core layers
- Existing npm wrapper and multi-platform packaging flow
- Existing codebase map under `.planning/codebase/`
- CI/build/release automation present
- Test coverage is currently light and behavior-focused tests are sparse

## Immediate Objective
Implement and validate support for the overseas Zhipu platform (`z.ai`) while keeping fork drift low enough that future upstream sync remains practical.

## Constraints
- Prefer compatibility with upstream architecture and user-facing behavior
- Avoid unnecessary divergence in packaging and release structure
- Preserve graceful degradation behavior expected by a Claude Code status line plugin
- Changes should be backed by targeted tests because current behavioral coverage is thin

## Success Signals
- `z.ai` base URL and platform detection work correctly
- Quota endpoint resolution for `z.ai` is confirmed and robust
- Status line output remains correct for both Zhipu CN and `z.ai`
- Documentation/config examples reflect both supported platforms
- New behavior is protected by automated tests

## Known Risks
- Current docs already claim ZAI support; actual support may be partial or unverified
- API shape or endpoint path may differ between `bigmodel.cn` and `api.z.ai`
- Config surface and runtime behavior already show some drift, so additions should avoid deepening that mismatch
