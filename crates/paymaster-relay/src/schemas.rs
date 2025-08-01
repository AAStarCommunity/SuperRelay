/// SuperPaymaster API Schemas
///
/// 本模块定义了SuperPaymaster系统的完整API数据模型、错误代码和示例
/// 用于生成高质量的开发者文档和客户端SDK
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

// 导入详细的schema定义
pub use crate::api_docs::*;

// ============================================================================
// 核心API数据模型
// ============================================================================

/// UserOperation 标准结构 (ERC-4337)
///
/// 表示一个完整的用户操作，包含所有必需的字段和Gas参数
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(
    title = "UserOperation",
    example = json!({
        "sender": "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266",
        "nonce": "0x0",
        "callData": "0xb61d27f6000000000000000000000000deadbeefdeadbeefdeadbeefdeadbeefdeadbeef0000000000000000000000000000000000000000000000000de0b6b3a76400000000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000000",
        "callGasLimit": "0x15F90",
        "verificationGasLimit": "0x15F90", 
        "preVerificationGas": "0x5208",
        "maxFeePerGas": "0x59682F00",
        "maxPriorityFeePerGas": "0x3B9ACA00",
        "signature": "0x",
        "initCode": "0x",
        "paymasterAndData": "0x"
    })
)]
pub struct UserOperation {
    /// 发送者账户地址 (Account Contract Address)
    #[schema(example = "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266")]
    pub sender: String,

    /// 账户nonce值
    #[schema(example = "0x0")]
    pub nonce: String,

    /// 调用数据
    #[schema(
        example = "0xb61d27f6000000000000000000000000deadbeefdeadbeefdeadbeefdeadbeefdeadbeef0000000000000000000000000000000000000000000000000de0b6b3a76400000000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000000"
    )]
    pub call_data: String,

    /// 执行Gas限制
    #[schema(example = "0x15F90")]
    pub call_gas_limit: String,

    /// 验证Gas限制  
    #[schema(example = "0x15F90")]
    pub verification_gas_limit: String,

    /// 预验证Gas
    #[schema(example = "0x5208")]
    pub pre_verification_gas: String,

    /// 最大Gas费用
    #[schema(example = "0x59682F00")]
    pub max_fee_per_gas: String,

    /// 最大优先级费用
    #[schema(example = "0x3B9ACA00")]
    pub max_priority_fee_per_gas: String,

    /// 账户签名
    #[schema(example = "0x")]
    pub signature: String,

    /// 账户初始化代码
    #[schema(example = "0x")]
    pub init_code: String,

    /// Paymaster数据
    #[schema(example = "0x")]
    pub paymaster_and_data: String,
}

// ============================================================================
// API请求/响应结构
// ============================================================================

/// 赞助UserOperation请求
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(
    title = "SponsorUserOperationRequest",
    example = json!({
        "userOperation": {
            "sender": "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266",
            "nonce": "0x0",
            "callData": "0xb61d27f6000000000000000000000000deadbeefdeadbeefdeadbeefdeadbeefdeadbeef0000000000000000000000000000000000000000000000000de0b6b3a76400000000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000000",
            "callGasLimit": "0x15F90",
            "verificationGasLimit": "0x15F90",
            "preVerificationGas": "0x5208", 
            "maxFeePerGas": "0x59682F00",
            "maxPriorityFeePerGas": "0x3B9ACA00",
            "signature": "0x",
            "initCode": "0x",
            "paymasterAndData": "0x"
        },
        "entryPoint": "0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789"
    })
)]
pub struct SponsorUserOperationRequest {
    /// 待赞助的UserOperation
    pub user_operation: UserOperation,

    /// EntryPoint合约地址
    #[schema(example = "0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789")]
    pub entry_point: String,
}

/// 赞助UserOperation响应
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(
    title = "SponsorUserOperationResponse",
    example = json!({
        "paymasterAndData": "0x70997970C51812dc3A010C7d01b50e0d17dc79C8000000000000000000000000000000000000000000000000000000006678c5500000000000000000000000000000000000000000000000000000000000000000",
        "userOpHash": "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
        "sponsored": true,
        "estimatedGasCost": "0x2710",
        "sponsorshipReason": "Approved by policy: default_allowlist"
    })
)]
pub struct SponsorUserOperationResponse {
    /// 完整的paymaster数据
    #[schema(
        example = "0x70997970C51812dc3A010C7d01b50e0d17dc79C8000000000000000000000000000000000000000000000000000000006678c5500000000000000000000000000000000000000000000000000000000000000000"
    )]
    pub paymaster_and_data: String,

    /// UserOperation哈希
    #[schema(example = "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef")]
    pub user_op_hash: String,

    /// 是否成功赞助
    pub sponsored: bool,

    /// 预估Gas成本
    #[schema(example = "0x2710")]
    pub estimated_gas_cost: String,

    /// 赞助原因
    #[schema(example = "Approved by policy: default_allowlist")]
    pub sponsorship_reason: String,
}

// ============================================================================
// 系统监控和状态结构
// ============================================================================

