use crate::config::{Config, DisplayMode, SegmentConfig};
use crate::core::segments::SegmentData;

pub struct StatusLineGenerator;

const WHITE: &str = "\x1b[37m";
const GRAY: &str = "\x1b[38;5;109m";
const RED: &str = "\x1b[38;5;196m";
const RESET: &str = "\x1b[0m";
const DOT_SEP: &str = " · ";

fn get_color_for_percentage(percentage: u8) -> &'static str {
    match percentage {
        0..=80 => "\x1b[38;5;46m",
        81..=90 => "\x1b[38;5;226m",
        _ => "\x1b[38;5;196m",
    }
}

impl StatusLineGenerator {
    pub fn generate(config: &Config, segments: Vec<(SegmentConfig, SegmentData)>) -> String {
        let mut output = Vec::new();

        for (seg_config, data) in segments.iter().filter(|(c, _)| c.enabled) {
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

    fn render_segment(config: &Config, seg_config: &SegmentConfig, data: &SegmentData) -> String {
        let icon = Self::get_icon(config, seg_config);
        let pct_color = data
            .metadata
            .get("percentage")
            .and_then(|s| s.parse::<u8>().ok())
            .map(get_color_for_percentage)
            .unwrap_or(GRAY);

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
            .map(|m| format!("{}{}{}", RED, m, RESET));

        // Secondary block (gray)
        let secondary_block = if data.secondary.is_empty() {
            None
        } else {
            Some(format!("{}{}{}", GRAY, data.secondary, RESET))
        };

        // Assemble: primary [· multiplier] [· secondary]
        let dot_sep = format!("{}{}{}", WHITE, DOT_SEP, RESET);
        let mut parts = vec![primary_block];
        if let Some(m) = multiplier_block {
            parts.push(m);
        }
        if let Some(s) = secondary_block {
            parts.push(s);
        }

        parts.join(&dot_sep)
    }

    fn get_icon(config: &Config, seg_config: &SegmentConfig) -> String {
        match config.style.resolved_mode() {
            DisplayMode::Emoji => seg_config.icon.emoji.clone(),
            DisplayMode::Ascii => seg_config.icon.ascii.clone(),
            DisplayMode::Auto => unreachable!("resolved_mode() never returns Auto"),
        }
    }

    fn format_separator(config: &Config) -> String {
        format!("{}{}{}{}", RESET, WHITE, config.style.separator, RESET)
    }
}

impl Default for StatusLineGenerator {
    fn default() -> Self {
        Self
    }
}
