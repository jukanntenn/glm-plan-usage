//! Token usage segment for displaying current token consumption.
//!
//! This segment shows the percentage of tokens used with a reset timer
//! and supports premium model multiplier calculations.

use super::{Segment, SegmentData};
use crate::api::SharedCache;
use crate::config::{Config, DisplayMode, InputData, SegmentConfig};
use std::sync::LazyLock;
use std::time::{SystemTime, UNIX_EPOCH};
use time::{format_description, Month, OffsetDateTime, UtcOffset};

const UTC_PLUS_8: UtcOffset = match UtcOffset::from_hms(8, 0, 0) {
    Ok(offset) => offset,
    #[allow(
        unreachable_code,
        reason = "const context requires unreachable!() for Err arm"
    )]
    Err(_) => unreachable!(),
};

static CLOCK_FORMAT: LazyLock<Vec<time::format_description::FormatItem<'static>>> =
    LazyLock::new(|| format_description::parse("[hour]:[minute]").expect("valid format"));

/// Timer display mode determining how reset time is shown.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TimerMode {
    /// Shows reset time as a clock (e.g., "14:00").
    Clock,
    /// Shows time remaining until reset as countdown (e.g., "2h 30m").
    Countdown,
}

impl TimerMode {
    /// Parses a timer mode from a string.
    ///
    /// Returns `Clock` for any value other than "countdown".
    fn from_str(s: &str) -> Self {
        match s {
            "countdown" => TimerMode::Countdown,
            _ => TimerMode::Clock,
        }
    }
}

/// Parses "HH:MM" format to minutes since midnight.
fn parse_hhmm(s: &str) -> Option<u16> {
    let (h, m) = s.split_once(':')?;
    let hours: u16 = h.parse().ok()?;
    let minutes: u16 = m.parse().ok()?;
    Some(hours * 60 + minutes)
}

/// Parses "YYYY-MM-DD" format to date components.
fn parse_ymd(s: &str) -> Option<(i32, Month, u8)> {
    let mut parts = s.split('-');
    let y: i32 = parts.next()?.parse().ok()?;
    let m: u8 = parts.next()?.parse().ok()?;
    let d: u8 = parts.next()?.parse().ok()?;
    let month = Month::try_from(m).ok()?;
    Some((y, month, d))
}

/// Formats remaining time as "H:MM" countdown.
fn format_countdown(reset_at: i64) -> String {
    #[expect(
        clippy::cast_possible_wrap,
        reason = "current epoch seconds fit in i64"
    )]
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;

    let remaining = reset_at.saturating_sub(now);

    if remaining <= 0 {
        return "0:00".to_string();
    }

    let hours = remaining / 3600;
    let minutes = (remaining % 3600) / 60;

    format!("{hours}:{minutes:02}")
}

/// Formats the reset timestamp as a local clock time "HH:MM".
fn format_clock_time(reset_at: i64) -> String {
    let Ok(utc_datetime) = OffsetDateTime::from_unix_timestamp(reset_at) else {
        return "--:--".to_string();
    };

    let local_datetime =
        utc_datetime.to_offset(UtcOffset::local_offset_at(utc_datetime).unwrap_or(UtcOffset::UTC));

    let format = CLOCK_FORMAT.clone();
    local_datetime
        .format(&format)
        .unwrap_or_else(|_| "--:--".to_string())
}

/// Formats the reset time according to the timer mode.
fn format_reset_time(reset_at: Option<i64>, timer_mode: TimerMode) -> String {
    let Some(at) = reset_at else {
        return "--:--".to_string();
    };

    match timer_mode {
        TimerMode::Countdown => format_countdown(at),
        TimerMode::Clock => format_clock_time(at),
    }
}

/// Returns the icon for the timer based on mode and display mode.
fn get_timer_icon(timer_mode: TimerMode, display_mode: DisplayMode) -> &'static str {
    match (timer_mode, display_mode) {
        (TimerMode::Countdown, DisplayMode::Ascii) => "!",
        (TimerMode::Countdown, _) => "⌛️",
        (TimerMode::Clock, DisplayMode::Ascii) => "@",
        (TimerMode::Clock, _) => "⏱\u{FE0F}",
    }
}

/// Returns the current minutes since midnight in UTC+8 timezone.
fn current_minutes_since_midnight() -> u16 {
    let now_utc8 = OffsetDateTime::now_utc().to_offset(UTC_PLUS_8);
    u16::from(now_utc8.hour()) * 60 + u16::from(now_utc8.minute())
}

