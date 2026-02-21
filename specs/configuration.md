# Configuration

> Config file format, extension patterns, and design decisions.

---

## Config Location

```bash
~/.claude/glm-plan-usage/config.toml
```

Initialize with: `glm-plan-usage init`

---

## Config Structure

```toml
[style]
mode = "auto"               # Display mode: auto, emoji, or ascii
separator = " | "           # Separator between segments

[[segments]]
id = "token_usage"          # Segment identifier
enabled = true

[segments.icon]
emoji = "🪙"               # Icon for emoji mode
ascii = "$"                # Icon for ascii mode

[segments.options]
show_timer = true           # Show timer (clock or countdown)
timer_mode = "clock"        # "clock" (default) | "countdown"

[[segments]]
id = "weekly_usage"
enabled = true

[segments.icon]
emoji = "🗓️"
ascii = "*"

[segments.options]

[[segments]]
id = "mcp_usage"
enabled = true

[segments.icon]
emoji = "🌐"
ascii = "#"

[segments.options]

[api]
timeout_ms = 5000           # Request timeout (currently not wired to client)
retry_attempts = 2          # Retry count (currently not wired to client)

[cache]
enabled = true
ttl_seconds = 300
```

---

## Extension Pattern

All config changes require updating **three locations**:

1. **`src/config/types.rs`** — Add struct field with `#[serde(default)]`
2. **`impl Default for Config`** — Provide default value
3. **`src/config_template.toml`** — Add option to embedded template with docs

**Why `#[serde(default)]`**: Allows adding new fields without breaking existing user configs.

---

## Available Segments

| ID             | Description                           | Output (Emoji)     | Output (ASCII)   | Options                                        |
| -------------- | ------------------------------------- | ------------------ | ---------------- | ---------------------------------------------- |
| `token_usage`  | Token usage percentage with timer     | `🪙 32% · ⏱ 14:30` | `$ 32% · @ 14:30` | `show_timer`, `timer_mode` (default: clock)    |
| `weekly_usage` | Weekly token quota percentage         | `🗓️ 24%`          | `* 24%`          | None                                           |
| `mcp_usage`    | MCP server usage count                | `🌐 20/100`        | `# 20/100`       | None                                           |

---

## Style Modes

Config file uses `snake_case` values (`"auto"`, `"emoji"`, `"ascii"`). Rust enum is PascalCase (`Auto`, `Emoji`, `Ascii`) with `#[serde(rename_all = "snake_case")]`.

| Mode    | Description                                    | Example Output     |
| ------- | ---------------------------------------------- | ------------------ |
| `auto`  | Auto-detect terminal capabilities (default)    | Emoji or ASCII     |
| `emoji` | Emoji icons from config                        | `🪙 32% · ⌛️ 1:44` |
| `ascii` | ASCII characters from config                   | `$ 32% · ⌛️ 1:44`  |

Auto mode detects terminal capabilities: checks for Windows Terminal, VS Code terminal, UTF-8 locale, and known-bad terminals (dumb, linux, screen). Falls back to ASCII if detection fails.

---

## Dynamic Coloring

Colors automatically change based on usage percentage:

| Range   | Color  | ANSI Code |
| ------- | ------ | --------- |
| 0-50%   | Green  | 46        |
| 51-80%  | Yellow | 226       |
| 81-100% | Red    | 196       |
| No data | Gray   | 109       |

Colors use ANSI 256-color codes for broad terminal compatibility. Not user-configurable to avoid conflicts with dynamic behavior.

---

## Output Format

- Segments joined by `style.separator` (default: `|`)
- Within each segment, primary and secondary text separated by `·`
- Timer format: `⏱ HH:MM` (clock mode) or `⌛️ H:MM` (countdown mode)

## Timer Modes

The token_usage segment supports two timer display modes:

| Mode       | Description                         | Emoji Icon | ASCII Icon |
| ---------- | ----------------------------------- | ---------- | ---------- |
| `clock`    | Display reset time as local HH:MM   | ⏱         | @          |
| `countdown`| Display time remaining until reset  | ⌛️        | !          |

Timer icons are determined by `timer_mode` + resolved `DisplayMode`, not by segment `IconConfig`. This keeps configuration simple.

---

## Default Values

| Setting              | Default                                      |
| -------------------- | -------------------------------------------- |
| `style.mode`         | `Auto`                                       |
| `style.separator`    | `" | "`                                      |
| `api.timeout_ms`     | `5000`                                       |
| `api.retry_attempts` | `2`                                          |
| `cache.enabled`      | `true`                                       |
| `cache.ttl_seconds`  | `300`                                        |
| `segments`           | `token_usage`, `weekly_usage`, `mcp_usage`   |

---

## Validation Rules

Run `glm-plan-usage check` to validate:

- At least one segment must be configured
- Segment IDs must be unique
- Segment IDs must be valid: `token_usage`, `weekly_usage`, `mcp_usage`

---

## CLI Overrides

| Flag         | Effect                         |
| ------------ | ------------------------------ |
| `--no-cache` | Disable cache for this run     |
| `--verbose`  | Print error messages to stderr |
