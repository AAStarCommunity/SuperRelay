use std::{sync::Arc, time::SystemTime};

use axum::{extract::State, http::StatusCode, response::Json, routing::get, Router};
use rundler_paymaster_relay::PaymasterRelayService;
use serde::{Deserialize, Serialize};
use tokio::time::Instant;
use tracing::{debug, error, info, warn};

use crate::{gateway::GatewayState, router::GatewayRouter};

/// Health check response structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    /// Overall system status
    pub status: SystemStatus,
    /// Timestamp of the check
    pub timestamp: u64,
    /// Uptime in seconds
    pub uptime_seconds: u64,
    /// Component status details
    pub components: ComponentsStatus,
    /// System metrics
    pub metrics: SystemMetrics,
}

/// Overall system status enum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SystemStatus {
    /// All systems operating normally
    Healthy,
    /// Some systems have warnings
    Degraded,
    /// Critical system issues detected
    Unhealthy,
}

/// Component status details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentsStatus {
    /// Gateway service status
    pub gateway: ComponentHealth,
    /// Paymaster service status
    pub paymaster: ComponentHealth,
    /// Pool service status
    pub pool: ComponentHealth,
    /// Router service status
    pub router: ComponentHealth,
}

/// Individual component health
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    /// Component status
    pub status: ComponentStatus,
    /// Last check timestamp
    pub last_check: u64,
    /// Response time in milliseconds
    pub response_time_ms: Option<u64>,
    /// Error message if unhealthy
    pub error: Option<String>,
}

/// Component status enum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ComponentStatus {
    /// Component operating normally
    Healthy,
    /// Component has warnings
    Warning,
    /// Component has errors
    Error,
    /// Component status unknown
    Unknown,
}

/// System metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    /// Memory usage in MB
    pub memory_usage_mb: f64,
    /// Active connections count
    pub active_connections: u32,
    /// Total requests processed
    pub total_requests: u64,
    /// Error rate (percentage)
    pub error_rate: f64,
}

/// Health checker service
#[derive(Clone)]
pub struct HealthChecker {
    start_time: Instant,
    request_counter: Arc<std::sync::atomic::AtomicU64>,
    error_counter: Arc<std::sync::atomic::AtomicU64>,
}

impl Default for HealthChecker {
    fn default() -> Self {
        Self::new()
    }
}