/// Returns whether the current time falls within the peak time range.
fn is_peak_time(peak_start: &str, peak_end: &str) -> Option<bool> {
    let current = current_minutes_since_midnight();
    let start = parse_hhmm(peak_start)?;
    let end = parse_hhmm(peak_end)?;
    Some(current >= start && current <= end)
}

/// Returns whether a promotional period is still active based on expiry date.
fn is_promo_active(expires: &str) -> Option<bool> {
    let now_utc8 = OffsetDateTime::now_utc().to_offset(UTC_PLUS_8);
    let (ey, em, ed) = parse_ymd(expires)?;
    let expires_date = time::Date::from_calendar_date(ey, em, ed).ok()?;
    Some(now_utc8.date() <= expires_date)
}

/// Calculates the usage multiplier based on model, time, and promo status.
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

    let Some(is_peak) = is_peak_time(&mc.peak_start, &mc.peak_end) else {
        return 1.0;
    };

    if is_peak {
        return mc.peak;
    }

    let promo_active = is_promo_active(&mc.promo.expires).unwrap_or(false);
    if promo_active {
        mc.promo.off_peak
    } else {
        mc.off_peak
    }
}

/// Formats a multiplier value as a string with "x" suffix.
fn format_multiplier(value: f64) -> String {
    #[expect(
        clippy::float_cmp,
        reason = "exact floor check to decide integer vs float formatting"
    )]
    if value == value.floor() {
        #[expect(
            clippy::cast_possible_truncation,
            reason = "floor() guarantees the value fits in i64"
        )]
        let int_val = value as i64;
        format!("{int_val}x")
    } else {
        format!("{value}x")
    }
}

/// Resolves whether to show timer and which mode from segment config.
fn resolve_timer_mode(segment_config: Option<&SegmentConfig>) -> (bool, TimerMode) {
    let show_timer_opt = segment_config
        .and_then(|s| s.options.get("show_timer"))
        .and_then(serde_json::Value::as_bool);
    let show_countdown_opt = segment_config
        .and_then(|s| s.options.get("show_countdown"))
        .and_then(serde_json::Value::as_bool);

    let show_timer = show_timer_opt.or(show_countdown_opt).unwrap_or(true);

    let timer_mode = if show_timer_opt.is_none() && show_countdown_opt.is_some() {
        TimerMode::Countdown
    } else {
        let mode_str = segment_config
            .and_then(|s| s.options.get("timer_mode"))
            .and_then(|v| v.as_str())
            .unwrap_or("clock");
        TimerMode::from_str(mode_str)
    };

    (show_timer, timer_mode)
}

super::segment_with_cache!(TokenUsageSegment);

