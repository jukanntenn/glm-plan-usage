# State

## Project
- Name: glm-plan-usage fork
- Type: brownfield fork / internal customization
- Current primary goal: verified support for overseas Zhipu platform `z.ai`
- Upstream strategy: keep compatibility where practical

## Existing artifacts
- Codebase map exists under `.planning/codebase/`
- Lightweight research added under `.planning/research/z-ai-support.md`
- Requirements and roadmap initialized

## Current focus
Phase 1 planning should center on verified `z.ai` support with tests before broader cleanup or refactoring.

## Risks to keep visible
- Behavioral coverage is sparse
- `z.ai` support is claimed in docs but may be only partially verified
- Config/runtime drift already exists and should not be worsened

## Next step
Run `/gsd-plan-phase 1`
