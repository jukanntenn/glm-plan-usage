# GLM Plan Usage - Claude Code Plugin

**AI Context File for Project Understanding and Development**

## Project Overview

This is a Claude Code plugin that displays GLM (ZHIPU/ZAI) coding plan usage statistics in the status bar. It's implemented in Rust with dual distribution via cargo build and npm package.

**Key Technologies:**
- **Rust** (edition 2021) - Core binary implementation
- **Claude Code** - Target integration platform
- **NPM packaging** - Distribution via npm registry
- **ureq** - HTTP client for API calls
- **serde/toml** - Configuration parsing
- **clap** - CLI argument parsing

**What it does:**
1. Reads JSON input from stdin (provided by Claude Code)
2. Fetches usage statistics from GLM API
3. Formats output with ANSI color codes
4. Displays in Claude Code's status bar

## Architecture

### Module Structure

```
glm-plan-usage/
├── src/
│   ├── main.rs              # Entry point, stdin parsing, CLI handling
│   ├── cli.rs               # Command-line argument definitions (Args struct)
│   ├── lib.rs               # Library interface, module exports
│   ├── config/
│   │   ├── mod.rs           # Config module exports
│   │   ├── types.rs         # All configuration structs (InputData, Config, etc.)
│   │   └── loader.rs        # Config file loading/parsing
│   ├── api/
│   │   ├── mod.rs           # API module exports
│   │   ├── client.rs        # GlmApiClient (HTTP requests, auth)
│   │   └── types.rs         # API response types, error types, Platform enum
│   └── core/
│       ├── mod.rs           # Core module exports
│       ├── statusline.rs    # StatusLineGenerator (segments orchestration)
│       └── segments/
│           ├── mod.rs       # Segment trait, SegmentData, SegmentStyle
│           └── glm_usage.rs # GlmUsageSegment (API integration, caching)
├── npm/
│   ├── main/
│   │   ├── package.json       # Main npm package (@jukanntenn/glm-plan-usage)
│   │   ├── bin/
│   │   │   └── glm-plan-usage.js  # NPM entry point
│   │   └── scripts/
│   │       └── postinstall.js  # Install script
│   ├── platforms/             # Platform-specific binary packages
│   │   ├── darwin-arm64/
│   │   ├── darwin-x64/
│   │   ├── linux-x64/
│   │   ├── linux-x64-musl/
│   │   ├── linux-arm64/
│   │   ├── linux-arm64-musl/
│   │   └── win32-x64/
│   └── scripts/
│       └── prepare-packages.js  # Package preparation script
└── build-all.sh             # Cross-platform build script
```

### Data Flow

```
┌─────────────────┐     ┌──────────────────┐     ┌─────────────────┐
│  Claude Code    │────▶│  main.rs:stdin   │────▶│ serde_json parse│
│  (statusLine)   │     │  read_stdin()    │     │  InputData      │
└─────────────────┘     └──────────────────┘     └────────┬────────┘
                                                          │
                                                          ▼
┌─────────────────┐     ┌──────────────────┐     ┌─────────────────┐
│  Claude Status  │◀────│ StatusLine       │◀────│ Segment::collect│
│     Bar         │     │ Generator        │     │ GlmUsageSegment │
└─────────────────┘     │ (format segments)│     │ (API + cache)   │
                        └──────────────────┘     └────────┬────────┘
                                                          │
                                                          ▼
                                                 ┌─────────────────┐
                                                 │  GLM API        │
                                                 │  /quota/limit   │
                                                 └─────────────────┘
```

### Key Modules and Responsibilities

| Module | File | Responsibility |
|--------|------|-----------------|
| **CLI Args** | `cli.rs` | Command-line argument definitions using clap derive (Args struct) |
| **CLI Handler** | `main.rs` | Parse args, handle `--init`, apply CLI overrides |
| **Config Loader** | `config/loader.rs` | Load TOML from `~/.claude/glm-plan-usage/config.toml` |
| **API Client** | `api/client.rs` | HTTP requests to GLM API with Bearer auth |
| **Segment Trait** | `core/segments/mod.rs` | Interface for pluggable status segments |
| **Usage Segment** | `core/segments/glm_usage.rs` | Fetch usage, apply caching, determine colors |
| **Status Generator** | `core/statusline.rs` | Combine segments, apply ANSI formatting |

