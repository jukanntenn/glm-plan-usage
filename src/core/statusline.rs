//! Status line string generation with ANSI colors.
//!
//! This module generates the final status line string by combining
//! segment data with colors and separators.

use crate::config::{Config, DisplayMode, SegmentConfig};
use crate::core::segments::SegmentData;

/// Generates the colored status line string from segment data.
#[derive(Default, Debug)]
pub struct StatusLineGenerator;

impl StatusLineGenerator {
    /// Create a new status line generator.
    #[must_use]
    #[allow(dead_code, reason = "public API, may be used by external callers")]
    pub fn new() -> Self {
        Self
    }
}

const WHITE: &str = "\x1b[37m";
const GRAY: &str = "\x1b[38;5;109m";
const RED: &str = "\x1b[38;5;196m";
const GREEN: &str = "\x1b[38;5;46m";
const YELLOW: &str = "\x1b[38;5;226m";
const RESET: &str = "\x1b[0m";
const DOT_SEP_WITH_COLORS: &str = "\x1b[37m · \x1b[0m";

/// Threshold for warning color (yellow) - green below this percentage.
///
/// Based on standard project management warning levels. Changing this
/// affects the color transition point in the status line display.
const PERCENTAGE_WARNING_THRESHOLD: u8 = 80;

/// Threshold for critical color (red) - yellow between warning and critical.
///
/// Standard critical threshold. Users should be alerted when approaching limit.
/// Changing this affects when the status line turns red.
const PERCENTAGE_CRITICAL_THRESHOLD: u8 = 90;

/// Returns the ANSI color code for a percentage value.
///
/// Green for 0-80%, yellow for 81-90%, red for 91%+.
/// Values exceeding 100% are treated as critical (red).
fn get_color_for_percentage(percentage: u8) -> &'static str {
    match percentage {
        0..=PERCENTAGE_WARNING_THRESHOLD => GREEN,
        81..=PERCENTAGE_CRITICAL_THRESHOLD => YELLOW,
        _ => RED,
    }
}

impl StatusLineGenerator {
    /// Generate a status line string from the given segments.
    ///
    /// Segments are rendered with icons, colors, and separators according to the config.
    #[must_use]
    pub fn generate(config: &Config, segments: &[(SegmentConfig, SegmentData)]) -> String {
        let mut output = Vec::new();

        for (seg_config, data) in segments {
            let rendered = Self::render_segment(config, seg_config, data);
            if !rendered.is_empty() {
                output.push(rendered);
            }
        }

        if output.is_empty() {
            return String::new();
        }

        let separator = Self::format_separator(config);
        output.join(&separator)
    }

    /// Renders a single segment with icon, colors, and text formatting.
    fn render_segment(config: &Config, seg_config: &SegmentConfig, data: &SegmentData) -> String {
        let icon = Self::get_icon(config, seg_config);
        let pct_color = data
            .metadata
            .get("percentage")
            .and_then(|s| s.parse::<u8>().ok())
            .map_or(GRAY, get_color_for_percentage);

        // Primary block: icon + primary text in percentage color
        let primary_block = if icon.is_empty() {
            format!("{}{}{}", pct_color, data.primary, RESET)
        } else {
            format!("{}{} {}{}", pct_color, icon, data.primary, RESET)
        };

        // Multiplier block (red, only when present)
        let multiplier_block = data
            .multiplier
            .as_deref()
            .filter(|m| !m.is_empty())
            .map(|m| format!("{RED}{m}{RESET}"));

        // Secondary block (gray)
        let secondary_block =
            (!data.secondary.is_empty()).then(|| format!("{}{}{}", GRAY, data.secondary, RESET));

        // Assemble: primary [· multiplier] [· secondary]
        let dot_sep = DOT_SEP_WITH_COLORS;
        let mut parts = vec![primary_block];
        if let Some(m) = multiplier_block {
            parts.push(m);
        }
        if let Some(s) = secondary_block {
            parts.push(s);
        }

        parts.join(dot_sep)
    }

    /// Returns the icon string based on the display mode and segment config.
    fn get_icon(config: &Config, seg_config: &SegmentConfig) -> String {
        match config.style.resolved_mode() {
            DisplayMode::Emoji => seg_config.icon.emoji.clone(),
            DisplayMode::Ascii => seg_config.icon.ascii.clone(),
            DisplayMode::Auto => unreachable!("resolved_mode() never returns Auto"),
        }
    }

    /// Returns the formatted separator string with ANSI colors.
    fn format_separator(config: &Config) -> String {
        format!("{}{}{}{}", RESET, WHITE, config.style.separator, RESET)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{IconConfig, SegmentConfig, StyleConfig};
    use std::collections::HashMap;

    fn test_config() -> Config {
        Config {
            style: StyleConfig {
                mode: DisplayMode::Ascii,
                separator: crate::config::DEFAULT_SEPARATOR.to_string(),
            },
            ..Config::default()
        }
    }

    fn test_segment_config(id: &str) -> SegmentConfig {
        SegmentConfig {
            id: id.to_string(),
            enabled: true,
            icon: IconConfig::new("$", "$"),
            options: HashMap::new(),
        }
    }

    #[test]
    fn test_generate_empty() {
        let config = test_config();
        let output = StatusLineGenerator::generate(&config, &[]);
        assert!(output.is_empty());
    }

    #[test]
    fn test_generate_single_segment() {
        let config = test_config();
        let seg_config = test_segment_config("token_usage");
        let data = SegmentData::new("50%").with_metadata("percentage", "50");
        let output = StatusLineGenerator::generate(&config, &[(seg_config, data)]);
        assert!(output.contains("50%"));
    }

    #[test]
    fn test_generate_multiple_segments() {
        let config = test_config();
        let seg1 = test_segment_config("token_usage");
        let data1 = SegmentData::new("50%").with_metadata("percentage", "50");
        let seg2 = test_segment_config("weekly_usage");
        let data2 = SegmentData::new("30%").with_metadata("percentage", "30");
        let output = StatusLineGenerator::generate(&config, &[(seg1, data1), (seg2, data2)]);
        assert!(output.contains("50%"));
        assert!(output.contains("30%"));
        assert!(output.contains(" | "));
    }

    #[test]
    fn test_generate_with_multiplier() {
        let config = test_config();
        let seg_config = test_segment_config("token_usage");
        let data = SegmentData::new("50%")
            .with_multiplier("3x")
            .with_metadata("percentage", "50");
        let output = StatusLineGenerator::generate(&config, &[(seg_config, data)]);
        assert!(output.contains("3x"));
    }
}
