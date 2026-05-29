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
cargo test                     # Run all tests (unit + e2e)
cargo test --test e2e          # Run e2e tests only
cargo test --lib               # Run unit tests only

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
├── e2e/                    # End-to-end CLI tests (assert_cmd + httpmock)
│   ├── main.rs              # Test entry point
│   ├── helpers.rs           # Shared test utilities
│   ├── fixtures/            # JSON fixtures (stdin input, API responses)
│   └── tests/               # Test modules by workflow
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

## E2E Testing

Tests are in `e2e/` using `assert_cmd` (binary invocation), `httpmock` (mock HTTP server), and `tempfile` (config isolation).

**How it works:**

- **Config isolation**: Each test sets `HOME` to a temp directory via `cmd.env("HOME", temp_dir)`. The CLI reads config from `~/.claude/glm-plan-usage/config.toml`, so this isolates tests from real config.
- **API mocking**: The API client detects platform from `ANTHROPIC_BASE_URL` (URL containing `zhipu`/`bigmodel.cn` → ZHIPU, `api.z.ai` → ZAI). To mock, the URL is set to `http://127.0.0.1:{port}/zhipu/api/anthropic` — this triggers ZHIPU detection, code strips `/anthropic`, and requests hit `http://127.0.0.1:{port}/zhipu/api/monitor/usage/quota/limit` where the mock server listens.
- **Stdin piping**: Use `cmd.write_stdin(json_string)` to feed stdin data (owned `String`, not `&String`).

**Test coverage**: `init`, `print`, `check` subcommands; stdin mode with mocked API; error/edge cases (invalid JSON, missing config, API failure, graceful degradation).
