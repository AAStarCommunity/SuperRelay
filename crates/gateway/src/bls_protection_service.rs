//! BLS聚合签名防护服务
//!
//! 提供企业级BLS聚合签名防护服务：
//! - 集成到Gateway的请求处理流程
//! - 自动化的聚合器监控和防护
//! - 实时性能监控和告警
//! - RESTful API接口用于管理和监控

use std::sync::Arc;

use alloy_primitives::{Address, Bytes, B256};
use anyhow::Result;
use axum::{
    extract::{Path, Query},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use rundler_types::user_operation::UserOperationVariant;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::bls_protection::{BlsProtectionConfig, BlsProtectionSystem, BlsValidationResult};

/// BLS防护服务请求
#[derive(Debug, Serialize, Deserialize)]
pub struct BlsValidationRequest {
    pub aggregator_address: Address,
    pub signature: Bytes,
    pub message_hash: B256,
}

/// BLS聚合验证请求
#[derive(Debug, Serialize, Deserialize)]
pub struct BlsAggregationRequest {
    pub aggregator_address: Address,
    pub signatures: Vec<Bytes>,
}

/// 黑名单管理请求
#[derive(Debug, Serialize, Deserialize)]
pub struct BlacklistRequest {
    pub address: Address,
    pub reason: String,
    pub duration_seconds: u64,
}

/// 可信聚合器管理请求
#[derive(Debug, Serialize, Deserialize)]
pub struct TrustedAggregatorRequest {
    pub address: Address,
}

/// API响应包装器
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn error(error: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(error),
        }
    }
}

/// BLS防护服务
#[derive(Debug)]
pub struct BlsProtectionService {
    protection_system: Arc<BlsProtectionSystem>,
    config: BlsProtectionConfig,
}

impl BlsProtectionService {
    /// 创建新的BLS防护服务
    pub fn new(config: BlsProtectionConfig) -> Result<Self> {
        let protection_system = Arc::new(BlsProtectionSystem::new(config.clone()));
        Ok(Self {
            protection_system,
            config,
        })
    }

    /// 获取防护系统的引用
    pub fn protection_system(&self) -> Arc<BlsProtectionSystem> {
        Arc::clone(&self.protection_system)
    }

    /// 处理UserOperation的BLS签名验证
    pub async fn validate_user_operation_bls(
        &self,
        user_op: &UserOperationVariant,
        aggregator_address: Option<Address>,
    ) -> Result<BlsValidationResult> {
        // 从UserOperation中提取BLS相关信息
        let (signature, message_hash) = self.extract_bls_info(user_op)?;

        if let Some(aggregator) = aggregator_address {
            debug!(
                "Validating BLS signature for aggregator {:?}, signature len: {}",
                aggregator,
                signature.len()
            );

            self.protection_system
                .validate_bls_signature(aggregator, &signature, &message_hash)
                .await
        } else {
            // 没有聚合器的情况，返回成功（非BLS签名）
            Ok(BlsValidationResult {
                is_valid: true,
                message: "Non-BLS UserOperation, skipping BLS validation".to_string(),
                aggregator_address: None,
                validation_time_ms: 0,
                security_issues: vec![],
            })
        }
    }

    /// 验证BLS聚合操作
    pub async fn validate_aggregation_request(
        &self,
        aggregator_address: Address,
        user_ops: &[UserOperationVariant],
    ) -> Result<BlsValidationResult> {
        // 从UserOperations中提取所有BLS签名
        let signatures: Result<Vec<Bytes>> = user_ops
            .iter()
            .map(|uo| Ok(self.extract_bls_info(uo)?.0))
            .collect();

        let signatures = signatures?;

        info!(
            "Validating BLS aggregation for {:?} with {} signatures",
            aggregator_address,
            signatures.len()
        );

        self.protection_system
            .validate_aggregation(aggregator_address, &signatures)
            .await
    }

