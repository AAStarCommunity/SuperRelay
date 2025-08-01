// Rate limiting middleware for RPC requests
use std::{
    collections::HashMap,
    net::IpAddr,
    sync::Arc,
    time::{Duration, Instant},
};

use jsonrpsee::types::ErrorObjectOwned;
use parking_lot::RwLock;
use tokio::time::{interval, MissedTickBehavior};

/// Token bucket for rate limiting
#[derive(Debug, Clone)]
pub(crate) struct TokenBucket {
    tokens: u32,
    capacity: u32,
    refill_rate: u32, // tokens per second
    last_refill: Instant,
}

impl TokenBucket {
    pub(crate) fn new(capacity: u32, refill_rate: u32) -> Self {
        Self {
            tokens: capacity,
            capacity,
            refill_rate,
            last_refill: Instant::now(),
        }
    }

    pub(crate) fn try_consume(&mut self, tokens: u32) -> bool {
        self.refill();

        if self.tokens >= tokens {
            self.tokens -= tokens;
            true
        } else {
            false
        }
    }

    fn refill(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill);
        let tokens_to_add = (elapsed.as_secs_f64() * self.refill_rate as f64) as u32;

        if tokens_to_add > 0 {
            self.tokens = (self.tokens + tokens_to_add).min(self.capacity);
            self.last_refill = now;
        }
    }
}

/// Rate limiter configuration
#[derive(Debug, Clone)]
pub struct RateLimiterConfig {
    /// Maximum requests per second per IP
    pub requests_per_second: u32,
    /// Burst capacity (tokens in bucket)
    pub burst_capacity: u32,
    /// Cleanup interval for expired entries
    pub cleanup_interval: Duration,
    /// Entry expiry time for inactive IPs
    pub entry_expiry: Duration,
}

impl Default for RateLimiterConfig {
    fn default() -> Self {
        Self {
            requests_per_second: 100,                  // 100 RPS per IP
            burst_capacity: 200,                       // Allow bursts up to 200
            cleanup_interval: Duration::from_secs(60), // Cleanup every minute
            entry_expiry: Duration::from_secs(300),    // Remove after 5 minutes
        }
    }
}

/// IP-based rate limiter
#[derive(Debug)]
pub struct RateLimiter {
    buckets: Arc<RwLock<HashMap<IpAddr, (TokenBucket, Instant)>>>,
    config: RateLimiterConfig,
}

impl RateLimiter {
    /// Create a new rate limiter with the given configuration
    pub fn new(config: RateLimiterConfig) -> Self {
        let limiter = Self {
            buckets: Arc::new(RwLock::new(HashMap::new())),
            config,
        };

        // Start cleanup task
        limiter.start_cleanup_task();

        limiter
    }

    /// Check if request is allowed for given IP
    pub fn check_rate_limit(&self, ip: IpAddr) -> bool {
        let mut buckets = self.buckets.write();
        let now = Instant::now();

        let entry = buckets.entry(ip).or_insert_with(|| {
            (
                TokenBucket::new(self.config.burst_capacity, self.config.requests_per_second),
                now,
            )
        });

        // Update last access time and check rate limit
        entry.1 = now;
        entry.0.try_consume(1)
    }

    /// Get current statistics
    pub fn get_stats(&self) -> RateLimiterStats {
        let buckets = self.buckets.read();
        RateLimiterStats {
            active_ips: buckets.len(),
            total_buckets: buckets.len(),
        }
    }

    fn start_cleanup_task(&self) {
        let buckets = Arc::clone(&self.buckets);
        let expiry_duration = self.config.entry_expiry;
        let cleanup_interval = self.config.cleanup_interval;

        tokio::spawn(async move {
            let mut interval = interval(cleanup_interval);
            interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

            loop {
                interval.tick().await;

                let now = Instant::now();
                let mut buckets = buckets.write();

                // Remove expired entries
                let expired_count = buckets.len();
                buckets.retain(|_ip, (_, last_access)| {
                    now.duration_since(*last_access) < expiry_duration
                });

                let removed = expired_count - buckets.len();
                if removed > 0 {
                    tracing::debug!("Rate limiter cleanup: removed {} expired entries", removed);
                }
            }
        });
    }
}

#[derive(Debug)]
/// Statistics for rate limiter monitoring
pub struct RateLimiterStats {
    /// Number of currently active IP addresses
    pub active_ips: usize,
    /// Total number of token buckets
    pub total_buckets: usize,
}

/// Rate limiting error
#[allow(dead_code)]
pub(crate) fn rate_limit_error() -> ErrorObjectOwned {
    ErrorObjectOwned::owned(
        -32005, // Custom error code for rate limiting
        "Rate limit exceeded",
        Some("Too many requests. Please slow down."),
    )
}

/// Extract client IP from request
#[allow(dead_code)]
pub(crate) fn extract_client_ip(
    headers: &http::HeaderMap,
    remote_addr: Option<std::net::SocketAddr>,
) -> Option<IpAddr> {
    // Try X-Forwarded-For header first (for reverse proxies)
    if let Some(forwarded) = headers.get("x-forwarded-for") {
        if let Ok(forwarded_str) = forwarded.to_str() {
            if let Some(ip) = forwarded_str.split(',').next() {
                if let Ok(parsed_ip) = ip.trim().parse::<IpAddr>() {
                    return Some(parsed_ip);
                }
            }
        }
    }

    // Try X-Real-IP header
    if let Some(real_ip) = headers.get("x-real-ip") {
        if let Ok(ip_str) = real_ip.to_str() {
            if let Ok(parsed_ip) = ip_str.parse::<IpAddr>() {
                return Some(parsed_ip);
            }
        }
    }

    // Fall back to remote address
    remote_addr.map(|addr| addr.ip())
}

#[cfg(test)]
mod tests {
    use std::net::Ipv4Addr;

    use super::*;

    #[test]
    fn test_token_bucket() {
        let mut bucket = TokenBucket::new(10, 5);

        // Should allow initial requests up to capacity
        for _ in 0..10 {
            assert!(bucket.try_consume(1));
        }

        // Should reject when empty
        assert!(!bucket.try_consume(1));
    }

    #[tokio::test]
    async fn test_rate_limiter() {
        let config = RateLimiterConfig {
            requests_per_second: 2,
            burst_capacity: 5,
            cleanup_interval: Duration::from_secs(60),
            entry_expiry: Duration::from_secs(300),
        };

        let limiter = RateLimiter::new(config);
        let test_ip = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));

        // Should allow initial burst
        for _ in 0..5 {
            assert!(limiter.check_rate_limit(test_ip));
        }

        // Should reject after burst exhausted
        assert!(!limiter.check_rate_limit(test_ip));
    }
}
