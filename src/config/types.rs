use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::OnceLock;

/// Input data from Claude Code (via stdin)
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct InputData {
    pub model: Option<ModelInfo>,
    pub workspace: Option<WorkspaceInfo>,
    pub transcript_path: Option<String>,
    #[serde(rename = "cost")]
    pub cost_info: Option<CostInfo>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    #[serde(rename = "display_name")]
    pub display_name: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct WorkspaceInfo {
    #[serde(rename = "current_dir")]
    pub current_dir: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct CostInfo {
    pub tokens: Option<f64>,
    pub cost: Option<f64>,
}

/// Display mode for icons and styling
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum DisplayMode {
    #[default]
    Auto,
    Emoji,
    Ascii,
}

/// Cached detection result for Auto mode
static DETECTED_MODE: OnceLock<DisplayMode> = OnceLock::new();

/// Detect terminal capabilities to determine best display mode
fn detect_display_mode() -> DisplayMode {
    // Windows without modern terminal
    if cfg!(windows) {
        let has_wt = std::env::var("WT_SESSION").is_ok();
        let has_vscode = std::env::var("TERM_PROGRAM").as_deref() == Ok("vscode");
        if !has_wt && !has_vscode {
            return DisplayMode::Ascii;
        }
    }
    // Known-bad Unix terminals
    if let Ok(term) = std::env::var("TERM") {
        if term == "dumb" || term == "linux" || term == "screen" {
            return DisplayMode::Ascii;
        }
    }
    // Non-UTF-8 locale on Unix
    if cfg!(unix) {
        let lang = std::env::var("LANG").unwrap_or_default();
        if !lang.contains("UTF-8") && !lang.contains("utf8") {
            return DisplayMode::Ascii;
        }
    }
    DisplayMode::Emoji
}

/// Plugin configuration
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    #[serde(default)]
    pub style: StyleConfig,
    #[serde(default)]
    pub segments: Vec<SegmentConfig>,
    #[serde(default)]
    pub api: ApiConfig,
    #[serde(default)]
    pub cache: CacheConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            style: StyleConfig::default(),
            segments: vec![
                SegmentConfig::token_usage(),
                SegmentConfig::weekly_usage(),
                SegmentConfig::mcp_usage(),
            ],
            api: ApiConfig::default(),
            cache: CacheConfig::default(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct StyleConfig {
    #[serde(default)]
    pub mode: DisplayMode,
    #[serde(default = "default_separator")]
    pub separator: String,
}

impl StyleConfig {
    /// Resolve Auto mode to concrete Emoji or Ascii at runtime (cached)
    pub fn resolved_mode(&self) -> DisplayMode {
        match self.mode {
            DisplayMode::Auto => *DETECTED_MODE.get_or_init(detect_display_mode),
            DisplayMode::Emoji => DisplayMode::Emoji,
            DisplayMode::Ascii => DisplayMode::Ascii,
        }
    }
}

impl Default for StyleConfig {
    fn default() -> Self {
        Self {
            mode: DisplayMode::default(),
            separator: default_separator(),
        }
    }
}

fn default_separator() -> String {
    " | ".to_string()
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SegmentConfig {
    pub id: String,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    #[serde(default)]
    pub icon: IconConfig,
    #[serde(default)]
    pub options: HashMap<String, serde_json::Value>,
}

impl SegmentConfig {
    pub fn token_usage() -> Self {
        Self {
            id: "token_usage".to_string(),
            enabled: true,
            icon: IconConfig::new("🪙", "$"),
            options: HashMap::new(),
        }
    }

    pub fn mcp_usage() -> Self {
        Self {
            id: "mcp_usage".to_string(),
            enabled: true,
            icon: IconConfig::new("🌐", "#"),
            options: HashMap::new(),
        }
    }

    pub fn weekly_usage() -> Self {
        Self {
            id: "weekly_usage".to_string(),
            enabled: true,
            icon: IconConfig::new("🗓️", "*"),
            options: HashMap::new(),
        }
    }
}

fn default_enabled() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IconConfig {
    #[serde(default)]
    pub emoji: String,
    #[serde(default)]
    pub ascii: String,
}

impl IconConfig {
    pub fn new(emoji: &str, ascii: &str) -> Self {
        Self {
            emoji: emoji.to_string(),
            ascii: ascii.to_string(),
        }
    }
}

impl Default for IconConfig {
    fn default() -> Self {
        Self::new("", "")
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ApiConfig {
    #[serde(default = "default_timeout")]
    pub timeout_ms: u64,
    #[serde(default = "default_retry")]
    pub retry_attempts: u32,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            timeout_ms: default_timeout(),
            retry_attempts: default_retry(),
        }
    }
}

fn default_timeout() -> u64 {
    5000
}

fn default_retry() -> u32 {
    2
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CacheConfig {
    #[serde(default = "default_cache_enabled")]
    pub enabled: bool,
    #[serde(default = "default_ttl")]
    pub ttl_seconds: u64,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            enabled: default_cache_enabled(),
            ttl_seconds: default_ttl(),
        }
    }
}

fn default_cache_enabled() -> bool {
    true
}

fn default_ttl() -> u64 {
    300
}
