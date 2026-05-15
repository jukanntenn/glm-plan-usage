//! API types for GLM usage data.
//!
//! This module defines types for API responses, error handling,
//! and platform detection for the GLM/ZHIPU API.

use serde::Deserialize;
use std::fmt;
use thiserror::Error;

/// Detected API platform (ZAI or ZHIPU).
///
/// Used to determine platform-specific API behavior.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    Zai,
    Zhipu,
}

impl Platform {
    /// Detect platform from a base URL string.
    ///
    /// Returns `None` if the URL does not match any known platform.
    #[must_use]
    pub fn detect_from_url(base_url: &str) -> Option<Self> {
        if base_url.contains("api.z.ai") {
            Some(Platform::Zai)
        } else if base_url.contains("bigmodel.cn") || base_url.contains("zhipu") {
            Some(Platform::Zhipu)
        } else {
            None
        }
    }
}

/// Errors that can occur when calling the GLM API.
#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Missing environment variable: {0}")]
    MissingEnvVar(String),

    #[error("HTTP request failed: {0}")]
    HttpError(String),

    #[error("API returned error: {0}")]
    ApiResponse(String),

    #[error("Failed to parse response: {0}")]
    ParseError(String),

    #[error("Platform detection failed")]
    PlatformDetectionFailed,
}

/// Response from the quota limit API endpoint.
#[derive(Debug, Deserialize)]
pub struct QuotaLimitResponse {
    /// Response code.
    #[expect(unused, reason = "deserialized from API, kept for documentation")]
    pub code: i32,
    /// Response message.
    pub msg: String,
    /// Quota limit data.
    pub data: QuotaLimitData,
    /// Whether the request was successful.
    pub success: bool,
}

/// Container for quota limit items from the API.
#[derive(Debug, Deserialize)]
pub struct QuotaLimitData {
    /// List of quota limit items.
    pub limits: Vec<QuotaLimitItem>,
}

impl QuotaLimitData {
    /// Find a quota matching the given type and unit.
    ///
    /// Returns `None` if no matching quota is found.
    #[must_use]
    pub fn find_quota(
        &self,
        quota_type: &str,
        unit: Option<i64>,
        time_window: &str,
    ) -> Option<QuotaUsage> {
        self.limits
            .iter()
            .find(|item| {
                if item.quota_type != quota_type {
                    return false;
                }
                match unit {
                    Some(u) => item.unit == u,
                    None => true,
                }
            })
            .map(|item| QuotaUsage {
                used: item.current_value,
                limit: item.usage,
                percentage: u8::try_from(item.percentage.clamp(0, 100)).unwrap_or(0),
                time_window: time_window.to_string(),
                reset_at: item.next_reset_time.map(|ms| ms / 1000),
            })
    }
}

/// Individual quota limit item from the API.
#[derive(Debug, Deserialize, Clone)]
pub struct QuotaLimitItem {
    /// Quota type identifier (e.g., "`TOKENS_LIMIT`", "`TIME_LIMIT`").
    #[serde(rename = "type")]
    pub quota_type: String,
    /// Time unit identifier.
    #[serde(default)]
    pub unit: i64,
    /// Total usage limit.
    #[serde(default)]
    pub usage: i64,
    /// Current usage value.
    #[serde(rename = "currentValue", default)]
    pub current_value: i64,
    /// Usage percentage (may exceed 100).
    pub percentage: i32,
    /// Next reset time as milliseconds since epoch.
    #[serde(rename = "nextResetTime", default)]
    pub next_reset_time: Option<i64>,
}

/// Response from the model usage API endpoint.
#[expect(dead_code, reason = "kept for API documentation")]
#[derive(Debug, Deserialize)]
pub struct ModelUsageResponse {
    pub code: Option<i32>,
    pub msg: Option<String>,
    pub data: Option<ModelUsageData>,
}

/// Token usage breakdown data.
#[expect(dead_code, reason = "kept for API documentation")]
#[derive(Debug, Deserialize)]
pub struct ModelUsageData {
    /// Total tokens used.
    pub total: Option<i64>,
    /// Tokens used for completions.
    pub completion_tokens: Option<i64>,
    /// Tokens used for prompts.
    pub prompt_tokens: Option<i64>,
}