    /// 启动后台清理任务
    pub async fn start_cleanup_tasks(self: Arc<Self>) -> Result<()> {
        let service = Arc::clone(&self);

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(
                std::time::Duration::from_secs(300), // 每5分钟清理一次
            );

            loop {
                interval.tick().await;

                if let Err(e) = service.protection_system.cleanup_expired_blacklist().await {
                    warn!("Failed to cleanup expired blacklist entries: {}", e);
                } else {
                    debug!("Completed blacklist cleanup cycle");
                }
            }
        });

        info!("BLS protection cleanup tasks started");
        Ok(())
    }

    /// 创建REST API路由
    pub fn create_api_routes(self: Arc<Self>) -> Router {
        Router::new()
            .route("/bls/validate", post(Self::api_validate_signature))
            .route("/bls/aggregate", post(Self::api_validate_aggregation))
            .route("/bls/status", get(Self::api_get_status))
            .route("/bls/blacklist", post(Self::api_blacklist_aggregator))
            .route("/bls/blacklist/:address", get(Self::api_check_blacklist))
            .route("/bls/trusted", post(Self::api_add_trusted))
            .route("/bls/trusted/:address", get(Self::api_remove_trusted))
            .route("/bls/stats/:address", get(Self::api_get_aggregator_stats))
            .with_state(self)
    }

    // 提取BLS相关信息
    fn extract_bls_info(&self, user_op: &UserOperationVariant) -> Result<(Bytes, B256)> {
        let signature = user_op.signature().clone();
        let hash = user_op.hash();
        Ok((signature, hash))
    }

    // API处理函数
    async fn api_validate_signature(
        axum::extract::State(service): axum::extract::State<Arc<Self>>,
        Json(request): Json<BlsValidationRequest>,
    ) -> Result<Json<ApiResponse<BlsValidationResult>>, StatusCode> {
        match service
            .protection_system
            .validate_bls_signature(
                request.aggregator_address,
                &request.signature,
                &request.message_hash,
            )
            .await
        {
            Ok(result) => Ok(Json(ApiResponse::success(result))),
            Err(e) => {
                warn!("BLS validation API error: {}", e);
                Ok(Json(ApiResponse::error(e.to_string())))
            }
        }
    }

    async fn api_validate_aggregation(
        axum::extract::State(service): axum::extract::State<Arc<Self>>,
        Json(request): Json<BlsAggregationRequest>,
    ) -> Result<Json<ApiResponse<BlsValidationResult>>, StatusCode> {
        match service
            .protection_system
            .validate_aggregation(request.aggregator_address, &request.signatures)
            .await
        {
            Ok(result) => Ok(Json(ApiResponse::success(result))),
            Err(e) => {
                warn!("BLS aggregation API error: {}", e);
                Ok(Json(ApiResponse::error(e.to_string())))
            }
        }
    }

    async fn api_get_status(
        axum::extract::State(service): axum::extract::State<Arc<Self>>,
    ) -> Result<Json<ApiResponse<crate::bls_protection::BlsProtectionStatus>>, StatusCode> {
        match service.protection_system.get_status().await {
            Ok(status) => Ok(Json(ApiResponse::success(status))),
            Err(e) => {
                warn!("BLS status API error: {}", e);
                Ok(Json(ApiResponse::error(e.to_string())))
            }
        }
    }

    async fn api_blacklist_aggregator(
        axum::extract::State(service): axum::extract::State<Arc<Self>>,
        Json(request): Json<BlacklistRequest>,
    ) -> Result<Json<ApiResponse<String>>, StatusCode> {
        match service
            .protection_system
            .blacklist_aggregator(request.address, request.reason, request.duration_seconds)
            .await
        {
            Ok(_) => Ok(Json(ApiResponse::success(
                "Aggregator blacklisted successfully".to_string(),
            ))),
            Err(e) => {
                warn!("BLS blacklist API error: {}", e);
                Ok(Json(ApiResponse::error(e.to_string())))
            }
        }
    }

    async fn api_check_blacklist(
        axum::extract::State(service): axum::extract::State<Arc<Self>>,
        Path(address): Path<String>,
    ) -> Result<Json<ApiResponse<bool>>, StatusCode> {
        let address = match address.parse::<Address>() {
            Ok(addr) => addr,
            Err(e) => {
                return Ok(Json(ApiResponse::error(format!("Invalid address: {}", e))));
            }
        };

        let blacklisted = service.protection_system.is_blacklisted(address).await;
        Ok(Json(ApiResponse::success(blacklisted)))
    }

    async fn api_add_trusted(
        axum::extract::State(service): axum::extract::State<Arc<Self>>,
        Json(request): Json<TrustedAggregatorRequest>,
    ) -> Result<Json<ApiResponse<String>>, StatusCode> {
        match service
            .protection_system
            .add_trusted_aggregator(request.address)
            .await
        {
            Ok(_) => Ok(Json(ApiResponse::success(
                "Trusted aggregator added successfully".to_string(),
            ))),
            Err(e) => {
                warn!("BLS trusted API error: {}", e);
                Ok(Json(ApiResponse::error(e.to_string())))
            }
        }
    }

    async fn api_remove_trusted(
        axum::extract::State(service): axum::extract::State<Arc<Self>>,
        Path(address): Path<String>,
    ) -> Result<Json<ApiResponse<String>>, StatusCode> {
        let address = match address.parse::<Address>() {
            Ok(addr) => addr,
            Err(e) => {
                return Ok(Json(ApiResponse::error(format!("Invalid address: {}", e))));
            }
        };

        match service
            .protection_system
            .remove_trusted_aggregator(address)
            .await
        {
            Ok(_) => Ok(Json(ApiResponse::success(
                "Trusted aggregator removed successfully".to_string(),
            ))),
            Err(e) => {
                warn!("BLS trusted removal API error: {}", e);
                Ok(Json(ApiResponse::error(e.to_string())))
            }
        }
    }

    async fn api_get_aggregator_stats(
        axum::extract::State(service): axum::extract::State<Arc<Self>>,
        Path(address): Path<String>,
    ) -> Result<
        Json<ApiResponse<Option<crate::bls_protection::AggregatorPerformanceStats>>>,
        StatusCode,
    > {
        let address = match address.parse::<Address>() {
            Ok(addr) => addr,
            Err(e) => {
                return Ok(Json(ApiResponse::error(format!("Invalid address: {}", e))));
            }
        };

        let stats = service
            .protection_system
            .get_aggregator_stats(address)
            .await;
        Ok(Json(ApiResponse::success(stats)))
    }
}

