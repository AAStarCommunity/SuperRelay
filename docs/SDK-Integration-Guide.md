# 🚀 AAstar SuperRelay SDK 集成指南

## 概览

SuperRelay 是 AAstar 生态的核心组件，为 DApp 和钱包提供企业级的 ERC-4337 Account Abstraction 服务。本指南将帮助您在 Node.js 项目中快速集成 SuperRelay。

## 📦 快速开始

### 1. 环境要求

- **Node.js**: 16.0.0+
- **npm/yarn**: 最新版本
- **SuperRelay**: 运行在 `http://localhost:3000`

### 2. 安装依赖

```bash
npm install ethers axios
# 或
yarn add ethers axios
```

### 3. 基础配置

```javascript
const { ethers } = require('ethers');
const axios = require('axios');

const CONFIG = {
    SUPER_RELAY_URL: 'http://localhost:3000',  // SuperRelay RPC 端点
    RPC_URL: 'http://localhost:8545',          // Anvil 测试网络
    ENTRY_POINT_ADDRESS: '0x5FbDB2315678afecb367f032d93F642f64180aa3',
    CHAIN_ID: 31337 // Anvil 默认链ID
};
```

## 🔧 核心功能集成

### 1. UserOperation 赞助 (pm_sponsorUserOperation)

这是 SuperRelay 的核心功能，为用户操作提供 gas 赞助。

```javascript
class SuperRelayClient {
    constructor(config = {}) {
        this.config = { ...CONFIG, ...config };
        this.provider = new ethers.JsonRpcProvider(this.config.RPC_URL);
    }

    /**
     * 为 UserOperation 获取 Paymaster 赞助
     * @param {Object} userOp - 用户操作对象
     * @param {string} entryPoint - EntryPoint 合约地址
     * @returns {Promise<string>} paymasterAndData
     */
    async sponsorUserOperation(userOp, entryPoint) {
        try {
            const response = await axios.post(this.config.SUPER_RELAY_URL, {
                jsonrpc: "2.0",
                id: Date.now(),
                method: "pm_sponsorUserOperation",
                params: [userOp, entryPoint]
            }, {
                headers: { 'Content-Type': 'application/json' }
            });

            if (response.data.error) {
                throw new Error(`SuperRelay Error: ${response.data.error.message}`);
            }

            return response.data.result;
        } catch (error) {
            console.error('赞助用户操作失败:', error);
            throw error;
        }
    }
}
```

### 2. 完整的 UserOperation 构建与提交

```javascript
/**
 * 创建并提交带 Paymaster 赞助的 UserOperation
 */
async function createSponsoredUserOperation(client, senderAddress, callData) {
    // 1. 构建基础 UserOperation
    const userOp = {
        sender: senderAddress,
        nonce: "0x0", // 实际应用中需要查询链上 nonce
        callData: callData || "0x",
        callGasLimit: "0x186A0",           // 100,000 gas
        verificationGasLimit: "0x186A0",   // 100,000 gas
        preVerificationGas: "0x5208",      // 21,000 gas
        maxFeePerGas: "0x3B9ACA00",        // 1 gwei
        maxPriorityFeePerGas: "0x3B9ACA00" // 1 gwei
    };

    try {
        // 2. 获取 Paymaster 赞助
        const paymasterAndData = await client.sponsorUserOperation(
            userOp,
            client.config.ENTRY_POINT_ADDRESS
        );

        console.log('✅ 获得 Paymaster 赞助:', paymasterAndData);

        // 3. 完成 UserOperation
        const sponsoredUserOp = {
            ...userOp,
            paymasterAndData: paymasterAndData,
            signature: "0x" // 需要钱包签名
        };

        return sponsoredUserOp;

    } catch (error) {
        console.error('❌ 创建赞助用户操作失败:', error.message);
        throw error;
    }
}
```

### 3. 标准 ERC-4337 API 调用

SuperRelay 完全兼容 ERC-4337 标准，支持所有标准方法：

