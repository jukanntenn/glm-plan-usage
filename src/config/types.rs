//! Configuration types for the GLM plan usage plugin.
//!
//! This module defines all configuration structures used by the plugin,
//! including input data from Claude Code, display settings, and segment configuration.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::OnceLock;

pub const DEFAULT_SEPARATOR: &str = " | ";

/// Input data received from Claude Code via stdin.
#[derive(Debug, Deserialize)]
pub struct InputData {
    #[serde(default)]
    #[allow(unused, reason = "deserialized from stdin, accessed by segments")]
    pub model: Option<ModelInfo>,
    #[serde(default)]
    #[allow(unused, reason = "deserialized from stdin, accessed by segments")]
    pub workspace: Option<WorkspaceInfo>,
    #[serde(default)]
    #[allow(unused, reason = "deserialized from stdin, accessed by segments")]
    pub transcript_path: Option<String>,
    #[serde(rename = "cost", default)]
    #[allow(unused, reason = "deserialized from stdin, accessed by segments")]
    pub cost_info: Option<CostInfo>,
}

/// Information about the current AI model.
#[derive(Debug, Deserialize)]
pub struct ModelInfo {
    #[serde(default)]
    #[allow(unused, reason = "deserialized from stdin, accessed by segments")]
    pub id: String,
    #[serde(rename = "display_name", default)]
    #[allow(unused, reason = "deserialized from stdin, accessed by segments")]
    pub display_name: Option<String>,
}

/// Information about the current workspace.
#[derive(Debug, Deserialize)]
pub struct WorkspaceInfo {
    #[serde(rename = "current_dir", default)]
    #[allow(unused, reason = "deserialized from stdin, accessed by segments")]
    pub current_dir: Option<String>,
}

/// Cost and token usage information.
#[derive(Debug, Deserialize)]
pub struct CostInfo {
    #[serde(default)]
    #[allow(unused, reason = "deserialized from stdin, accessed by segments")]
    pub tokens: Option<f64>,
    #[serde(default)]
    #[allow(unused, reason = "deserialized from stdin, accessed by segments")]
    pub cost: Option<f64>,
}

/// Display mode for icons and styling.
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

/// Detects terminal capabilities to determine the best display mode.
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

/// Global configuration for the GLM plan usage plugin.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    /// Visual styling configuration.
    #[serde(default)]
    pub style: StyleConfig,
    /// Segment configurations.
    #[serde(default)]
    pub segments: Vec<SegmentConfig>,
    /// API connection settings.
    #[serde(default)]
    pub api: ApiConfig,
    /// Cache behavior settings.
    #[serde(default)]
    pub cache: CacheConfig,
    /// Multiplier calculation settings.
    #[serde(default)]
    pub multiplier: MultiplierConfig,
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
            multiplier: MultiplierConfig::default(),
        }
    }
}

impl Config {
    /// Merge user-defined segments with defaults, preserving user order.
    #[must_use]
    pub fn merge_default_segments(mut self) -> Self {
        let defaults = Config::default().segments;
        let user_ids: Vec<_> = self.segments.iter().map(|s| s.id.clone()).collect();
        for seg in defaults {
            if !user_ids.contains(&seg.id) {
                self.segments.push(seg);
            }
        }
        self
    }
}

/// Visual styling configuration for the status line.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct StyleConfig {
    /// Display mode (Auto, Emoji, or Ascii).
    #[serde(default)]
    pub mode: DisplayMode,
    /// Separator string between segments.
    #[serde(default = "default_separator")]
    pub separator: String,
}

impl StyleConfig {
    /// Resolve the display mode, detecting terminal capabilities if set to Auto.
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
    DEFAULT_SEPARATOR.to_string()
}

/// Configuration for a single status line segment.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SegmentConfig {
    /// Segment identifier (e.g., `token_usage`, `weekly_usage`, `mcp_usage`).
    #[serde(default)]
    pub id: String,
    /// Whether the segment is enabled.
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    /// Icon configuration (emoji and ASCII variants).
    #[serde(default)]
    pub icon: IconConfig,
    /// Segment-specific options.
    #[serde(default)]
    pub options: HashMap<String, serde_json::Value>,
}

