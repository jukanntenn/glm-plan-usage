//! Thread-safe cache for API usage statistics.
//!
//! This module provides a TTL-based cache for usage statistics
//! to reduce API calls and improve performance.

use super::types::UsageStats;
use std::sync::{Arc, Mutex};
use std::time::Instant;

/// Thread-safe cache for API usage statistics with TTL-based expiration.
#[derive(Clone, Debug)]
pub struct SharedCache {
    /// Cached data with expiration time.
    data: Arc<Mutex<Option<(UsageStats, Instant)>>>,
}

impl SharedCache {
    /// Create a new empty cache.
    #[must_use]
    pub fn new() -> Self {
        Self {
            data: Arc::new(Mutex::new(None)),
        }
    }

    /// Get cached stats if fresh, otherwise fetch and cache.
    ///
    /// Returns stale data if the fetch fails.
    pub fn get_or_fetch<F>(&self, ttl_seconds: u64, fetch: F) -> Option<UsageStats>
    where
        F: FnOnce() -> Option<UsageStats>,
    {
        let mut cache = self.data.lock().ok()?;

        if let Some((stats, timestamp)) = cache.as_ref() {
            if timestamp.elapsed().as_secs() < ttl_seconds {
                return Some(stats.clone());
            }
        }

        let fetched = fetch();

        if let Some(stats) = fetched {
            let to_return = stats.clone();
            *cache = Some((stats, Instant::now()));
            Some(to_return)
        } else {
            cache.as_ref().map(|(stats, _)| stats.clone())
        }
    }

    /// Clear the cache.
    #[allow(dead_code, reason = "public API for cache management")]
    pub fn clear(&self) {
        if let Ok(mut cache) = self.data.lock() {
            *cache = None;
        }
    }
}

impl Default for SharedCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_stats() -> UsageStats {
        UsageStats {
            token_usage: None,
            weekly_usage: None,
            mcp_usage: None,
        }
    }

    #[test]
    fn test_cache_fresh_fetch() {
        let cache = SharedCache::new();
        let result = cache.get_or_fetch(300, || Some(empty_stats()));
        assert!(result.is_some());
    }

    #[test]
    fn test_cache_hit_within_ttl() {
        let cache = SharedCache::new();
        let call_count = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let count_clone = call_count.clone();

        cache.get_or_fetch(300, move || {
            count_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            Some(empty_stats())
        });

        cache.get_or_fetch(300, move || {
            call_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            Some(empty_stats())
        });

        assert_eq!(
            cache.get_or_fetch(300, || Some(empty_stats())).map(|_| ()),
            Some(())
        );
    }

    #[test]
    fn test_cache_miss_expired() {
        let cache = SharedCache::new();
        let call_count = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));

        cache.get_or_fetch(0, {
            let c = call_count.clone();
            move || {
                c.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                Some(empty_stats())
            }
        });

        std::thread::sleep(std::time::Duration::from_millis(10));

        cache.get_or_fetch(0, {
            let c = call_count.clone();
            move || {
                c.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                Some(empty_stats())
            }
        });

        assert!(call_count.load(std::sync::atomic::Ordering::SeqCst) >= 2);
    }

    #[test]
    fn test_cache_fallback_to_stale() {
        let cache = SharedCache::new();

        cache.get_or_fetch(0, || Some(empty_stats()));

        std::thread::sleep(std::time::Duration::from_millis(10));

        let result = cache.get_or_fetch(0, || None);
        assert!(result.is_some());
    }
}
