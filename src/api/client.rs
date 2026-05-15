//! API client for GLM usage data endpoints.
//!
//! This module provides the `GlmApiClient` for fetching usage statistics
//! from the GLM/ZHIPU API.

use super::types::{ApiError, Platform, QuotaLimitResponse, UsageStats};
use anyhow::Result;
use std::time::Duration;
use ureq::{Agent, Request};

const AUTH_TOKEN_ENV: &str = "ANTHROPIC_AUTH_TOKEN";
const BASE_URL_ENV: &str = "ANTHROPIC_BASE_URL";
const DEFAULT_BASE_URL: &str = "https://open.bigmodel.cn/api/anthropic";
const QUOTA_LIMIT_ENDPOINT: &str = "/monitor/usage/quota/limit";

/// Quota type for token-based limits.
const TOKENS_LIMIT: &str = "TOKENS_LIMIT";

/// Quota type for time-based limits (MCP tool usage).
const TIME_LIMIT: &str = "TIME_LIMIT";

/// Period unit value for the 5-hour billing window.
///
/// The API identifies the short-period token quota by unit=3.
const PERIOD_UNIT_5H: i64 = 3;

/// Period unit value for the weekly billing window.
///
/// The API identifies the weekly token quota by unit=6.
const PERIOD_UNIT_WEEKLY: i64 = 6;

/// Delay between retry attempts in milliseconds.
///
/// Short delay to allow transient network issues to resolve without
/// overloading the API. Too short and we hammer the server; too long
/// and we degrade user experience.
const RETRY_DELAY_MS: u64 = 100;

/// GLM API client
#[derive(Debug)]
pub struct GlmApiClient {
    /// HTTP agent for making requests.
    agent: Agent,
    /// Base URL for the API.
    base_url: String,
    /// Authentication token.
    token: String,
    /// Number of retry attempts on failure.
    retry_attempts: u32,
}

impl GlmApiClient {
    /// Create a new API client from environment variables.
    ///
    /// Reads `ANTHROPIC_AUTH_TOKEN` and `ANTHROPIC_BASE_URL` from the environment.
    ///
    /// # Errors
    ///
    /// Returns an error if the auth token is missing or the platform cannot be detected.
    pub fn from_env(timeout: Duration, retry_attempts: u32) -> Result<Self> {
        let token = std::env::var(AUTH_TOKEN_ENV)
            .map_err(|e| ApiError::MissingEnvVar(format!("{AUTH_TOKEN_ENV}: {e}")))?;

        let base_url = std::env::var(BASE_URL_ENV).unwrap_or_else(|_| DEFAULT_BASE_URL.to_string());

        let platform =
            Platform::detect_from_url(&base_url).ok_or(ApiError::PlatformDetectionFailed)?;

        // Fix base URL for ZHIPU platform (remove /anthropic suffix for monitor API)
        let base_url = if platform == Platform::Zhipu {
            base_url.replace("/anthropic", "")
        } else {
            base_url
        };

        let agent = ureq::AgentBuilder::new().timeout(timeout).build();

        Ok(Self {
            agent,
            base_url,
            token,
            retry_attempts,
        })
    }

    /// Fetch usage statistics from the GLM API.
    ///
    /// Retries up to `retry_attempts` times with 100ms delays between attempts.
    ///
    /// # Errors
    ///
    /// Returns an error if all API attempts fail due to HTTP errors, API errors, or parse failures.
    pub fn fetch_usage_stats(&self) -> Result<UsageStats> {
        for _ in 0..self.retry_attempts {
            match self.try_fetch_usage_stats() {
                Ok(stats) => return Ok(stats),
                Err(_) => std::thread::sleep(Duration::from_millis(RETRY_DELAY_MS)),
            }
        }
        self.try_fetch_usage_stats()
    }

    /// Attempts to fetch usage stats from the API once.
    fn try_fetch_usage_stats(&self) -> Result<UsageStats> {
        // Fetch quota limits (contains all the data we need)
        let url = format!("{}{}", self.base_url, QUOTA_LIMIT_ENDPOINT);

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

        let token_usage = quota_response
            .data
            .find_quota(TOKENS_LIMIT, Some(PERIOD_UNIT_5H), "5h");
        let weekly_usage =
            quota_response
                .data
                .find_quota(TOKENS_LIMIT, Some(PERIOD_UNIT_WEEKLY), "weekly");
        let mcp_usage = quota_response.data.find_quota(TIME_LIMIT, None, "30d");

        Ok(UsageStats {
            token_usage,
            weekly_usage,
            mcp_usage,
        })
    }

    /// Creates an authenticated HTTP request with Bearer token and content type.
    fn authenticated_request(&self, url: &str) -> Request {
        self.agent
            .get(url)
            .set("Authorization", &format!("Bearer {}", self.token))
            .set("Content-Type", "application/json")
    }
}
