# HTTP REST vs JSON-RPC 协议对比指南

## 📋 概述

SuperRelay 支持两种 API 协议：**HTTP REST** 和 **JSON-RPC**。本文档详细说明两种协议的区别、使用场景以及 SuperRelay 中的转换机制。

## 🎯 快速对比

| 特性 | HTTP REST | JSON-RPC |
|------|-----------|----------|
| **设计理念** | 资源导向 (Resource-Oriented) | 方法调用导向 (Method-Oriented) |
| **URL 结构** | 语义化路径 `/api/v1/sponsor` | 单一端点 `/` |
| **操作标识** | HTTP 动词 (POST/GET/PUT/DELETE) | 方法名 (`pm_sponsorUserOperation`) |
| **数据格式** | 灵活 (JSON/XML/Form) | 严格 JSON-RPC 2.0 规范 |
| **错误处理** | HTTP 状态码 (400/404/500) | 自定义错误码 (-32602/-32603) |
| **批量操作** | 需要多次请求 | 原生支持批量调用 |
| **缓存支持** | HTTP 缓存机制 | 不支持标准缓存 |
| **生态工具** | Postman, Swagger, API网关 | 区块链工具链, web3.js |

## 📖 协议详解

### HTTP REST 协议

REST (Representational State Transfer) 是一种**资源导向**的架构风格，将 API 设计为对资源的操作。

#### 核心原则
1. **资源标识**: 每个资源都有唯一的 URL
2. **统一接口**: 使用标准 HTTP 动词操作资源
3. **无状态**: 每个请求包含完整的处理信息
4. **可缓存**: 支持 HTTP 缓存机制

#### REST 请求示例

**赞助 UserOperation**:
```http
POST /api/v1/sponsor HTTP/1.1
Content-Type: application/json
Accept: application/json

{
  "user_operation": {
    "sender": "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266",
    "nonce": "0x0",
    "initCode": "0x",
    "callData": "0xb61d27f6...",
    "callGasLimit": "0x30D40",
    "verificationGasLimit": "0x186A0",
    "preVerificationGas": "0xC350",
    "maxFeePerGas": "0x59682F00",
    "maxPriorityFeePerGas": "0x59682F00",
    "paymasterAndData": "0x",
    "signature": "0xff...1c"
  },
  "entry_point": "0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789"
}
```

**成功响应**:
```http
HTTP/1.1 200 OK
Content-Type: application/json

{
  "paymaster_and_data": "0x70997970C51812dc3A010C7d01b50e0d17dc79C8000000000000000000000000000000000000000000000000000000006678c5500000000000000000000000000000000000000000000000000000000000000000"
}
```

**错误响应**:
```http
HTTP/1.1 400 Bad Request
Content-Type: application/json

{
  "code": -32602,
  "message": "Invalid user operation format",
  "data": {
    "field": "sender",
    "error": "Invalid address format"
  }
}
```

**其他 REST 端点**:
```http
GET /health                    # 健康检查
GET /api/v1/balance           # 获取余额状态 (未来)
GET /api/v1/policies          # 获取策略状态 (未来)
POST /api/v1/sponsor          # 赞助 UserOperation
```

### JSON-RPC 协议

JSON-RPC 是一种**方法调用导向**的远程过程调用协议，广泛应用于区块链生态系统。

#### 核心特性
1. **方法调用**: 直接调用远程方法
2. **严格规范**: JSON-RPC 2.0 标准格式
3. **批量支持**: 一次请求多个方法调用
4. **ID 匹配**: 请求和响应通过 ID 关联

#### JSON-RPC 请求示例

**赞助 UserOperation**:
```http
POST / HTTP/1.1
Content-Type: application/json

{
  "jsonrpc": "2.0",
  "method": "pm_sponsorUserOperation",
  "params": [
    {
      "sender": "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266",
      "nonce": "0x0",
      "initCode": "0x",
      "callData": "0xb61d27f6...",
      "callGasLimit": "0x30D40",
      "verificationGasLimit": "0x186A0",
      "preVerificationGas": "0xC350",
      "maxFeePerGas": "0x59682F00",
      "maxPriorityFeePerGas": "0x59682F00",
      "paymasterAndData": "0x",
      "signature": "0xff...1c"
    },
    "0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789"
  ],
  "id": 1
}
```

**成功响应**:
```http
HTTP/1.1 200 OK
Content-Type: application/json

{
  "jsonrpc": "2.0",
  "result": "0x70997970C51812dc3A010C7d01b50e0d17dc79C8000000000000000000000000000000000000000000000000000000006678c5500000000000000000000000000000000000000000000000000000000000000000",
  "id": 1
}
```

**错误响应**:
```http
HTTP/1.1 200 OK
Content-Type: application/json

{
  "jsonrpc": "2.0",
  "error": {
    "code": -32602,
    "message": "Invalid params",
    "data": "Invalid user operation format"
  },
  "id": 1
}
```

