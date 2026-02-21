pub mod mcp_usage;
pub mod token_usage;
pub mod weekly_usage;

use crate::config::{Config, InputData};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct SegmentData {
    pub primary: String,
    pub secondary: String,
    pub metadata: HashMap<String, String>,
}

impl SegmentData {
    pub fn new(primary: impl Into<String>) -> Self {
        Self {
            primary: primary.into(),
            secondary: String::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn with_secondary(mut self, secondary: impl Into<String>) -> Self {
        self.secondary = secondary.into();
        self
    }

    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

#[allow(dead_code)]
pub trait Segment: Send + Sync {
    fn id(&self) -> &str;

    fn is_enabled(&self, config: &Config) -> bool {
        config
            .segments
            .iter()
            .find(|s| s.id == self.id())
            .map(|s| s.enabled)
            .unwrap_or(false)
    }

    fn collect(&self, input: &InputData, config: &Config) -> Option<SegmentData>;
}

pub use mcp_usage::McpUsageSegment;
pub use token_usage::TokenUsageSegment;
pub use weekly_usage::WeeklyUsageSegment;