#[cfg(test)]
mod tests {
    use alloy_primitives::{address, b256};
    use rundler_types::user_operation::v0_7;

    use super::*;
    use crate::bls_protection::BlsProtectionConfig;

    #[tokio::test]
    async fn test_bls_protection_service_creation() -> Result<()> {
        let config = BlsProtectionConfig::default();
        let service = BlsProtectionService::new(config)?;

        let status = service.protection_system.get_status().await?;
        assert!(status.enabled);

        Ok(())
    }

    #[tokio::test]
    async fn test_user_operation_validation() -> Result<()> {
        let config = BlsProtectionConfig::default();
        let service = BlsProtectionService::new(config)?;

        let user_op = UserOperationVariant::V0_7(v0_7::UserOperation::default());
        let aggregator = Some(address!("1234567890123456789012345678901234567890"));

        let result = service
            .validate_user_operation_bls(&user_op, aggregator)
            .await?;

        // Should have some validation result
        assert!(result.aggregator_address.is_some());

        Ok(())
    }

    #[tokio::test]
    async fn test_non_bls_user_operation() -> Result<()> {
        let config = BlsProtectionConfig::default();
        let service = BlsProtectionService::new(config)?;

        let user_op = UserOperationVariant::V0_7(v0_7::UserOperation::default());

        let result = service.validate_user_operation_bls(&user_op, None).await?;

        // Should pass without aggregator
        assert!(result.is_valid);
        assert!(result.message.contains("Non-BLS"));
        assert!(result.aggregator_address.is_none());

        Ok(())
    }

    #[tokio::test]
    async fn test_aggregation_validation() -> Result<()> {
        let config = BlsProtectionConfig::default();
        let service = BlsProtectionService::new(config)?;

        let aggregator = address!("1234567890123456789012345678901234567890");
        let user_ops = vec![
            UserOperationVariant::V0_7(v0_7::UserOperation::default()),
            UserOperationVariant::V0_7(v0_7::UserOperation::default()),
        ];

        let result = service
            .validate_aggregation_request(aggregator, &user_ops)
            .await?;

        // Should have validation result for aggregation
        assert!(result.aggregator_address.is_some());
        assert_eq!(result.aggregator_address.unwrap(), aggregator);

        Ok(())
    }

    #[tokio::test]
    async fn test_api_routes_creation() {
        let config = BlsProtectionConfig::default();
        let service = Arc::new(BlsProtectionService::new(config).unwrap());

        let _router = service.create_api_routes();

        // API routes created successfully
        assert!(true);
    }
}
