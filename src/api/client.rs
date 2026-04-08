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
    match platform {
        Platform::Zhipu => base_url
            .replace("/api/anthropic", "/api")
            .replace("/anthropic", ""),
        Platform::Zai => base_url.to_string(),
    }
}

/// Build the full quota URL from normalized base URL.
/// Exported for testing; internal use only.
fn build_quota_url(normalized_base: &str) -> String {
    format!("{}/monitor/usage/quota/limit", normalized_base)
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
        // CN input with trailing slash → still works
        let input = "https://open.bigmodel.cn/api/anthropic/";
        let platform = Platform::Zhipu;
        let normalized = normalize_base_url(input, platform);
        let quota_url = build_quota_url(&normalized);

        // Note: double slash is okay - URL parser will handle it
        assert!(quota_url.contains("open.bigmodel.cn/api/monitor"));
    }

    #[test]
    fn test_quota_endpoint_resolution_zai_standard() {
        // Test 1: Standard z.ai input: https://api.z.ai/api/anthropic
        let input = "https://api.z.ai/api/anthropic";
        let platform = Platform::Zai;
        let normalized = normalize_base_url(input, platform);
        let quota_url = build_quota_url(&normalized);

        // Expected: no normalization → keep as-is then append monitor path
        assert_eq!(
            quota_url,
            "https://api.z.ai/api/anthropic/monitor/usage/quota/limit"
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

        // Expected: no normalization for z.ai, keep all path segments
        assert_eq!(
            quota_url,
            "https://api.z.ai/api/paas/v4//monitor/usage/quota/limit"
        );
        // Double trailing slash is acceptable - HTTP handles it
    }
}
