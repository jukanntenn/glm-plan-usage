use super::types::UsageStats;
use std::sync::{Arc, Mutex};
use std::time::Instant;

#[derive(Clone)]
pub struct SharedCache {
    data: Arc<Mutex<Option<(UsageStats, Instant)>>>,
}

impl SharedCache {
    pub fn new() -> Self {
        Self {
            data: Arc::new(Mutex::new(None)),
        }
    }

    pub fn get_or_fetch<F>(&self, ttl_seconds: u64, fetch: F) -> Option<UsageStats>
    where
        F: FnOnce() -> Option<UsageStats>,
    {
        let mut cache = self.data.lock().unwrap();

        if let Some((stats, timestamp)) = cache.as_ref() {
            if timestamp.elapsed().as_secs() < ttl_seconds {
                return Some(stats.clone());
            }
        }

        let stale = cache.as_ref().map(|(stats, _)| stats.clone());
        let fetched = fetch();

        if let Some(stats) = fetched {
            *cache = Some((stats.clone(), Instant::now()));
            Some(stats)
        } else {
            stale
        }
    }

    #[allow(dead_code)]
    pub fn clear(&self) {
        let mut cache = self.data.lock().unwrap();
        *cache = None;
    }
}

impl Default for SharedCache {
    fn default() -> Self {
        Self::new()
    }
}
