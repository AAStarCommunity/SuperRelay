# SuperPaymaster 用户手册

SuperPaymaster 是一个企业级的 ERC-4337 Account Abstraction Paymaster 解决方案，为不同类型的用户提供完整的 gas 代付服务。

## 🎯 三种用户角色使用指南

### 🏗️ 运营者：运营 Bundler 和 Paymaster 服务

作为 SuperRelay 服务的运营者，你需要部署和维护整个基础设施。

#### 一次性初始化设置

**1. 代码库准备**
```bash
# 克隆 SuperRelay 代码库
git clone https://github.com/你的用户名/super-relay.git
cd super-relay

# 初始化和更新 submodules
git submodule update --init --recursive

# 构建项目
cargo build --release
```

**2. 环境配置**
```bash
# 复制环境配置模板
cp .env.example .env

# 编辑配置文件，设置私钥和网络参数
vi .env
```

必需的环境变量：
```bash
# Paymaster 私钥（推荐使用 AWS KMS）
PAYMASTER_PRIVATE_KEY=your_private_key_here

# EntryPoint 合约地址
ENTRY_POINT_ADDRESS=0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789

# RPC 节点配置
ETHEREUM_RPC_URL=https://your-ethereum-rpc.com
SEPOLIA_RPC_URL=https://your-sepolia-rpc.com

# 监控配置
PROMETHEUS_PORT=8080
SWAGGER_PORT=9000
```

**3. 启动服务**

**选项A：直接启动**
```bash
# 使用推荐的 super-relay 命令（未来版本）
./target/release/super-relay node --config config/config.toml

# 或当前版本
cargo run --bin rundler -- node --config config/config.toml
```

**选项B：Docker 部署**
```bash
# 构建 Docker 镜像
docker build -t super-relay:latest .

# 启动容器（带负载均衡）
docker run -d \
  --name super-relay-1 \
  -p 3000:3000 \
  -p 9000:9000 \
  -p 8080:8080 \
  --env-file .env \
  super-relay:latest

# 多实例负载均衡（pod 级别）
docker-compose up -d --scale super-relay=3
```

#### 日常监控和运维

**监控入口汇总**
- **Swagger UI**: http://localhost:9000/swagger-ui/
- **健康检查**: http://localhost:9000/health
- **API 统计**: http://localhost:9000/metrics
- **Prometheus 指标**: http://localhost:8080/metrics
- **主 RPC 服务**: http://localhost:3000

**关键监控指标**
```bash
# 健康检查
curl http://localhost:9000/health | jq .

# 查看实时指标
curl http://localhost:9000/metrics | jq .

# Prometheus 格式指标
curl http://localhost:8080/metrics
```

**运维操作清单**
1. **资金检查**: 定期检查 EntryPoint 余额，确保有足够 ETH 支付 gas
2. **日志监控**: 监控错误日志和性能指标
3. **服务重启**: 配置自动重启机制，确保高可用性
4. **备份恢复**: 定期备份配置和私钥（加密存储）

**告警配置**
- EntryPoint 余额不足（< 0.1 ETH）
- 服务响应时间过长（> 5秒）
- 错误率过高（> 5%）
- 内存使用过高（> 80%）

---

### 👨‍💻 开发者：使用 API 提交免 gas UserOperation

作为应用开发者，你将集成 SuperRelay API 来为用户提供免 gas 交易体验。

#### 快速开始

**1. 获取 API 访问权限**
```bash
# 目前无需 API key，提供账户地址即可
# 用户额度限制：
# - 日限额：100 次操作
# - 频率限额：每分钟 10 次
# - 总额度：基于账户地址的信誉评分
```

**2. 测试账户可访问性**
```bash
# 使用 curl 测试基础连接
curl -X POST http://localhost:9000/api/v1/sponsor \
  -H "Content-Type: application/json" \
  -d '{
    "user_op": {
      "sender": "0x1234567890123456789012345678901234567890",
      "nonce": "0x0",
      "initCode": "0x",
      "callData": "0x",
      "callGasLimit": "0x5208",
      "verificationGasLimit": "0x5208",
      "preVerificationGas": "0x5208",
      "maxFeePerGas": "0x3b9aca00",
      "maxPriorityFeePerGas": "0x3b9aca00",
      "paymasterAndData": "0x",
      "signature": "0x"
    },
    "entry_point": "0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789"
  }'
```

#### 集成示例代码

**JavaScript/TypeScript 集成**
```typescript
// SuperPaymaster 客户端类
class SuperPaymasterClient {
  constructor(private baseUrl: string = 'http://localhost:9000') {}

  async sponsorUserOperation(
    userOp: UserOperation,
    entryPoint: string
  ): Promise<string> {
    const response = await fetch(`${this.baseUrl}/api/v1/sponsor`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        user_op: userOp,
        entry_point: entryPoint
      })
    });

    if (!response.ok) {
      throw new Error(`API Error: ${response.status}`);
    }

    const result = await response.json();
    return result.user_op_hash;
  }

  async getHealthStatus() {
    const response = await fetch(`${this.baseUrl}/health`);
    return response.json();
  }
}

// 使用示例
const client = new SuperPaymasterClient();

// 赞助一个 UserOperation
const userOpHash = await client.sponsorUserOperation({
  sender: "0x...",
  nonce: "0x0",
  // ... 其他字段
}, "0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789");

console.log('User operation hash:', userOpHash);
```

