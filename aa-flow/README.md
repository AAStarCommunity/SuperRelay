# ERC-4337 Account Abstraction Flow Tests

本目录包含完整的 ERC-4337 Account Abstraction 测试套件，专门用于验证 SuperRelay 项目的 Bundler 功能。

## 📁 文件结构

```
aa-flow/
├── README.md                       # 本文档
├── package.json                    # Node.js 项目配置
├── .env.example                    # 环境变量模板
├── .gitignore                      # Git 忽略规则
├── ERC4337-AB-Test-Guide.md        # 详细测试指南
└── src/
    ├── testTransferWithBundler.js  # 主要测试脚本 - PNT 转账
    ├── testWithProperSignature.js  # 签名方法测试
    └── testPNTTransferFixed.js     # PNT 转账专用脚本
```

## 🚀 快速开始

### 1. 环境准备

```bash
cd aa-flow
npm install ethers@5.7.2
```

### 2. 配置环境变量

复制 `.env.example` 为 `.env` 并填入实际值：

```bash
cp .env.example .env
```

编辑 `.env` 文件：
```env
NODE_HTTP=https://eth-sepolia.g.alchemy.com/v2/YOUR_API_KEY
BUNDLER_URL=https://rundler-superrelay.fly.dev
PRIVATE_KEY_A=0xYOUR_PRIVATE_KEY
```

### 3. 运行测试

```bash
# 主要测试 - PNT 代币转账
npm run test

# 签名方法验证测试
npm run test:signature

# PNT 转账专用测试 (5 PNT)
npm run test:pnt 5

# 批量转账测试
npm run test:pnt-batch
```

## 📋 测试脚本说明

### 1. `testTransferWithBundler.js`
**主要测试脚本** - 完整的 ERC-4337 PNT 代币转账流程

**功能:**
- ✅ 检查账户余额
- ✅ 构建 UserOperation
- ✅ 计算正确的签名 (Ethereum Signed Message 格式)
- ✅ 发送到 Bundler
- ✅ 等待交易确认
- ✅ 验证转账结果

**运行:**
```bash
node src/testTransferWithBundler.js
# 或
npm run test
```

### 2. `testWithProperSignature.js`
**签名验证测试** - 验证不同签名方法的有效性

**功能:**
- 🔐 测试 EIP-712 签名方法 (v0.6 中无效)
- 🔐 测试 Ethereum Signed Message 签名 (v0.6 正确方法)
- 🔐 测试原始哈希签名
- ✅ 验证签名恢复

**运行:**
```bash
node src/testWithProperSignature.js
# 或
npm run test:signature
```

### 3. `testPNTTransferFixed.js`
**PNT 转账专用** - 灵活的 PNT 代币转账测试

**功能:**
- 💰 代币信息查询
- 💸 自定义转账金额
- 👥 自定义接收地址
- 🔄 批量转账测试
- 📊 详细余额分析

**运行:**
```bash
# 转账 5 PNT 到默认地址
node src/testPNTTransferFixed.js 5

# 转账 2.5 PNT 到指定地址
node src/testPNTTransferFixed.js 2.5 0x742d35Cc6634C0532925a3b8D0A40F4b7F3

# 批量转账测试
node src/testPNTTransferFixed.js batch
# 或
npm run test:pnt-batch
```

## 🏗️ 关键技术发现

### 1. **签名格式** (重要!)
SimpleAccount v0.6 使用 **Ethereum Signed Message** 格式，而非 EIP-712：

```javascript
// ✅ 正确方法
const signature = await wallet.signMessage(ethers.utils.arrayify(userOpHash));

// ❌ 错误方法 (会导致 AA23 错误)
const signature = await wallet._signTypedData(domain, types, userOp);
```

### 2. **Gas 估算**
必须使用真实签名进行 gas 估算，不能使用 dummy 签名：

```javascript
// ✅ 使用真实签名
userOp.signature = await signUserOpForSimpleAccount(...);

// ❌ 使用 dummy 签名会导致估算错误
userOp.signature = "0xff".repeat(65);
```

### 3. **CallData 结构**
PNT 转账的 CallData 结构：
```
execute(address dest, uint256 value, bytes calldata func)
├── dest: PNT_TOKEN_ADDRESS
├── value: 0 (不转 ETH)
└── func: transfer(address to, uint256 amount)
    ├── to: 接收地址
    └── amount: 转账金额 (wei)
```

## 📊 成功案例数据

### 实际转账记录
- **交易哈希**: `0xa601891378597635bba88ac797d63294fa7a60e6d37654c8c232d4291b7c7e01`
- **转账金额**: 5 PNT
- **发送方**: SimpleAccount (0x6ff9A269085C79001e647b3D56C9176841A19935)
- **接收方**: Contract Account A (0x6ff9A269085C79001e647b3D56C9176841A19935)

### 余额变化
```
发送方: 180 PNT → 175 PNT (-5 PNT)
接收方: 328 PNT → 333 PNT (+5 PNT)
```

### Gas 使用
```
callGasLimit: 70,000 gas
verificationGasLimit: 70,000 gas
preVerificationGas: 21,000 gas
总计: ~161,000 gas
```

## 🔧 故障排除

### 常见错误

#### 1. **AA23 - 签名验证失败**
```
原因: 使用了错误的签名格式
解决: 使用 wallet.signMessage() 而非 EIP-712
```

#### 2. **Gas 费用不足**
```
原因: 网络费用波动或估算不准确
解决: 增加 gas limit 或使用动态费用
```

#### 3. **余额不足**
```
原因: SimpleAccount 没有足够的 PNT 代币
解决: 先向 SimpleAccount 转入 PNT 代币
```

#### 4. **Bundler 连接失败**
```
原因: Bundler URL 错误或服务未启动
解决: 检查 BUNDLER_URL 环境变量
```

## 🌐 网络配置

### Sepolia 测试网
- **Chain ID**: 11155111
- **RPC URL**: `https://eth-sepolia.g.alchemy.com/v2/YOUR_API_KEY`
- **区块浏览器**: https://sepolia.etherscan.io

### 关键合约地址
```javascript
const contracts = {
    entryPoint: "0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789",    // ERC-4337 EntryPoint v0.6
    factory: "0x9406Cc6185a346906296840746125a0E44976454",       // SimpleAccountFactory
    pntToken: "0x7D1AfA7B718fb893dB30A3aBc0Cfc608AaCfeBB0"       // PNT Test Token
};
```

## 📖 参考文档

- [ERC4337-AB-Test-Guide.md](./ERC4337-AB-Test-Guide.md) - 详细测试指南
- [ERC-4337 规范](https://eips.ethereum.org/EIPS/eip-4337)
- [SimpleAccount 实现](../account-abstraction/contracts/accounts/SimpleAccount.sol)

## 🤝 贡献

本测试套件是 SuperRelay 项目的一部分。如有问题或建议，请提交 Issue 或 PR。

---

**注意**: 本测试套件基于 SimpleAccount v0.6 和 EntryPoint v0.6 实现。不同版本的合约可能需要调整签名方法。