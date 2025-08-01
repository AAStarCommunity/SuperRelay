# SuperRelay 用户安装指南

## 📋 系统要求

### 最低配置
- **操作系统**: Linux/macOS/Windows (推荐 Ubuntu 20.04+)
- **内存**: 最少 4GB RAM (推荐 8GB+)
- **存储**: 最少 20GB 可用空间
- **网络**: 稳定的互联网连接

### 软件依赖
- **Rust**: 1.75+ (最新稳定版)
- **Node.js**: 23.0+ (如需运行demo)
- **Git**: 2.30+
- **Docker**: 20.10+ (可选，用于容器化部署)

## 🚀 快速安装

### 方式一：从源码安装 (推荐)

```bash
# 1. 克隆仓库
git clone https://github.com/your-org/super-relay.git
cd super-relay

# 2. 安装 Rust (如果未安装)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# 3. 编译安装
cargo build --release --all

# 4. 验证安装
./target/release/super-relay --version
```

### 方式二：Docker 安装

```bash
# 1. 拉取镜像
docker pull super-relay:latest

# 2. 运行容器
docker run -d \
  --name super-relay \
  -p 3000:3000 \
  -p 9000:9000 \
  -e SIGNER_PRIVATE_KEYS="your_private_key" \
  -e NODE_HTTP="https://your-ethereum-node" \
  super-relay:latest
```

## ⚙️ 配置设置

### 1. 环境变量配置

创建 `.env` 文件 (仅开发环境):
```bash
# Ethereum 网络配置
NETWORK=mainnet
NODE_HTTP=https://eth-mainnet.alchemyapi.io/v2/your-api-key

# 私钥配置 (生产环境使用环境变量)
SIGNER_PRIVATE_KEYS=0x1234567890abcdef...

# Paymaster 配置
PAYMASTER_PRIVATE_KEY=0xabcdef1234567890...
```

### 2. 配置文件设置

复制并编辑配置文件:
```bash
cp config/config.toml config/production.toml
# 编辑 production.toml 根据你的需求
```

主要配置项:
- **网络设置**: RPC节点地址、网络ID
- **服务端口**: API服务端口配置
- **速率限制**: API调用频率控制
- **Paymaster策略**: Gas赞助规则配置

## 🔧 服务启动

### 开发环境启动
```bash
# 启动完整服务
cargo run --bin super-relay node --config config/config.toml

# 或使用便捷脚本
./scripts/start_dev_server.sh
```

### 生产环境启动
```bash
# 使用systemd管理服务
sudo cp scripts/super-relay.service /etc/systemd/system/
sudo systemctl enable super-relay
sudo systemctl start super-relay

# 检查服务状态
sudo systemctl status super-relay
```

## 🌐 服务访问

安装成功后，可以通过以下地址访问服务:

- **JSON-RPC API**: http://localhost:3000
- **Swagger UI**: http://localhost:9000/swagger-ui/
- **健康检查**: http://localhost:9000/health
- **监控指标**: http://localhost:8080/metrics

## 🧪 验证安装

### 1. 服务状态检查
```bash
# 检查所有服务状态
cargo run --bin super-relay status

# 或使用curl检查
curl http://localhost:9000/health
```

### 2. API功能测试
```bash
# 测试基础RPC功能
curl -X POST http://localhost:3000 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"eth_chainId","params":[],"id":1}'

# 测试Paymaster功能
curl -X POST http://localhost:3000 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"pm_sponsorUserOperation","params":[...],"id":1}'
```

## 🔄 更新升级

### 从源码更新
```bash
# 1. 拉取最新代码
git pull origin main

# 2. 重新编译
cargo build --release --all

# 3. 重启服务
sudo systemctl restart super-relay
```

### Docker更新
```bash
# 1. 拉取新镜像
docker pull super-relay:latest

# 2. 停止旧容器
docker stop super-relay

# 3. 启动新容器
docker run --name super-relay-new [same-parameters-as-before]
```

## 🚨 故障排除

### 常见问题

**Q: 编译失败 "error: failed to compile"**
A: 确保Rust版本 >= 1.75，运行 `rustup update`

**Q: 服务启动失败 "bind: address already in use"**  
A: 检查端口占用 `lsof -i :3000`，或修改配置文件中的端口

**Q: RPC调用失败 "network connection error"**
A: 检查NODE_HTTP配置是否正确，确保网络连接正常

### 日志查看
```bash
# 查看服务日志
sudo journalctl -u super-relay -f

# 查看应用日志
tail -f logs/super-relay.log
```

### 支持联系
- **GitHub Issues**: https://github.com/your-org/super-relay/issues
- **文档Wiki**: https://github.com/your-org/super-relay/wiki
- **社区讨论**: https://discord.gg/your-community

## 📄 许可证

SuperRelay 基于 MIT/Apache-2.0 双重许可证发布。详见 [LICENSE](../LICENSE) 文件。