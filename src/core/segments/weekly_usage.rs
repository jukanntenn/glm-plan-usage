use super::{Segment, SegmentData};
use crate::api::{GlmApiClient, SharedCache, UsageStats};
use crate::config::{Config, InputData};

pub struct WeeklyUsageSegment {
    cache: SharedCache,
}

impl WeeklyUsageSegment {
    pub fn new() -> Self {
        Self {
            cache: SharedCache::new(),
        }
    }

    pub fn with_cache(cache: SharedCache) -> Self {
        Self { cache }
    }

    fn fetch_usage_stats(&self) -> Option<UsageStats> {
        GlmApiClient::from_env().ok()?.fetch_usage_stats().ok()
    }
}

impl Default for WeeklyUsageSegment {
    fn default() -> Self {
        Self::new()
    }
}

impl Segment for WeeklyUsageSegment {
    fn id(&self) -> &str {
        "weekly_usage"
    }

    fn collect(&self, _input: &InputData, config: &Config) -> Option<SegmentData> {
        let stats = if config.cache.enabled {
            self.cache
                .get_or_fetch(config.cache.ttl_seconds, || self.fetch_usage_stats())
        } else {
            self.fetch_usage_stats()
        }?;

        let weekly = stats.weekly_usage.as_ref()?;

        let primary = format!("{}%", weekly.percentage);

        Some(SegmentData::new(primary).with_metadata("percentage", weekly.percentage.to_string()))
    }
}
