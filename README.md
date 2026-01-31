# GLM Usage Status Plugin

A Claude Code plugin that displays GLM (ZHIPU/ZAI) coding plan usage statistics in the status bar.

## Features

- **Real-time Usage Tracking**: Displays token and MCP usage percentages
- **Color-coded Warnings**:
  - Green (0-79%): Normal usage
  - Yellow (80-94%): High usage
  - Red (95-100%): Critical usage
- **Smart Caching**: 5-minute cache to reduce API calls
- **Automatic Platform Detection**: Supports both ZAI and ZHIPU platforms
- **Graceful Degradation**: Silently fails on API errors

## Installation

### Method 1: Via npm (Recommended ⭐)

The easiest way to install and manage the plugin:

```bash
npm install -g glm-plan-usage
```

This installs the plugin globally and makes it available as `glm-plan-usage` in your terminal.

**Initialize Configuration**:

```bash
glm-plan-usage --init
```

**Verify Installation**:

```bash
glm-plan-usage --help
```

**Supported Platforms**:
- ✅ Linux (x64, arm64)
- ✅ macOS (Intel, Apple Silicon)
- ✅ Windows (x64)

---

### Method 2: Manual Installation from Source

Build from source if you prefer or need customization:

#### 1. Build

```bash
cd glm-plan-usage
cargo build --release
```

#### 2. Install

```bash
mkdir -p ~/.claude/glm-plan-usage
cp target/release/glm-plan-usage ~/.claude/glm-plan-usage/
chmod +x ~/.claude/glm-plan-usage/glm-plan-usage
```

#### 3. Initialize Configuration

```bash
~/.claude/glm-plan-usage/glm-plan-usage --init
```

This creates a default config at `~/.claude/glm-plan-usage/config.toml`:

```toml
[style]
mode = "plain"
separator = " | "

[[segments]]
id = "glm_usage"
enabled = true

[segments.colors]
text = { c256 = 109 }  # Default green

[segments.styles]
text_bold = true

[api]
timeout_ms = 5000
retry_attempts = 2

[cache]
enabled = true
ttl_seconds = 300
```

### 4. Configure Claude Code

**If installed via npm**:

Edit `~/.config/claude-code/settings.json`:

```json
{
  "statusLine": {
    "type": "command",
    "command": "glm-plan-usage",
    "padding": 0
  }
}
```

**If installed manually**:

```json
{
  "statusLine": {
    "type": "command",
    "command": "~/.claude/glm-plan-usage/glm-plan-usage",
    "padding": 0
  }
}
```

### 5. Set Environment Variables

Add to your `~/.bashrc` or `~/.zshrc`:

```bash
export ANTHROPIC_AUTH_TOKEN="your-token-here"
export ANTHROPIC_BASE_URL="https://open.bigmodel.cn/api/anthropic"
```

## Usage

The plugin automatically displays usage in the format:

```
T:42% M:15%
```

Where:
- `T:42%` = Token usage (5-hour window)
- `M:15%` = MCP usage (30-day window)

### Using the npm-installed Version

**Initialize config**:
```bash
glm-plan-usage --init
```

**Test the plugin**:
```bash
# Set environment variables
export ANTHROPIC_AUTH_TOKEN="your-token"
export ANTHROPIC_BASE_URL="https://open.bigmodel.cn/api/anthropic"

# Test with sample input
echo '{"model":{"id":"test"}}' | glm-plan-usage
```

**Update config manually** (if needed):
```bash
# Edit config directly
nano ~/.claude/glm-plan-usage/config.toml

# Or reinitialize
glm-plan-usage --init
```

## CLI Options

```bash
glm-plan-usage [OPTIONS]

Options:
  --init        Initialize configuration file
  --verbose     Enable verbose output
  --no-cache    Disable cache
  -h, --help    Print help
```

## Configuration

### Style Settings

```toml
[style]
mode = "plain"  # plain, nerd_font, powerline
separator = " | "
```

### Segment Colors

```toml
[[segments]]
id = "glm_usage"
enabled = true

[segments.colors]
text = { c256 = 109 }  # 256-color palette
# OR RGB:
# text = { r = 34, g = 197, b = 94 }
```

### API Settings

```toml
[api]
timeout_ms = 5000      # HTTP timeout
retry_attempts = 2     # Number of retries
```

### Cache Settings

```toml
[cache]
enabled = true
ttl_seconds = 300      # 5 minutes
```

## API Endpoints

The plugin queries the GLM API:

- **Quota Limits**: `GET /api/monitor/usage/quota/limit`
- **Token Usage**: `GET /api/monitor/usage/model-usage`
- **MCP Usage**: `GET /api/monitor/usage/tool-usage`

## Error Handling

The plugin gracefully handles errors:
- Missing environment variables → No output
- API timeout → Uses cached data (if available)
- API errors → Uses cached data (if available)
- No cache → Silent failure

Use `--verbose` to debug issues:

```bash
# If installed via npm
glm-plan-usage --verbose < input.json

# If installed manually
~/.claude/glm-plan-usage/glm-plan-usage --verbose < input.json
```

## Updating

### Via npm

```bash
# Update to the latest version
npm update -g glm-plan-usage

# Or reinstall
npm install -g glm-plan-usage@latest
```

### Manual Installation

```bash
# Rebuild from source
cd glm-plan-usage
cargo build --release

# Reinstall
cp target/release/glm-plan-usage ~/.claude/glm-plan-usage/
```

## Development

### Project Structure

```
glm-plan-usage/
├── Cargo.toml
├── README.md
└── src/
    ├── main.rs              # Entry point
    ├── cli.rs               # CLI parsing
    ├── lib.rs               # Library root
    ├── config/
    │   ├── mod.rs
    │   ├── types.rs         # Config types
    │   └── loader.rs        # Config loading
    ├── core/
    │   ├── mod.rs
    │   ├── statusline.rs    # ANSI rendering
    │   └── segments/
    │       ├── mod.rs
    │       └── glm_usage.rs # GLM usage segment
    └── api/
        ├── mod.rs
        ├── client.rs        # HTTP client
        └── types.rs         # API types
```

### Build Release

```bash
cargo build --release
```

The binary is stripped and optimized for size.

## License

MIT