impl HealthChecker {
    /// Create a new health checker
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            request_counter: Arc::new(std::sync::atomic::AtomicU64::new(0)),
            error_counter: Arc::new(std::sync::atomic::AtomicU64::new(0)),
        }
    }

    /// Register a request
    pub fn record_request(&self) {
        self.request_counter
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    /// Register an error
    pub fn record_error(&self) {
        self.error_counter
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    /// Perform comprehensive health check
    pub async fn check_health(&self, state: &GatewayState) -> HealthStatus {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let uptime = self.start_time.elapsed().as_secs();

        // Check individual components
        let gateway_health = self.check_gateway_health().await;
        let paymaster_health = self.check_paymaster_health(&state.paymaster_service).await;
        let pool_health = self.check_pool_health().await;
        let router_health = self.check_router_health(&state.router).await;

        // Determine overall status
        let overall_status = self.determine_overall_status(&[
            &gateway_health,
            &paymaster_health,
            &pool_health,
            &router_health,
        ]);

        // Collect system metrics
        let metrics = self.collect_system_metrics().await;

        HealthStatus {
            status: overall_status,
            timestamp: now,
            uptime_seconds: uptime,
            components: ComponentsStatus {
                gateway: gateway_health,
                paymaster: paymaster_health,
                pool: pool_health,
                router: router_health,
            },
            metrics,
        }
    }

    /// Check gateway component health
    async fn check_gateway_health(&self) -> ComponentHealth {
        let start = Instant::now();

        // Basic gateway health check - verify we can process requests
        let status = ComponentStatus::Healthy;
        let response_time = start.elapsed().as_millis() as u64;

        ComponentHealth {
            status,
            last_check: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            response_time_ms: Some(response_time),
            error: None,
        }
    }

    /// Check paymaster service health
    async fn check_paymaster_health(
        &self,
        paymaster_service: &Option<Arc<PaymasterRelayService>>,
    ) -> ComponentHealth {
        let start = Instant::now();

        let (status, error) = match paymaster_service {
            Some(_service) => {
                // TODO: Add actual paymaster health check logic
                // For now, just check if service exists
                (ComponentStatus::Healthy, None)
            }
            None => (
                ComponentStatus::Warning,
                Some("Paymaster service not configured".to_string()),
            ),
        };

        let response_time = start.elapsed().as_millis() as u64;

        ComponentHealth {
            status,
            last_check: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            response_time_ms: Some(response_time),
            error,
        }
    }

    /// Check pool service health
    async fn check_pool_health(&self) -> ComponentHealth {
        let start = Instant::now();

        // TODO: Add actual pool health check logic
        // For now, assume healthy
        let status = ComponentStatus::Healthy;
        let response_time = start.elapsed().as_millis() as u64;

        ComponentHealth {
            status,
            last_check: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            response_time_ms: Some(response_time),
            error: None,
        }
    }

    /// Check router service health
    async fn check_router_health(&self, _router: &GatewayRouter) -> ComponentHealth {
        let start = Instant::now();

        // Basic router health check
        let status = ComponentStatus::Healthy;
        let response_time = start.elapsed().as_millis() as u64;

        ComponentHealth {
            status,
            last_check: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            response_time_ms: Some(response_time),
            error: None,
        }
    }

    /// Determine overall system status from component statuses
    fn determine_overall_status(&self, components: &[&ComponentHealth]) -> SystemStatus {
        let mut has_error = false;
        let mut has_warning = false;

        for component in components {
            match component.status {
                ComponentStatus::Error => has_error = true,
                ComponentStatus::Warning => has_warning = true,
                ComponentStatus::Healthy | ComponentStatus::Unknown => {}
            }
        }

        if has_error {
            SystemStatus::Unhealthy
        } else if has_warning {
            SystemStatus::Degraded
        } else {
            SystemStatus::Healthy
        }
    }

    /// Collect system metrics
    async fn collect_system_metrics(&self) -> SystemMetrics {
        let total_requests = self
            .request_counter
            .load(std::sync::atomic::Ordering::Relaxed);
        let total_errors = self
            .error_counter
            .load(std::sync::atomic::Ordering::Relaxed);

        let error_rate = if total_requests > 0 {
            (total_errors as f64 / total_requests as f64) * 100.0
        } else {
            0.0
        };

        // Get memory usage (approximation)
        let memory_usage_mb = self.get_memory_usage_mb().await;

        SystemMetrics {
            memory_usage_mb,
            active_connections: 0, // TODO: Track actual connections
            total_requests,
            error_rate,
        }
    }

    /// Get current memory usage in MB
    async fn get_memory_usage_mb(&self) -> f64 {
        // This is a simplified implementation
        // In production, you might want to use a more sophisticated approach
        std::process::Command::new("ps")
            .args(["-o", "rss=", "-p", &std::process::id().to_string()])
            .output()
            .ok()
            .and_then(|output| {
                String::from_utf8(output.stdout)
                    .ok()?
                    .trim()
                    .parse::<f64>()
                    .ok()
            })
            .map(|kb| kb / 1024.0) // Convert KB to MB
            .unwrap_or(0.0)
    }
}

/// Health check endpoint handler
pub async fn health_check(
    State(state): State<GatewayState>,
) -> Result<Json<HealthStatus>, StatusCode> {
    debug!("Processing health check request");

    // For now, create a new health checker each time
    // In production, you might want to maintain a singleton
    let health_checker = HealthChecker::new();
    let health_status = health_checker.check_health(&state).await;

    match health_status.status {
        SystemStatus::Healthy => {
            info!("Health check passed: all systems healthy");
        }
        SystemStatus::Degraded => {
            warn!("Health check shows degraded performance");
        }
        SystemStatus::Unhealthy => {
            error!("Health check failed: system unhealthy");
        }
    }

    Ok(Json(health_status))
}

/// Readiness check endpoint handler (simpler check for load balancers)
pub async fn readiness_check(State(_state): State<GatewayState>) -> Result<StatusCode, StatusCode> {
    // Simple readiness check - just return OK if service is running
    debug!("Processing readiness check request");
    Ok(StatusCode::OK)
}

/// Liveness check endpoint handler (basic service alive check)
pub async fn liveness_check() -> StatusCode {
    debug!("Processing liveness check request");
    StatusCode::OK
}

/// Create health check routes
pub fn health_routes() -> Router<GatewayState> {
    Router::new()
        .route("/health", get(health_check))
        .route("/ready", get(readiness_check))
        .route("/live", get(liveness_check))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_health_checker_creation() {
        let checker = HealthChecker::new();
        assert!(checker.start_time.elapsed().as_millis() < 100);
    }

    #[tokio::test]
    async fn test_request_recording() {
        let checker = HealthChecker::new();
        checker.record_request();
        checker.record_request();

        let count = checker
            .request_counter
            .load(std::sync::atomic::Ordering::Relaxed);
        assert_eq!(count, 2);
    }

    #[tokio::test]
    async fn test_error_recording() {
        let checker = HealthChecker::new();
        checker.record_error();

        let count = checker
            .error_counter
            .load(std::sync::atomic::Ordering::Relaxed);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_overall_status_determination() {
        let checker = HealthChecker::new();

        let healthy_component = ComponentHealth {
            status: ComponentStatus::Healthy,
            last_check: 0,
            response_time_ms: Some(10),
            error: None,
        };

        let warning_component = ComponentHealth {
            status: ComponentStatus::Warning,
            last_check: 0,
            response_time_ms: Some(20),
            error: Some("Warning message".to_string()),
        };

        let error_component = ComponentHealth {
            status: ComponentStatus::Error,
            last_check: 0,
            response_time_ms: Some(100),
            error: Some("Error message".to_string()),
        };

        // All healthy
        assert_eq!(
            checker.determine_overall_status(&[&healthy_component, &healthy_component]),
            SystemStatus::Healthy
        );

        // Has warning
        assert_eq!(
            checker.determine_overall_status(&[&healthy_component, &warning_component]),
            SystemStatus::Degraded
        );

        // Has error
        assert_eq!(
            checker.determine_overall_status(&[&healthy_component, &error_component]),
            SystemStatus::Unhealthy
        );
    }
}