/// Response from the tool usage API endpoint.
#[expect(dead_code, reason = "kept for API documentation")]
#[derive(Debug, Deserialize)]
pub struct ToolUsageResponse {
    pub code: Option<i32>,
    pub msg: Option<String>,
    pub data: Option<ToolUsageData>,
}

/// Tool usage total data.
#[expect(dead_code, reason = "kept for API documentation")]
#[derive(Debug, Deserialize)]
pub struct ToolUsageData {
    pub total: Option<i64>,
}

/// Combined usage statistics from all API endpoints.
#[derive(Debug, Clone)]
#[allow(
    clippy::struct_field_names,
    reason = "field names match API domain terminology"
)]
pub struct UsageStats {
    /// Token usage for current period (e.g., 5h limit).
    pub token_usage: Option<QuotaUsage>,
    /// Token usage for weekly period.
    pub weekly_usage: Option<QuotaUsage>,
    /// MCP tool usage for 30-day period.
    pub mcp_usage: Option<QuotaUsage>,
}

/// Individual quota usage with normalized percentage.
#[derive(Debug, Clone)]
pub struct QuotaUsage {
    /// Amount used.
    pub used: i64,
    /// Total limit.
    pub limit: i64,
    /// Usage percentage (0-100).
    pub percentage: u8,
    /// Time window for this quota (e.g., "5h", "weekly", "30d").
    #[allow(dead_code, reason = "kept for documentation and future use")]
    pub time_window: String,
    /// Reset timestamp in seconds (converted from milliseconds).
    pub reset_at: Option<i64>,
}

impl fmt::Display for QuotaUsage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}%", self.percentage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_data() -> QuotaLimitData {
        QuotaLimitData {
            limits: vec![
                QuotaLimitItem {
                    quota_type: "TOKENS_LIMIT".to_string(),
                    unit: 3,
                    usage: 100000,
                    current_value: 50000,
                    percentage: 50,
                    next_reset_time: Some(1700000000000),
                },
                QuotaLimitItem {
                    quota_type: "TOKENS_LIMIT".to_string(),
                    unit: 6,
                    usage: 500000,
                    current_value: 250000,
                    percentage: 50,
                    next_reset_time: Some(1700000000000),
                },
                QuotaLimitItem {
                    quota_type: "TIME_LIMIT".to_string(),
                    unit: 0,
                    usage: 100,
                    current_value: 30,
                    percentage: 30,
                    next_reset_time: None,
                },
            ],
        }
    }

    #[test]
    fn test_find_quota_by_type_and_unit() {
        let data = test_data();
        let result = data.find_quota("TOKENS_LIMIT", Some(3), "5h");
        assert!(result.is_some());
        let quota = result.unwrap();
        assert_eq!(quota.used, 50000);
        assert_eq!(quota.limit, 100000);
        assert_eq!(quota.percentage, 50);
        assert_eq!(quota.time_window, "5h");
    }

    #[test]
    fn test_find_quota_wrong_unit() {
        let data = test_data();
        let result = data.find_quota("TOKENS_LIMIT", Some(999), "5h");
        assert!(result.is_none());
    }

    #[test]
    fn test_find_quota_wrong_type() {
        let data = test_data();
        let result = data.find_quota("NONEXISTENT", Some(3), "5h");
        assert!(result.is_none());
    }

    #[test]
    fn test_find_quota_none_unit() {
        let data = test_data();
        let result = data.find_quota("TIME_LIMIT", None, "30d");
        assert!(result.is_some());
        let quota = result.unwrap();
        assert_eq!(quota.percentage, 30);
    }

    #[test]
    fn test_platform_detect_zai() {
        assert_eq!(
            Platform::detect_from_url("https://api.z.ai/v1"),
            Some(Platform::Zai)
        );
    }

    #[test]
    fn test_platform_detect_zhipu_bigmodel() {
        assert_eq!(
            Platform::detect_from_url("https://open.bigmodel.cn/api/anthropic"),
            Some(Platform::Zhipu)
        );
    }

    #[test]
    fn test_platform_detect_zhipu_keyword() {
        assert_eq!(
            Platform::detect_from_url("https://zhipu.example.com"),
            Some(Platform::Zhipu)
        );
    }

    #[test]
    fn test_platform_detect_unknown() {
        assert_eq!(Platform::detect_from_url("https://api.example.com"), None);
    }
}