**Python 集成**
```python
import requests
import json

class SuperPaymasterClient:
    def __init__(self, base_url='http://localhost:9000'):
        self.base_url = base_url
    
    def sponsor_user_operation(self, user_op, entry_point):
        """赞助一个 UserOperation"""
        response = requests.post(
            f'{self.base_url}/api/v1/sponsor',
            headers={'Content-Type': 'application/json'},
            json={
                'user_op': user_op,
                'entry_point': entry_point
            }
        )
        
        response.raise_for_status()
        return response.json()['user_op_hash']
    
    def get_health_status(self):
        """获取服务健康状态"""
        response = requests.get(f'{self.base_url}/health')
        return response.json()

# 使用示例
client = SuperPaymasterClient()

user_op = {
    "sender": "0x...",
    "nonce": "0x0",
    # ... 其他字段
}

try:
    user_op_hash = client.sponsor_user_operation(
        user_op, 
        "0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789"
    )
    print(f'User operation hash: {user_op_hash}')
except requests.exceptions.RequestException as e:
    print(f'API Error: {e}')
```

#### API 文档和工具

**文档入口**
- **交互式文档**: http://localhost:9000/swagger-ui/
- **代码示例生成**: 
  - curl: http://localhost:9000/codegen/curl/sponsor
  - JavaScript: http://localhost:9000/codegen/javascript/sponsor
  - Python: http://localhost:9000/codegen/python/sponsor

**支持的 UserOperation 版本**
- **ERC-4337 v0.6**: `/examples/v06`
- **ERC-4337 v0.7**: `/examples/v07`

---

### 👤 最终用户：使用免 gas 安全加密账户

作为最终用户，你可以享受免 gas 费的安全账户体验。

#### 免费账户申请

**个人用户**
1. **邮箱注册**: 发送邮件到 support@superrelay.com
2. **免费额度**: 每月 3 次免费转账
3. **账户特性**: 
   - 安全的多签账户
   - 社交恢复功能
   - 免基础日常 gas 费

**社区用户（增强权益）**
1. **ENS 设置**: 注册 .eth 域名
2. **社区注册**: 使用 COS72 注册社区身份
3. **增强额度**: 
   - 个人：每月 5 次免费操作
   - 社区合约：完全免 gas
   - 特殊活动：赞助空投和积分

#### 用户操作指南

**基础操作**
- ✅ 转账：支持 ETH 和 ERC-20 代币
- ✅ 合约交互：DeFi、NFT、DAO 投票
- ✅ 批量操作：一次交易执行多个操作
- ✅ 社交恢复：通过朋友恢复账户访问

**高级功能**
- 🎯 **智能代付**: 自动选择最优 gas 策略
- 🔐 **多重签名**: 企业级安全保护
- 📱 **移动友好**: 支持钱包 App 集成
- 🌐 **跨链操作**: 多网络无缝体验

**社区福利**
- 🎁 **积分任务**: 参与社区活动获得积分
- 🎉 **活动赞助**: 特殊事件期间的额外免费额度
- 🏆 **声誉系统**: 基于使用行为的信誉评级
- 💫 **空投权益**: 优先获得新项目代币空投

#### 使用流程

**1. 账户创建**
```
用户邮箱申请 → 身份验证 → 账户激活 → 获得初始额度
```

**2. 日常使用**
```
发起交易 → 系统检查额度 → 自动代付 gas → 交易确认
```

**3. 额度管理**
```
查看剩余额度 → 申请额外权益 → 社区身份认证 → 获得增强服务
```

---

## 📚 相关文档链接

### 技术文档
- **架构分析**: [docs/Architecture-Analysis.md](./Architecture-Analysis.md)
- **API 分析**: [docs/API-Analysis.md](./API-Analysis.md)
- **测试指南**: [docs/Testing-Analysis.md](./Testing-Analysis.md)

### 部署文档
- **部署指南**: [docs/Deploy.md](./Deploy.md)
- **安装手册**: [docs/Install.md](./Install.md)
- **配置参考**: [config/](../config/)

### 系统架构
- **aggregators**: [docs/architecture/aggregators.md](./architecture/aggregators.md)
- **builder**: [docs/architecture/builder.md](./architecture/builder.md)
- **pool**: [docs/architecture/pool.md](./architecture/pool.md)
- **rpc**: [docs/architecture/rpc.md](./architecture/rpc.md)

### 评估报告
- **综合评估**: [docs/Comprehensive-Review.md](./Comprehensive-Review.md)
- **测试总结**: [docs/Testing-Summary.md](./Testing-Summary.md)
- **变更日志**: [docs/Changes.md](./Changes.md)

---

## 🆘 支持与帮助

### 开发者支持
- **GitHub Issues**: https://github.com/你的用户名/super-relay/issues
- **文档站点**: https://superrelay.gitbook.io/
- **开发者论坛**: https://forum.superrelay.com/

### 运营者支持
- **技术支持**: devops@superrelay.com
- **监控告警**: alerts@superrelay.com
- **紧急联系**: emergency@superrelay.com

### 最终用户支持
- **用户支持**: support@superrelay.com
- **社区 Discord**: https://discord.gg/superrelay
- **教程视频**: https://youtube.com/@superrelay

---

*SuperPaymaster - 让 Web3 交易更简单、更安全、更经济* 🚀 