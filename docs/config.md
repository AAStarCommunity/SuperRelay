# SuperRelay + AirAccount 项目配置文档

**版本**: v1.0
**更新日期**: 2025-09-06
**状态**: Active Configuration

## 🌐 跨链EntryPoint合约地址

### EntryPoint v0.8 (最新版本)
- **Ethereum Mainnet**: `0x4337084d9e255ff0702461cf8895ce9e3b5ff108`
- **Sepolia Testnet**: `0x4337084d9e255ff0702461cf8895ce9e3b5ff108`
- **OP Mainnet**: `0x4337084d9e255ff0702461cf8895ce9e3b5ff108`
- **OP Sepolia**: `0x4337084d9e255ff0702461cf8895ce9e3b5ff108`

### EntryPoint v0.7 (生产可用)
- **Ethereum Mainnet**: `0x0000000071727De22E5E9d8BAf0edAc6f37da032`
- **Sepolia Testnet**: `0x0000000071727De22E5E9d8BAf0edAc6f37da032`
- **OP Mainnet**: `0x0000000071727De22E5E9d8BAf0edAc6f37da032`
- **OP Sepolia**: `0x0000000071727De22E5E9d8BAf0edAc6f37da032`

### EntryPoint v0.6 (传统版本)
- **Ethereum Mainnet**: `0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789`
- **Sepolia Testnet**: `0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789`
- **OP Mainnet**: `0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789`
- **OP Sepolia**: `0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789`

## 🏦 业务合约地址

### Sepolia Testnet (主要开发环境)
- **SBT NFT 合约**: `0xBfde68c232F2248114429DDD9a7c3Adbff74bD7f`
- **PNTs ERC20 合约**: `0x3e7B771d4541eC85c8137e950598Ac97553a337a`
- **SuperRelay Paymaster**: `0x3720B69B7f30D92FACed624c39B1fd317408774B`

### Ethereum Mainnet (生产环境 - 待部署)
- **SBT NFT 合约**: `TBD`
- **PNTs ERC20 合约**: `TBD`
- **SuperRelay Paymaster**: `TBD`

### OP Mainnet (Layer 2 生产环境 - 待部署)
- **SBT NFT 合约**: `TBD`
- **PNTs ERC20 合约**: `TBD`
- **SuperRelay Paymaster**: `TBD`

### OP Sepolia (Layer 2 测试环境 - 待部署)
- **SBT NFT 合约**: `TBD`
- **PNTs ERC20 合约**: `TBD`
- **SuperRelay Paymaster**: `TBD`

## 🌐 RPC 端点配置

### Ethereum Mainnet
```
Primary RPC: https://eth-mainnet.g.alchemy.com/v2/[API_KEY]
Backup RPC: https://mainnet.infura.io/v3/[PROJECT_ID]
Chain ID: 1
```

### Sepolia Testnet
```
Primary RPC: https://eth-sepolia.g.alchemy.com/v2/[API_KEY]
Backup RPC: https://sepolia.infura.io/v3/[PROJECT_ID]
Chain ID: 11155111
```

### OP Mainnet
```
Primary RPC: https://opt-mainnet.g.alchemy.com/v2/[API_KEY]
Backup RPC: https://mainnet.optimism.io
Chain ID: 10
```

### OP Sepolia
```
Primary RPC: https://opt-sepolia.g.alchemy.com/v2/[API_KEY]
Backup RPC: https://sepolia.optimism.io
Chain ID: 11155420
```

## 💰 定价机制配置

### PNTs 汇率 (可动态调整)
```
1 PNTs = 0.001 ETH (基础汇率)
最小Gas费用 = 21000 * gasPrice
Gas价格缓冲 = 1.2x (20%缓冲)
```

### Paymaster 押金配置
```
Sepolia: 0.1 ETH (测试用)
Mainnet: 1.0 ETH (生产用)
OP Mainnet: 0.5 ETH (Layer 2优化)
OP Sepolia: 0.05 ETH (Layer 2测试)
```

## 🔐 TEE TA 配置参数

### AirAccount TA 设置
```rust
pub struct TAGlobalConfig {
    // 支持的EntryPoint版本
    pub supported_versions: Vec<&'static str> = vec!["0.6", "0.7", "0.8"];

    // 默认使用版本
    pub default_version: &'static str = "0.7";

    // 安全参数
    pub max_nonce_window: u64 = 1000;
    pub signature_timeout: u64 = 300; // 5分钟
    pub max_paymaster_count: usize = 100;
}
```

### 双重签名验证配置
```rust
pub struct DualSignatureConfig {
    pub require_sbt: bool = true;
    pub min_pnts_balance: u64 = 1000;
    pub max_gas_limit: u64 = 10_000_000;
    pub anti_replay_window: u64 = 3600; // 1小时
}
```

## 📊 监控和日志配置

### SuperRelay 监控端点
```
健康检查: http://localhost:3001/health
指标收集: http://localhost:3001/metrics
性能监控: http://localhost:3001/perf
```

### AirAccount KMS 监控端点
```
服务状态: http://localhost:3002/health
TEE状态: http://localhost:3002/kms-ta/status
API文档: http://localhost:3002/api-docs
```

## 🔧 开发环境配置

### 本地开发端口
```
SuperRelay: 3001
AirAccount KMS: 3002
QEMU TEE: 虚拟化端口
```

### 测试私钥 (仅用于开发)
```
Paymaster测试密钥: 0x59c6995e998f97436e73cb5c6d1c2c7e4a65e2d78ab0b8c5b9fb9a5a8b8f8b8d
⚠️ 生产环境必须使用环境变量或硬件钱包
```

## 🚨 安全配置

### 配置验证合约 (计划实现)
```solidity
contract SuperRelayConfigRegistry {
    address public constant ETHEREUM_MAINNET = 0x[TBD];
    address public constant SEPOLIA_TESTNET = 0x[TBD];
    address public constant OP_MAINNET = 0x[TBD];
    address public constant OP_SEPOLIA = 0x[TBD];

    mapping(bytes32 => bool) public validConfigHashes;
}
```

### 配置哈希验证
```
每次TA启动时验证配置完整性
配置变更需要通过链上合约验证
多重签名确认配置更新
```

## 📝 版本兼容性矩阵

| 组件 | v0.6 支持 | v0.7 支持 | v0.8 支持 | 备注 |
|------|----------|----------|----------|------|
| SuperRelay | ✅ | ✅ | 🔄 计划 | |
| AirAccount TA | ✅ | 🔄 开发中 | ⏳ 待定 | |
| KMS API | ✅ | ✅ | ⏳ 待定 | |

## 🔄 配置更新流程

1. **开发环境**: 直接修改此配置文件
2. **测试环境**: 通过配置管理工具部署
3. **生产环境**: 多重签名 + 链上验证后部署

## 📞 紧急联系配置

```
开发团队响应: 15分钟内
测试网络问题: 1小时内
生产网络问题: 5分钟内
安全事件响应: 立即
```

---

**配置维护者**: SuperRelay 开发团队
**最后更新**: 2025-09-06
**下次审查**: 2025-10-06