## Development Guidelines

### Adding a New Status Segment

1. **Create segment module** in `src/core/segments/`:

```rust
// src/core/segments/my_segment.rs
use super::Segment;
use crate::config::{Config, InputData};
use crate::core::segments::{SegmentData, SegmentStyle};

pub struct MySegment;

impl Segment for MySegment {
    fn id(&self) -> &str {
        "my_segment"
    }

    fn collect(&self, input: &InputData, config: &Config) -> Option<SegmentData> {
        // Your logic here
        Some(SegmentData {
            text: "My Data".to_string(),
            style: SegmentStyle {
                color: Some((255, 0, 0)),
                color_256: None,
                bold: false,
            },
        })
    }
}
```

2. **Export from segments module** (`src/core/segments/mod.rs`):

```rust
pub mod my_segment;
pub use my_segment::MySegment;
```

3. **Register in main.rs**:

```rust
use core::{StatusLineGenerator, MySegment};

let generator = StatusLineGenerator::new()
    .add_segment(Box::new(GlmUsageSegment::new()))
    .add_segment(Box::new(MySegment));
```

### Configuration Extension Pattern

All config changes require updating three locations:

1. **`src/config/types.rs`** - Add struct field with `#[serde(default)]`
2. **`impl Default for Config`** - Provide default value
3. **`config.toml`** (default template) - Document the option

Example:

```rust
// types.rs
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    // ... existing fields
    #[serde(default)]
    pub my_feature: MyFeatureConfig,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MyFeatureConfig {
    #[serde(default = "default_my_setting")]
    pub my_setting: bool,
}

fn default_my_setting() -> bool {
    true
}
```

### Error Handling Philosophy

**Graceful Degradation**: The plugin should never cause Claude Code to fail.

- API errors → Return cached data or `None`
- Missing config → Use defaults
- Invalid stdin → Continue with empty `InputData`
- No env vars → Return `None` from segment

Pattern used in `glm_usage.rs:50-52`:

```rust
Err(_) => {
    // Return cached data if available
    self.cache.lock().unwrap().as_ref().map(|e| e.stats.clone())
}
```

## API Integration

### GLM API Endpoints

| Endpoint | Purpose | Response Type |
|----------|---------|---------------|
| `/monitor/usage/quota/limit` | Get all quota limits | `QuotaLimitResponse` |

### Authentication

```bash
export ANTHROPIC_AUTH_TOKEN="your-token-here"
export ANTHROPIC_BASE_URL="https://open.bigmodel.cn/api/anthropic"
```

The `GlmApiClient::from_env()` reads these and adds `Authorization: Bearer {token}` header.

**Important**: ZHIPU platform URL transformation (client.rs:27-33):
```
https://open.bigmodel.cn/api/anthropic
    → https://open.bigmodel.cn/api/monitor/usage/quota/limit
```

### Caching Strategy

- **TTL**: 5 minutes (configurable via `cache.ttl_seconds`)
- **Storage**: `Arc<Mutex<Option<CacheEntry>>>` in segment struct
- **Fallback**: On API error, return stale cached data

Cache check flow (`glm_usage.rs:26-33`):

```rust
if config.cache.enabled {
    if let Some(entry) = self.cache.lock().unwrap().as_ref() {
        if entry.timestamp.elapsed() < Duration::from_secs(config.cache.ttl_seconds) {
            return Some(entry.stats.clone());
        }
    }
}
```

### Color-coded Warning Levels

| Percentage | Color (256) | ANSI Escape |
|------------|-------------|-------------|
| 0-79% | 109 (Green) | `\x1b[38;5;109m` |
| 80-94% | 226 (Yellow) | `\x1b[38;5;226m` |
| 95-100% | 196 (Red) | `\x1b[38;5;196m` |

## Testing

### Run Tests

```bash
cargo test
```

### Test Coverage Status

The project currently has **no formal tests**. This is an area for improvement.

