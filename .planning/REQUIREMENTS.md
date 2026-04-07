# Requirements

## Problem statement
This fork needs dependable support for the overseas Zhipu platform (`z.ai`). The repository already signals partial or intended ZAI support, but behavior is not sufficiently verified and current research/docs are still centered on the mainland Zhipu endpoint family.

## Product goal
Enable users of the fork to use the plugin with `z.ai` as confidently as with the existing Zhipu CN flow, while preserving upstream compatibility where practical.

## Scope
### In scope
- Platform detection and base URL handling for `z.ai`
- Correct quota endpoint resolution and request behavior for `z.ai`
- Status-line compatibility for `z.ai` responses if the response shape matches or can be safely normalized
- Targeted automated tests for the new/verified behavior
- Documentation updates for configuration and troubleshooting

### Out of scope
- Large architectural refactors unrelated to platform support
- New status line segments unrelated to GLM usage
- Major npm/release redesign
- Fork rebranding or package renaming in this initialization cycle

## Functional requirements
1. The plugin must recognize `z.ai` base URLs as a supported platform.
2. The API client must resolve the correct quota endpoint for `z.ai` without breaking the existing ZHIPU CN path.
3. The plugin must continue to degrade gracefully when auth, network, or API parsing fails.
4. The status line output format must remain stable for existing users unless a change is explicitly required.
5. Documentation must describe how to configure the plugin for both ZHIPU CN and `z.ai`.

## Quality requirements
1. Add automated tests for platform detection logic.
2. Add automated tests for base URL transformation or endpoint resolution logic.
3. Preserve existing CI quality gates: `cargo test`, `cargo fmt -- --check`, `cargo clippy -- -D warnings`.
4. Avoid increasing drift between config schema and runtime behavior.

## Compatibility requirements
1. Keep public CLI behavior unchanged unless required for `z.ai` support.
2. Keep repo structure and release/distribution flow compatible with upstream where possible.
3. Avoid introducing breaking config changes for existing users.

## Acceptance criteria
- A user can set `ANTHROPIC_BASE_URL` for `z.ai` and the plugin successfully attempts the correct usage endpoint.
- Existing ZHIPU CN behavior still works.
- New tests cover the supported platform-routing behavior.
- README/troubleshooting docs clearly explain overseas configuration.
