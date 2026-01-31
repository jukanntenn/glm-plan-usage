use super::types::*;
use anyhow::Result;
use std::time::Duration;
use ureq::{Agent, Request};

/// GLM API client
pub struct GlmApiClient {
    agent: Agent,
    base_url: String,
    token: String,
    platform: Platform,
}

impl GlmApiClient {
    /// Create client from environment variables
    pub fn from_env() -> Result<Self> {
        let token = std::env::var("ANTHROPIC_AUTH_TOKEN")
            .map_err(|_| ApiError::MissingEnvVar("ANTHROPIC_AUTH_TOKEN".to_string()))?;

        let base_url = std::env::var("ANTHROPIC_BASE_URL")
            .unwrap_or_else(|_| "https://open.bigmodel.cn/api/anthropic".to_string());

        let platform = Platform::detect_from_url(&base_url)
            .ok_or(ApiError::PlatformDetectionFailed)?;

        // Fix base URL for ZHIPU platform (remove /anthropic suffix for monitor API)
        let base_url = if platform == Platform::ZHIPU {
            base_url
                .replace("/api/anthropic", "/api")
                .replace("/anthropic", "")
        } else {
            base_url
        };

        let agent = ureq::AgentBuilder::new()
            .timeout(Duration::from_secs(5))
            .build();

        Ok(Self {
            agent,
            base_url,
            token,
            platform,
        })
    }

    /// Fetch complete usage statistics (simplified - all data from quota/limit endpoint)
    pub fn fetch_usage_stats(&self) -> Result<UsageStats> {
        // Retry logic
        let mut last_error = None;

        for attempt in 0..=2 {
            match self.try_fetch_usage_stats() {
                Ok(stats) => return Ok(stats),
                Err(e) => {
                    last_error = Some(e);
                    if attempt < 2 {
                        std::thread::sleep(Duration::from_millis(100));
                    }
                }
            }
        }

        Err(last_error.unwrap())
    }

    fn try_fetch_usage_stats(&self) -> Result<UsageStats> {
        // Fetch quota limits (contains all the data we need)
        let url = format!("{}/monitor/usage/quota/limit", self.base_url);

        let response = self
            .authenticated_request(&url)
            .call()
            .map_err(|e| ApiError::HttpError(e.to_string()))?;

        if response.status() != 200 {
            return Err(ApiError::ApiError(format!(
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
            return Err(ApiError::ApiError(quota_response.msg).into());
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
                percentage: item.percentage.max(0).min(100) as u8,
                time_window: "5h".to_string(),
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
                percentage: item.percentage.max(0).min(100) as u8,
                time_window: "30d".to_string(),
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
