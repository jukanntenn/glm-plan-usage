# @jukanntenn/glm-plan-usage

GLM Plan Usage - StatusLine plugin for Claude Code

## Installation

```bash
npm install -g @jukanntenn/glm-plan-usage
```

For users experiencing network issues, use npm mirror for faster installation:

```bash
npm install -g @jukanntenn/glm-plan-usage --registry https://registry.npmmirror.com
```

## Features

- 📊 **Real-time Usage Tracking**: Display Token and MCP usage percentages
- 🗓️ **Weekly Quota Support**: Display weekly Token usage (new plan users only)
- 🎨 **Color-coded Warnings**: Green (0-50%), Yellow (51-80%), Red (81-100%)
- ⚡ **Smart Caching**: 5-minute cache to reduce API calls
- 🔍 **Auto Platform Detection**: Supports ZAI and ZHIPU platforms
- 🌍 **Cross-platform Support**: Works on Windows, macOS, and Linux

## Usage

Add to your Claude Code `settings.json`:

**Linux/macOS:**

```json
{
  "statusLine": {
    "type": "command",
    "command": "~/.claude/glm-plan-usage/glm-plan-usage",
    "padding": 0
  }
}
```

**Windows:**

```json
{
  "statusLine": {
    "type": "command",
    "command": "$HOME/.claude/glm-plan-usage/glm-plan-usage.exe",
    "padding": 0
  }
}
```

> **Note:** Older versions of Claude Code may require Windows-style paths, such as `%USERPROFILE%\.claude\glm-plan-usage\glm-plan-usage.exe`.

Restart Claude Code, the status bar will display:

```text
🪙 32% · ⏱ 14:30 | 🗓️ 24% | 🌐 20/100
   │  │    │        │       │     └─ MCP usage (used/total)
   │  │    │        │       └─ Segment separator
   │  │    │        └─ Weekly quota percentage (new plan users)
   │  │    └─ Token reset time (clock mode)
   │  └─ Internal separator
   └─ Token usage percentage
```

## Environment Variables

**Note:** These variables are typically already configured in your Claude Code `settings.json`. If not, you can set them manually:

**Linux/macOS:**

```bash
export ANTHROPIC_AUTH_TOKEN="your-token-here"
export ANTHROPIC_BASE_URL="https://open.bigmodel.cn/api/anthropic"
```

**Windows (Command Prompt):**

```cmd
set ANTHROPIC_AUTH_TOKEN=your-token-here
set ANTHROPIC_BASE_URL=https://open.bigmodel.cn/api/anthropic
```

**Windows (PowerShell):**

```powershell
$env:ANTHROPIC_AUTH_TOKEN="your-token-here"
$env:ANTHROPIC_BASE_URL="https://open.bigmodel.cn/api/anthropic"
```

## More Information

- GitHub: https://github.com/jukanntenn/glm-plan-usage
- Issues: https://github.com/jukanntenn/glm-plan-usage/issues
- License: MIT
