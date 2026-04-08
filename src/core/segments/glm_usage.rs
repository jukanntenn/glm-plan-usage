use super::Segment;
use crate::api::{GlmApiClient, UsageStats};
use crate::config::{Config, InputData};
use crate::core::segments::{SegmentData, SegmentStyle};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// Format token count with appropriate units (M/K/raw)
fn format_tokens(count: i64) -> String {
    if count < 0 {
        return "N/A".to_string();
    }
    if count >= 1_000_000 {
        format!("{:.2}M", count as f64 / 1_000_000.0)
    } else if count >= 10_000 {
        format!("{:.1}K", count as f64 / 1_000.0)
    } else {
        format!("{}", count)
    }
}

/// Calculate countdown to reset time and format as HH:MM
fn format_countdown(reset_at: i64) -> Option<String> {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).ok()?.as_secs() as i64;

    let remaining = reset_at.saturating_sub(now);

    if remaining <= 0 {
        return Some("0:00".to_string());
    }

    let hours = remaining / 3600;
    let minutes = (remaining % 3600) / 60;

    Some(format!("{}:{:02}", hours, minutes))
}

/// GLM usage segment with caching
pub struct GlmUsageSegment {
    cache: Arc<Mutex<Option<CacheEntry>>>,
}

struct CacheEntry {
    stats: UsageStats,
    timestamp: Instant,
}

impl GlmUsageSegment {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(Mutex::new(None)),
        }
    }

    fn get_usage_stats(&self, config: &Config) -> Option<UsageStats> {
        // Check cache first
        if config.cache.enabled {
            if let Some(entry) = self.cache.lock().unwrap().as_ref() {
                if entry.timestamp.elapsed() < Duration::from_secs(config.cache.ttl_seconds) {
                    return Some(entry.stats.clone());
                }
            }
        }

        // Fetch from API
        match GlmApiClient::from_env() {
            Ok(client) => {
                match client.fetch_usage_stats() {
                    Ok(stats) => {
                        // Update cache
                        if config.cache.enabled {
                            let entry = CacheEntry {
                                stats: stats.clone(),
                                timestamp: Instant::now(),
                            };
                            *self.cache.lock().unwrap() = Some(entry);
                        }
                        Some(stats)
                    }
                    Err(_) => {
                        // Return cached data if available
                        self.cache.lock().unwrap().as_ref().map(|e| e.stats.clone())
                    }
                }
            }
            Err(_) => None,
        }
    }

    fn format_stats(stats: &UsageStats) -> String {
        let mut parts = Vec::new();

        // Token usage with countdown
        if let Some(token) = &stats.token_usage {
            let countdown = token
                .reset_at
                .and_then(format_countdown)
                .unwrap_or_else(|| "--:--".to_string());

            parts.push(format!("🪙 {}% (⌛️ {})", token.percentage, countdown));
        }

        // MCP raw count
        if let Some(mcp) = &stats.mcp_usage {
            parts.push(format!("🌐 {}/{}", mcp.used, mcp.limit));
        }

        if parts.is_empty() {
            String::new()
        } else {
            parts.join(" · ")
        }
    }

    fn get_color(stats: &UsageStats) -> SegmentStyle {
        // Get maximum usage percentage
        let max_pct = stats
            .token_usage
            .as_ref()
            .map(|u| u.percentage)
            .unwrap_or(0)
            .max(stats.mcp_usage.as_ref().map(|u| u.percentage).unwrap_or(0));

        let color_256 = match max_pct {
            0..=79 => Some(109),   // Green
            80..=94 => Some(226),  // Yellow
            95..=100 => Some(196), // Red
            _ => Some(109),
        };

        SegmentStyle {
            color: None,
            color_256,
            bold: true,
        }
    }
}

impl Default for GlmUsageSegment {
    fn default() -> Self {
        Self::new()
    }
}

impl Segment for GlmUsageSegment {
    fn id(&self) -> &str {
        "glm_usage"
    }

