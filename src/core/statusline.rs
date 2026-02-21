use crate::config::{Config, DisplayMode, SegmentConfig};
use crate::core::segments::SegmentData;

pub struct StatusLineGenerator;

/// Get ANSI 256 color code based on usage percentage
/// - Green (0-50%): Normal usage
/// - Yellow (51-80%): Warning zone
/// - Red (81-100%): Critical usage
fn get_color_for_percentage(percentage: u8) -> String {
    match percentage {
        0..=50 => "\x1b[38;5;46m".to_string(),   // Bright green
        51..=80 => "\x1b[38;5;226m".to_string(), // Bright yellow
        _ => "\x1b[38;5;196m".to_string(),       // Bright red
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

        let text = if data.secondary.is_empty() {
            data.primary.clone()
        } else {
            format!("{} · {}", data.primary, data.secondary)
        };

        // Apply dynamic color based on percentage
        let colored_output = if let Some(percentage_str) = data.metadata.get("percentage") {
            if let Ok(percentage) = percentage_str.parse::<u8>() {
                let color_code = get_color_for_percentage(percentage);
                if icon.is_empty() {
                    format!("{}{}\x1b[0m", color_code, text)
                } else {
                    format!("{}{} {}\x1b[0m", color_code, icon, text)
                }
            } else {
                // Failed to parse percentage, use gray
                if icon.is_empty() {
                    format!("\x1b[38;5;109m{}\x1b[0m", text)
                } else {
                    format!("\x1b[38;5;109m{} {}\x1b[0m", icon, text)
                }
            }
        } else {
            // No percentage available, use gray
            if icon.is_empty() {
                format!("\x1b[38;5;109m{}\x1b[0m", text)
            } else {
                format!("\x1b[38;5;109m{} {}\x1b[0m", icon, text)
            }
        };

        colored_output
    }

    fn get_icon(config: &Config, seg_config: &SegmentConfig) -> String {
        match config.style.resolved_mode() {
            DisplayMode::Emoji => seg_config.icon.emoji.clone(),
            DisplayMode::Ascii => seg_config.icon.ascii.clone(),
            DisplayMode::Auto => unreachable!("resolved_mode() never returns Auto"),
        }
    }

    fn format_separator(config: &Config) -> String {
        format!("\x1b[0m\x1b[37m{}\x1b[0m", config.style.separator)
    }
}

impl Default for StatusLineGenerator {
    fn default() -> Self {
        Self
    }
}
