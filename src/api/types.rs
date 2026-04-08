use serde::Deserialize;
use std::fmt;
use thiserror::Error;

/// Platform detection from base URL
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    Zai,
    Zhipu,
}

impl Platform {
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

/// API error types
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

/// Quota limit response (actual ZHIPU API format)
#[derive(Debug, Deserialize)]
pub struct QuotaLimitResponse {
    #[allow(dead_code)]
    pub code: i32,
    pub msg: String,
    pub data: QuotaLimitData,
    pub success: bool,
}

#[derive(Debug, Deserialize)]
pub struct QuotaLimitData {
    pub limits: Vec<QuotaLimitItem>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct QuotaLimitItem {
    #[serde(rename = "type")]
    pub quota_type: String,
    #[serde(default)]
    pub usage: i64,
    #[serde(rename = "currentValue", default)]
    pub current_value: i64,
    pub percentage: i32,
    #[serde(rename = "nextResetTime", default)]
    pub next_reset_time: Option<i64>, // Millisecond timestamp
}

/// Model usage response
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct ModelUsageResponse {
    pub code: Option<i32>,
    pub msg: Option<String>,
    pub data: Option<ModelUsageData>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct ModelUsageData {
    pub total: Option<i64>,
    pub completion_tokens: Option<i64>,
    pub prompt_tokens: Option<i64>,
}

/// Tool usage response
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct ToolUsageResponse {
    pub code: Option<i32>,
    pub msg: Option<String>,
    pub data: Option<ToolUsageData>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct ToolUsageData {
    pub total: Option<i64>,
}

/// Combined usage statistics
#[derive(Debug, Clone)]
pub struct UsageStats {
    pub token_usage: Option<QuotaUsage>,
    pub mcp_usage: Option<QuotaUsage>,
}

/// Individual quota usage
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct QuotaUsage {
    pub used: i64,
    pub limit: i64,
    pub percentage: u8,
    pub time_window: String,
    pub reset_at: Option<i64>, // Second-level timestamp (converted from ms)
}

impl fmt::Display for QuotaUsage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}%", self.percentage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_from_url_zai_anthropic_path() {
        // Test 1: z.ai with /api/anthropic suffix should detect as Zai
        assert_eq!(
            Platform::detect_from_url("https://api.z.ai/api/anthropic"),
            Some(Platform::Zai)
        );
    }

    #[test]
    fn test_detect_from_url_zai_root() {
        // Test z.ai without suffix should still detect as Zai
        assert_eq!(
            Platform::detect_from_url("https://api.z.ai"),
            Some(Platform::Zai)
        );
    }

    #[test]
    fn test_detect_from_url_zhipu_cn_standard() {
        // Test 2: Standard CN input should detect as Zhipu
        assert_eq!(
            Platform::detect_from_url("https://open.bigmodel.cn/api/anthropic"),
            Some(Platform::Zhipu)
        );
    }

    #[test]
    fn test_detect_from_url_zhipu_cn_alternative() {
        // Test alternative CN domain patterns
        assert_eq!(
            Platform::detect_from_url("https://api.zhipu.com"),
            Some(Platform::Zhipu)
        );
    }

    #[test]
    fn test_detect_from_url_unknown_host() {
        // Test 3: Unknown hosts return None (fail closed)
        assert_eq!(
            Platform::detect_from_url("https://api.openai.com/v1"),
            None
        );
    }

    #[test]
    fn test_detect_from_url_empty() {
        // Edge case: empty string
        assert_eq!(Platform::detect_from_url(""), None);
    }
}