**批量请求**:
```json
[
  {
    "jsonrpc": "2.0",
    "method": "pm_sponsorUserOperation",
    "params": [userOp1, entryPoint],
    "id": 1
  },
  {
    "jsonrpc": "2.0",
    "method": "pm_sponsorUserOperation",
    "params": [userOp2, entryPoint],
    "id": 2
  }
]
```

## 🔄 SuperRelay 转换机制

### 架构设计

SuperRelay 采用**双协议支持**的架构设计，提供最大的兼容性：

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────────┐
│   REST 客户端    │───▶│   HTTP REST API   │───▶│                     │
│  (Web/Mobile)   │    │  (端口 9000)     │    │  PaymasterRelay     │
├─────────────────┤    ├──────────────────┤    │   核心业务逻辑        │
│ JSON-RPC 客户端  │───▶│   JSON-RPC API   │───▶│  (PaymasterRelay-   │
│  (区块链工具)    │    │  (端口 3000)     │    │   ApiServerImpl)    │
└─────────────────┘    └──────────────────┘    └─────────────────────┘
                            ↑                           ↑
                       协议转换层                  统一业务逻辑
```

### 转换实现

#### 1. REST 到 RPC 转换 (`api_handlers.rs`)

```rust
#[utoipa::path(
    post,
    path = "/api/v1/sponsor",
    request_body = SponsorUserOperationRequest,
    responses(
        (status = 200, description = "Successfully sponsored", body = SponsorUserOperationResponse),
        (status = 400, description = "Invalid request", body = ErrorResponse),
    ),
    tag = "paymaster"
)]
pub async fn sponsor_user_operation_handler(
    State(rpc_service): State<Arc<PaymasterRelayApiServerImpl>>,
    Json(request): Json<SponsorUserOperationRequest>,
) -> Result<Json<SponsorUserOperationResponse>, (StatusCode, Json<ErrorResponse>)> {

    // 1. 调用内部 JSON-RPC 实现
    match rpc_service.sponsor_user_operation(
        request.user_op,      // REST 请求体 → RPC 参数
        request.entry_point
    ).await {
        Ok(paymaster_and_data) => {
            // 2. RPC 响应 → REST 响应
            Ok(Json(SponsorUserOperationResponse {
                paymaster_and_data,
            }))
        }
        Err(rpc_error) => {
            // 3. RPC 错误 → HTTP 状态码
            let error_response = ErrorResponse {
                code: rpc_error.code(),
                message: rpc_error.message().to_string(),
                data: rpc_error.data().cloned(),
            };

            let status_code = match rpc_error.code() {
                -32600 => StatusCode::BAD_REQUEST,      // Invalid Request
                -32601 => StatusCode::NOT_FOUND,        // Method not found
                -32602 => StatusCode::BAD_REQUEST,      // Invalid params
                -32603 => StatusCode::INTERNAL_SERVER_ERROR, // Internal error
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };

            Err((status_code, Json(error_response)))
        }
    }
}
```

#### 2. 数据结构映射

**REST 请求结构**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SponsorUserOperationRequest {
    /// 用户操作数据
    pub user_op: serde_json::Value,

    /// EntryPoint 合约地址
    #[schema(example = "0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789")]
    pub entry_point: String,
}
```

**RPC 方法签名**:
```rust
async fn sponsor_user_operation(
    &self,
    user_op: serde_json::Value,     // 对应 request.user_op
    entry_point: String,            // 对应 request.entry_point
) -> Result<String, ErrorObjectOwned>;
```

#### 3. 错误码映射

| RPC 错误码 | HTTP 状态码 | 含义 |
|-----------|-------------|------|
| -32600 | 400 Bad Request | 无效的请求格式 |
| -32601 | 404 Not Found | 方法不存在 |
| -32602 | 400 Bad Request | 无效的参数 |
| -32603 | 500 Internal Server Error | 内部错误 |
| -32000 ~ -32099 | 500 Internal Server Error | 服务器自定义错误 |

## 🎯 使用场景选择

### 选择 HTTP REST 的场景

✅ **推荐使用 REST** 的情况：
- **Web 前端开发**: React, Vue, Angular 应用
- **移动应用开发**: iOS, Android 原生应用
- **API 测试**: 使用 Postman, Insomnia 等工具
- **微服务集成**: 集成到现有的 REST API 架构
- **API 网关**: 通过 Kong, Nginx 等网关路由
- **缓存需求**: 需要利用 HTTP 缓存机制
- **标准化要求**: 需要 OpenAPI 规范文档

**REST 调用示例**:
```javascript
// 前端 JavaScript 调用
const sponsorUserOp = async (userOperation, entryPoint) => {
  try {
    const response = await fetch('/api/v1/sponsor', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        user_operation: userOperation,
        entry_point: entryPoint
      })
    });

    if (!response.ok) {
      throw new Error(`HTTP ${response.status}: ${response.statusText}`);
    }

    const result = await response.json();
    return result.paymaster_and_data;
  } catch (error) {
    console.error('赞助失败:', error);
    throw error;
  }
};
```