impl SegmentConfig {
    /// Create the default token usage segment configuration.
    #[must_use]
    pub fn token_usage() -> Self {
        Self {
            id: "token_usage".to_string(),
            enabled: true,
            icon: IconConfig::new("🪙", "$"),
            options: HashMap::new(),
        }
    }

    /// Create the default MCP usage segment configuration.
    #[must_use]
    pub fn mcp_usage() -> Self {
        Self {
            id: "mcp_usage".to_string(),
            enabled: true,
            icon: IconConfig::new("🌐", "#"),
            options: HashMap::new(),
        }
    }

    /// Create the default weekly usage segment configuration.
    #[must_use]
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

/// Icon configuration with both emoji and ASCII variants.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IconConfig {
    /// Emoji icon for modern terminals (e.g., "🪙").
    #[serde(default)]
    pub emoji: String,
    /// ASCII icon for legacy terminals (e.g., "$").
    #[serde(default)]
    pub ascii: String,
}

impl IconConfig {
    /// Create a new icon config with the given emoji and ASCII representations.
    #[must_use]
    pub fn new(emoji: &str, ascii: &str) -> Self {
        Self {
            emoji: emoji.to_string(),
            ascii: ascii.to_string(),
        }
    }
}

/// API connection settings.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ApiConfig {
    /// Request timeout in milliseconds.
    #[serde(default = "default_timeout")]
    pub timeout_ms: u64,
    /// Number of retry attempts on failure.
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

/// Cache behavior configuration.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CacheConfig {
    /// Enable or disable caching.
    #[serde(default = "default_cache_enabled")]
    pub enabled: bool,
    /// Time-to-live for cached data in seconds.
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

/// Multiplier calculation settings.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MultiplierConfig {
    /// Model ID substrings that identify premium models.
    #[serde(default = "default_premium_models")]
    pub premium_models: Vec<String>,
    /// Peak hours start time in UTC+8 (HH:MM format).
    #[serde(default = "default_peak_start")]
    pub peak_start: String,
    /// Peak hours end time in UTC+8 (HH:MM format).
    #[serde(default = "default_peak_end")]
    pub peak_end: String,
    /// Multiplier value during peak hours.
    #[serde(default = "default_peak")]
    pub peak: f64,
    /// Multiplier value during off-peak hours.
    #[serde(default = "default_off_peak")]
    pub off_peak: f64,
    /// Promotional pricing configuration.
    #[serde(default)]
    pub promo: PromoConfig,
}

impl Default for MultiplierConfig {
    fn default() -> Self {
        Self {
            premium_models: default_premium_models(),
            peak_start: default_peak_start(),
            peak_end: default_peak_end(),
            peak: default_peak(),
            off_peak: default_off_peak(),
            promo: PromoConfig::default(),
        }
    }
}

fn default_premium_models() -> Vec<String> {
    vec![
        "glm-5".to_string(),
        "glm-5.1".to_string(),
        "glm-5.2".to_string(),
        "glm-5-turbo".to_string(),
    ]
}

fn default_peak_start() -> String {
    "14:00".to_string()
}

fn default_peak_end() -> String {
    "18:00".to_string()
}

fn default_peak() -> f64 {
    3.0
}

fn default_off_peak() -> f64 {
    2.0
}

/// Promotional pricing configuration with reduced multiplier.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PromoConfig {
    /// Off-peak multiplier during promotional period.
    #[serde(default = "default_promo_off_peak")]
    pub off_peak: f64,
    /// Promo expiry date (YYYY-MM-DD format, inclusive).
    #[serde(default = "default_promo_expires")]
    pub expires: String,
}

impl Default for PromoConfig {
    fn default() -> Self {
        Self {
            off_peak: default_promo_off_peak(),
            expires: default_promo_expires(),
        }
    }
}

fn default_promo_off_peak() -> f64 {
    1.0
}

