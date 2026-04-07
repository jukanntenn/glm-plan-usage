# CONVENTIONS

## Language and style conventions
- Rust-first codebase with idiomatic module layout by responsibility
- Uses typed structs/enums for config and API payloads
- Derive-heavy DTO style with `serde` attributes for wire/config compatibility
- Simple builder chaining for segment registration
- Naming follows standard Rust conventions (`snake_case`, `PascalCase`)

## Operational conventions
- Prefer graceful degradation instead of crashing Claude Code
- Environment variables are the integration boundary for auth/base URL
- Defaults are centralized in config types
- CLI overrides are applied after config load

## Formatting and lint conventions
- CI enforces `cargo fmt`
- CI enforces `cargo clippy -D warnings`
- Cargo tests are part of standard validation

## Packaging conventions
- Rust crate version and npm package version are expected to stay aligned
- npm main package depends on per-platform optional packages
- Release automation is tag-driven (`v*`)

## Documentation conventions
- Bilingual top-level README files
- `CLAUDE.md` contains contributor-oriented architecture and workflow notes
- `docs/plans/` stores dated design/process notes rather than executable specs

## Gaps / inconsistencies in conventions
- Some documented details in `CLAUDE.md` are richer than enforced behavior and may drift from code
- `ApiConfig` fields exist in config but current HTTP client/retry behavior is effectively hardcoded in implementation
- Formatting helpers exist for richer token display, but current visible output is intentionally minimal
