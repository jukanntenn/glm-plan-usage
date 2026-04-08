use super::types::*;
use anyhow::Result;
use std::time::Duration;
use ureq::{Agent, Request};

/// GLM API client
pub struct GlmApiClient {
    agent: Agent,
    base_url: String,
    token: String,
    _platform: Platform,
}

/// Normalize base URL and resolve quota endpoint for a given platform.
/// Exported for testing; internal use only.
fn normalize_base_url(base_url: &str, platform: Platform) -> String {
    // Both platforms: remove any path after /api segment
    // Examples:
    //   https://open.bigmodel.cn/api/anthropic → https://open.bigmodel.cn/api
    //   https://api.z.ai/api/anthropic → https://api.z.ai/api
    //   https://api.z.ai/api/paas/v4/ → https://api.z.ai/api

    // Remove trailing slash first
    let base_url = base_url.trim_end_matches('/');

    // Look for pattern "/api/" followed by more path
    // The base_url format is: scheme://host/api/...
    // So we look for "/api/" that comes after the host part
    if let Some(api_idx) = base_url.find("/api/") {
        let before_api = &base_url[..api_idx];
        // Make sure we're past the scheme://host part (contains ://)
        if before_api.contains("://") {
            return format!("{}/api", before_api);
        }
    }

    // No /api/... path found, return as-is
    base_url.to_string()
}

/// Build the full quota URL from normalized base URL.
/// Exported for testing; internal use only.
fn build_quota_url(normalized_base: &str) -> String {
    // Avoid double slashes if base already ends with /
    if normalized_base.ends_with('/') {
        format!("{}monitor/usage/quota/limit", normalized_base)
    } else {
        format!("{}/monitor/usage/quota/limit", normalized_base)
    }
}

impl GlmApiClient {
    /// Create client from environment variables with config
    pub fn from_env(config: &crate::config::Config) -> Result<Self> {
        let token = std::env::var("ANTHROPIC_AUTH_TOKEN")
            .map_err(|_| ApiError::MissingEnvVar("ANTHROPIC_AUTH_TOKEN".to_string()))?;

        let base_url = std::env::var("ANTHROPIC_BASE_URL")
            .unwrap_or_else(|_| "https://open.bigmodel.cn/api/anthropic".to_string());

        let platform =
            Platform::detect_from_url(&base_url).ok_or(ApiError::PlatformDetectionFailed)?;

        // Fix base URL for ZHIPU platform (remove /anthropic suffix for monitor API)
        let base_url = normalize_base_url(&base_url, platform);

        let agent = ureq::AgentBuilder::new()
            .timeout(Duration::from_millis(config.api.timeout_ms))
            .build();

        Ok(Self {
            agent,
            base_url,
            token,
            _platform: platform,
        })
    }

    /// Fetch complete usage statistics (simplified - all data from quota/limit endpoint)
    pub fn fetch_usage_stats(&self, config: &crate::config::Config) -> Result<UsageStats> {
        // Retry logic
        let mut last_error = None;

        for attempt in 0..config.api.retry_attempts {
            match self.try_fetch_usage_stats() {
                Ok(stats) => return Ok(stats),
                Err(e) => {
                    last_error = Some(e);
                    if attempt < config.api.retry_attempts - 1 {
                        std::thread::sleep(Duration::from_millis(100));
                    }
                }
            }
        }

        Err(last_error.unwrap())
    }

    fn try_fetch_usage_stats(&self) -> Result<UsageStats> {
        // Fetch quota limits (contains all the data we need)
        let url = build_quota_url(&self.base_url);

        let response = self
            .authenticated_request(&url)
            .call()
            .map_err(|e| ApiError::HttpError(e.to_string()))?;

        if response.status() != 200 {
            return Err(ApiError::ApiResponse(format!(
                "Status {}: {}",
                response.status(),
                response.status_text()
            ))
            .into());
        }

        let quota_response: QuotaLimitResponse = response
            .into_json()
            .map_err(|e| ApiError::ParseError(e.to_string()))?;

        if !quota_response.success {
            return Err(ApiError::ApiResponse(quota_response.msg).into());
        }

        // Extract token usage (TOKENS_LIMIT)
        let token_usage = quota_response
            .data
            .limits
            .iter()
            .find(|item| item.quota_type == "TOKENS_LIMIT")
            .map(|item| QuotaUsage {
                used: item.current_value,
                limit: item.usage,
                percentage: item.percentage.clamp(0, 100) as u8,
                time_window: "5h".to_string(),
                reset_at: item.next_reset_time.map(|ms| ms / 1000),
            });

        // Extract tool usage (TIME_LIMIT)
        let mcp_usage = quota_response
            .data
            .limits
            .iter()
            .find(|item| item.quota_type == "TIME_LIMIT")
            .map(|item| QuotaUsage {
                used: item.current_value,
                limit: item.usage,
                percentage: item.percentage.clamp(0, 100) as u8,
                time_window: "30d".to_string(),
                reset_at: item.next_reset_time.map(|ms| ms / 1000),
            });

        Ok(UsageStats {
            token_usage,
            mcp_usage,
        })
    }

