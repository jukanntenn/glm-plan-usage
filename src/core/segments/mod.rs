//! Segment implementations for status line display.
//!
//! This module provides segment types that collect usage data
//! and format it for display in the status bar.

pub mod mcp_usage;
pub mod token_usage;
pub mod weekly_usage;

use crate::api::{GlmApiClient, SharedCache, UsageStats};
use crate::config::{Config, InputData};
use std::collections::HashMap;
use std::time::Duration;

macro_rules! segment_with_cache {
    ($name:ident) => {
        #[derive(Debug)]
        pub struct $name {
            cache: SharedCache,
        }

        impl $name {
            pub fn new() -> Self {
                Self {
                    cache: SharedCache::new(),
                }
            }

            pub fn with_cache(cache: SharedCache) -> Self {
                Self { cache }
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }
    };
}

pub(crate) use segment_with_cache;

/// Fetches usage statistics from the API or cache.
///
/// Returns `None` if the fetch fails or cache is disabled.
pub(super) fn fetch_usage(config: &Config, cache: &SharedCache) -> Option<UsageStats> {
    let timeout = Duration::from_millis(config.api.timeout_ms);
    let retries = config.api.retry_attempts;
    let fetch = || {
        GlmApiClient::from_env(timeout, retries)
            .ok()?
            .fetch_usage_stats()
            .ok()
    };
    if config.cache.enabled {
        cache.get_or_fetch(config.cache.ttl_seconds, fetch)
    } else {
        fetch()
    }
}

#[derive(Debug, Clone, Default)]
pub struct SegmentData {
    /// Main display text (e.g., "50%").
    pub primary: String,
    /// Supplemental text (e.g., "2.0x" multiplier).
    pub secondary: String,
    /// Usage multiplier when applicable.
    pub multiplier: Option<String>,
    /// Additional rendering data (percentage, color, etc.).
    pub metadata: HashMap<String, String>,
}

impl SegmentData {
    pub fn new(primary: impl Into<String>) -> Self {
        Self {
            primary: primary.into(),
            secondary: String::new(),
            multiplier: None,
            metadata: HashMap::new(),
        }
    }

    #[must_use]
    pub fn with_secondary(mut self, secondary: impl Into<String>) -> Self {
        self.secondary = secondary.into();
        self
    }

    #[must_use]
    pub fn with_multiplier(mut self, multiplier: impl Into<String>) -> Self {
        self.multiplier = Some(multiplier.into());
        self
    }

    #[must_use]
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl std::fmt::Display) -> Self {
        self.metadata.insert(key.into(), value.to_string());
        self
    }
}

/// Trait for segments that collect and format usage data for display.
///
/// Implementors must be thread-safe (`Send + Sync`) as segments may be
/// accessed concurrently. The `collect` method returns `None` if
/// data cannot be retrieved, enabling graceful degradation.
pub trait Segment: Send + Sync {
    fn collect(&self, input: &InputData, config: &Config) -> Option<SegmentData>;
}

#[doc(inline)]
pub use mcp_usage::McpUsageSegment;
#[doc(inline)]
pub use token_usage::TokenUsageSegment;
#[doc(inline)]
pub use weekly_usage::WeeklyUsageSegment;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_segmentdata_new() {
        let data = SegmentData::new("50%");
        assert_eq!(data.primary, "50%");
        assert!(data.secondary.is_empty());
        assert!(data.multiplier.is_none());
        assert!(data.metadata.is_empty());
    }

    #[test]
    fn test_segmentdata_with_metadata() {
        let data = SegmentData::new("50%").with_metadata("percentage", "50");
        assert_eq!(data.metadata.get("percentage"), Some(&"50".to_string()));
    }

    #[test]
    fn test_segmentdata_chained_metadata() {
        let data = SegmentData::new("50%")
            .with_metadata("percentage", "50")
            .with_metadata("used", "50000");
        assert_eq!(data.metadata.get("percentage"), Some(&"50".to_string()));
        assert_eq!(data.metadata.get("used"), Some(&"50000".to_string()));
    }

    #[test]
    fn test_segmentdata_with_secondary() {
        let data = SegmentData::new("50%").with_secondary("⏱ 14:00");
        assert_eq!(data.primary, "50%");
        assert_eq!(data.secondary, "⏱ 14:00");
        assert!(data.multiplier.is_none());
    }

    #[test]
    fn test_segmentdata_with_multiplier() {
        let data = SegmentData::new("50%").with_multiplier("3x");
        assert_eq!(data.primary, "50%");
        assert_eq!(data.multiplier, Some("3x".to_string()));
    }

    #[test]
    fn test_segmentdata_chained_builders() {
        let data = SegmentData::new("75%")
            .with_secondary("! 2:30")
            .with_multiplier("2.5x")
            .with_metadata("percentage", "75");
        assert_eq!(data.primary, "75%");
        assert_eq!(data.secondary, "! 2:30");
        assert_eq!(data.multiplier, Some("2.5x".to_string()));
        assert_eq!(data.metadata.get("percentage"), Some(&"75".to_string()));
    }

    #[test]
    fn test_segmentdata_default() {
        let data = SegmentData::default();
        assert!(data.primary.is_empty());
        assert!(data.secondary.is_empty());
        assert!(data.multiplier.is_none());
        assert!(data.metadata.is_empty());
    }

    #[test]
    fn test_fetch_usage_cache_disabled() {
        let mut config = Config::default();
        config.cache.enabled = false;
        // Without env vars, from_env will fail → fetch returns None
        let cache = SharedCache::new();
        let result = fetch_usage(&config, &cache);
        assert!(result.is_none());
    }

    #[test]
    fn test_fetch_usage_cache_enabled() {
        let mut config = Config::default();
        config.cache.enabled = true;
        config.cache.ttl_seconds = 300;
        // Without env vars, from_env will fail → fetch returns None
        let cache = SharedCache::new();
        let result = fetch_usage(&config, &cache);
        assert!(result.is_none());
    }
}
