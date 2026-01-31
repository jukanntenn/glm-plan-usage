# Usage Examples

## Basic Usage

Once installed and configured, the plugin will automatically display usage in the Claude Code status bar:

```
T:42% M:15%
```

Where:
- `T:42%` = Token usage (5-hour rolling window)
- `M:15%` = MCP usage (30-day rolling window)

## Color Coding

The plugin uses color-coded warnings:

**Green (0-79%)**: Normal usage
```
T:42% M:15%  ← Green text
```

**Yellow (80-94%)**: High usage warning
```
T:87% M:92%  ← Yellow text
```

**Red (95-100%)**: Critical usage
```
T:98% M:97%  ← Red text
```

## Manual Testing

### Test with Real API Credentials

```bash
# Set environment variables
export ANTHROPIC_AUTH_TOKEN="your-actual-token"
export ANTHROPIC_BASE_URL="https://open.bigmodel.cn/api/anthropic"

# Test with sample input
echo '{"model":{"id":"claude-sonnet-4","display_name":"Sonnet 4"},"workspace":{"current_dir":"/home/alice/workspace"},"transcript_path":"/tmp/transcript.json"}' | \
~/.claude/glm-plan-usage/glm-plan-usage
```

Expected output (with ANSI colors):
```
T:42% M:15%
```

### Test Without Cache

```bash
echo '{"model":{"id":"claude-sonnet-4"}}' | \
~/.claude/glm-plan-usage/glm-plan-usage --no-cache
```

### Test with Verbose Output

```bash
echo '{"model":{"id":"claude-sonnet-4"}}' | \
~/.claude/glm-plan-usage/glm-plan-usage --verbose
```

This will print debug information to stderr.

## Configuration Examples

### Change Colors

Edit `~/.claude/glm-plan-usage/config.toml`:

```toml
[segments.colors.text]
c256 = 226  # Yellow instead of green

# Or use RGB:
# r = 34
# g = 197
# b = 94
```

### Disable Bold Text

```toml
[segments.styles]
text_bold = false
```

### Adjust Cache Duration

```toml
[cache]
enabled = true
ttl_seconds = 600  # 10 minutes instead of 5
```

### Increase Timeout

```toml
[api]
timeout_ms = 10000  # 10 seconds instead of 5
retry_attempts = 3
```

### Change Separator

```toml
[style]
separator = " • "
```

Output: `T:42% • M:15%`

## Integration with Claude Code

### Basic Configuration

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

### Custom Configuration Path

If you installed the binary elsewhere:

```json
{
  "statusLine": {
    "type": "command",
    "command": "/usr/local/bin/glm-plan-usage",
    "padding": 0
  }
}
```

## Troubleshooting

### No Output in Status Bar

1. Check environment variables are set:
```bash
echo $ANTHROPIC_AUTH_TOKEN
echo $ANTHROPIC_BASE_URL
```

2. Test manually:
```bash
echo '{"model":{"id":"test"}}' | ~/.claude/glm-plan-usage/glm-plan-usage --verbose
```

3. Check Claude Code logs for errors

### API Timeout

Increase timeout in config:
```toml
[api]
timeout_ms = 10000
```

### Colors Not Showing

Ensure your terminal supports ANSI colors. Test with:
```bash
echo -e "\x1b[38;5;109mGreen text\x1b[0m"
```

### Cache Issues

Disable cache temporarily:
```bash
echo '{"model":{"id":"test"}}' | ~/.claude/glm-plan-usage/glm-plan-usage --no-cache
```

## Performance

- **First call**: ~500ms (3 sequential API calls)
- **Cached call**: <20ms
- **Binary size**: 3.1MB
- **Memory usage**: ~2MB

## Platform-Specific Notes

### ZHIPU Platform

```bash
export ANTHROPIC_BASE_URL="https://open.bigmodel.cn/api/anthropic"
```

### ZAI Platform

```bash
export ANTHROPIC_BASE_URL="https://api.z.ai/api/paas/v4/"
```

The plugin auto-detects the platform from the base URL.
