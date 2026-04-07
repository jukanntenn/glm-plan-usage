# STACK

## Primary technology
- Language: Rust 2021 (`Cargo.toml`)
- Binary crate with small library surface (`src/main.rs`, `src/lib.rs`)
- CLI parsing via `clap`
- Serialization/config via `serde`, `serde_json`, `toml`
- HTTP client via `ureq`
- Error handling via `anyhow` and `thiserror`
- User-home/config path detection via `dirs`

## Runtime model
- Command-style Claude Code status line plugin
- Reads JSON payload from stdin
- Reads auth/base URL from environment variables
- Fetches GLM quota data over HTTPS
- Writes formatted ANSI-colored status text to stdout

## Build and packaging
- Rust build driven by Cargo
- Release profile tuned for small binaries (`lto`, `strip`, single codegen unit)
- GitHub Actions CI for test/fmt/clippy/build
- Cross-platform release workflow builds Linux/macOS/Windows binaries
- Dual distribution:
  - native binaries from GitHub Releases
  - npm wrapper package plus platform packages under `npm/`

## Supported platforms
- Linux x64 / ARM64
- Linux musl x64 / ARM64
- macOS x64 / ARM64
- Windows x64

## Tooling present
- `cargo test`
- `cargo fmt -- --check`
- `cargo clippy -- -D warnings`
- GitHub Actions for CI/release automation

## Notable implementation choices
- Synchronous HTTP requests with short timeout and light retry loop
- In-memory cache inside the status segment
- No async runtime, database, or server component