```javascript
class SuperRelayERC4337Client extends SuperRelayClient {

    /**
     * 获取支持的 EntryPoint 地址列表
     */
    async getSupportedEntryPoints() {
        return this.callMethod("eth_supportedEntryPoints", []);
    }

    /**
     * 估算 UserOperation 的 gas
     */
    async estimateUserOperationGas(userOp, entryPoint) {
        return this.callMethod("eth_estimateUserOperationGas", [userOp, entryPoint]);
    }

    /**
     * 发送 UserOperation 到内存池
     */
    async sendUserOperation(userOp, entryPoint) {
        return this.callMethod("eth_sendUserOperation", [userOp, entryPoint]);
    }

    /**
     * 获取 UserOperation 收据
     */
    async getUserOperationReceipt(userOpHash) {
        return this.callMethod("eth_getUserOperationReceipt", [userOpHash]);
    }

    /**
     * 通用 RPC 调用方法
     */
    async callMethod(method, params) {
        try {
            const response = await axios.post(this.config.SUPER_RELAY_URL, {
                jsonrpc: "2.0",
                id: Date.now(),
                method: method,
                params: params
            });

            if (response.data.error) {
                throw new Error(`RPC Error: ${response.data.error.message}`);
            }

            return response.data.result;
        } catch (error) {
            console.error(`调用 ${method} 失败:`, error);
            throw error;
        }
    }
}
```

## 🎯 实用示例

### 示例 1: 简单的 gas 赞助

```javascript
async function simpleGasSponsorDemo() {
    const client = new SuperRelayClient();

    // 测试账户地址 (Anvil 默认)
    const testAccount = "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266";

    try {
        // 创建赞助的用户操作
        const sponsoredUserOp = await createSponsoredUserOperation(
            client,
            testAccount,
            "0x" // 空调用数据
        );

        console.log('🎉 成功创建赞助用户操作:', {
            sender: sponsoredUserOp.sender,
            paymasterAndData: sponsoredUserOp.paymasterAndData,
            callGasLimit: sponsoredUserOp.callGasLimit
        });

    } catch (error) {
        console.error('❌ Demo 失败:', error.message);
    }
}

// 运行示例
simpleGasSponsorDemo();
```

### 示例 2: 批量操作赞助

```javascript
async function batchSponsorDemo() {
    const client = new SuperRelayClient();
    const accounts = [
        "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266",
        "0x70997970C51812dc3A010C7d01b50e0d17dc79C8"
    ];

    console.log('🚀 开始批量赞助演示...');

    for (let i = 0; i < accounts.length; i++) {
        try {
            console.log(`\n📋 处理账户 ${i + 1}/${accounts.length}: ${accounts[i]}`);

            const sponsoredUserOp = await createSponsoredUserOperation(
                client,
                accounts[i],
                "0x"
            );

            console.log(`✅ 账户 ${accounts[i]} 赞助成功`);
            console.log(`   📦 PaymasterAndData: ${sponsoredUserOp.paymasterAndData.slice(0, 20)}...`);

        } catch (error) {
            console.error(`❌ 账户 ${accounts[i]} 赞助失败:`, error.message);
        }
    }
}
```

### 示例 3: 健康检查和服务状态

```javascript
async function healthCheckDemo() {
    const client = new SuperRelayClient();

    try {
        // 检查服务健康状态
        const healthResponse = await axios.get(`${client.config.SUPER_RELAY_URL}/health`);
        console.log('🏥 服务健康状态:', healthResponse.data);

        // 检查支持的 EntryPoint
        const entryPoints = await client.getSupportedEntryPoints();
        console.log('🎯 支持的 EntryPoint:', entryPoints);

        // 获取网络信息
        const provider = new ethers.JsonRpcProvider(client.config.RPC_URL);
        const network = await provider.getNetwork();
        console.log('🌐 网络信息:', {
            chainId: network.chainId,
            name: network.name
        });

    } catch (error) {
        console.error('❌ 健康检查失败:', error.message);
    }
}
```

## 🔧 开发工具和脚本

### 快速启动脚本

项目提供了完整的开发环境启动脚本：

```bash
# 1. 启动 Anvil 测试网络
./scripts/start_anvil.sh

# 2. 部署 EntryPoint 合约
./scripts/deploy_entrypoint.sh

# 3. 设置测试账户
./scripts/setup_test_accounts.sh

# 4. 启动 SuperRelay 服务
./scripts/start_superrelay.sh

# 5. 运行集成测试
./scripts/test_integration.sh
```

### 运行 Demo 示例