/// 系统健康状态
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[schema(
    title = "HealthStatus",
    example = json!({
        "status": "HEALTHY",
        "version": "0.2.0",
        "uptime": "72h15m30s",
        "timestamp": "2025-01-03T14:30:00Z",
        "components": {
            "signer": "UP",
            "policy_engine": "UP", 
            "rpc_server": "UP",
            "eth_connection": "UP"
        },
        "performance": {
            "avg_response_time_ms": 4.2,
            "requests_per_second": 25.8,
            "error_rate": 0.012
        }
    })
)]
pub struct HealthStatus {
    /// 整体状态
    #[schema(example = "HEALTHY")]
    pub status: String,

    /// 系统版本
    #[schema(example = "0.2.0")]
    pub version: String,

    /// 运行时间
    #[schema(example = "72h15m30s")]
    pub uptime: String,

    /// 时间戳
    #[schema(example = "2025-01-03T14:30:00Z")]
    pub timestamp: String,

    /// 组件状态
    pub components: ComponentStatus,

    /// 性能指标
    pub performance: PerformanceMetrics,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ComponentStatus {
    /// 签名服务状态
    #[schema(example = "UP")]
    pub signer: String,

    /// 策略引擎状态
    #[schema(example = "UP")]
    pub policy_engine: String,

    /// RPC服务器状态
    #[schema(example = "UP")]
    pub rpc_server: String,

    /// 以太坊连接状态
    #[schema(example = "UP")]
    pub eth_connection: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PerformanceMetrics {
    /// 平均响应时间 (毫秒)
    #[schema(example = 4.2)]
    pub avg_response_time_ms: f64,

    /// 每秒请求数
    #[schema(example = 25.8)]
    pub requests_per_second: f64,

    /// 错误率
    #[schema(example = 0.012)]
    pub error_rate: f64,
}

/// 余额状态详情
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[schema(
    title = "BalanceStatus",
    example = json!({
        "paymaster_balance": "10000.0",
        "entry_point_deposit": "2.0",  
        "status": "HEALTHY",
        "last_updated": "2025-01-03T14:30:00Z",
        "thresholds": {
            "paymaster_min": "1.0",
            "deposit_min": "0.5"
        },
        "addresses": {
            "paymaster": "0x70997970C51812dc3A010C7d01b50e0d17dc79C8",
            "entry_point": "0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789"
        }
    })
)]
pub struct BalanceStatus {
    /// Paymaster账户余额 (ETH)
    #[schema(example = "10000.0")]
    pub paymaster_balance: String,

    /// EntryPoint存款余额 (ETH)
    #[schema(example = "2.0")]
    pub entry_point_deposit: String,

    /// 余额状态
    #[schema(example = "HEALTHY")]
    pub status: String,

    /// 最后更新时间
    #[schema(example = "2025-01-03T14:30:00Z")]
    pub last_updated: String,

    /// 阈值配置
    pub thresholds: BalanceThresholds,

    /// 相关地址
    pub addresses: BalanceAddresses,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct BalanceThresholds {
    /// Paymaster最小余额
    #[schema(example = "1.0")]
    pub paymaster_min: String,

    /// EntryPoint最小存款
    #[schema(example = "0.5")]
    pub deposit_min: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct BalanceAddresses {
    /// Paymaster地址
    #[schema(example = "0x70997970C51812dc3A010C7d01b50e0d17dc79C8")]
    pub paymaster: String,

    /// EntryPoint地址
    #[schema(example = "0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789")]
    pub entry_point: String,
}

/// 策略状态信息
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[schema(
    title = "PolicyStatus",
    example = json!({
        "enabled": true,
        "active_policies": 3,
        "policy_file": "/etc/paymaster/policies.toml",
        "last_updated": "2025-01-03T14:30:00Z",
        "policies": {
            "sender_allowlist": {
                "enabled": true,
                "addresses": 150
            },
            "gas_limit": {
                "enabled": true, 
                "max_gas": "0x15F90"
            },
            "rate_limit": {
                "enabled": true,
                "requests_per_hour": 100
            }
        }
    })
)]
pub struct PolicyStatus {
    /// 策略引擎是否启用
    pub enabled: bool,

    /// 活跃策略数量
    pub active_policies: u32,

    /// 策略配置文件路径
    pub policy_file: String,

    /// 最后更新时间
    pub last_updated: String,

    /// 具体策略详情
    pub policies: serde_json::Value,
}

/// 系统性能指标
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[schema(
    title = "SystemMetrics",
    example = json!({
        "total_requests": 12847,
        "successful_requests": 12650,
        "failed_requests": 197,
        "avg_response_time_ms": 4.2,
        "p95_response_time_ms": 8.7,
        "p99_response_time_ms": 15.3,
        "requests_per_second": 25.8,
        "error_rate": 0.015,
        "memory_usage_mb": 145.6,
        "cpu_usage_percent": 12.3,
        "uptime_seconds": 259847
    })
)]
pub struct SystemMetrics {
    /// 总请求数
    pub total_requests: u64,

    /// 成功请求数
    pub successful_requests: u64,

    /// 失败请求数
    pub failed_requests: u64,

    /// 平均响应时间 (毫秒)
    pub avg_response_time_ms: f64,

    /// P95响应时间 (毫秒)
    pub p95_response_time_ms: f64,

    /// P99响应时间 (毫秒)
    pub p99_response_time_ms: f64,

    /// 每秒请求数
    pub requests_per_second: f64,

    /// 错误率
    pub error_rate: f64,

    /// 内存使用量 (MB)
    pub memory_usage_mb: f64,

    /// CPU使用率 (%)
    pub cpu_usage_percent: f64,

    /// 运行时间 (秒)
    pub uptime_seconds: u64,
}

/// API使用统计
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[schema(
    title = "ApiStatistics",
    example = json!({
        "total_calls": 12847,
        "calls_by_method": {
            "pm_sponsorUserOperation": 12000,
            "health": 500,
            "balance": 200,
            "metrics": 147
        },
        "response_times": {
            "pm_sponsorUserOperation": {
                "avg": 4.2,
                "p95": 8.7,
                "p99": 15.3
            }
        },
        "error_rates": {
            "pm_sponsorUserOperation": 0.012,
            "overall": 0.015
        },
        "peak_rps": 45.7,
        "peak_time": "2025-01-03T10:30:00Z"
    })
)]
pub struct ApiStatistics {
    /// 总调用次数
    pub total_calls: u64,

    /// 按方法分组的调用次数
    pub calls_by_method: serde_json::Value,

    /// 响应时间统计
    pub response_times: serde_json::Value,

    /// 错误率统计
    pub error_rates: serde_json::Value,

    /// 峰值RPS
    pub peak_rps: f64,

    /// 峰值时间
    pub peak_time: String,
}

// ============================================================================
// 错误码和错误响应
// ============================================================================

/// 标准错误响应
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[schema(
    title = "ErrorResponse",
    example = json!({
        "code": -32602,
        "message": "Invalid params",
        "data": {
            "field": "userOperation.sender",
            "reason": "Invalid address format",
            "received": "invalid_address"
        }
    })
)]
pub struct ErrorResponse {
    /// JSON-RPC错误代码
    #[schema(example = -32602)]
    pub code: i32,

    /// 错误信息
    #[schema(example = "Invalid params")]
    pub message: String,

    /// 错误详情
    pub data: Option<serde_json::Value>,
}

// ============================================================================
// 错误代码常量定义
// ============================================================================

/// SuperPaymaster错误代码定义
pub struct ErrorCodes;

impl ErrorCodes {
    // JSON-RPC标准错误码
    pub const PARSE_ERROR: i32 = -32700;
    pub const INVALID_REQUEST: i32 = -32600;
    pub const METHOD_NOT_FOUND: i32 = -32601;
    pub const INVALID_PARAMS: i32 = -32602;
    pub const INTERNAL_ERROR: i32 = -32603;

    // SuperPaymaster业务错误码 (从-32000开始)
    pub const POLICY_REJECTED: i32 = -32000;
    pub const INSUFFICIENT_BALANCE: i32 = -32001;
    pub const INVALID_USER_OPERATION: i32 = -32002;
    pub const SIGNING_ERROR: i32 = -32003;
    pub const ENTRYPOINT_ERROR: i32 = -32004;
    pub const RATE_LIMITED: i32 = -32005;
    pub const SECURITY_VIOLATION: i32 = -32006;
    pub const MAINTENANCE_MODE: i32 = -32007;
}

/// 错误代码映射和描述
pub fn get_error_description(code: i32) -> &'static str {
    match code {
        ErrorCodes::PARSE_ERROR => "Parse error - Invalid JSON",
        ErrorCodes::INVALID_REQUEST => "Invalid Request - Malformed JSON-RPC request",
        ErrorCodes::METHOD_NOT_FOUND => "Method not found - Unknown API method",
        ErrorCodes::INVALID_PARAMS => "Invalid params - Parameter validation failed",
        ErrorCodes::INTERNAL_ERROR => "Internal error - Server internal error",
        ErrorCodes::POLICY_REJECTED => "Policy rejected - UserOperation rejected by policy engine",
        ErrorCodes::INSUFFICIENT_BALANCE => "Insufficient balance - Paymaster lacks funds",
        ErrorCodes::INVALID_USER_OPERATION => "Invalid UserOperation - Validation failed",
        ErrorCodes::SIGNING_ERROR => "Signing error - Failed to sign UserOperation",
        ErrorCodes::ENTRYPOINT_ERROR => "EntryPoint error - EntryPoint contract interaction failed",
        ErrorCodes::RATE_LIMITED => "Rate limited - Too many requests",
        ErrorCodes::SECURITY_VIOLATION => "Security violation - Request blocked by security filter",
        ErrorCodes::MAINTENANCE_MODE => "Maintenance mode - Service temporarily unavailable",
        _ => "Unknown error code",
    }
}