Suggested test additions:
- Unit tests for `Platform::detect_from_url()`
- Unit tests for color calculation logic
- Integration tests with mock API server

## Release Process

### Version Bumping

Must update **both** files:
1. `Cargo.toml` - `version` field in `[package]`
2. `npm/main/package.json` - `version` field

### Build Targets

Run `./build-all.sh` to build for all platforms:

```
- x86_64-unknown-linux-gnu
- x86_64-unknown-linux-musl
- aarch64-unknown-linux-gnu
- aarch64-unknown-linux-musl
- x86_64-apple-darwin
- aarch64-apple-darwin
- x86_64-pc-windows-msvc
```

Binaries are placed in `lib/` for NPM packaging.

### File Locations After Installation

| File | Location |
|------|----------|
| Binary | `~/.claude/glm-plan-usage/glm-plan-usage` |
| Config | `~/.claude/glm-plan-usage/config.toml` |
| Claude Settings | `~/.config/claude-code/settings.json` |

### Claude Settings Format

```json
{
  "statusLine": {
    "type": "command",
    "command": "glm-plan-usage",
    "padding": 0
  }
}
```

For manual builds, use full path: `"command": "~/.claude/glm-plan-usage/glm-plan-usage"`

## Common Code Patterns

### Reading Input (stdin → JSON)

```rust
let input_text = read_stdin()?;
let input: InputData = serde_json::from_str(&input_text)
    .unwrap_or_else(|_| InputData::default());
```

### ANSI Color Formatting

```rust
let output = format!(
    "\x1b[38;5;{}m\x1b[1m{}\x1b[0m",  // color + bold + reset
    color_code, text
);
```

### Segment Registration Chain

```rust
let generator = StatusLineGenerator::new()
    .add_segment(Box::new(Segment1::new()))
    .add_segment(Box::new(Segment2::new()));
```

## CLI Reference

```
--init        Initialize ~/.claude/glm-plan-usage/config.toml
--verbose     Print errors to stderr (for debugging)
--no-cache    Disable cache for this run
--help        Show usage
```

## Troubleshooting Development Issues

### Binary Not Found

```bash
# Check if binary is installed
ls -la ~/.claude/glm-plan-usage/glm-plan-usage

# Check npm bin path
npm bin -g
```

### Config Not Loading

```bash
# Run with verbose to see errors
echo '{"model": {"id": "test"}}' | glm-plan-usage --verbose

# Check config syntax
cat ~/.claude/glm-plan-usage/config.toml
```

### API Authentication Issues

```bash
# Verify env vars are set
echo $ANTHROPIC_AUTH_TOKEN
echo $ANTHROPIC_BASE_URL

# Test API directly
curl -H "Authorization: Bearer $ANTHROPIC_AUTH_TOKEN" \
  "$ANTHROPIC_BASE_URL/monitor/usage/quota/limit"
```

### Colors Not Showing

Test terminal 256-color support:

```bash
echo -e "\x1b[38;5;109mGreen\x1b[0m"
echo -e "\x1b[38;5;226mYellow\x1b[0m"
echo -e "\x1b[38;5;196mRed\x1b[0m"
```

## Platform Detection Logic

The `Platform` enum automatically detects the GLM platform from `ANTHROPIC_BASE_URL`:

```rust
// api/types.rs:13-21
pub fn detect_from_url(base_url: &str) -> Option<Self> {
    if base_url.contains("api.z.ai") {
        Some(Platform::ZAI)
    } else if base_url.contains("bigmodel.cn") || base_url.contains("zhipu") {
        Some(Platform::ZHIPU)
    } else {
        None
    }
}
```

**Supported Platforms:**
- **ZHIPU**: `https://open.bigmodel.cn/api/anthropic`
- **ZAI**: `https://api.z.ai/api/paas/v4/`

## Dependencies

| Crate | Purpose |
|-------|---------|
| clap | CLI argument parsing |
| serde/serde_json | JSON serialization |
| toml | Config file parsing |
| ureq | Simple HTTP client |
| anyhow | Error handling |
| thiserror | Custom error types |
| chrono | Time handling (unused, consider removing) |
| dirs | Platform-specific directory paths |

## License

MIT License - See `LICENSE` file.
