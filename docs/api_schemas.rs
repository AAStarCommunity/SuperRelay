/// SuperPaymaster API Schemas Documentation
/// 
/// 本文件定义了SuperPaymaster系统的完整API数据模型、错误代码和示例
/// 用于生成高质量的开发者文档和客户端SDK

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use serde_json::json;

// ============================================================================
// 核心API数据模型
// ============================================================================

/// UserOperation 标准结构 (ERC-4337)
/// 
/// 表示一个完整的用户操作，包含所有必需的字段和Gas参数
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(
    description = "ERC-4337标准UserOperation结构，包含执行账户抽象操作所需的所有参数",
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
    #[schema(
        description = "智能合约账户地址，必须是已部署的ERC-4337兼容账户",
        example = "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266",
        format = "address"
    )]
    pub sender: String,
    
    /// 账户nonce值
    #[schema(
        description = "防重放攻击的nonce值，每个账户的序列号",
        example = "0x0",
        format = "hex"
    )]
    pub nonce: String,
    
    /// 调用数据
    #[schema(
        description = "要执行的函数调用数据，编码后的方法调用",
        example = "0xb61d27f6000000000000000000000000deadbeefdeadbeefdeadbeefdeadbeefdeadbeef0000000000000000000000000000000000000000000000000de0b6b3a76400000000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000000",
        format = "hex"
    )]
    pub call_data: String,
    
    /// 执行Gas限制
    #[schema(
        description = "执行callData所需的最大Gas量",
        example = "0x15F90",
        format = "hex"
    )]
    pub call_gas_limit: String,
    
    /// 验证Gas限制  
    #[schema(
        description = "账户验证所需的最大Gas量",
        example = "0x15F90", 
        format = "hex"
    )]
    pub verification_gas_limit: String,
    
    /// 预验证Gas
    #[schema(
        description = "UserOperation预处理所需的Gas量",
        example = "0x5208",
        format = "hex"
    )]
    pub pre_verification_gas: String,
    
    /// 最大Gas费用
    #[schema(
        description = "愿意支付的最大Gas价格 (wei)",
        example = "0x59682F00",
        format = "hex"
    )]
    pub max_fee_per_gas: String,
    
    /// 最大优先级费用
    #[schema(
        description = "给矿工的最大小费 (wei)",
        example = "0x3B9ACA00",
        format = "hex"
    )]
    pub max_priority_fee_per_gas: String,
    
    /// 账户签名
    #[schema(
        description = "账户对UserOperation哈希的签名",
        example = "0x1234567890abcdef...",
        format = "hex"
    )]
    pub signature: String,
    
    /// 账户初始化代码
    #[schema(
        description = "如果账户未部署，用于创建账户的初始化代码",
        example = "0x",
        format = "hex"
    )]
    pub init_code: String,
    
    /// Paymaster数据
    #[schema(
        description = "Paymaster地址和相关数据，由SuperPaymaster填充",
        example = "0x",
        format = "hex"
    )]
    pub paymaster_and_data: String,
}

// ============================================================================
// API请求/响应结构
// ============================================================================

/// 赞助UserOperation请求
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(
    description = "请求SuperPaymaster赞助UserOperation的完整请求结构",
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
    #[schema(description = "完整的UserOperation结构，paymasterAndData字段将被SuperPaymaster填充")]
    pub user_operation: UserOperation,
    
    /// EntryPoint合约地址
    #[schema(
        description = "ERC-4337 EntryPoint合约地址，用于验证和执行UserOperation",
        example = "0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789",
        format = "address"
    )]
    pub entry_point: String,
}

/// 赞助UserOperation响应
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(
    description = "SuperPaymaster赞助响应，包含填充了paymaster信息的paymasterAndData字段",
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
        description = "包含Paymaster地址、有效期和签名的完整数据字符串",
        example = "0x70997970C51812dc3A010C7d01b50e0d17dc79C8000000000000000000000000000000000000000000000000000000006678c5500000000000000000000000000000000000000000000000000000000000000000",
        format = "hex"
    )]
    pub paymaster_and_data: String,
    
    /// UserOperation哈希
    #[schema(
        description = "UserOperation的唯一标识哈希",
        example = "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
        format = "hex"
    )]
    pub user_op_hash: String,
    
    /// 是否成功赞助
    #[schema(description = "表示是否成功获得赞助")]
    pub sponsored: bool,
    
    /// 预估Gas成本
    #[schema(
        description = "预估的总Gas成本 (wei)",
        example = "0x2710",
        format = "hex"
    )]
    pub estimated_gas_cost: String,
    
    /// 赞助原因
    #[schema(
        description = "通过赞助的策略规则说明",
        example = "Approved by policy: default_allowlist"
    )]
    pub sponsorship_reason: String,
}

// ============================================================================
// 系统监控和状态结构
// ============================================================================

/// 系统健康状态
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[schema(
    description = "SuperPaymaster系统的整体健康状态信息",
    example = json!({
        "status": "HEALTHY",
        "version": "0.1.10",
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
    #[schema(
        description = "系统整体健康状态",
        example = "HEALTHY"
    )]
    pub status: String,
    
    /// 系统版本
    #[schema(example = "0.1.10")]
    pub version: String,
    
    /// 运行时间
    #[schema(example = "72h15m30s")]
    pub uptime: String,
    
    /// 时间戳
    #[schema(example = "2025-01-03T14:30:00Z")]
    pub timestamp: String,
    
    /// 组件状态
    #[schema(description = "各个子系统的状态")]
    pub components: ComponentStatus,
    
    /// 性能指标
    #[schema(description = "系统性能统计")]
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
    description = "Paymaster账户和EntryPoint存款的详细余额信息",
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

// ============================================================================
// 错误码和错误响应
// ============================================================================

/// 标准错误响应
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[schema(
    description = "API调用失败时的标准错误响应格式",
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
    #[schema(
        description = "标准JSON-RPC错误代码或自定义业务错误代码",
        example = -32602
    )]
    pub code: i32,
    
    /// 错误信息
    #[schema(
        description = "人类可读的错误描述",
        example = "Invalid params"
    )]
    pub message: String,
    
    /// 错误详情
    #[schema(description = "可选的错误详细信息和调试数据")]
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