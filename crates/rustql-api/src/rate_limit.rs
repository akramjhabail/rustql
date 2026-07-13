use std::sync::Arc;
use std::collections::HashMap;
use std::time::{Instant, Duration};
use tokio::sync::Mutex;

#[derive(Debug, Clone)]
pub struct RateLimiter {
    requests: Arc<Mutex<HashMap<String, Vec<Instant>>>>,
    max_requests: u32,
    window_secs: u64,
}

impl RateLimiter {
    pub fn new(max_requests: u32, window_secs: u64) -> Self {
        RateLimiter {
            requests: Arc::new(Mutex::new(HashMap::new())),
            max_requests,
            window_secs,
        }
    }

    pub async fn check(&self, ip: &str) -> bool {
        let mut requests = self.requests.lock().await;
        let now = Instant::now();
        let window = Duration::from_secs(self.window_secs);
        let entry = requests.entry(ip.to_string()).or_insert_with(Vec::new);
        entry.retain(|t| now.duration_since(*t) < window);
        if entry.len() < self.max_requests as usize {
            entry.push(now);
            true
        } else {
            false
        }
    }

    pub async fn remaining(&self, ip: &str) -> u32 {
        let requests = self.requests.lock().await;
        let now = Instant::now();
        let window = Duration::from_secs(self.window_secs);
        if let Some(entry) = requests.get(ip) {
            let active = entry.iter()
                .filter(|t| now.duration_since(**t) < window)
                .count();
            self.max_requests.saturating_sub(active as u32)
        } else {
            self.max_requests
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rate_limiter_allows_requests() {
        let limiter = RateLimiter::new(5, 60);
        assert!(limiter.check("127.0.0.1").await);
        assert!(limiter.check("127.0.0.1").await);
        assert!(limiter.check("127.0.0.1").await);
    }

    #[tokio::test]
    async fn test_rate_limiter_blocks_excess() {
        let limiter = RateLimiter::new(3, 60);
        limiter.check("127.0.0.1").await;
        limiter.check("127.0.0.1").await;
        limiter.check("127.0.0.1").await;
        assert!(!limiter.check("127.0.0.1").await);
    }

    #[tokio::test]
    async fn test_different_ips() {
        let limiter = RateLimiter::new(2, 60);
        limiter.check("192.168.1.1").await;
        limiter.check("192.168.1.1").await;
        assert!(!limiter.check("192.168.1.1").await);
        assert!(limiter.check("192.168.1.2").await);
    }
}