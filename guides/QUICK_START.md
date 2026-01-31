# Quick Start Guide

## Installation (One Command)

```bash
curl -sSL https://raw.githubusercontent.com/your-repo/glm-plan-usage/main/install.sh | bash
```

Or manually:

```bash
git clone https://github.com/your-repo/glm-plan-usage.git
cd glm-plan-usage
cargo build --release
mkdir -p ~/.claude/glm-plan-usage
cp target/release/glm-plan-usage ~/.claude/glm-plan-usage/
~/.claude/glm-plan-usage/glm-plan-usage --init
```

## Setup (3 Steps)

### 1. Set Environment Variables

Add to `~/.bashrc` or `~/.zshrc`:

```bash
export ANTHROPIC_AUTH_TOKEN="your-token-here"
export ANTHROPIC_BASE_URL="https://open.bigmodel.cn/api/anthropic"
```

### 2. Configure Claude Code

Edit `~/.config/claude-code/settings.json`:

```json
{
  "statusLine": {
    "type": "command",
    "command": "~/.claude/glm-plan-usage/glm-plan-usage",
    "padding": 0
  }
}
```

### 3. Restart Claude Code

That's it! The status bar will show: `T:42% M:15%`

## Output Format

```
T:42% M:15%
│   │  └─ MCP usage (30-day window)
│   └─ Percentage
└─ Token usage (5-hour window)
```

## Colors

- 🟢 **Green** (0-79%): Normal
- 🟡 **Yellow** (80-94%): High
- 🔴 **Red** (95-100%): Critical

## Testing

```bash
# Quick test
echo '{"model":{"id":"test"}}' | ~/.claude/glm-plan-usage/glm-plan-usage

# With credentials
export ANTHROPIC_AUTH_TOKEN="your-token"
export ANTHROPIC_BASE_URL="https://open.bigmodel.cn/api/anthropic"
echo '{"model":{"id":"test"}}' | ~/.claude/glm-plan-usage/glm-plan-usage
```

## Troubleshooting

**No output?**
```bash
# Check env vars
echo $ANTHROPIC_AUTH_TOKEN

# Test with verbose
~/.claude/glm-plan-usage/glm-plan-usage --verbose < input.json
```

**Colors not showing?**
```bash
# Test terminal colors
echo -e "\x1b[38;5;109mGreen\x1b[0m"
```

**API timeout?**
```toml
# Edit config.toml
[api]
timeout_ms = 10000
```

## Customization

Edit `~/.claude/glm-plan-usage/config.toml`:

```toml
[style]
separator = " • "  # Change separator

[segments.colors.text]
c256 = 226  # Change to yellow

[cache]
ttl_seconds = 600  # 10 minutes
```

## CLI Options

```bash
--init        # Create config file
--verbose     # Debug output
--no-cache    # Disable cache
--help        # Show help
```

## Uninstall

```bash
rm -rf ~/.claude/glm-plan-usage
# Remove statusLine from Claude Code settings
```

## Need Help?

- Full docs: `README.md`
- Examples: `USAGE_EXAMPLES.md`
- Implementation: `IMPLEMENTATION_NOTES.md`
- Issues: https://github.com/your-repo/glm-plan-usage/issues
