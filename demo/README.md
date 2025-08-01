# SuperPaymaster Demo

这个目录包含了SuperPaymaster的完整演示和测试工具。

## 🚀 快速开始

### 一句话API测试（假设服务已启动）

```bash
# 核心能力测试 - JSON-RPC API
curl -X POST http://localhost:3000 -H "Content-Type: application/json" -d '{"jsonrpc":"2.0","id":1,"method":"pm_sponsorUserOperation","params":[{"sender":"0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266","nonce":"0x0","initCode":"0x","callData":"0x","callGasLimit":"0x186A0","verificationGasLimit":"0x186A0","preVerificationGas":"0x5208","maxFeePerGas":"0x3B9ACA00","maxPriorityFeePerGas":"0x3B9ACA00","paymasterAndData":"0x","signature":"0x"},"0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789"]}' | jq '.result'
```

## 📁 文件说明

### 核心Demo文件
- **`superPaymasterDemo.js`** - 完整的Node.js演示程序，展示所有核心功能
- **`interactive-demo.html`** - Web UI交互式演示页面
- **`curl-test.sh`** - 简单的curl命令测试脚本

### 配置文件
- **`package.json`** - Node.js依赖配置
- **`package-lock.json`** - 锁定版本

## 🎮 使用方法

### 方法1: 命令行测试脚本
```bash
# 运行所有API测试
./curl-test.sh

# 设置自定义服务器地址
SUPER_RELAY_URL=http://your-server:3000 ./curl-test.sh
```

### 方法2: Node.js完整演示
```bash
# 安装依赖
npm install

# 运行演示程序
node superPaymasterDemo.js

# 查看帮助
node superPaymasterDemo.js --help
```

### 方法3: Web交互界面
```bash
# 在浏览器中打开
open interactive-demo.html

# 或者使用简单HTTP服务器
python3 -m http.server 8080
# 然后访问 http://localhost:8080/interactive-demo.html
```

## ⚙️ 配置说明

### 环境变量
```bash
# SuperRelay服务地址
export SUPER_RELAY_URL="http://localhost:3000"

# EntryPoint合约地址
export ENTRY_POINT_ADDRESS="0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789"

# 本地Anvil RPC地址
export RPC_URL="http://localhost:8545"
```

### 测试账户
Demo使用Anvil默认测试账户：
- **用户账户**: `0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266`
- **Paymaster账户**: `0x70997970C51812dc3A010C7d01b50e0d17dc79C8`

## 🔧 前置条件

1. **启动Anvil测试网络**
```bash
anvil
```

2. **部署EntryPoint合约**
```bash
# 在项目根目录执行
./scripts/deploy_entrypoint.sh
```

3. **启动SuperRelay服务**
```bash
# 方法1: 使用rundler直接启动
cargo run --bin rundler -- \
  --rpc.listen 127.0.0.1:3000 \
  --eth-client-address http://localhost:8545 \
  --chain-id 31337 \
  --entry-points 0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789 \
  --paymaster.enabled true

# 方法2: 使用配置文件启动（推荐）
./scripts/start_dev_server.sh
```

## 📊 测试功能

### 核心功能测试
1. **用户操作赞助** - `pm_sponsorUserOperation`
2. **多版本支持** - ERC-4337 v0.6 和 v0.7
3. **策略检查** - 基于白名单的访问控制
4. **Gas抽象** - 用户无需持有ETH支付gas费用

### API端点测试
- **JSON-RPC**: `POST /` - 标准ERC-4337 JSON-RPC接口
- **REST API**: `POST /api/v1/sponsor` - RESTful接口
- **健康检查**: `GET /health` - 服务状态检查
- **指标监控**: `GET /metrics` - 服务指标
- **Swagger UI**: `GET /swagger-ui/` - API文档界面

## 🐛 故障排除

### 常见问题

1. **连接失败**
   ```bash
   # 检查服务是否启动
   curl -s http://localhost:3000/health
   ```

2. **Policy Rejected错误**
   ```bash
   # 检查用户地址是否在白名单中
   # 默认白名单: 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266
   ```

3. **EntryPoint未找到**
   ```bash
   # 重新部署EntryPoint合约
   ./scripts/deploy_entrypoint.sh
   ```

## 🔗 相关链接

- **Swagger UI**: http://localhost:3000/swagger-ui/
- **健康检查**: http://localhost:3000/health
- **指标监控**: http://localhost:3000/metrics
- **项目文档**: [../docs/](../docs/)

## 📝 示例响应

### 成功响应
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
}
```

### 错误响应
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "error": {
    "code": -32604,
    "message": "Policy rejected: Sender 0x... is not in the allowlist"
  }
}
```

---

✨ **提示**: 更多详细信息请查看 [superPaymasterDemo.js](./superPaymasterDemo.js) 中的完整实现和注释。