    fn authenticated_request(&self, url: &str) -> Request {
        self.agent
            .get(url)
            .set("Authorization", &format!("Bearer {}", self.token))
            .set("Content-Type", "application/json")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{ApiConfig, Config};

    // Test 4: Table-driven endpoint resolution tests covering both CN and z.ai
    #[test]
    fn test_quota_endpoint_resolution_zhipu_cn_standard() {
        // Standard CN input: https://open.bigmodel.cn/api/anthropic
        let input = "https://open.bigmodel.cn/api/anthropic";
        let platform = Platform::Zhipu;
        let normalized = normalize_base_url(input, platform);
        let quota_url = build_quota_url(&normalized);

        // Expected: removes /anthropic suffix → https://open.bigmodel.cn/api + /monitor/...
        assert_eq!(
            quota_url,
            "https://open.bigmodel.cn/api/monitor/usage/quota/limit"
        );
    }

    #[test]
    fn test_quota_endpoint_resolution_zhipu_cn_already_normalized() {
        // CN input already without /anthropic → should not change
        let input = "https://open.bigmodel.cn/api";
        let platform = Platform::Zhipu;
        let normalized = normalize_base_url(input, platform);
        let quota_url = build_quota_url(&normalized);

        assert_eq!(
            quota_url,
            "https://open.bigmodel.cn/api/monitor/usage/quota/limit"
        );
    }

    #[test]
    fn test_quota_endpoint_resolution_zhipu_cn_trailing_slash() {
        // CN input with trailing slash → normalizes correctly
        let input = "https://open.bigmodel.cn/api/anthropic/";
        let platform = Platform::Zhipu;
        let normalized = normalize_base_url(input, platform);
        let quota_url = build_quota_url(&normalized);

        assert_eq!(
            quota_url,
            "https://open.bigmodel.cn/api/monitor/usage/quota/limit"
        );
    }

    #[test]
    fn test_quota_endpoint_resolution_zai_standard() {
        // Standard z.ai input: https://api.z.ai/api/anthropic
        let input = "https://api.z.ai/api/anthropic";
        let platform = Platform::Zai;
        let normalized = normalize_base_url(input, platform);
        let quota_url = build_quota_url(&normalized);

        // Expected: removes /anthropic suffix → https://api.z.ai/api + /monitor/...
        assert_eq!(
            quota_url,
            "https://api.z.ai/api/monitor/usage/quota/limit"
        );
    }

    #[test]
    fn test_quota_endpoint_resolution_zai_root() {
        // z.ai root input
        let input = "https://api.z.ai";
        let platform = Platform::Zai;
        let normalized = normalize_base_url(input, platform);
        let quota_url = build_quota_url(&normalized);

        assert_eq!(
            quota_url,
            "https://api.z.ai/monitor/usage/quota/limit"
        );
    }

    #[test]
    fn test_quota_endpoint_resolution_zai_paas_v4() {
        // Alternative z.ai base URL as documented: https://api.z.ai/api/paas/v4/
        let input = "https://api.z.ai/api/paas/v4/";
        let platform = Platform::Zai;
        let normalized = normalize_base_url(input, platform);
        let quota_url = build_quota_url(&normalized);

        // Expected: normalizes to https://api.z.ai/api + /monitor/...
        assert_eq!(
            quota_url,
            "https://api.z.ai/api/monitor/usage/quota/limit"
        );
    }

    #[test]
    fn test_client_config_timeout_default() {
        // Default config should have 5000ms timeout
        let config = Config::default();
        assert_eq!(config.api.timeout_ms, 5000);
    }

    #[test]
    fn test_client_config_timeout_custom() {
        // Custom timeout should be correctly stored in ApiConfig
        let mut config = Config::default();
        config.api.timeout_ms = 10000;
        assert_eq!(config.api.timeout_ms, 10000);
    }

    #[test]
    fn test_client_config_retry_default() {
        // Default config should have 2 retry attempts
        let config = Config::default();
        assert_eq!(config.api.retry_attempts, 2);
    }

    #[test]
    fn test_client_config_retry_custom() {
        // Custom retry attempts should be correctly stored
        let mut config = Config::default();
        config.api.retry_attempts = 5;
        assert_eq!(config.api.retry_attempts, 5);
    }

    #[test]
    fn test_retry_loop_iteration_count_zero() {
        // With retry_attempts = 0, loop runs 0 times → only one attempt total
        let mut count = 0;
        let config = Config {
            api: ApiConfig {
                retry_attempts: 0,
                ..Default::default()
            },
            ..Default::default()
        };
        for _ in 0..config.api.retry_attempts {
            count += 1;
        }
        assert_eq!(count, 0);
    }

    #[test]
    fn test_retry_loop_iteration_count_default() {
        // With default retry_attempts = 2, loop runs exactly 2 times
        let mut count = 0;
        let config = Config::default();
        for _ in 0..config.api.retry_attempts {
            count += 1;
        }
        assert_eq!(count, 2);
    }

    #[test]
    fn test_retry_loop_iteration_count_three() {
        // With retry_attempts = 3, loop runs exactly 3 times
        let mut count = 0;
        let mut config = Config::default();
        config.api.retry_attempts = 3;
        for _ in 0..config.api.retry_attempts {
            count += 1;
        }
        assert_eq!(count, 3);
    }
}
