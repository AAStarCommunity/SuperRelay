# jsonrpsee 命名空间机制详解

## 🔧 技术原理

### 1. RPC Trait 定义
```rust
#[rpc(client, server, namespace = "pm")]
#[async_trait]
pub trait PaymasterRelayApi {
    #[method(name = "sponsorUserOperation")]  // 内部方法名
    async fn sponsor_user_operation(...)
}
```

### 2. jsonrpsee 自动处理
```
内部定义: namespace="pm" + method="sponsorUserOperation"
        ↓ (jsonrpsee 框架自动拼接)
JSON-RPC 方法名: "pm_sponsorUserOperation"
```

### 3. 实际的 JSON-RPC 调用格式
```json
{
  "jsonrpc": "2.0",
  "method": "pm_sponsorUserOperation",  // 必须带 pm_ 前缀
  "params": [...],
  "id": 1
}
```

## 🚫 为什么不能去掉前缀

### 如果调用不带前缀的方法：
```json
{
  "jsonrpc": "2.0",
  "method": "sponsorUserOperation",  // 没有前缀
  "params": [...],
  "id": 1
}
```

**结果**:
```json
{
  "jsonrpc": "2.0",
  "error": {
    "code": -32601,
    "message": "Method not found"
  },
  "id": 1
}
```

## ✅ 正确理解

1. **内部实现**: Rust trait 中用简洁的方法名 `sponsorUserOperation`
2. **框架处理**: jsonrpsee 自动添加命名空间前缀 `pm_`
3. **外部调用**: 所有 JSON-RPC 客户端必须使用 `pm_sponsorUserOperation`

## 📋 命名空间的作用

### 避免方法名冲突
```rust
// 不同服务可能有相同的方法名
namespace = "pm"    → pm_sponsorUserOperation
namespace = "eth"   → eth_sponsorUserOperation
namespace = "debug" → debug_sponsorUserOperation
```

### 标准化 API 命名
- `pm_*` 代表 Paymaster 相关方法
- `eth_*` 代表以太坊标准方法
- 符合 JSON-RPC 生态约定

## 🔄 WebUI 调用链

```
WebUI Swagger → HTTP Request → Swagger 代理 → JSON-RPC 调用
                                    ↓
                              "pm_sponsorUserOperation"
                                    ↓
                              PaymasterRelayApi trait
                                    ↓
                              sponsor_user_operation() 实际执行
```

## 🎯 总结

**WebUI 必须使用 `pm_sponsorUserOperation`** 是因为：

1. **框架要求**: jsonrpsee 的命名空间机制
2. **协议标准**: JSON-RPC 2.0 的方法路由规则
3. **服务发现**: 服务器只注册了带前缀的方法名

这不是可选的设计决定，而是框架的**技术要求**。