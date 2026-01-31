# Quick Guide: Installing from npm

## Installation

```bash
npm install -g glm-plan-usage
```

## Setup

### 1. Initialize Configuration

```bash
glm-plan-usage --init
```

This creates `~/.claude/glm-plan-usage/config.toml`

### 2. Set Environment Variables

Add to `~/.bashrc` or `~/.zshrc`:

```bash
export ANTHROPIC_AUTH_TOKEN="your-token-here"
export ANTHROPIC_BASE_URL="https://open.bigmodel.cn/api/anthropic"
```

### 3. Configure Claude Code

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

### 4. Restart Claude Code

The status bar will show: `T:42% M:78%`

## Usage

```bash
# Show help
glm-plan-usage --help

# Test the plugin
echo '{"model":{"id":"test"}}' | glm-plan-usage

# Verbose mode (for debugging)
glm-plan-usage --verbose < input.json

# Disable cache
glm-plan-usage --no-cache < input.json
```

## Update

```bash
npm update -g glm-plan-usage
```

## Uninstall

```bash
npm uninstall -g glm-plan-usage
rm -rf ~/.claude/glm-plan-usage
```

## Troubleshooting

### Command not found?

```bash
# Add npm global bin to PATH
export PATH="$(npm config get prefix)/bin:$PATH"
```

### Check installation

```bash
npm list -g glm-plan-usage
glm-plan-usage --version
```

### Reinstall

```bash
npm uninstall -g glm-plan-usage
npm install -g glm-plan-usage
```

## Supported Platforms

- ✅ Linux (x64, arm64)
- ✅ macOS (Intel, Apple Silicon)
- ✅ Windows (x64)

## More Information

- Full documentation: [README.md](README.md)
- Publishing guide: [NPM_PUBLISHING_GUIDE.md](NPM_PUBLISHING_GUIDE.md)
- Feature summary: [NPM_FEATURE_SUMMARY.md](NPM_FEATURE_SUMMARY.md)
