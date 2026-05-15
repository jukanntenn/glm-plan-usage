//! MCP usage segment for displaying MCP tool time consumption.
//!
//! This segment shows the used vs limit time for MCP tool usage
//! within the 30-day billing period.

use super::{Segment, SegmentData};
use crate::api::SharedCache;
use crate::config::{Config, InputData};

super::segment_with_cache!(McpUsageSegment);

/// Collects MCP usage data for display in the status line.
impl Segment for McpUsageSegment {
    fn collect(&self, _input: &InputData, config: &Config) -> Option<SegmentData> {
        let stats = super::fetch_usage(config, &self.cache)?;

        let mcp = stats.mcp_usage.as_ref()?;
        let primary = format!("{}/{}", mcp.used, mcp.limit);

        let out = SegmentData::new(primary).with_metadata("percentage", mcp.percentage);

        Some(out)
    }
}
