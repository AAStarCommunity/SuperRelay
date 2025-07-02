# Deploy Guide

本文档提供 Super-Relay 项目的部署、初始化和维护指南，面向运维和开发同学。

## 系统要求

### 最低要求
- **操作系统**: Linux (Ubuntu 20.04+), macOS (10.15+)
- **内存**: 4GB RAM
- **存储**: 10GB 可用空间
- **网络**: 稳定的互联网连接，访问 Ethereum 节点

### 推荐配置
- **内存**: 8GB+ RAM
- **CPU**: 4+ 核心
- **存储**: 50GB+ SSD
- **网络**: 专用 RPC 节点或高质量的 RPC 服务

## 构建环境准备

### 1. 安装 Rust 工具链
```bash
# 安装 Rust (使用 rustup)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# 验证安装
rustc --version
cargo --version
```

### 2. 安装系统依赖
```bash
# Ubuntu/Debian
sudo apt update
sudo apt install -y build-essential pkg-config libssl-dev clang

# macOS (需要 Xcode Command Line Tools)
xcode-select --install

# 或使用 Homebrew
brew install llvm
```

### 3. 克隆项目
```bash
git clone <your-super-relay-repository>
cd super-relay

# 初始化子模块
git submodule update --init --recursive
```

## 编译与构建

### 开发环境构建
```bash
# 编译 (Debug 模式)
cargo build

# 运行测试
cargo test

# 特定模块测试
cargo test --package rundler-paymaster-relay
```

### 生产环境构建
```bash
# Release 模式编译 (优化版本)
cargo build --release

# 二进制文件位置
ls -la target/release/rundler
```

### 交叉编译 (可选)
```bash
# 为 Linux 构建 (在 macOS 上)
cargo install cross
cross build --target x86_64-unknown-linux-gnu --release
```

## 配置文件

### 1. 基础配置
创建 `config.toml`:
```toml
# RPC 端点配置
[rpc]
listen_address = "127.0.0.1:3000"
max_connections = 100

# Entry Point 合约地址
[entry_points]
v0_6 = "0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789"
v0_7 = "0x0000000071727De22E5E9d8BAf0edAc6f37da032"

# 链配置
[chain]
id = 1  # Mainnet = 1, Sepolia = 11155111
rpc_url = "https://eth-mainnet.alchemyapi.io/v2/YOUR_API_KEY"
```

### 2. Paymaster 配置
创建 `paymaster-policies.toml`:
```toml
[default]
senders = [
    "0x1234567890123456789012345678901234567890",
    "0xabcdefabcdefabcdefabcdefabcdefabcdefabcd"
]

[premium_policy]
senders = ["0x9876543210987654321098765432109876543210"]
max_gas_limit = 1000000
```

### 3. 环境变量
创建 `.env` 文件:
```bash
# Paymaster 私钥 (用于签名)
PAYMASTER_PRIVATE_KEY=0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef

# 日志级别
RUST_LOG=info,rundler=debug

# 可选: AWS KMS 配置 (未来版本)
# AWS_REGION=us-east-1
# AWS_ACCESS_KEY_ID=your_key
# AWS_SECRET_ACCESS_KEY=your_secret
```

## 部署步骤

### 1. 基础部署
```bash
# 1. 准备工作目录
sudo mkdir -p /opt/super-relay
sudo chown $USER:$USER /opt/super-relay
cd /opt/super-relay

# 2. 复制编译好的二进制文件
cp /path/to/build/target/release/rundler ./

# 3. 复制配置文件
cp config.toml .
cp paymaster-policies.toml .
cp .env .

# 4. 设置权限
chmod +x rundler
chmod 600 .env  # 保护私钥文件
```

### 2. 系统服务配置 (Systemd)
创建 `/etc/systemd/system/super-relay.service`:
```ini
[Unit]
Description=Super-Relay Paymaster Service
After=network.target

[Service]
Type=simple
User=super-relay
Group=super-relay
WorkingDirectory=/opt/super-relay
Environment=RUST_LOG=info
EnvironmentFile=/opt/super-relay/.env
ExecStart=/opt/super-relay/rundler \
    --rpc.listen-address 0.0.0.0:3000 \
    --paymaster.enabled \
    --paymaster.policy-file /opt/super-relay/paymaster-policies.toml \
    node --rpc.url ws://localhost:8546
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
```

