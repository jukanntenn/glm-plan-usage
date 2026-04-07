# STRUCTURE

## Top-level layout
- `src/` — Rust implementation
- `npm/` — npm wrapper, platform packages, release prep scripts
- `docs/` — research and planning notes
- `.github/workflows/` — CI and release automation
- `screenshots/` — demo assets

## Rust source structure
- `src/main.rs` — process entrypoint, stdin parsing, config load, generator wiring
- `src/lib.rs` — module exports
- `src/cli.rs` — CLI flags
- `src/api/`
  - `client.rs` — HTTP client and retry logic
  - `types.rs` — platform enum, API DTOs, errors, usage models
- `src/config/`
  - `types.rs` — config/input schemas and defaults
  - `loader.rs` — config file path, init, load behavior
- `src/core/`
  - `statusline.rs` — segment orchestration and ANSI formatting
  - `segments/mod.rs` — segment trait and render data types
  - `segments/glm_usage.rs` — only current segment implementation

## npm structure
- `npm/main/` — main public package, bin shim, postinstall hook
- `npm/platforms/*` — template package manifests for platform-specific binary packages
- `npm/scripts/prepare-packages.js` — release-time package generation

## Documentation structure
- `README.md`, `README_en.md` — install/config/usage docs
- `docs/api-research.md` — API investigation notes
- `docs/plans/` — dated design/test planning notes
- `CLAUDE.md` — rich project context and contributor guidance

## Structural observations
- Repo is compact and cohesive
- Most business logic is concentrated in `src/core/segments/glm_usage.rs` and `src/api/client.rs`
- Only one segment exists today, but structure allows more segments later
- npm and Rust concerns are cleanly separated by directory
