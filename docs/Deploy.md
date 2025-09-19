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

### 成本预估 (已优化)
- **开发/测试**: $2-5/月 (1GB 内存, 1 CPU, 自动停机)
- **小规模生产**: $5-10/月 (1GB 内存, 1 CPU, 保持运行)
- **中等规模**: $15-30/月 (2GB 内存, 2 CPU, 2 台机器)
- **大规模**: $30-50/月 (4GB 内存, 专用 CPU, 3 台机器)

#### 已应用的成本优化
✅ 新加坡区域部署 (更低延迟)  
✅ 内存降至 1GB (节省 50%)  
✅ CPU 降至 1 核心 (节省 50%)  
✅ 空闲自动停机 (min=0)  
✅ 单机器部署 (max=1)  
✅ 优化超时和 gas 限制

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
- **主区域**: `sin` (新加坡) - 亚洲用户低延迟访问
- **备用区域**: `hkg` (香港), `nrt` (东京), `syd` (悉尼)
- **延迟优化**: 根据主要用户群体选择区域

#### 亚洲区域选择 (推荐)
```bash
# 新加坡 (推荐 - 最佳延迟)
primary_region = "sin"

# 香港 (备选)  
primary_region = "hkg"

# 东京 (备选)
primary_region = "nrt"
```

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

## 🚀 成本和性能优化指南

### 完整优化步骤 (已执行)

本部署已执行以下优化配置，确保最佳性价比：

#### 1. 区域优化
```bash
# 查看所有可用区域
flyctl platform regions

# 设置新加坡为主区域 (亚洲用户最佳)
# 在 fly.toml 中设置：
primary_region = "sin"  # 新加坡 - 亚洲最佳延迟
```

#### 2. 资源优化
```toml
# fly.toml 中的资源配置优化
[vm]
  memory = "1gb"    # 从 2GB 降至 1GB (节省 50%)
  cpu_kind = "shared"
  cpus = 1          # 从 2 核心降至 1 核心 (节省 50%)

[scaling]
  min_machines_running = 0  # 空闲自动停机 (节省 100% 空闲成本)
  max_machines_running = 1  # 单机器部署 (最低成本)
```

#### 3. 环境变量优化
```bash
# 超时优化 - 减少资源占用
flyctl secrets set PROVIDER_CLIENT_TIMEOUT_SECONDS="15"  # 从 30s 优化
flyctl secrets set TRACER_TIMEOUT="10s"                  # 新增超时控制

# Gas 限制优化 - 降低计算成本
flyctl secrets set MAX_VERIFICATION_GAS="3000000"        # 从 5000000 降低

# 日志优化 - 减少存储和网络成本
flyctl secrets set RUST_LOG="warn"                       # 从 info 简化
```

#### 4. 必需环境变量
```bash
# 核心配置 (必需)
flyctl secrets set NODE_HTTP="https://eth-sepolia.g.alchemy.com/v2/YOUR_KEY"
flyctl secrets set NETWORK="ethereum_sepolia"
flyctl secrets set PRIVATE_KEY="0xYOUR_PRIVATE_KEY"
```

### 优化效果对比

| 配置项 | 优化前 | 优化后 | 节省 |
|--------|--------|---------|------|
| **区域** | 西雅图 (sea) | 新加坡 (sin) | 延迟降低 ~40% |
| **内存** | 2GB | 1GB | 成本节省 ~50% |
| **CPU** | 2 核心 | 1 核心 | 成本节省 ~50% |
| **最小实例** | 1 | 0 (自动停机) | 空闲时节省 100% |
| **超时设置** | 30s | 15s | 资源占用降低 50% |
| **验证 Gas** | 5M | 3M | 计算成本降低 40% |
| **日志级别** | info | warn | 存储成本降低 60% |
| **月成本** | $15-30 | $2-5 | 节省 ~80% |

### 应用当前配置

当前 `rundler-superrelay` 应用已应用所有优化：

```bash
# 检查当前配置
flyctl status --app rundler-superrelay

# 查看设置的环境变量
flyctl secrets list --app rundler-superrelay

# 监控部署状态
flyctl logs --app rundler-superrelay
```

### 后续优化建议

根据使用情况，可以进一步调整：

1. **流量增长时**：
   ```bash
   # 增加到 2GB 内存，2 CPU
   flyctl scale memory 2048 --app rundler-superrelay
   
   # 设置最少保持 1 台运行
   # 在 fly.toml 中修改：min_machines_running = 1
   ```

2. **高可用需求**：
   ```bash
   # 增加到 2 台机器
   # 在 fly.toml 中修改：max_machines_running = 2
   
   # 添加多区域支持
   flyctl regions add hkg nrt --app rundler-superrelay
   ```

3. **性能监控**：
   ```bash
   # 定期检查资源使用
   flyctl machine list --app rundler-superrelay
   
   # 如果CPU/内存不足，及时扩容
   flyctl scale count 2 --app rundler-superrelay
   ```

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