/// Collects token usage data for display in the status line.
impl Segment for TokenUsageSegment {
    fn collect(&self, input: &InputData, config: &Config) -> Option<SegmentData> {
        let stats = super::fetch_usage(config, &self.cache)?;

        let token = stats.token_usage.as_ref()?;

        let primary = format!("{}%", token.percentage);

        let resolved_mode = config.style.resolved_mode();

        let segment_config = config.segments.iter().find(|s| s.id == "token_usage");

        let (show_timer, timer_mode) = resolve_timer_mode(segment_config);

        let secondary = if show_timer {
            let timer_icon = get_timer_icon(timer_mode, resolved_mode);
            let formatted_time = format_reset_time(token.reset_at, timer_mode);
            format!("{timer_icon} {formatted_time}")
        } else {
            String::new()
        };

        let show_multiplier = segment_config
            .and_then(|s| s.options.get("show_multiplier"))
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(true);

        let multiplier_value = calculate_multiplier(input, config);
        let multiplier_str = (show_multiplier && multiplier_value > 1.0)
            .then(|| format_multiplier(multiplier_value));

        let mut out = SegmentData::new(primary)
            .with_secondary(secondary)
            .with_metadata("percentage", token.percentage);

        if let Some(m) = multiplier_str {
            out = out.with_multiplier(m);
        }

        Some(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hhmm_valid() {
        assert_eq!(parse_hhmm("14:00"), Some(840));
        assert_eq!(parse_hhmm("00:00"), Some(0));
        assert_eq!(parse_hhmm("23:59"), Some(1439));
    }

    #[test]
    fn test_parse_hhmm_invalid() {
        assert_eq!(parse_hhmm("abc"), None);
        assert_eq!(parse_hhmm(""), None);
        assert_eq!(parse_hhmm("14"), None);
        assert_eq!(parse_hhmm("abc:def"), None);
    }

    #[test]
    fn test_parse_ymd_valid() {
        let result = parse_ymd("2026-06-30");
        assert!(result.is_some());
        let (y, m, d) = result.unwrap();
        assert_eq!(y, 2026);
        assert_eq!(m, time::Month::June);
        assert_eq!(d, 30);
    }

    #[test]
    fn test_parse_ymd_invalid() {
        assert_eq!(parse_ymd("2026-13-01"), None);
        assert_eq!(parse_ymd("abc"), None);
        assert_eq!(parse_ymd(""), None);
    }

    #[test]
    fn test_format_countdown_future() {
        let far_future = 2000000000i64;
        let result = format_countdown(far_future);
        assert!(result.contains(':'));
        assert!(!result.is_empty());
    }

    #[test]
    fn test_format_countdown_past() {
        let past = 1000i64;
        assert_eq!(format_countdown(past), "0:00");
    }

    #[test]
    fn test_format_multiplier_integer() {
        assert_eq!(format_multiplier(3.0), "3x");
        assert_eq!(format_multiplier(1.0), "1x");
        assert_eq!(format_multiplier(2.0), "2x");
    }

    #[test]
    fn test_format_multiplier_fractional() {
        assert_eq!(format_multiplier(2.5), "2.5x");
        assert_eq!(format_multiplier(1.5), "1.5x");
    }

    #[test]
    fn test_is_peak_time_valid_format() {
        let result = is_peak_time("14:00", "18:00");
        assert!(result.is_some());
    }

    #[test]
    fn test_is_peak_time_invalid_format() {
        assert_eq!(is_peak_time("abc", "def"), None);
    }

    #[test]
    fn test_is_peak_time_empty() {
        assert_eq!(is_peak_time("", ""), None);
    }

    #[test]
    fn test_is_promo_active_future() {
        assert_eq!(is_promo_active("2099-12-31"), Some(true));
    }

    #[test]
    fn test_is_promo_active_past() {
        assert_eq!(is_promo_active("2000-01-01"), Some(false));
    }

    #[test]
    fn test_is_promo_active_invalid() {
        assert_eq!(is_promo_active("abc"), None);
    }

    #[test]
    fn test_format_reset_time_none_countdown() {
        assert_eq!(format_reset_time(None, TimerMode::Countdown), "--:--");
    }

    #[test]
    fn test_format_reset_time_none_clock() {
        assert_eq!(format_reset_time(None, TimerMode::Clock), "--:--");
    }

    #[test]
    fn test_format_reset_time_countdown_past() {
        assert_eq!(format_reset_time(Some(1000), TimerMode::Countdown), "0:00");
    }

    #[test]
    fn test_format_reset_time_clock_valid() {
        let result = format_reset_time(Some(2000000000), TimerMode::Clock);
        assert!(result.contains(':'));
    }

    #[test]
    fn test_get_timer_icon_countdown_ascii() {
        assert_eq!(
            get_timer_icon(TimerMode::Countdown, DisplayMode::Ascii),
            "!"
        );
    }

    #[test]
    fn test_get_timer_icon_clock_ascii() {
        assert_eq!(get_timer_icon(TimerMode::Clock, DisplayMode::Ascii), "@");
    }

    #[test]
    fn test_get_timer_icon_countdown_emoji() {
        let icon = get_timer_icon(TimerMode::Countdown, DisplayMode::Emoji);
        assert!(!icon.is_ascii());
    }

    #[test]
    fn test_get_timer_icon_clock_emoji() {
        let icon = get_timer_icon(TimerMode::Clock, DisplayMode::Emoji);
        assert!(!icon.is_ascii());
    }

    #[test]
    fn test_format_countdown_zero() {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        assert_eq!(format_countdown(now), "0:00");
    }

    #[test]
    fn test_format_countdown_ancient_timestamp() {
        assert_eq!(format_countdown(1000), "0:00");
    }

    #[test]
    fn test_format_countdown_one_hour() {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        let result = format_countdown(now + 3600);
        assert_eq!(result, "1:00");
    }

    #[test]
    fn test_format_countdown_large_value() {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        let result = format_countdown(now + 86400);
        assert_eq!(result, "24:00");
    }

    #[test]
    fn test_timer_mode_from_str_countdown() {
        assert_eq!(TimerMode::from_str("countdown"), TimerMode::Countdown);
    }

    #[test]
    fn test_timer_mode_from_str_clock() {
        assert_eq!(TimerMode::from_str("clock"), TimerMode::Clock);
    }

    #[test]
    fn test_timer_mode_from_str_unknown_defaults_clock() {
        assert_eq!(TimerMode::from_str("unknown"), TimerMode::Clock);
        assert_eq!(TimerMode::from_str(""), TimerMode::Clock);
    }

    #[test]
    fn test_resolve_timer_mode_none() {
        let (show, mode) = resolve_timer_mode(None);
        assert!(show);
        assert_eq!(mode, TimerMode::Clock);
    }

    #[test]
    fn test_resolve_timer_mode_show_timer_false() {
        let mut config = SegmentConfig::token_usage();
        config
            .options
            .insert("show_timer".to_string(), serde_json::Value::Bool(false));
        let (show, mode) = resolve_timer_mode(Some(&config));
        assert!(!show);
        assert_eq!(mode, TimerMode::Clock);
    }

    #[test]
    fn test_resolve_timer_mode_show_timer_true_explicit() {
        let mut config = SegmentConfig::token_usage();
        config
            .options
            .insert("show_timer".to_string(), serde_json::Value::Bool(true));
        let (show, mode) = resolve_timer_mode(Some(&config));
        assert!(show);
        assert_eq!(mode, TimerMode::Clock);
    }

    #[test]
    fn test_resolve_timer_mode_show_countdown_backward_compat() {
        // When show_timer is absent but show_countdown is present → Countdown mode
        let mut config = SegmentConfig::token_usage();
        config.options.remove("show_timer");
        config
            .options
            .insert("show_countdown".to_string(), serde_json::Value::Bool(true));
        let (show, mode) = resolve_timer_mode(Some(&config));
        assert!(show);
        assert_eq!(mode, TimerMode::Countdown);
    }

    #[test]
    fn test_resolve_timer_mode_timer_mode_countdown() {
        let mut config = SegmentConfig::token_usage();
        config.options.insert(
            "timer_mode".to_string(),
            serde_json::Value::String("countdown".to_string()),
        );
        let (show, mode) = resolve_timer_mode(Some(&config));
        assert!(show);
        assert_eq!(mode, TimerMode::Countdown);
    }

    #[test]
    fn test_calculate_multiplier_no_model() {
        let input: InputData = serde_json::from_str("{}").unwrap();
        let config = Config::default();
        assert_eq!(calculate_multiplier(&input, &config), 1.0);
    }

    #[test]
    fn test_calculate_multiplier_non_premium_model() {
        let input: InputData =
            serde_json::from_str(r#"{"model": {"id": "claude-sonnet"}}"#).unwrap();
        let config = Config::default();
        assert_eq!(calculate_multiplier(&input, &config), 1.0);
    }

    #[test]
    fn test_calculate_multiplier_glm_5_2_recognized_as_premium() {
        // GLM-5.2 must be recognized as a premium model. With promo forced
        // inactive, a premium model returns peak (3.0) or off_peak (2.0) —
        // both >= off_peak — never the 1.0 returned for non-premium models,
        // regardless of time of day.
        let input: InputData = serde_json::from_str(r#"{"model": {"id": "glm-5.2"}}"#).unwrap();
        let mut config = Config::default();
        config.multiplier.promo.expires = "2000-01-01".to_string();
        let multiplier = calculate_multiplier(&input, &config);
        assert!(
            multiplier >= config.multiplier.off_peak,
            "glm-5.2 should be premium, got {multiplier}"
        );
    }

    #[test]
    fn test_current_minutes_since_midnight() {
        let mins = current_minutes_since_midnight();
        assert!(mins < 1440);
    }

    #[test]
    fn test_format_clock_time_invalid_timestamp() {
        // i64::MIN is an invalid timestamp
        let result = format_clock_time(i64::MIN);
        assert_eq!(result, "--:--");
    }

    #[test]
    fn test_is_promo_active_invalid_date() {
        assert_eq!(is_promo_active("not-a-date"), None);
        assert_eq!(is_promo_active("2026-13-01"), None);
        assert_eq!(is_promo_active(""), None);
    }
}