```bash
# curl 命令行调用
curl -X POST "http://localhost:9000/api/v1/sponsor" \
  -H "Content-Type: application/json" \
  -d '{
    "user_operation": {
      "sender": "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266",
      "nonce": "0x0",
      "callData": "0x"
    },
    "entry_point": "0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789"
  }'
```

### 选择 JSON-RPC 的场景

✅ **推荐使用 JSON-RPC** 的情况：
- **区块链工具**: web3.js, ethers.js 等库
- **DApp 开发**: 去中心化应用后端
- **批量操作**: 需要在一次请求中调用多个方法
- **以太坊生态集成**: 与其他以太坊服务保持一致
- **高性能要求**: 减少 HTTP 开销
- **RPC 代理**: 通过 RPC 代理服务调用

**JSON-RPC 调用示例**:
```javascript
// web3.js 集成
const Web3 = require('web3');

const web3 = new Web3('http://localhost:3000');

const sponsorUserOp = async (userOperation, entryPoint) => {
  try {
    const result = await web3.currentProvider.send({
      jsonrpc: '2.0',
      method: 'pm_sponsorUserOperation',
      params: [userOperation, entryPoint],
      id: Date.now()
    });

    if (result.error) {
      throw new Error(`RPC Error ${result.error.code}: ${result.error.message}`);
    }

    return result.result;
  } catch (error) {
    console.error('赞助失败:', error);
    throw error;
  }
};

// 批量调用示例
const batchSponsor = async (operations) => {
  const requests = operations.map((op, index) => ({
    jsonrpc: '2.0',
    method: 'pm_sponsorUserOperation',
    params: [op.userOperation, op.entryPoint],
    id: index + 1
  }));

  const results = await web3.currentProvider.send(requests);
  return results.map(r => r.result);
};
```

```bash
# curl JSON-RPC 调用
curl -X POST "http://localhost:3000" \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "pm_sponsorUserOperation",
    "params": [
      {
        "sender": "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266",
        "nonce": "0x0",
        "callData": "0x"
      },
      "0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789"
    ],
    "id": 1
  }'
```

## 🚀 部署和配置

### 启动双协议服务

**方式1: 分别启动两个服务**
```bash
# 启动 JSON-RPC 服务 (端口 3000)
./target/release/super-relay node --rpc-addr 0.0.0.0:3000

# 启动 HTTP REST 服务 (端口 9000)
./target/release/super-relay api-server --bind-addr 0.0.0.0:9000
```

**方式2: 使用脚本启动**
```bash
# 使用启动脚本
./scripts/start_superrelay.sh    # JSON-RPC 服务
./scripts/start_web_ui.sh        # HTTP REST + Swagger UI
```

**方式3: 代码集成**
```rust
use rundler_paymaster_relay::{start_api_server, PaymasterRelayApiServerImpl};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建服务实例
    let service = create_paymaster_service().await?;
    let rpc_impl = Arc::new(PaymasterRelayApiServerImpl::new(service));

    // 启动 HTTP REST 服务器
    start_api_server("0.0.0.0:9000", rpc_impl).await?;
    Ok(())
}
```

### 验证服务状态

```bash
# 检查 JSON-RPC 服务
curl -X POST http://localhost:3000 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"eth_chainId","params":[],"id":1}'

# 检查 HTTP REST 服务
curl http://localhost:9000/health

# 访问 Swagger UI 文档
open http://localhost:9000/swagger-ui/
```

## 📚 相关文档

- [API 参考文档](./API_REFERENCE.md) - 完整的 API 接口文档
- [utoipa 迁移报告](./UTOIPA_MIGRATION_REPORT.md) - 技术架构变更详情
- [开发者指南](./DEVELOPER_GUIDE.md) - 开发环境搭建和最佳实践
- [部署指南](./DEPLOYMENT_GUIDE.md) - 生产环境部署说明

## 🔧 故障排除

### 常见问题

**Q: REST API 调用返回 404 错误**
```
A: 检查是否启动了 HTTP REST 服务 (端口 9000)，而不是只启动了 JSON-RPC 服务 (端口 3000)
```

**Q: JSON-RPC 调用提示方法不存在**
```
A: 确认方法名使用 pm_ 前缀，例如 pm_sponsorUserOperation 而不是 sponsorUserOperation
```

**Q: Swagger UI 无法加载**
```
A: 检查服务是否在端口 9000 启动，访问 http://localhost:9000/swagger-ui/
```

**Q: CORS 错误**
```
A: HTTP REST 服务已配置 CORS，如果仍有问题，检查请求头和域名配置
```

### 调试技巧

**开启调试日志**:
```bash
RUST_LOG=debug ./target/release/super-relay api-server
```

**使用 curl 测试**:
```bash
# 测试 REST API
curl -v -X POST http://localhost:9000/api/v1/sponsor \
  -H "Content-Type: application/json" \
  -d '{"user_operation":{},"entry_point":"0x..."}'

# 测试 JSON-RPC
curl -v -X POST http://localhost:3000 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"pm_sponsorUserOperation","params":[{},"0x..."],"id":1}'
```

---

**本文档版本**: v1.0
**最后更新**: 2025-08-12
**适用版本**: SuperRelay v0.2.0+