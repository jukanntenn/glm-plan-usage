# glm-plan-usage

A Claude Code status bar plugin that displays GLM (ZHIPU/ZAI) coding plan usage statistics. Shows token usage percentage, countdown to reset, and MCP usage in real-time.

## Tech Stack

Core: Rust (edition 2021) + clap + ureq + serde + toml; Packaging: npm

## Commands

```bash
# Build & Run
cargo build --release          # Build release binary
./build-all.sh                 # Cross-platform build (7 targets)

# Lint & Format
cargo fmt                      # Format code
cargo clippy -- -D warnings    # Run linter

# Testing
cargo test                     # Run tests

# CLI
glm-plan-usage init            # Initialize config
glm-plan-usage print           # Print current config
glm-plan-usage check           # Validate config
```

## Project Structure

For detailed module organization, see `specs/directory-structure.md`.

```text
glm-plan-usage/
├── src/
│   ├── main.rs              # Entry point, stdin parsing, CLI handling
│   ├── cli.rs               # CLI argument definitions (clap derive)
│   ├── lib.rs               # Library interface
│   ├── config/              # Configuration loading and types
│   ├── api/                 # API client, cache, response types
│   └── core/                # Status line generation and segments
├── npm/                     # NPM packaging (main + platform binaries)
├── specs/                   # Design specifications
└── build-all.sh             # Cross-platform build script
```

## Standards

MUST FOLLOW THESE RULES, NO EXCEPTIONS

- Graceful degradation: the plugin must never cause Claude Code to fail. All segments return `Option<SegmentData>`, never panic.
- All config fields must have `#[serde(default)]` to allow adding new fields without breaking existing configs.
- Silent by default: no stderr output unless `--verbose` is set. API failures return `None`, not errors.
- Only add meaningful comments explaining why (not what) something is done.