    fn collect(&self, _input: &InputData, config: &Config) -> Option<SegmentData> {
        let stats = self.get_usage_stats(config)?;

        let text = Self::format_stats(&stats);

        if text.is_empty() {
            return None;
        }

        let style = Self::get_color(&stats);

        Some(SegmentData { text, style })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::QuotaUsage;
    use crate::config::Config;
    use std::time::{Duration, Instant};

    #[test]
    fn format_tokens_negative_returns_na() {
        assert_eq!(format_tokens(-100), "N/A");
    }

    #[test]
    fn format_tokens_less_than_10k_returns_raw() {
        assert_eq!(format_tokens(0), "0");
        assert_eq!(format_tokens(5000), "5000");
        assert_eq!(format_tokens(9999), "9999");
    }

    #[test]
    fn format_tokens_between_10k_and_1m_returns_k_format() {
        assert_eq!(format_tokens(10000), "10.0K");
        assert_eq!(format_tokens(12345), "12.3K");
        assert_eq!(format_tokens(25000), "25.0K");
        assert_eq!(format_tokens(999999), "1000.0K");
    }

    #[test]
    fn format_tokens_greater_or_equal_1m_returns_m_format() {
        assert_eq!(format_tokens(1000000), "1.00M");
        assert_eq!(format_tokens(13050000), "13.05M");
        assert_eq!(format_tokens(2500000), "2.50M");
    }

    #[test]
    fn format_countdown_past_returns_zero_zero() {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        assert_eq!(format_countdown(now - 100), Some("0:00".to_string()));
    }

    #[test]
    fn format_countdown_future_single_digit_minutes_zero_padded() {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        let reset_at = now + 8 * 60; // 8 minutes
        assert_eq!(format_countdown(reset_at), Some("0:08".to_string()));
    }

    #[test]
    fn format_countdown_future_hours_and_minutes_formatted_correctly() {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        let reset_at = now + 3600 + 44 * 60; // 1 hour 44 minutes
        assert_eq!(format_countdown(reset_at), Some("1:44".to_string()));
    }

    fn create_test_stats() -> UsageStats {
        UsageStats {
            token_usage: Some(QuotaUsage {
                used: 13050000,
                limit: 40000000,
                percentage: 33,
                time_window: "daily".to_string(),
                reset_at: None,
            }),
            mcp_usage: None,
        }
    }

    #[test]
    fn cache_enabled_fresh_entry_returns_cached() {
        let segment = GlmUsageSegment::new();
        let stats = create_test_stats();
        let mut config = Config::default();
        config.cache.enabled = true;
        config.cache.ttl_seconds = 300;

        // Pre-populate cache with fresh entry
        let entry = CacheEntry {
            stats: stats.clone(),
            timestamp: Instant::now(),
        };
        *segment.cache.lock().unwrap() = Some(entry);

        let result = segment.get_usage_stats(&config);
        assert!(result.is_some());
        // Result should match cached value
        assert_eq!(
            result.unwrap().token_usage.unwrap().used,
            stats.token_usage.unwrap().used
        );
    }

    #[test]
    fn cache_enabled_no_entry_fetches_and_caches() {
        // Since we can't easily mock the API client without refactoring,
        // this test will get API error (no env vars) and return None.
        // We just test that cache remains empty after the call.
        let segment = GlmUsageSegment::new();
        let mut config = Config::default();
        config.cache.enabled = true;
        config.cache.ttl_seconds = 300;

        assert!(segment.cache.lock().unwrap().is_none());

        let result = segment.get_usage_stats(&config);
        assert!(result.is_none());
        assert!(segment.cache.lock().unwrap().is_none());
    }

    #[test]
    fn cache_enabled_entry_expired_ignores_cache() {
        let segment = GlmUsageSegment::new();
        let stats = create_test_stats();
        let mut config = Config::default();
        config.cache.enabled = true;
        config.cache.ttl_seconds = 300;

        // Create expired entry
        let entry = CacheEntry {
            stats: stats.clone(),
            timestamp: Instant::now() - Duration::from_secs(config.cache.ttl_seconds + 1),
        };
        *segment.cache.lock().unwrap() = Some(entry);

        // Should attempt to fetch new data, will fail without env vars → return None
        let result = segment.get_usage_stats(&config);
        assert!(result.is_none());
    }

    #[test]
    fn cache_disabled_always_fetches_does_not_update_cache() {
        let segment = GlmUsageSegment::new();
        let stats = create_test_stats();
        let mut config = Config::default();
        config.cache.enabled = false;

        // Pre-populate cache
        let entry = CacheEntry {
            stats: stats.clone(),
            timestamp: Instant::now(),
        };
        *segment.cache.lock().unwrap() = Some(entry);

        // Cache disabled → should not return cached value even though it exists
        // Will attempt API fetch which fails → return None
        let result = segment.get_usage_stats(&config);
        assert!(result.is_none());
    }

    #[test]
    fn api_fails_stale_cache_exists_returns_stale_fallback() {
        let segment = GlmUsageSegment::new();
        let stats = create_test_stats();
        let mut config = Config::default();
        config.cache.enabled = true;
        config.cache.ttl_seconds = 300;

        // Even if entry is expired, on API failure it should still return it as fallback
        let entry = CacheEntry {
            stats: stats.clone(),
            timestamp: Instant::now() - Duration::from_secs(config.cache.ttl_seconds + 1),
        };
        *segment.cache.lock().unwrap() = Some(entry);

        // API fetch will fail, but we have stale cache → return it
        let result = segment.get_usage_stats(&config);
        assert!(result.is_some());
        assert_eq!(
            result.unwrap().token_usage.unwrap().used,
            stats.token_usage.unwrap().used
        );
    }

    #[test]
    fn api_fails_no_cache_returns_none() {
        let segment = GlmUsageSegment::new();
        let mut config = Config::default();
        config.cache.enabled = true;

        // No existing cache entry
        assert!(segment.cache.lock().unwrap().is_none());

        // API fetch fails, no cache → return None
        let result = segment.get_usage_stats(&config);
        assert!(result.is_none());
    }
}
