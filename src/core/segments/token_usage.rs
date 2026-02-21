use super::{Segment, SegmentData};
use crate::api::{GlmApiClient, SharedCache, UsageStats};
use crate::config::{Config, DisplayMode, InputData};
use std::time::{SystemTime, UNIX_EPOCH};
use time::{format_description, OffsetDateTime};

/// Timer display mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TimerMode {
    Clock,
    Countdown,
}

impl TimerMode {
    fn from_str(s: &str) -> Self {
        match s {
            "countdown" => TimerMode::Countdown,
            _ => TimerMode::Clock, // default to clock
        }
    }
}

pub struct TokenUsageSegment {
    cache: SharedCache,
}

impl TokenUsageSegment {
    pub fn new() -> Self {
        Self {
            cache: SharedCache::new(),
        }
    }

    pub fn with_cache(cache: SharedCache) -> Self {
        Self { cache }
    }

    fn format_countdown(reset_at: i64) -> String {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        let remaining = reset_at.saturating_sub(now);

        if remaining <= 0 {
            return "0:00".to_string();
        }

        let hours = remaining / 3600;
        let minutes = (remaining % 3600) / 60;

        format!("{}:{:02}", hours, minutes)
    }

    fn format_clock_time(reset_at: i64) -> String {
        let utc_datetime = match OffsetDateTime::from_unix_timestamp(reset_at) {
            Ok(dt) => dt,
            Err(_) => return "--:--".to_string(),
        };

        // Try to get local offset, fallback to UTC
        let local_datetime = match time::UtcOffset::local_offset_at(utc_datetime) {
            Ok(offset) => utc_datetime.to_offset(offset),
            Err(_) => utc_datetime, // fallback to UTC
        };

        let format = format_description::parse("[hour]:[minute]").unwrap();
        local_datetime
            .format(&format)
            .unwrap_or_else(|_| "--:--".to_string())
    }

    fn fetch_usage_stats(&self) -> Option<UsageStats> {
        GlmApiClient::from_env().ok()?.fetch_usage_stats().ok()
    }

    fn get_timer_icon(timer_mode: TimerMode, display_mode: DisplayMode) -> &'static str {
        match timer_mode {
            TimerMode::Countdown => match display_mode {
                DisplayMode::Ascii => "!",
                _ => "⌛️",
            },
            TimerMode::Clock => match display_mode {
                DisplayMode::Ascii => "@",
                _ => "⏱\u{FE0F}",
            },
        }
    }
}

impl Default for TokenUsageSegment {
    fn default() -> Self {
        Self::new()
    }
}

impl Segment for TokenUsageSegment {
    fn id(&self) -> &str {
        "token_usage"
    }

    fn collect(&self, _input: &InputData, config: &Config) -> Option<SegmentData> {
        let stats = if config.cache.enabled {
            self.cache
                .get_or_fetch(config.cache.ttl_seconds, || self.fetch_usage_stats())
        } else {
            self.fetch_usage_stats()
        }?;

        let token = stats.token_usage.as_ref()?;

        let primary = format!("{}%", token.percentage);

        // Get resolved display mode for icon selection
        let resolved_mode = config.style.resolved_mode();

        // Read timer options with backward compatibility
        let segment_config = config.segments.iter().find(|s| s.id == "token_usage");

        // Check show_timer first, fall back to show_countdown for backward compat
        let show_timer = segment_config
            .and_then(|s| s.options.get("show_timer"))
            .and_then(|v| v.as_bool())
            .or_else(|| {
                // Fallback to show_countdown for backward compatibility
                segment_config
                    .and_then(|s| s.options.get("show_countdown"))
                    .and_then(|v| v.as_bool())
            })
            .unwrap_or(true);

        // Read timer_mode, default to "clock"
        let timer_mode_str = segment_config
            .and_then(|s| s.options.get("timer_mode"))
            .and_then(|v| v.as_str())
            .unwrap_or("clock");

        // Determine timer_mode, with backward compat for show_countdown
        let timer_mode = if segment_config
            .and_then(|s| s.options.get("show_timer"))
            .is_none()
            && segment_config
                .and_then(|s| s.options.get("show_countdown"))
                .is_some()
        {
            // Old config using show_countdown: preserve countdown behavior
            TimerMode::Countdown
        } else {
            TimerMode::from_str(timer_mode_str)
        };

        let secondary = if show_timer {
            let timer_icon = Self::get_timer_icon(timer_mode, resolved_mode);

            let formatted_time = match timer_mode {
                TimerMode::Countdown => token
                    .reset_at
                    .map(Self::format_countdown)
                    .unwrap_or_else(|| "--:--".to_string()),
                TimerMode::Clock => token
                    .reset_at
                    .map(Self::format_clock_time)
                    .unwrap_or_else(|| "--:--".to_string()),
            };

            format!("{} {}", timer_icon, formatted_time)
        } else {
            String::new()
        };

        let out = SegmentData::new(primary)
            .with_secondary(secondary)
            .with_metadata("percentage", token.percentage.to_string());

        Some(out)
    }
}
