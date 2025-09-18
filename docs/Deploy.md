# Rundler Fly.io 部署指南

## 版本: v0.1.18

本文档描述了如何将 Rundler ERC-4337 bundler 部署到 Fly.io 平台的完整流程。

## 🎯 部署概述

### 架构特点
- **零重构**: 基于现有 Rundler 代码，无需修改业务逻辑
- **容器化**: 使用 Docker 多阶段构建优化镜像大小
- **自动扩缩**: 支持基于负载的自动扩缩容
- **健康监控**: 内置健康检查和指标收集
- **安全性**: 非 root 用户运行，环境变量管理敏感信息

### 成本预估
- **开发/测试**: $2-5/月 (1 台 shared-cpu-1x 机器)
- **小规模生产**: $10-15/月 (2 台 shared-cpu-2x 机器)
- **中等规模**: $30-50/月 (3 台 dedicated-cpu-2x 机器)

## 📋 前置准备

### 1. Fly.io 账户设置
```bash
# 安装 Fly.io CLI
curl -L https://fly.io/install.sh | sh

# 登录账户
flyctl auth login

# 验证安装
flyctl version
```

### 2. 环境变量配置
需要配置以下关键环境变量：

#### 必需变量
```bash
NODE_HTTP=<ethereum_rpc_endpoint>     # 以太坊节点 RPC URL
NETWORK=<network_name>                # 网络名称 (ethereum, ethereum_sepolia, etc.)
```

#### 可选变量 (已有默认值)
```bash
RUST_LOG=info                         # 日志级别
MAX_VERIFICATION_GAS=5000000         # 最大验证 gas
PROVIDER_CLIENT_TIMEOUT_SECONDS=30   # 提供者超时
```

### 3. 密钥管理
```bash
# 设置以太坊节点 RPC URL (必需)
flyctl secrets set NODE_HTTP="https://your-ethereum-rpc-endpoint.com"

# 设置网络 (可选，默认为 ethereum_sepolia)
flyctl secrets set NETWORK="ethereum"

# 如果需要私钥进行交易签名 (Builder 组件)
flyctl secrets set PRIVATE_KEY="your_private_key_here"

# 如果使用 AWS KMS 等外部签名服务
flyctl secrets set AWS_ACCESS_KEY_ID="your_aws_key"
flyctl secrets set AWS_SECRET_ACCESS_KEY="your_aws_secret"
```

## 🚀 部署流程

### 步骤 1: 准备代码
```bash
# 确保在 deploy 分支
git checkout deploy

# 验证关键文件存在
ls -la Dockerfile fly.toml

# 验证配置文件
ls -la bin/rundler/chain_specs/
```

### 步骤 2: 创建 Fly.io 应用
```bash
# 创建新应用 (如果还未创建)
flyctl apps create rundler-superrelay

# 或者使用现有配置初始化
flyctl launch --no-deploy
```

### 步骤 3: 配置资源和环境
```bash
# 设置必需的密钥
flyctl secrets set NODE_HTTP="https://your-rpc-endpoint.com"

# 可选：调整资源配置
flyctl scale memory 2048  # 2GB 内存
flyctl scale count 1      # 开始时使用 1 台机器
```

### 步骤 4: 执行部署
```bash
# 构建并部署
flyctl deploy

# 监控部署进度
flyctl logs -f
```

### 步骤 5: 验证部署
```bash
# 检查应用状态
flyctl status

# 检查健康状态
flyctl checks list

# 测试 RPC 端点
curl -X POST https://rundler-superrelay.fly.dev \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"web3_clientVersion","params":[],"id":1}'

# 检查指标端点
curl https://rundler-superrelay.fly.dev:8080/health
```

## 📊 监控和管理

### 查看日志
```bash
# 实时日志
flyctl logs -f

# 历史日志
flyctl logs --lines 100

# 按组件过滤日志
flyctl logs -f | grep "rundler"
```

### 性能监控
```bash
# 查看应用指标
flyctl status

# 查看机器使用情况
flyctl machine list

# 监控内存和 CPU 使用
flyctl machine status <machine_id>
```