fn default_promo_expires() -> String {
    "2026-09-30".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_premium_models_includes_glm_5_2() {
        let models = default_premium_models();
        assert!(
            models.iter().any(|m| m == "glm-5.2"),
            "glm-5.2 must be in default premium_models: {models:?}"
        );
    }

    #[test]
    fn test_default_promo_expires_end_of_september() {
        // Off-peak 1x deduction benefit extended from end of June to end of September.
        assert_eq!(default_promo_expires(), "2026-09-30");
    }

    #[test]
    fn test_merge_segments_empty() {
        let config = Config {
            segments: vec![],
            ..Config::default()
        };
        let merged = config.merge_default_segments();
        assert_eq!(merged.segments.len(), 3);
    }

    #[test]
    fn test_merge_segments_partial() {
        let config = Config {
            segments: vec![SegmentConfig::token_usage()],
            ..Config::default()
        };
        let merged = config.merge_default_segments();
        assert_eq!(merged.segments.len(), 3);
        assert_eq!(merged.segments[0].id, "token_usage");
    }

    #[test]
    fn test_merge_segments_full() {
        let config = Config::default();
        let merged = config.merge_default_segments();
        assert_eq!(merged.segments.len(), 3);
    }

    #[test]
    fn test_merge_segments_preserves_order() {
        let config = Config {
            segments: vec![SegmentConfig::mcp_usage(), SegmentConfig::token_usage()],
            ..Config::default()
        };
        let merged = config.merge_default_segments();
        assert_eq!(merged.segments[0].id, "mcp_usage");
        assert_eq!(merged.segments[1].id, "token_usage");
        assert_eq!(merged.segments[2].id, "weekly_usage");
    }

    #[test]
    fn test_iconconfig_default_empty() {
        let icon = IconConfig::default();
        assert!(icon.emoji.is_empty());
        assert!(icon.ascii.is_empty());
    }

    #[test]
    fn test_iconconfig_new() {
        let icon = IconConfig::new("🪙", "$");
        assert_eq!(icon.emoji, "🪙");
        assert_eq!(icon.ascii, "$");
    }

    #[test]
    fn test_segment_config_token_usage_icons() {
        let seg = SegmentConfig::token_usage();
        assert_eq!(seg.icon.emoji, "🪙");
        assert_eq!(seg.icon.ascii, "$");
    }

    #[test]
    fn test_resolved_mode_emoji() {
        let style = StyleConfig {
            mode: DisplayMode::Emoji,
            separator: DEFAULT_SEPARATOR.to_string(),
        };
        assert_eq!(style.resolved_mode(), DisplayMode::Emoji);
    }

    #[test]
    fn test_resolved_mode_ascii() {
        let style = StyleConfig {
            mode: DisplayMode::Ascii,
            separator: DEFAULT_SEPARATOR.to_string(),
        };
        assert_eq!(style.resolved_mode(), DisplayMode::Ascii);
    }

    #[test]
    fn test_resolved_mode_auto_never_auto() {
        let style = StyleConfig {
            mode: DisplayMode::Auto,
            separator: DEFAULT_SEPARATOR.to_string(),
        };
        let resolved = style.resolved_mode();
        assert_ne!(resolved, DisplayMode::Auto);
        assert!(resolved == DisplayMode::Emoji || resolved == DisplayMode::Ascii);
    }

    #[test]
    fn test_merge_default_segments_fills_all() {
        let config = Config::default();
        let merged = config.merge_default_segments();
        let ids: Vec<&str> = merged.segments.iter().map(|s| s.id.as_str()).collect();
        assert!(ids.contains(&"token_usage"));
        assert!(ids.contains(&"weekly_usage"));
        assert!(ids.contains(&"mcp_usage"));
    }

    #[test]
    fn test_merge_default_segments_user_override() {
        let mut seg = SegmentConfig::token_usage();
        seg.enabled = false;
        let config = Config {
            segments: vec![seg],
            ..Config::default()
        };
        let merged = config.merge_default_segments();
        let token_seg = merged
            .segments
            .iter()
            .find(|s| s.id == "token_usage")
            .unwrap();
        assert!(!token_seg.enabled);
    }

    #[test]
    fn test_merge_default_segments_no_duplicates() {
        let config = Config {
            segments: vec![SegmentConfig::token_usage()],
            ..Config::default()
        };
        let merged = config.merge_default_segments();
        let token_count = merged
            .segments
            .iter()
            .filter(|s| s.id == "token_usage")
            .count();
        assert_eq!(token_count, 1);
    }
}
