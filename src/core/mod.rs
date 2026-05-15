//! Status line generation and segment data collection.
//!
//! This module provides the core functionality for generating status line output,
//! including segment implementations and the status line generator.

mod segments;
mod statusline;

#[doc(inline)]
pub use segments::{McpUsageSegment, Segment, SegmentData, TokenUsageSegment, WeeklyUsageSegment};
#[doc(inline)]
pub use statusline::StatusLineGenerator;
