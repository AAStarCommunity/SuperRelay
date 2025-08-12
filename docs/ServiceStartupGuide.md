# SuperRelay 服务启动指南

**更新时间**: 2025-08-05
**适用版本**: SuperRelay v0.1.5

## 🚀 快速启动

### 1. 前置条件
```bash
# 确保已安装必要工具
# - Rust 1.75+
# - Foundry (anvil, cast)
# - curl (用于测试)

# 构建SuperRelay
cargo build --package super-relay
```

### 2. 启动测试环境

#### 启动本地测试链
```bash
# 终端1: 启动Anvil本地测试链
anvil --port 8545 --host 0.0.0.0 --chain-id 31337
```

#### 启动SuperRelay服务
```bash
# 终端2: 设置环境变量并启动SuperRelay Gateway
export PAYMASTER_PRIVATE_KEY=0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80

# 方式1: 使用Gateway模式 (推荐)
./target/debug/super-relay gateway \
    --config config/config.toml \
    --host 0.0.0.0 \
    --port 3000 \
    --enable-paymaster \
    --paymaster-private-key "$PAYMASTER_PRIVATE_KEY"

# 方式2: 使用Node兼容模式
./target/debug/super-relay node --config config/config.toml
```

### 3. 验证服务状态

#### 基础连接测试
```bash
# 测试RPC接口
curl -X POST http://localhost:3000 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc": "2.0", "id": 1, "method": "eth_supportedEntryPoints", "params": []}'

# 预期响应
{"id":1,"jsonrpc":"2.0","result":["0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789"]}
```

#### Paymaster接口测试
```bash
# 测试Paymaster功能
curl -X POST http://localhost:3000 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "pm_sponsorUserOperation",
    "params": [{
      "sender": "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266",
      "nonce": "0x0",
      "callData": "0x",
      "callGasLimit": "0x186A0",
      "verificationGasLimit": "0x186A0",
      "preVerificationGas": "0x5208",
      "maxFeePerGas": "0x3B9ACA00",
      "maxPriorityFeePerGas": "0x3B9ACA00"
    }, "0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789"]
  }'
```

## 📋 命令详解

### SuperRelay 命令结构
```
super-relay <COMMAND>

Commands:
  gateway  # 企业级API Gateway模式 (推荐)
  node     # 兼容模式，内部调用rundler
  pool     # 独立Pool服务
  builder  # 独立Builder服务
  admin    # 管理工具
  version  # 版本信息
  status   # 服务状态
```

### Gateway模式参数
```bash
super-relay gateway [OPTIONS]

Options:
  --config <CONFIG>                    # 配置文件路径
  --host <HOST>                        # 绑定主机地址 [default: 127.0.0.1]
  --port <PORT>                        # 绑定端口 [default: 3000]
  --enable-paymaster                   # 启用Paymaster服务
  --paymaster-private-key <KEY>        # Paymaster私钥
  --paymaster-policy-file <FILE>       # Paymaster策略文件
```

## 🔧 配置说明

### 主要配置文件: config/config.toml
```toml
[node]
http_api = "0.0.0.0:3000"
network = "dev"
node_http = "http://localhost:8545"

[paymaster_relay]
enabled = true
private_key = "${PAYMASTER_PRIVATE_KEY}"
policy_file = "config/paymaster-policies.toml"

[rate_limiting]
enabled = true
requests_per_second = 100
```

## 🧪 测试脚本

### 运行基础功能测试
```bash
# 运行基础Gateway功能测试
./scripts/test_basic_gateway.sh

# 运行综合规范符合性测试
./scripts/test_spec_comprehensive.sh

# 运行健康检查测试
./scripts/test_health_system.sh
```

## ⚠️ 故障排除

### 常见问题

1. **端口占用错误**
   ```bash
   # 检查端口占用
   lsof -i :3000
   lsof -i :8545

   # 清理进程
   pkill -f "super-relay|anvil"
   ```

2. **配置文件错误**
   ```bash
   # 检查配置文件格式
   cat config/config.toml | head -20

   # 验证环境变量
   echo $PAYMASTER_PRIVATE_KEY
   ```

3. **依赖项问题**
   ```bash
   # 重新构建
   cargo clean
   cargo build --package super-relay

   # 检查Foundry安装
   anvil --version
   cast --version
   ```

## 🔒 生产环境配置

### 安全配置建议
1. **私钥管理**: 使用环境变量而非配置文件
2. **网络绑定**: 生产环境使用内网IP
3. **速率限制**: 根据业务需求调整
4. **监控告警**: 配置Prometheus指标收集

### 生产启动示例
```bash
# 生产环境启动
export PAYMASTER_PRIVATE_KEY="your-production-key"

./target/release/super-relay gateway \
    --config config/production.toml \
    --host 10.0.1.100 \
    --port 3000 \
    --enable-paymaster \
    --paymaster-private-key "$PAYMASTER_PRIVATE_KEY"
```

---

**注意事项**:
- 测试环境使用Anvil本地链，生产环境需要连接实际区块链网络
- Paymaster私钥必须有足够ETH用于Gas赞助
- 企业级部署需要考虑负载均衡和高可用配置
- 定期更新依赖项和安全补丁