### 3. 启动服务
```bash
# 创建用户
sudo useradd -r -s /bin/false super-relay

# 设置文件权限
sudo chown -R super-relay:super-relay /opt/super-relay

# 启用并启动服务
sudo systemctl daemon-reload
sudo systemctl enable super-relay
sudo systemctl start super-relay

# 检查状态
sudo systemctl status super-relay
```

## 运行参数

### 基本启动命令
```bash
./rundler \
    --rpc.listen-address 0.0.0.0:3000 \
    --paymaster.enabled \
    --paymaster.policy-file ./paymaster-policies.toml \
    node \
    --rpc.url ws://localhost:8546 \
    --chain-id 1
```

### 完整参数示例
```bash
./rundler \
    --rpc.listen-address 0.0.0.0:3000 \
    --rpc.max-connections 1000 \
    --builder.enabled \
    --pool.enabled \
    --paymaster.enabled \
    --paymaster.policy-file ./policies.toml \
    node \
    --rpc.url ws://your-ethereum-node:8546 \
    --chain-id 1 \
    --entry-points 0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789
```

## 监控与维护

### 1. 日志监控
```bash
# 查看实时日志
sudo journalctl -u super-relay -f

# 查看最近的日志
sudo journalctl -u super-relay --since "1 hour ago"

# 查看错误日志
sudo journalctl -u super-relay -p err
```

### 2. 性能监控
```bash
# 检查服务状态
curl http://localhost:3000/health

# 检查 RPC 端点
curl -X POST http://localhost:3000 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"web3_clientVersion","params":[],"id":1}'
```

### 3. 配置热重载
```bash
# 重新加载策略配置 (如果支持)
curl -X POST http://localhost:3000 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"pm_admin_reloadPolicies","params":[],"id":1}'
```

## 安全建议

### 1. 私钥管理
- 使用强随机生成的私钥
- 定期轮换私钥
- 考虑使用 AWS KMS 或 Azure Key Vault
- 永远不要在日志中暴露私钥

### 2. 网络安全
- 使用防火墙限制访问端口
- 启用 HTTPS/WSS (生产环境)
- 使用 VPN 或专用网络
- 定期更新系统和依赖

### 3. 访问控制
```bash
# 设置严格的文件权限
chmod 700 /opt/super-relay
chmod 600 /opt/super-relay/.env
chown -R super-relay:super-relay /opt/super-relay
```

## 故障排查

### 1. 常见问题
- **编译失败**: 检查 Rust 工具链版本，确保子模块已初始化
- **启动失败**: 检查配置文件格式，验证 RPC 端点连接
- **签名错误**: 验证私钥格式和权限

### 2. 调试模式
```bash
# 启用详细日志
RUST_LOG=debug ./rundler [options]

# 检查配置
./rundler --help
```

### 3. 备份与恢复
```bash
# 备份配置
tar -czf super-relay-backup-$(date +%Y%m%d).tar.gz \
  /opt/super-relay/*.toml \
  /opt/super-relay/.env

# 监控磁盘空间
df -h /opt/super-relay
```

## 更新流程

### 1. 更新代码
```bash
# 停止服务
sudo systemctl stop super-relay

# 更新代码
git pull origin main
git submodule update --recursive

# 重新编译
cargo build --release

# 备份当前版本
cp /opt/super-relay/rundler /opt/super-relay/rundler.backup

# 部署新版本
cp target/release/rundler /opt/super-relay/

# 重启服务
sudo systemctl start super-relay
```

### 2. 回滚步骤
```bash
# 如果新版本有问题，回滚到备份版本
sudo systemctl stop super-relay
cp /opt/super-relay/rundler.backup /opt/super-relay/rundler
sudo systemctl start super-relay
```

## 生产环境检查清单

- [ ] 系统依赖已安装
- [ ] 防火墙配置正确
- [ ] SSL 证书配置 (如需要)
- [ ] 监控系统配置
- [ ] 日志轮转配置
- [ ] 备份策略实施
- [ ] 私钥安全存储
- [ ] 性能基准测试完成
- [ ] 灾难恢复计划制定

## 联系信息

- **技术支持**: [技术团队邮箱]
- **文档**: 查看 `docs/` 目录下的其他文档
- **问题报告**: 提交 GitHub Issue 