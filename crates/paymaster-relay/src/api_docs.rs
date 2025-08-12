/// SuperPaymaster API Documentation
///
/// 本模块定义了所有API的请求/响应结构和OpenAPI注解
/// 用于生成高质量的Swagger UI文档
use serde::{Deserialize, Serialize};
use utoipa::{OpenApi, ToSchema};

// 导入详细的schema定义
// 注意：实际使用时需要调整路径或将schemas移到合适位置
pub use crate::schemas::*;

/// API文档主结构
#[derive(OpenApi)]
#[openapi(
    paths(
        pm_sponsor_user_operation,
        get_balance_status,
        get_policies_status,
        get_system_metrics,
        health_check,
        get_api_statistics
    ),
    components(
        schemas(
            UserOperation,
            SponsorUserOperationRequest,
            SponsorUserOperationResponse,
            HealthStatus,
            ComponentStatus,
            PerformanceMetrics,
            BalanceStatus,
            BalanceThresholds,
            BalanceAddresses,
            PolicyStatus,
            SystemMetrics,
            ErrorResponse,
            ApiStatistics
        )
    ),
    tags(
        (name = "paymaster", description = "ERC-4337 Paymaster赞助操作 - 核心业务API"),
        (name = "monitoring", description = "系统监控和健康检查 - 运维监控API"),
        (name = "management", description = "策略和配置管理 - 管理员API"),
        (name = "statistics", description = "API使用统计和性能指标 - 分析API")
    ),
    info(
        title = "SuperPaymaster Enterprise API",
        version = "0.2.0",
        description = r#"
# SuperPaymaster Enterprise Account Abstraction Solution

SuperPaymaster是基于ERC-4337标准的企业级账户抽象Paymaster解决方案，提供高性能、安全可靠的Gas费用赞助服务。

## 主要特性

- 🚀 **高性能**: 基于Rust构建，支持25+ TPS
- 🔒 **企业安全**: 多层安全策略和风险控制
- 📊 **实时监控**: Prometheus指标和健康检查
- 🎯 **灵活策略**: TOML配置驱动的策略引擎
- 🔗 **标准兼容**: 完全符合ERC-4337规范

## API设计原则

1. **RESTful风格**: 清晰的资源导向接口设计
2. **标准错误码**: JSON-RPC 2.0兼容的错误处理
3. **详细文档**: 完整的请求/响应示例和错误说明
4. **版本管理**: 语义化版本控制，向后兼容

## 认证和安全

- API密钥认证 (生产环境)
- IP白名单限制
- 速率限制和反DDoS保护
- 请求签名验证

## 监控和告警

- 实时性能指标
- 系统健康状态检查
- 自动故障检测和恢复
- 完整的审计日志

更多信息请访问: https://superpaymaster.docs
        "#,
        contact(
            name = "SuperPaymaster Team",
            email = "support@superpaymaster.io",
            url = "https://superpaymaster.io"
        ),
        license(
            name = "MIT",
            url = "https://opensource.org/licenses/MIT"
        )
    ),
    servers(
        (url = "http://localhost:3000", description = "开发环境 - SuperRelay主服务（实际处理请求）"),
        (url = "http://localhost:9000", description = "Swagger UI服务器（文档界面）"),
        (url = "http://localhost:8545", description = "Anvil本地区块链（测试环境）"),
        (url = "https://api.superpaymaster.io", description = "生产环境API")
    ),
    external_docs(
        description = "SuperPaymaster完整文档",
        url = "https://docs.superpaymaster.io"
    )
)]
pub struct ApiDoc;

/// UserOperation请求结构
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({
    "sender": "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266",
    "nonce": "0x0",
    "callData": "0x",
    "callGasLimit": "0x10000",
    "verificationGasLimit": "0x10000",
    "preVerificationGas": "0x5000",
    "maxFeePerGas": "0x3b9aca00",
    "maxPriorityFeePerGas": "0x3b9aca00",
    "signature": "0x",
    "initCode": "0x",
    "paymasterAndData": "0x"
}))]
pub struct UserOperationRequest {
    /// 发送者地址
    #[schema(example = "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266")]
    pub sender: String,

    /// Nonce值
    #[schema(example = "0x0")]
    pub nonce: String,

    /// 调用数据
    #[schema(example = "0x")]
    pub call_data: String,

    /// 调用Gas限制
    #[schema(example = "0x10000")]
    pub call_gas_limit: String,

    /// 验证Gas限制
    #[schema(example = "0x10000")]
    pub verification_gas_limit: String,

    /// 预验证Gas
    #[schema(example = "0x5000")]
    pub pre_verification_gas: String,

    /// 最大Gas费用
    #[schema(example = "0x3b9aca00")]
    pub max_fee_per_gas: String,

