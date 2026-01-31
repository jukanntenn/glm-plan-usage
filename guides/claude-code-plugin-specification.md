# Claude Code Command Plugin Specification

**Version:** 1.0
**Status:** Stable
**Based on:** CCometixLine Reference Implementation

---

## Table of Contents

1. [Overview](#overview)
2. [Plugin Architecture](#plugin-architecture)
3. [Data Flow](#data-flow)
4. [Input/Output Specification](#inputoutput-specification)
5. [Configuration System](#configuration-system)
6. [Implementation Guide](#implementation-guide)
7. [Installation Methods](#installation-methods)
8. [Best Practices](#best-practices)
9. [Reference Implementation](#reference-implementation)

---

## Overview

Claude Code supports **command-based plugins** that extend its UI functionality through external programs. These plugins receive context data from Claude Code, process it, and return formatted output for display.

### Supported Hook Types

| Hook Type | Purpose | Output Format |
|-----------|---------|---------------|
| `statusLine` | Display custom status bar content | Plain text with ANSI colors |
| (Future) | Additional hooks may be added | TBD |

### Key Characteristics

- **Synchronous Execution**: Claude Code waits for plugin completion before rendering
- **JSON Communication**: All data passed via stdin as JSON
- **Text Output**: Plugins return formatted text via stdout
- **Low Latency**: Should complete within 50-100ms for responsive UI
- **Stateless**: Each execution is independent (no persistent connection)

---

## Plugin Architecture

### System Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                         Claude Code                              │
│                                                                   │
│  ┌─────────────┐    ┌──────────────┐    ┌─────────────────┐    │
│  │   Session   │───>│  Hook System │───>│ Process Spawner │    │
│  │   Context   │    │              │    │                 │    │
│  └─────────────┘    └──────────────┘    └────────┬────────┘    │
│                                                   │             │
│                                                   ▼             │
├───────────────────────────────────────────────────────────────────┤
│                              IPC Boundary                         │
│                                                                   │
│  JSON Input ────────────────────────────────────> stdin          │
│  Text Output <──────────────────────────────────── stdout         │
├───────────────────────────────────────────────────────────────────┤
│                                                                   │
│  ┌──────────────────────────────────────────────────────────┐    │
│  │                     Plugin Process                        │    │
│  │                                                            │    │
│  │  ┌──────────┐   ┌─────────────┐   ┌──────────────────┐   │    │
│  │  │ Parse    │──>│ Collect     │──>│ Render/Format    │   │    │
│  │  │ JSON     │   │ Data        │   │ Output           │   │    │
│  │  └──────────┘   └─────────────┘   └────────┬─────────┘   │    │
│  │                                       │               │    │
│  │  ┌─────────────────────────────────────┴─────────────┐   │    │
│  │  │              Segments/Modules                     │   │    │
│  │  │  • Model      • Git       • Directory            │   │    │
│  │  │  • Context    • Usage     • Custom Data          │   │    │
│  │  └──────────────────────────────────────────────────┘   │    │
│  └──────────────────────────────────────────────────────────┘    │
│                                                                   │
└───────────────────────────────────────────────────────────────────┘
```

### Component Responsibilities

| Component | Responsibility |
|-----------|----------------|
| **Claude Code Hook System** | Detect plugin commands, prepare context data, spawn process, capture output |
| **Plugin Entry Point** (`main.rs`) | Parse CLI args, load config, read stdin, dispatch to renderer |
| **Config System** | Load user preferences, themes, segment configuration |
| **Segment Collectors** | Extract data from input (git info, file parsing, external commands) |
| **Renderer** | Apply colors, separators, formatting, output final text |

---

## Data Flow

### Execution Sequence

```
1. Claude Code UI Refresh Event
   │
   ├─> Gather session context (model, workspace, transcript path)
   │
   ├─> Serialize to JSON
   │
   ├─> Spawn plugin process
   │   │
   │   ├─> Write JSON to plugin stdin
   │   │
   │   ├─> Wait for completion (timeout: ~1-5 seconds)
   │   │
   │   └─> Read plugin stdout
   │
   ├─> Decode ANSI escape sequences
   │
   └─> Render in Claude Code UI (status bar)
```

### Plugin Processing Steps

```rust
// 1. Parse command-line arguments
let cli = Cli::parse();

// 2. Load configuration
let config = Config::load()?;

// 3. Read JSON input from Claude Code
let stdin = io::stdin();
let input: InputData = serde_json::from_reader(stdin.lock())?;

// 4. Collect data from all enabled segments
let segments_data = collect_all_segments(&config, &input);

// 5. Generate final output
let generator = StatusLineGenerator::new(config);
let output = generator.generate(segments_data);

// 6. Return to Claude Code
println!("{}", output);
```

---

## Input/Output Specification

### Input Format (JSON from Claude Code)

Claude Code sends session context via stdin. Plugin must handle optional fields gracefully.

```typescript
interface InputData {
  // Model information
  model: {
    id: string;           // e.g., "claude-sonnet-4-5-20250929"
    display_name: string; // e.g., "Sonnet 4.5"
  };

  // Workspace information
  workspace: {
    current_dir: string;  // Absolute path to working directory
  };

  // Path to conversation transcript JSON file
  transcript_path: string;

  // Optional: API usage data
  cost?: {
    total_cost_usd?: number;
    total_duration_ms?: number;
    total_api_duration_ms?: number;
    total_lines_added?: number;
    total_lines_removed?: number;
  };

  // Optional: Output style configuration
  output_style?: {
    name: string;
  };
}
```

### Transcript File Format

The transcript file contains conversation history with token usage data:

```typescript
interface TranscriptEntry {
  type?: string;          // "user" | "assistant" | "tool_use"
  message?: {
    usage?: {
      // Anthropic format
      input_tokens?: number;
      output_tokens?: number;
      cache_creation_input_tokens?: number;
      cache_read_input_tokens?: number;

      // OpenAI format (alternative fields)
      prompt_tokens?: number;
      completion_tokens?: number;
      total_tokens?: number;
    };
  };
  leafUuid?: string;
  uuid?: string;
  parentUuid?: string;
  summary?: string;
}
```

### Output Format (Text to Claude Code)

Plugin outputs plain text with ANSI color codes:

```
[ANSI_COLOR_BOLD][ANSI_FG_GREEN]Sonnet 4.5[RESET] [POWERLINE_ARROW] [ANSI_BOLD][ANSI_FG_BLUE]CCometixLine[RESET] [POWERLINE_ARROW] [ANSI_BOLD][ANSI_FG_YELLOW]main* ✗[RESET] [POWERLINE_ARROW] [ANSI_BOLD][ANSI_FG_CYAN]42% · 84k tokens[RESET]
```

#### ANSI Escape Sequences

| Code | Purpose |
|------|---------|
| `\x1b[1m` | Bold text |
| `\x1b[22m` | Reset bold |
| `\x1b[38;5;Nm` | Set foreground color (N = 0-255) |
| `\x1b[48;5;Nm` | Set background color (N = 0-255) |
| `\x1b[38;2;R;G;Bm` | RGB foreground (R,G,B = 0-255) |
| `\x1b[0m` | Reset all attributes |

#### Example Colored Output

```rust
// Bold green text
println!("\x1b[1m\x1b[38;5;82m{}\x1b[0m", "Sonnet 4.5");

// Using RGB color
println!("\x1b[1m\x1b[38;2;86;182;194m{}\x1b[0m", "CCometixLine");
```

---

## Configuration System

### Configuration File Location

```
~/.claude/<plugin-name>/config.toml
```

### TOML Structure

```toml
# Style configuration
[style]
mode = "powerline"  # "plain" | "nerd_font" | "powerline"
separator = "  "

# Theme selection
theme = "nord"  # Built-in theme name

# Segment configuration
[[segments]]
id = "directory"
enabled = true

[segments.icon]
plain = ""
nerd_font = ""

[segments.colors]
text = { c256 = 109 }  # 256-color palette
icon = { c256 = 109 }
background = null

[segments.styles]
text_bold = true

[[segments]]
id = "git"
enabled = true

[segments.options]
show_sha = false
show_remote_tracking = true

[[segments]]
id = "model"
enabled = true

[[segments]]
id = "context_window"
enabled = true

[segments.options]
show_percentage = true
show_tokens = true
```

### Color Specification

```toml
# 16 basic colors
color = { c16 = 2 }  # 0-15

# 256-color palette
color = { c256 = 109 }  # 0-255

# RGB color
color = { r = 86, g = 182, b = 194 }

# Or use named colors (resolved by plugin)
color = "green"
```

### Theme Presets

Themes provide pre-configured color schemes and segment arrangements:

| Theme | Style | Colors |
|-------|-------|--------|
| `nord` | Powerline | Nord-inspired cool palette |
| `gruvbox` | Plain | Warm, retro colors |
| `dracula` | Nerd Font | Dark, high-contrast |
| `minimal` | Plain | Minimal icons, clean look |
| `cometix` | Powerline | Default CCometixLine theme |

---

## Implementation Guide

### Project Structure

```
plugin-project/
├── Cargo.toml              # Rust project manifest
├── README.md               # Documentation
├── src/
│   ├── main.rs             # Entry point
│   ├── cli.rs              # Command-line argument parsing
│   ├── config/
│   │   ├── mod.rs          # Config module
│   │   ├── types.rs        # Data structures
│   │   └── loader.rs       # Config file I/O
│   ├── core/
│   │   ├── mod.rs          # Core module
│   │   ├── statusline.rs   # Renderer
│   │   └── segments/
│   │       ├── mod.rs      # Segment trait
│   │       ├── model.rs
│   │       ├── directory.rs
│   │       ├── git.rs
│   │       └── context_window.rs
│   └── ui/
│       ├── mod.rs          # UI module
│       ├── themes.rs       # Theme presets
│       └── configurator.rs # TUI (optional)
└── npm/
    └── main/
        ├── package.json    # npm package manifest
        └── bin/
            └── ccline.js   # Node.js wrapper
```

### Core Segment Trait

```rust
use serde::Deserialize;
use std::collections::HashMap;

// Input data from Claude Code
#[derive(Deserialize)]
pub struct InputData {
    pub model: Model,
    pub workspace: Workspace,
    pub transcript_path: String,
    pub cost: Option<Cost>,
    pub output_style: Option<OutputStyle>,
}

// Segment output data
pub struct SegmentData {
    pub primary: String,          // Main display text
    pub secondary: String,        // Additional info (tooltip, etc.)
    pub metadata: HashMap<String, String>,  // Raw data for debugging
}

// Segment trait - all segments must implement
pub trait Segment: Send + Sync {
    fn collect(&self, input: &InputData) -> Option<SegmentData>;
}
```

### Example Segment Implementation

```rust
pub struct DirectorySegment {
    pub show_full_path: bool,
}

impl Segment for DirectorySegment {
    fn collect(&self, input: &InputData) -> Option<SegmentData> {
        let name = if self.show_full_path {
            input.workspace.current_dir.clone()
        } else {
            // Extract last component of path
            input.workspace.current_dir
                .split('/')
                .next_back()
                .unwrap_or(&input.workspace.current_dir)
                .to_string()
        };

        Some(SegmentData {
            primary: name,
            secondary: input.workspace.current_dir.clone(),
            metadata: {
                let mut map = HashMap::new();
                map.insert("full_path".to_string(), input.workspace.current_dir.clone());
                map
            },
        })
    }
}
```

### Rendering Pipeline

```rust
pub struct StatusLineGenerator {
    config: Config,
}

impl StatusLineGenerator {
    pub fn generate(&self, segments: Vec<(SegmentConfig, SegmentData)>) -> String {
        let mut output = String::new();

        for (i, (config, data)) in segments.iter().enumerate() {
            // Apply colors
            let colored = self.colorize(&data.primary, &config.colors);

            output.push_str(&colored);

            // Add separator between segments
            if i < segments.len() - 1 {
                output.push_str(&self.get_separator(config));
            }
        }

        output
    }

    fn colorize(&self, text: &str, color: &ColorConfig) -> String {
        let mut result = String::new();

        // Apply bold
        if config.styles.text_bold {
            result.push_str("\x1b[1m");
        }

        // Apply foreground color
        if let Some(fg) = &color.text {
            match fg {
                AnsiColor::Color256 { c256 } => {
                    result.push_str(&format!("\x1b[38;5;{}m", c256));
                }
                AnsiColor::Rgb { r, g, b } => {
                    result.push_str(&format!("\x1b[38;2;{};{};{}m", r, g, b));
                }
                _ => {}
            }
        }

        result.push_str(text);
        result.push_str("\x1b[0m");  // Reset

        result
    }

    fn get_separator(&self, config: &SegmentConfig) -> &str {
        match self.config.style.mode {
            StyleMode::Powerline => " ",
            StyleMode::NerdFont => "  ",
            StyleMode::Plain => " | ",
        }
    }
}
```

---

## Installation Methods

### Method 1: npm Package (Recommended)

**Package.json structure:**

```json
{
  "name": "@your-org/your-plugin",
  "version": "1.0.0",
  "description": "Claude Code plugin description",
  "bin": {
    "your-plugin": "./bin/your-plugin.js"
  },
  "files": [
    "bin/",
    "dist/"
  ],
  "os": [
    "darwin",
    "linux",
    "win32"
  ],
  "cpu": [
    "x64",
    "arm64"
  ]
}
```

**Node.js wrapper (`bin/your-plugin.js`):**

```javascript
#!/usr/bin/env node

const path = require('path');
const { execFileSync } = require('child_process');

// Detect platform and binary name
const platform = process.platform === 'win32' ? 'windows' : process.platform;
const arch = process.arch;
const binaryName = platform === 'windows' ? 'your-plugin.exe' : 'your-plugin';

// Resolve binary path
const binaryPath = path.join(__dirname, '..', 'dist', binaryName);

try {
  // Forward all arguments and stdin/stdout to binary
  execFileSync(binaryPath, process.argv.slice(2), {
    stdio: 'inherit'
  });
} catch (error) {
  process.exit(error.status || 1);
}
```

**User installation:**

```bash
npm install -g @your-org/your-plugin
```

**Claude Code configuration:**

```json
{
  "statusLine": {
    "type": "command",
    "command": "your-plugin",
    "padding": 0
  }
}
```

### Method 2: Manual Binary Installation

```bash
# Create directory
mkdir -p ~/.claude/your-plugin

# Download binary
wget https://github.com/your-org/your-plugin/releases/latest/download/your-plugin-linux-x64

# Install
chmod +x your-plugin-linux-x64
mv your-plugin-linux-x64 ~/.claude/your-plugin/your-plugin
```

**Claude Code configuration:**

```json
{
  "statusLine": {
    "type": "command",
    "command": "~/.claude/your-plugin/your-plugin",
    "padding": 0
  }
}
```

### Method 3: Build from Source

```bash
git clone https://github.com/your-org/your-plugin.git
cd your-plugin
cargo build --release

# Install
mkdir -p ~/.claude/your-plugin
cp target/release/your-plugin ~/.claude/your-plugin/
chmod +x ~/.claude/your-plugin/your-plugin
```

---

## Best Practices

### Performance

| Practice | Recommendation | Rationale |
|----------|----------------|-----------|
| Startup time | < 10ms | UI responsiveness |
| Total execution | < 50ms | Prevent UI lag |
| Git operations | Use `git rev-parse` | Faster than porcelain commands |
| File parsing | Stream large files | Avoid loading entire transcript |
| Caching | Cache expensive operations | Improve repeat performance |

### Error Handling

```rust
// Gracefully handle missing data
fn collect(&self, input: &InputData) -> Option<SegmentData> {
    // Return None if data unavailable
    // Claude Code will simply not display this segment

    let git_info = self.get_git_info(&input.workspace.current_dir)?;

    Some(SegmentData { /* ... */ })
}

// Don't fail entire plugin for one segment error
let segments_data: Vec<_> = segments
    .iter()
    .filter_map(|seg| seg.collect(input).ok())
    .collect();
```

### Cross-Platform Compatibility

```rust
// Handle path separators
fn extract_dir_name(path: &str) -> &str {
    // Try Unix separator, then Windows
    path.split('/').next_back()
        .or_else(|| path.split('\\').next_back())
        .unwrap_or(path)
}

// Handle line endings
fn normalize_output(text: String) -> String {
    text.replace("\r\n", "\n").replace('\r', "\n")
}
```

### Configuration Design

```rust
// Provide sensible defaults
impl Default for Config {
    fn default() -> Self {
        Self {
            style: StyleConfig {
                mode: StyleMode::Plain,
                separator: " | ".to_string(),
            },
            segments: vec![
                SegmentConfig::default_directory(),
                SegmentConfig::default_git(),
                SegmentConfig::default_model(),
            ],
            theme: "default".to_string(),
        }
    }
}

// Initialize on first run
if let Err(_) = Config::load() {
    Config::init()?;  // Create default config
}
```

### User Experience

1. **Interactive Configuration**: Provide `--config` flag for TUI setup
2. **Theme Selection**: Include multiple built-in themes
3. **Documentation**: Clear README with examples
4. **Auto-Discovery**: Use npm PATH for easy installation
5. **Update Notifications**: Optional update check functionality

---

## Reference Implementation

### CCometixLine Analysis

**Repository:** https://github.com/Haleclipse/CCometixLine

**Key Files:**

| File | Lines | Purpose |
|------|-------|---------|
| `src/main.rs` | ~145 | Entry point, CLI parsing, plugin orchestration |
| `src/config/types.rs` | ~417 | Data structures, input format, config schema |
| `src/core/statusline.rs` | ~250 | Rendering pipeline, color application |
| `src/core/segments/git.rs` | ~180 | Git status detection |
| `src/core/segments/context_window.rs` | ~120 | Token usage calculation |

**Architecture Summary:**

1. **Segment-based modular design** - Each component independently collects data
2. **TOML configuration** - User-editable with TUI fallback
3. **Theme system** - Presets with override capability
4. **Cross-platform** - Handles Unix/Windows paths, static/dynamic binaries
5. **npm distribution** - JavaScript wrapper for Rust binary

**Performance Characteristics:**

- **Binary size**: ~2-3 MB (stripped)
- **Memory usage**: ~1-2 MB
- **Execution time**: ~10-30ms typical
- **Startup overhead**: < 5ms

---

## Appendix

### Complete Example: Minimal Plugin

```rust
use serde::Deserialize;
use std::io::{self, Read};

#[derive(Deserialize)]
struct InputData {
    model: Model,
    workspace: Workspace,
}

#[derive(Deserialize)]
struct Model {
    id: String,
    display_name: String,
}

#[derive(Deserialize)]
struct Workspace {
    current_dir: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Read JSON from stdin
    let mut input_str = String::new();
    io::stdin().read_to_string(&mut input_str)?;
    let input: InputData = serde_json::from_str(&input_str)?;

    // Extract directory name
    let dir_name = input.workspace.current_dir
        .split('/')
        .next_back()
        .unwrap_or("Unknown");

    // Output statusline
    println!("{} | {}", input.model.display_name, dir_name);

    Ok(())
}
```

**Cargo.toml:**

```toml
[package]
name = "minimal-cc-plugin"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

### ANSI Color Reference

**Basic Colors (16-color):**

| Code | Color | Code | Color |
|------|-------|------|-------|
| 0 | Black | 8 | Bright Black |
| 1 | Red | 9 | Bright Red |
| 2 | Green | 10 | Bright Green |
| 3 | Yellow | 11 | Bright Yellow |
| 4 | Blue | 12 | Bright Blue |
| 5 | Magenta | 13 | Bright Magenta |
| 6 | Cyan | 14 | Bright Cyan |
| 7 | White | 15 | Bright White |

**256-color Palette Ranges:**

| Range | Color Space |
|-------|-------------|
| 0-7 | Standard colors |
| 8-15 | High intensity colors |
| 16-231 | 6×6×6 color cube |
| 232-255 | Grayscale ramp |

### Claude Code Settings Location

| Platform | Settings Path |
|----------|---------------|
| Linux | `~/.config/claude-code/settings.json` |
| macOS | `~/Library/Application Support/Claude Code/settings.json` |
| Windows | `%APPDATA%\claude-code\settings.json` |

---

## Changelog

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2025-01-30 | Initial specification based on CCometixLine v1.1.0 |

---

## License

This specification is provided as reference documentation for building Claude Code plugins. It may be used freely as a guide for implementation.

**Reference Implementation:** [CCometixLine](https://github.com/Haleclipse/CCometixLine) by Haleclipse - Licensed under MIT License
