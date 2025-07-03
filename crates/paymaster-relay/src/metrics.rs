//! Prometheus metrics for PaymasterRelay
//!
//! This module provides comprehensive monitoring for paymaster operations,
//! including request metrics, business KPIs, and system health indicators.

use std::time::{Duration, Instant};

use anyhow::Result;
use metrics::{counter, gauge, histogram};

/// PaymasterRelay metrics collection and reporting
#[derive(Debug, Clone)]
pub struct PaymasterMetrics {
    /// Service start time for uptime calculation
    pub start_time: Instant,
}

impl PaymasterMetrics {
    /// Create a new metrics instance
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
        }
    }

    /// Record a successful paymaster request
    pub fn record_request_success(&self, duration: Duration) {
        counter!("paymaster_requests_total", "status" => "success").increment(1);
        histogram!("paymaster_request_duration_seconds").record(duration.as_secs_f64());
    }

    /// Record a failed paymaster request
    pub fn record_request_failure(&self, error_type: &str, duration: Duration) {
        counter!("paymaster_requests_total", "status" => "error", "error_type" => error_type.to_string()).increment(1);
        histogram!("paymaster_request_duration_seconds").record(duration.as_secs_f64());
    }

    /// Record gas sponsored amount
    pub fn record_gas_sponsored(&self, amount: u64) {
        counter!("paymaster_gas_sponsored_total").increment(amount);
        gauge!("paymaster_gas_sponsored_latest").set(amount as f64);
    }

    /// Record policy violation
    pub fn record_policy_violation(&self, policy_type: &str) {
        counter!("paymaster_policy_violations_total", "policy_type" => policy_type.to_string())
            .increment(1);
    }

    /// Record signature operation
    pub fn record_signature_operation(&self, success: bool, duration: Duration) {
        let status = if success { "success" } else { "failure" };
        counter!("paymaster_signature_operations_total", "status" => status.to_string())
            .increment(1);
        histogram!("paymaster_signature_duration_seconds").record(duration.as_secs_f64());
    }

    /// Record pool submission
    pub fn record_pool_submission(&self, success: bool) {
        let status = if success { "success" } else { "failure" };
        counter!("paymaster_pool_submissions_total", "status" => status.to_string()).increment(1);
    }

    /// Update success rate gauge
    pub fn update_success_rate(&self, success_rate: f64) {
        gauge!("paymaster_success_rate").set(success_rate);
    }

    /// Update active connections gauge
    pub fn update_active_connections(&self, count: u64) {
        gauge!("paymaster_active_connections").set(count as f64);
    }

    /// Update memory usage gauge
    pub fn update_memory_usage(&self, usage_mb: u64) {
        gauge!("paymaster_memory_usage_mb").set(usage_mb as f64);
    }

    /// Update request queue depth
    pub fn update_queue_depth(&self, depth: u64) {
        gauge!("paymaster_queue_depth").set(depth as f64);
    }

    /// Record UserOperation validation
    pub fn record_validation(&self, success: bool, duration: Duration) {
        let status = if success { "success" } else { "failure" };
        counter!("paymaster_validations_total", "status" => status.to_string()).increment(1);
        histogram!("paymaster_validation_duration_seconds").record(duration.as_secs_f64());
    }

    /// Get service uptime
    pub fn get_uptime(&self) -> Duration {
        self.start_time.elapsed()
    }

    /// Record error by category
    pub fn record_error(&self, category: &str, error: &str) {
        counter!("paymaster_errors_total", "category" => category.to_string(), "error" => error.to_string()).increment(1);
    }

    /// Record response time histogram
    pub fn record_response_time(&self, endpoint: &str, duration: Duration) {
        histogram!("paymaster_response_time_seconds", "endpoint" => endpoint.to_string())
            .record(duration.as_secs_f64());
    }

    /// Update health status
    pub fn update_health_status(&self, healthy: bool) {
        gauge!("paymaster_health_status").set(if healthy { 1.0 } else { 0.0 });
    }
}

impl Default for PaymasterMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Format duration for human-readable display
pub fn format_duration(duration: Duration) -> String {
    let total_seconds = duration.as_secs();
    let days = total_seconds / 86400;
    let hours = (total_seconds % 86400) / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;

    if days > 0 {
        format!("{}d {}h {}m {}s", days, hours, minutes, seconds)
    } else if hours > 0 {
        format!("{}h {}m {}s", hours, minutes, seconds)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, seconds)
    } else {
        format!("{}s", seconds)
    }
}

/// Metrics initialization and setup
pub fn init_metrics() -> Result<()> {
    // Initialize metrics recorder
    // This is typically done by the metrics-exporter-prometheus crate
    // when starting the Prometheus exporter
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(Duration::from_secs(30)), "30s");
        assert_eq!(format_duration(Duration::from_secs(90)), "1m 30s");
        assert_eq!(format_duration(Duration::from_secs(3661)), "1h 1m 1s");
        assert_eq!(format_duration(Duration::from_secs(90061)), "1d 1h 1m 1s");
    }

    #[test]
    fn test_metrics_creation() {
        let metrics = PaymasterMetrics::new();
        assert!(metrics.get_uptime().as_nanos() > 0);
    }

    #[test]
    fn test_record_operations() {
        let metrics = PaymasterMetrics::new();

        // Test recording various operations
        metrics.record_request_success(Duration::from_millis(100));
        metrics.record_request_failure("validation_error", Duration::from_millis(50));
        metrics.record_gas_sponsored(1000000);
        metrics.record_policy_violation("rate_limit");
        metrics.record_signature_operation(true, Duration::from_millis(10));
        metrics.record_pool_submission(true);

        // Test gauge updates
        metrics.update_success_rate(0.95);
        metrics.update_active_connections(10);
        metrics.update_memory_usage(256);
        metrics.update_queue_depth(5);
        metrics.update_health_status(true);
    }
}