### 扩缩容管理
```bash
# 手动扩容
flyctl scale count 3

# 调整内存
flyctl scale memory 4096

# 查看当前配置
flyctl scale show
```

## 🔧 配置调优

### 网络配置
- **主区域**: `sea` (西雅图) - 对区块链工作负载友好
- **备用区域**: `ewr` (纽约), `fra` (法兰克福)
- **延迟优化**: 根据主要用户群体选择区域

### 性能调优
```toml
# fly.toml 中的关键配置
[vm]
  memory = "2gb"    # 根据负载调整
  cpu_kind = "shared"  # 或 "dedicated" for 高性能
  cpus = 2

[http_service.concurrency]
  soft_limit = 100
  hard_limit = 200
```

### 环境变量优化
```bash
# 连接池大小
flyctl secrets set MAX_CONCURRENT_CONNECTIONS="100"

# 超时配置
flyctl secrets set PROVIDER_CLIENT_TIMEOUT_SECONDS="30"

# 日志级别
flyctl secrets set RUST_LOG="info"  # debug, info, warn, error
```

## 🔍 故障排除

### 常见问题

#### 1. 构建失败
```bash
# 检查 Dockerfile 语法
docker build -t rundler-test .

# 查看构建日志
flyctl logs --lines 50
```

#### 2. 应用启动失败
```bash
# 检查环境变量
flyctl secrets list

# 验证 RPC 连接
curl -X POST $NODE_HTTP \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"web3_clientVersion","params":[],"id":1}'
```

#### 3. 健康检查失败
```bash
# 检查端口配置
flyctl machine list

# 测试健康端点
flyctl ssh console
curl localhost:8080/health
```

#### 4. 内存不足
```bash
# 查看内存使用
flyctl machine status

# 增加内存
flyctl scale memory 4096
```

### 调试命令
```bash
# SSH 进入容器
flyctl ssh console

# 查看进程状态
flyctl ssh console -C "ps aux"

# 查看磁盘使用
flyctl ssh console -C "df -h"

# 查看网络连接
flyctl ssh console -C "netstat -tlnp"
```

## 🛡️ 安全最佳实践

### 1. 密钥管理
- 使用 `flyctl secrets` 管理敏感信息
- 定期轮换 API 密钥和私钥
- 不要在代码中硬编码密钥

### 2. 网络安全
- 启用 HTTPS 强制跳转
- 配置适当的 CORS 策略
- 使用内部端口隔离

### 3. 运行时安全
- 非 root 用户运行应用
- 最小化容器权限
- 定期更新基础镜像

## 📈 扩展部署

### 多区域部署
```bash
# 部署到多个区域
flyctl regions add ewr fra

# 查看区域状态
flyctl regions list
```

### 分离服务部署
如需将 RPC、Pool、Builder 分离部署：

1. 修改 `fly.toml` 中的 `[processes]` 配置
2. 为每个服务创建独立的应用
3. 配置内部通信网络

### 持久化存储
如需持久化数据：
```bash
# 创建卷
flyctl volumes create rundler_data --size 10

# 在 fly.toml 中配置挂载
[[mounts]]
  source = "rundler_data"
  destination = "/data"
```

## 📚 参考资源

- [Fly.io 官方文档](https://fly.io/docs/)
- [Rundler GitHub 仓库](https://github.com/alchemyplatform/rundler)
- [ERC-4337 规范](https://eips.ethereum.org/EIPS/eip-4337)
- [Docker 最佳实践](https://docs.docker.com/develop/dev-best-practices/)

## 🆘 获取帮助

如遇到问题，可以：
1. 查看 `flyctl logs` 了解错误信息
2. 参考本文档的故障排除部分
3. 联系开发团队或提交 GitHub Issue
4. 查阅 Fly.io 社区论坛

---

**注意**: 本文档基于 Rundler v0.9.0 和 Fly.io 当前 API。在部署前请确保版本兼容性。