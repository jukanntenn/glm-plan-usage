# GLM Plan Usage

GLM usage status plugin for Claude Code.

## Installation

```bash
npm install -g glm-plan-usage
```

## Setup for Claude Code

Add the following to your `~/.config/claude-code/settings.json`:

```json
{
  "statusLine": {
    "type": "command",
    "command": "glm-plan-usage"
  }
}
```

## Configuration

Create a configuration file at `~/.claude/glm-plan-usage/config.toml`:

```toml
# GLM API Configuration
[api]
# Base URL for GLM API (auto-detected from environment)
# base_url = "https://open.bigmodel.cn/api/anthropic"

# Display Configuration
[display]
# Show remaining token count
show_remaining = true
# Show percentage used
show_percentage = true
# Show color-coded warnings
colorize = true

# Cache Configuration
[cache]
# Enable caching of API responses
enabled = true
# Cache TTL in seconds
ttl_seconds = 300
```

## Environment Variables

- `ANTHROPIC_AUTH_TOKEN`: Your GLM API token
- `ANTHROPIC_BASE_URL`: Base URL for GLM API (e.g., `https://open.bigmodel.cn/api/anthropic`)

## CLI Usage

```bash
# Initialize default configuration
glm-plan-usage --init

# Show verbose error messages
glm-plan-usage --verbose

# Disable cache for this run
glm-plan-usage --no-cache
```

## Platforms Supported

- macOS (x64, ARM64/Apple Silicon)
- Linux (x64, ARM64)
- Windows (x64)

## License

MIT
