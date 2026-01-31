[English](README.md) | [中文](README_zh.md)

# GLM Usage Status Plugin

A Claude Code plugin that displays GLM (ZHIPU/ZAI) coding plan usage statistics in the status bar.

## Features

- **Real-time Usage Tracking**: Token and MCP usage percentages
- **Color-coded Warnings**: Green (0-79%), Yellow (80-94%), Red (95-100%)
- **Smart Caching**: 5-minute cache to reduce API calls
- **Automatic Platform Detection**: Supports ZAI and ZHIPU platforms
- **Graceful Degradation**: Silently fails on API errors

## Installation

### Via npm (Recommended)

```bash
npm install -g glm-plan-usage
glm-plan-usage --init
```

### Manual Build

```bash
git clone https://github.com/your-repo/glm-plan-usage.git
cd glm-plan-usage
cargo build --release
mkdir -p ~/.claude/glm-plan-usage
cp target/release/glm-plan-usage ~/.claude/glm-plan-usage/
~/.claude/glm-plan-usage/glm-plan-usage --init
```

## Setup

### 1. Set Environment Variables

Add to `~/.bashrc` or `~/.zshrc`:

**ZHIPU platform:**
```bash
export ANTHROPIC_AUTH_TOKEN="your-token-here"
export ANTHROPIC_BASE_URL="https://open.bigmodel.cn/api/anthropic"
```

**ZAI platform:**
```bash
export ANTHROPIC_AUTH_TOKEN="your-token-here"
export ANTHROPIC_BASE_URL="https://api.z.ai/api/paas/v4/"
```

### 2. Configure Claude Code

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

For manual installation, use: `"command": "~/.claude/glm-plan-usage/glm-plan-usage"`

### 3. Restart Claude Code

The status bar will show: `T:42% M:15%`

## Output Format

```
T:42% M:15%
│   │  └─ MCP usage (30-day window)
│   └─ Token usage (5-hour window)
└─ Percentage
```

## Configuration

Edit `~/.claude/glm-plan-usage/config.toml`:

```toml
[style]
separator = " | "

[[segments]]
id = "glm_usage"
enabled = true

[segments.colors]
text = { c256 = 109 }  # Green (256-color palette)

[segments.styles]
text_bold = true

[api]
timeout_ms = 5000
retry_attempts = 2

[cache]
enabled = true
ttl_seconds = 300  # 5 minutes
```

## CLI Options

```bash
--init        Initialize configuration file
--verbose     Enable verbose output
--no-cache    Disable cache
--help        Show help
```

## Troubleshooting

**No output?**
```bash
# Check environment variables
echo $ANTHROPIC_AUTH_TOKEN

# Test with verbose mode
glm-plan-usage --verbose < input.json
```

**Colors not showing?**
```bash
# Test terminal colors
echo -e "\x1b[38;5;109mGreen\x1b[0m"
```

## Updating

**Via npm:**
```bash
npm update -g glm-plan-usage
```

**Manual:**
```bash
cargo build --release
cp target/release/glm-plan-usage ~/.claude/glm-plan-usage/
```

## Documentation

- [Quick Start Guide](guides/QUICK_START.md)
- [Implementation Notes](guides/IMPLEMENTATION_NOTES.md)
- [NPM Publishing Guide](guides/NPM_PUBLISHING_GUIDE.md)

## License

MIT
