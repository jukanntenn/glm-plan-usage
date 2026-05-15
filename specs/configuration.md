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
timeout_ms = 5000           # Request timeout in milliseconds
retry_attempts = 2          # Retry count

[multiplier]
premium_models = ["glm-5", "glm-5.1", "glm-5-turbo"]
peak_start = "14:00"        # Peak hours start (UTC+8)
peak_end = "18:00"          # Peak hours end (UTC+8)
peak = 3.0                  # Peak hours consumption rate
off_peak = 2.0              # Off-peak hours consumption rate

[multiplier.promo]
off_peak = 1.0              # Promotional off-peak rate
expires = "2026-06-30"      # Promo expiry date (inclusive)

[cache]
enabled = true
ttl_seconds = 300
```

---

## Extension Pattern

All config changes require updating **four locations**:

1. **`src/config/types.rs`** — Add struct field with `#[serde(default)]`
2. **`impl Default for Config`** — Provide default value
3. **`src/config_template.toml`** — Add option to embedded template with docs
4. **`src/core/segments/mod.rs`** — If SegmentData structure changes (e.g., new fields)

**Why `#[serde(default)]`**: Allows adding new fields without breaking existing user configs.

---

## Available Segments

| ID             | Description                           | Output (Emoji)     | Output (ASCII)   | Options                                        |
| -------------- | ------------------------------------- | ------------------ | ---------------- | ---------------------------------------------- |
| `token_usage`  | Token usage percentage with timer     | `🪙 32% · 3x · ⏱ 14:30` | `$ 32% · 3x · @ 14:30` | `show_timer`, `timer_mode` (default: clock), `show_multiplier` (default: true) |
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

Colors are applied per segment part:

| Part                    | Color Rule                                  | ANSI Code |
| ----------------------- | ------------------------------------------- | --------- |
| Icon + Primary (%)      | Dynamic based on usage percentage           | see below |
| Internal separator (·)  | White                                       | 37        |
| Secondary (timer/clock) | Gray                                        | 109       |
| Multiplier (e.g., 3x)   | Red                                         | 196       |

Primary color changes based on usage percentage:

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
- Within each segment: primary [· multiplier] [· secondary]
- Multiplier (e.g., `3x`) shown only when > 1x and `show_multiplier` is true
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
| `multiplier.premium_models` | `["glm-5", "glm-5.1", "glm-5-turbo"]` |
| `multiplier.peak_start`     | `"14:00"`                              |
| `multiplier.peak_end`       | `"18:00"`                              |
| `multiplier.peak`           | `3.0`                                  |
| `multiplier.off_peak`       | `2.0`                                  |
| `multiplier.promo.off_peak` | `1.0`                                  |
| `multiplier.promo.expires`  | `"2026-06-30"`                         |
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
