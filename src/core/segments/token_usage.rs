use super::{Segment, SegmentData};
use crate::api::{GlmApiClient, SharedCache, UsageStats};
use crate::config::{Config, DisplayMode, InputData};
use std::time::{SystemTime, UNIX_EPOCH};
use time::{format_description, Month, OffsetDateTime, UtcOffset};

const UTC_PLUS_8: UtcOffset = match UtcOffset::from_hms(8, 0, 0) {
    Ok(offset) => offset,
    Err(_) => unreachable!(),
};

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
            _ => TimerMode::Clock,
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

        let local_datetime = match UtcOffset::local_offset_at(utc_datetime) {
            Ok(offset) => utc_datetime.to_offset(offset),
            Err(_) => utc_datetime,
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

    fn parse_hhmm(s: &str) -> Option<u16> {
        let (h, m) = s.split_once(':')?;
        let hours: u16 = h.parse().ok()?;
        let minutes: u16 = m.parse().ok()?;
        Some(hours * 60 + minutes)
    }

    fn is_peak_time(peak_start: &str, peak_end: &str) -> Option<bool> {
        let now_utc8 = OffsetDateTime::now_utc().to_offset(UTC_PLUS_8);
        let current = (now_utc8.hour() as u16) * 60 + now_utc8.minute() as u16;
        let start = Self::parse_hhmm(peak_start)?;
        let end = Self::parse_hhmm(peak_end)?;
        Some(current >= start && current <= end)
    }

    fn is_promo_active(expires: &str) -> Option<bool> {
        let now_utc8 = OffsetDateTime::now_utc().to_offset(UTC_PLUS_8);
        let (ey, em, ed) = Self::parse_ymd(expires)?;
        let expires_date = time::Date::from_calendar_date(ey, em, ed).ok()?;
        Some(now_utc8.date() <= expires_date)
    }

    fn parse_ymd(s: &str) -> Option<(i32, Month, u8)> {
        let mut parts = s.split('-');
        let y: i32 = parts.next()?.parse().ok()?;
        let m: u8 = parts.next()?.parse().ok()?;
        let d: u8 = parts.next()?.parse().ok()?;
        let month = Month::try_from(m).ok()?;
        Some((y, month, d))
    }

    fn calculate_multiplier(input: &InputData, config: &Config) -> f64 {
        let model_id = match input.model.as_ref() {
            Some(m) => &m.id,
            None => return 1.0,
        };

        let mc = &config.multiplier;
        let model_lower = model_id.to_lowercase();
        let is_premium = mc
            .premium_models
            .iter()
            .any(|pm| model_lower.contains(&pm.to_lowercase()));

        if !is_premium {
            return 1.0;
        }

        let is_peak = match Self::is_peak_time(&mc.peak_start, &mc.peak_end) {
            Some(v) => v,
            None => return 1.0,
        };

        if is_peak {
            return mc.peak;
        }

        let promo_active = Self::is_promo_active(&mc.promo.expires).unwrap_or(false);
        if promo_active {
            mc.promo.off_peak
        } else {
            mc.off_peak
        }
    }

    fn format_multiplier(value: f64) -> String {
        if value == value.floor() {
            format!("{}x", value as i64)
        } else {
            format!("{}x", value)
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

    fn collect(&self, input: &InputData, config: &Config) -> Option<SegmentData> {
        let stats = if config.cache.enabled {
            self.cache
                .get_or_fetch(config.cache.ttl_seconds, || self.fetch_usage_stats())
        } else {
            self.fetch_usage_stats()
        }?;

        let token = stats.token_usage.as_ref()?;

        let primary = format!("{}%", token.percentage);

        let resolved_mode = config.style.resolved_mode();

        let segment_config = config.segments.iter().find(|s| s.id == "token_usage");

        let show_timer = segment_config
            .and_then(|s| s.options.get("show_timer"))
            .and_then(|v| v.as_bool())
            .or_else(|| {
                segment_config
                    .and_then(|s| s.options.get("show_countdown"))
                    .and_then(|v| v.as_bool())
            })
            .unwrap_or(true);

        let timer_mode_str = segment_config
            .and_then(|s| s.options.get("timer_mode"))
            .and_then(|v| v.as_str())
            .unwrap_or("clock");

        let timer_mode = if segment_config
            .and_then(|s| s.options.get("show_timer"))
            .is_none()
            && segment_config
                .and_then(|s| s.options.get("show_countdown"))
                .is_some()
        {
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

        let show_multiplier = segment_config
            .and_then(|s| s.options.get("show_multiplier"))
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let multiplier_value = Self::calculate_multiplier(input, config);
        let multiplier_str = if show_multiplier && multiplier_value > 1.0 {
            Some(Self::format_multiplier(multiplier_value))
        } else {
            None
        };

        let mut out = SegmentData::new(primary)
            .with_secondary(secondary)
            .with_metadata("percentage", token.percentage.to_string());

        if let Some(m) = multiplier_str {
            out = out.with_multiplier(m);
        }

        Some(out)
    }
}