    /// 最大优先级费用
    #[schema(example = "0x3b9aca00")]
    pub max_priority_fee_per_gas: String,

    /// 签名
    #[schema(example = "0x")]
    pub signature: String,

    /// 初始化代码
    #[schema(example = "0x")]
    pub init_code: String,

    /// Paymaster数据
    #[schema(example = "0x")]
    pub paymaster_and_data: String,
}

/// 赞助UserOperation请求
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SponsorUserOperationRequest {
    /// UserOperation数据
    pub user_operation: UserOperationRequest,

    /// EntryPoint合约地址
    #[schema(example = "0xe7f1725E7734CE288F8367e1Bb143E90bb3F0512")]
    pub entry_point: String,
}

/// 赞助UserOperation响应
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({
    "paymasterAndData": "0x70997970C51812dc3A010C7d01b50e0d17dc79C8000000000000000000000000000000000000000000000000000000006678c5500000000000000000000000000000000000000000000000000000000000000000"
}))]
pub struct SponsorUserOperationResponse {
    /// Paymaster和数据字符串，包含赞助信息
    #[schema(
        example = "0x70997970C51812dc3A010C7d01b50e0d17dc79C8000000000000000000000000000000000000000000000000000000006678c5500000000000000000000000000000000000000000000000000000000000000000"
    )]
    pub paymaster_and_data: String,
}

/// 错误响应结构
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ErrorResponse {
    /// 错误代码
    #[schema(example = -32602)]
    pub code: i32,

    /// 错误信息
    #[schema(example = "Invalid user operation format")]
    pub message: String,

    /// 错误详情
    pub data: Option<serde_json::Value>,
}

/// 余额状态
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct BalanceStatus {
    /// Paymaster余额（ETH）
    #[schema(example = "10000.0")]
    pub paymaster_balance: String,

    /// EntryPoint存款（ETH）
    #[schema(example = "2.0")]
    pub entry_point_deposit: String,

    /// 状态
    #[schema(example = "HEALTHY")]
    pub status: String,
}

/// 策略状态
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PolicyStatus {
    /// 策略是否启用
    #[schema(example = true)]
    pub enabled: bool,

    /// 活跃策略数量
    #[schema(example = 3)]
    pub active_policies: u32,

    /// 策略文件路径
    #[schema(example = "/etc/paymaster/policies.toml")]
    pub policy_file: String,
}

/// 系统指标
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SystemMetrics {
    /// 总请求数
    #[schema(example = 1234)]
    pub total_requests: u64,

    /// 成功请求数
    #[schema(example = 1200)]
    pub successful_requests: u64,

    /// 失败请求数
    #[schema(example = 34)]
    pub failed_requests: u64,

    /// 平均响应时间（毫秒）
    #[schema(example = 150.5)]
    pub avg_response_time_ms: f64,
}

/// 健康状态
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct HealthStatus {
    /// 服务状态
    #[schema(example = "UP")]
    pub status: String,

    /// 版本信息
    #[schema(example = "0.1.9")]
    pub version: String,

    /// 启动时间
    #[schema(example = "2024-01-01T12:00:00Z")]
    pub uptime: String,
}

// Path handler functions - these need to be defined for the OpenAPI paths to work

/// 赞助UserOperation端点
#[utoipa::path(
    post,
    path = "/pm_sponsorUserOperation",
    request_body = SponsorUserOperationRequest,
    responses(
        (status = 200, description = "Successfully sponsored user operation", body = SponsorUserOperationResponse),
        (status = 400, description = "Invalid request", body = ErrorResponse)
    ),
    tag = "paymaster"
)]
pub async fn pm_sponsor_user_operation() {}

/// 获取余额状态
#[utoipa::path(
    get,
    path = "/balance",
    responses(
        (status = 200, description = "Balance status", body = BalanceStatus)
    ),
    tag = "monitoring"
)]
pub async fn get_balance_status() {}

/// 获取策略状态
#[utoipa::path(
    get,
    path = "/policies",
    responses(
        (status = 200, description = "Policy status", body = PolicyStatus)
    ),
    tag = "management"
)]
pub async fn get_policies_status() {}

/// 获取系统指标
#[utoipa::path(
    get,
    path = "/metrics",
    responses(
        (status = 200, description = "System metrics", body = SystemMetrics)
    ),
    tag = "monitoring"
)]
pub async fn get_system_metrics() {}

/// 健康检查
#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "Health status", body = HealthStatus)
    ),
    tag = "monitoring"
)]
pub async fn health_check() {}

/// 获取API使用统计
#[utoipa::path(
    get,
    path = "/statistics",
    responses(
        (status = 200, description = "API statistics", body = ApiStatistics)
    ),
    tag = "statistics"
)]
pub async fn get_api_statistics() {}