```bash
# 进入 demo 目录
cd demo

# 安装依赖
npm install

# 运行 SuperPaymaster 演示
npm run demo
# 或
node superPaymasterDemo.js
```

## 📚 重要配置文件

### 环境配置 (.env)

```bash
# SuperRelay 服务配置
SUPER_RELAY_URL=http://localhost:3000
RPC_URL=http://localhost:8545

# 合约地址 (部署后获得)
ENTRY_POINT_ADDRESS=0x5FbDB2315678afecb367f032d93F642f64180aa3

# 测试账户 (Anvil 默认账户)
TEST_PRIVATE_KEY=0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80
PAYMASTER_PRIVATE_KEY=0x59c6995e998f97a5a0044966f0945389dc9e86dae88c6a2440f60b6c4b9f78c2
```

### SuperRelay 配置 (config/config.toml)

关键配置项：

```toml
[paymaster_relay]
enabled = true
chain_id = 31337
entry_point = "0x5FbDB2315678afecb367f032d93F642f64180aa3"

[http_server]
host = "127.0.0.1"
port = 3000
cors_enabled = true

[rate_limiting]
enabled = true
requests_per_minute = 100
```

## 🔍 故障排除

### 常见问题和解决方案

1. **连接被拒绝 (ECONNREFUSED)**
   ```bash
   # 检查 SuperRelay 服务状态
   curl http://localhost:3000/health

   # 重启服务
   ./scripts/start_superrelay.sh
   ```

2. **EntryPoint 地址错误**
   ```bash
   # 重新部署 EntryPoint 合约
   ./scripts/deploy_entrypoint.sh

   # 检查生成的合约地址
   cat .env | grep ENTRY_POINT_ADDRESS
   ```

3. **Gas 估算失败**
   ```javascript
   // 增加 gas limit
   const userOp = {
       // ...其他参数
       callGasLimit: "0x30D40",        // 200,000 gas
       verificationGasLimit: "0x30D40", // 200,000 gas
   };
   ```

## 📖 API 参考文档

### 访问 Swagger UI

SuperRelay 提供完整的 OpenAPI 文档：

```bash
# 启动 API 文档服务器
./scripts/start_api_server.sh

# 访问 Swagger UI
open http://localhost:9000/swagger-ui/
```

### 主要 API 端点

| API 方法 | 描述 | 参数 |
|----------|------|------|
| `pm_sponsorUserOperation` | 获取 Paymaster 赞助 | `userOp`, `entryPoint` |
| `eth_supportedEntryPoints` | 获取支持的 EntryPoint | 无 |
| `eth_estimateUserOperationGas` | 估算 gas | `userOp`, `entryPoint` |
| `eth_sendUserOperation` | 发送用户操作 | `userOp`, `entryPoint` |
| `eth_getUserOperationReceipt` | 获取操作收据 | `userOpHash` |

## 🎯 最佳实践

1. **错误处理**: 始终使用 try-catch 包装 API 调用
2. **Gas 优化**: 根据操作复杂度调整 gas limit
3. **重试机制**: 对网络错误实现指数退避重试
4. **监控告警**: 监控 Paymaster 余额和服务状态
5. **测试环境**: 使用 Anvil 进行本地开发测试

## 🚀 生产环境部署

### 环境配置

```bash
# 生产环境配置
SUPER_RELAY_URL=https://paymaster.yourdomain.com
RPC_URL=https://mainnet.infura.io/v3/YOUR_PROJECT_ID
ENTRY_POINT_ADDRESS=0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789
```

### 启动生产服务

```bash
# 使用生产环境脚本
./scripts/start_production.sh

# 或手动启动
./target/release/super-relay node \
    --paymaster-relay \
    --rpc-url $RPC_URL \
    --entry-points $ENTRY_POINT_ADDRESS \
    --port 3000
```

---

## 🔗 相关资源

- **项目仓库**: [GitHub - SuperRelay](https://github.com/AAStarCommunity/SuperRelay)
- **技术文档**: [docs/](../docs/)
- **API 文档**: [Swagger UI](http://localhost:9000/swagger-ui/)
- **演示代码**: [demo/](../demo/)
- **问题反馈**: [GitHub Issues](https://github.com/AAStarCommunity/SuperRelay/issues)

---

*本指南持续更新中，如有问题或建议，欢迎提交 Issue 或 PR。*