use super::Segment;
use crate::api::{GlmApiClient, UsageStats};
use crate::config::{Config, InputData};
use crate::core::segments::{SegmentData, SegmentStyle};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

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

        if let Some(token) = &stats.token_usage {
            parts.push(format!("T:{}%", token.percentage));
        }

        if let Some(mcp) = &stats.mcp_usage {
            parts.push(format!("M:{}%", mcp.percentage));
        }

        if parts.is_empty() {
            String::new()
        } else {
            parts.join(" ")
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
