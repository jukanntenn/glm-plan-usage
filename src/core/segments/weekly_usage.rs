//! Weekly usage segment for displaying weekly token consumption.
//!
//! This segment shows the percentage of tokens used within
//! the current weekly billing period.

use super::{Segment, SegmentData};
use crate::api::SharedCache;
use crate::config::{Config, InputData};

super::segment_with_cache!(WeeklyUsageSegment);

/// Collects weekly usage data for display in the status line.
impl Segment for WeeklyUsageSegment {
    fn collect(&self, _input: &InputData, config: &Config) -> Option<SegmentData> {
        let stats = super::fetch_usage(config, &self.cache)?;

        let weekly = stats.weekly_usage.as_ref()?;

        let primary = format!("{}%", weekly.percentage);

        Some(SegmentData::new(primary).with_metadata("percentage", weekly.percentage))
    }
}
