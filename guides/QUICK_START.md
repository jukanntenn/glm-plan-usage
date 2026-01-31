# Quick Start Guide

## Installation

### Via npm (Recommended)

```bash
npm install -g glm-plan-usage
glm-plan-usage --init
```

### Via install.sh

```bash
curl -sSL https://raw.githubusercontent.com/your-repo/glm-plan-usage/main/install.sh | bash
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

## Setup (3 Steps)

### 1. Set Environment Variables

Add to `~/.bashrc` or `~/.zshrc`:

**For ZHIPU platform:**
```bash
export ANTHROPIC_AUTH_TOKEN="your-token-here"
export ANTHROPIC_BASE_URL="https://open.bigmodel.cn/api/anthropic"
```

**For ZAI platform:**
```bash
export ANTHROPIC_AUTH_TOKEN="your-token-here"
export ANTHROPIC_BASE_URL="https://api.z.ai/api/paas/v4/"
```

### 2. Configure Claude Code

Edit `~/.config/claude-code/settings.json`:

**If installed via npm:**
```json
{
  "statusLine": {
    "type": "command",
    "command": "glm-plan-usage",
    "padding": 0
  }
}
```

**If installed manually:**
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

## Color Coding

- 🟢 **Green** (0-79%): Normal usage
- 🟡 **Yellow** (80-94%): High usage
- 🔴 **Red** (95-100%): Critical usage

## Testing

```bash
# Quick test
echo '{"model":{"id":"test"}}' | glm-plan-usage

# With credentials
export ANTHROPIC_AUTH_TOKEN="your-token"
export ANTHROPIC_BASE_URL="https://open.bigmodel.cn/api/anthropic"
echo '{"model":{"id":"test"}}' | glm-plan-usage

# Verbose mode (debug output)
glm-plan-usage --verbose < input.json

# Disable cache
glm-plan-usage --no-cache < input.json
```

## Customization

Edit `~/.claude/glm-plan-usage/config.toml`:

```toml
[style]
separator = " • "  # Change separator (default: " | ")

[segments.colors.text]
c256 = 226  # Change to yellow (default: 109 = green)

[segments.styles]
text_bold = false  # Disable bold text

[cache]
ttl_seconds = 600  # 10 minutes (default: 300)

[api]
timeout_ms = 10000  # 10 seconds (default: 5000)
retry_attempts = 3
```

## CLI Options

```bash
--init        # Create config file
--verbose     # Debug output
--no-cache    # Disable cache
--help        # Show help
```

## Troubleshooting

**No output?**
```bash
# Check env vars
echo $ANTHROPIC_AUTH_TOKEN
echo $ANTHROPIC_BASE_URL

# Test with verbose
glm-plan-usage --verbose < input.json
```

**Colors not showing?**
```bash
# Test terminal colors
echo -e "\x1b[38;5;109mGreen text\x1b[0m"
```

**API timeout?**
```toml
# Edit config.toml
[api]
timeout_ms = 10000
```

**Cache issues?**
```bash
# Disable cache temporarily
glm-plan-usage --no-cache < input.json
```

## Performance

- **First call**: ~500ms (3 sequential API calls)
- **Cached call**: <20ms
- **Binary size**: 3.1MB
- **Memory usage**: ~2MB

## Update

**Via npm:**
```bash
npm update -g glm-plan-usage
```

**Manual:**
```bash
cd glm-plan-usage
cargo build --release
cp target/release/glm-plan-usage ~/.claude/glm-plan-usage/
```

## Uninstall

```bash
# Via npm
npm uninstall -g glm-plan-usage

# Remove config
rm -rf ~/.claude/glm-plan-usage

# Remove statusLine from Claude Code settings
```

## More Information

- **Full documentation**: [README.md](../README.md)
- **Developer notes**: [IMPLEMENTATION_NOTES.md](IMPLEMENTATION_NOTES.md)
- **Publishing guide**: [NPM_PUBLISHING_GUIDE.md](NPM_PUBLISHING_GUIDE.md)
- **Issues**: https://github.com/your-repo/glm-plan-usage/issues
