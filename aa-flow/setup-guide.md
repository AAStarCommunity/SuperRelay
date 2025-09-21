# 🔧 AA Flow 测试环境设置指南

## ❌ 当前问题

测试失败的原因：

1. **RPC 连接问题** - 当前使用的是 demo RPC，连接不稳定
2. **PNT 代币余额为 0** - SimpleAccount 没有足够的 PNT 代币
3. **可能的网络问题** - 合约地址可能不在当前网络上

## ✅ 解决方案

### 1. 配置正确的 RPC 端点

编辑 `.env` 文件：

```bash
# 获取免费的 Alchemy API Key: https://www.alchemy.com/
NODE_HTTP="https://eth-sepolia.g.alchemy.com/v2/YOUR_ACTUAL_API_KEY"

# 或使用其他可靠的 Sepolia RPC
# NODE_HTTP="https://sepolia.infura.io/v3/YOUR_INFURA_KEY"
```

### 2. 使用正确的私钥

确保 `.env` 中的私钥是控制 SimpleAccount 的 EOA：

```bash
PRIVATE_KEY_A="0xYOUR_ACTUAL_PRIVATE_KEY"
```

### 3. 确保账户有足够的资产

#### 检查余额
```bash
# 在区块浏览器中检查
# SimpleAccount: https://sepolia.etherscan.io/address/0x6ff9A269085C79001e647b3D56C9176841A19935
# PNT Token: https://sepolia.etherscan.io/address/0x7D1AfA7B718fb893dB30A3aBc0Cfc608AaCfeBB0
```

#### 如果余额为 0，需要：
1. **获取 Sepolia ETH** (用于 gas 费用)
   - 使用 Sepolia faucet: https://sepoliafaucet.com/
   - 发送到 SimpleAccount 地址

2. **获取 PNT 测试代币**
   - 如果 PNT 是自定义代币，需要 mint 一些到 SimpleAccount
   - 或者从其他账户转移一些 PNT

## 🧪 验证设置

### 运行连接测试
```bash
node checkBalance.js
```

期望看到：
```
✅ 当前区块: 9247xxx
SimpleAccount ETH: 0.1 ETH (至少需要 0.01 ETH 用于 gas)
代币名称: Some Token Name
代币符号: PNT
PNT 余额: 100.0 PNT (需要 > 1 PNT 进行测试)
```

### 运行转账测试
```bash
npm run test:pnt 1
```

## 📋 成功案例参考

根据之前的文档，成功的配置应该有：

- **SimpleAccount**: 180 PNT 余额
- **Contract Account A**: 328 PNT 余额
- **成功转账**: 5 PNT (交易哈希: 0xa601891...)

## 🚨 如果仍然失败

1. **验证网络** - 确保所有地址都在 Sepolia 网络上
2. **检查合约** - 验证 PNT 代币合约是否部署在正确地址
3. **使用不同的 RPC** - 尝试其他 Sepolia RPC 端点
4. **检查私钥权限** - 确保私钥控制的地址是 SimpleAccount 的 owner

## 💡 快速解决方案

如果只是想测试 Bundler 功能而不关心具体代币：

1. 更新测试脚本使用一个**确定存在**的 ERC20 代币
2. 或者创建一个模拟测试，只测试 UserOperation 构建和签名
3. 或者使用一个**有资产的测试账户**

---

**注意**: 当前的地址和私钥可能是示例数据，需要替换为实际的测试环境配置。