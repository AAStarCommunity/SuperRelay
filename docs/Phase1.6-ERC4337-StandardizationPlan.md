# Phase 1.6: ERC-4337标准化与架构重构改进计划

**版本**: v1.0  
**日期**: 2025-09-03  
**状态**: 待执行

## 🎯 改进目标

基于Phase 1.5的成果，针对用户反馈进行深度改进：
1. **符合ERC-4337最新标准**的UserOperation结构
2. **实现真正的ECDSA签名算法**替换mock实现  
3. **重构双重验证逻辑**以正确处理SBT+余额验证
4. **集成Sepolia链合约**进行真实链上验证
5. **标准化输入输出格式**确保生产环境可用

## 🔍 当前问题分析

### 1. **ERC-4337标准合规性问题**
- **问题**: 使用旧版UserOperation结构，缺少新字段
- **影响**: 与最新bundler不兼容，无法在生产环境使用
- **解决**: 更新为最新ERC-4337 v0.6标准

### 2. **签名算法实现问题**  
- **问题**: Mock签名过短(13字符)，非标准ECDSA
- **影响**: 无法通过链上验证，bundler拒绝处理
- **解决**: 实现标准65字节secp256k1签名

### 3. **双重验证逻辑偏差**
- **问题**: 对`paymasterVerified`字段理解错误
- **影响**: 业务逻辑不符合实际需求
- **解决**: 重构为SBT+PNTs余额验证逻辑

### 4. **缺乏链上集成**
- **问题**: 未连接真实合约，无法验证实际状态
- **影响**: 无法处理真实业务场景
- **解决**: 集成Sepolia链合约调用

## 📊 技术规格清单

### **链上资源 (Sepolia Testnet)**
| **资源** | **地址** | **用途** |
|---------|----------|----------|
| SBT NFT 合约 | `0xBfde68c232F2248114429DDD9a7c3Adbff74bD7f` | 用户资格验证 |
| PNTs ERC20 | `0x3e7B771d4541eC85c8137e950598Ac97553a337a` | Gas费用支付 |
| Paymaster 合约 | `0x3720B69B7f30D92FACed624c39B1fd317408774B` | ERC-4337 v0.6 |
| EntryPoint | `0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789` | 官方入口合约 |

### **定价机制**
- **汇率**: 1 PNTs = 0.001 ETH (可调节)
- **计费**: 根据gas估算检查PNTs余额
- **预存**: Paymaster已充值0.1 ETH

## 🏗️ 详细实施计划

## **阶段1: ERC-4337标准化改造** (预计2-3天)

### 1.1 更新UserOperation结构体
```typescript
// 新版ERC-4337 v0.6标准结构
interface UserOperation {
    sender: string;              // 发送方地址
    nonce: string;              // 防重放随机数
    factory: string;            // 账户工厂地址 (新增)
    factoryData: string;        // 工厂数据 (新增)
    callData: string;           // 调用数据
    callGasLimit: string;       // 调用gas限制
    verificationGasLimit: string; // 验证gas限制
    preVerificationGas: string;  // 预验证gas
    maxFeePerGas: string;       // 最大gas费
    maxPriorityFeePerGas: string; // 最大优先费
    paymaster: string;          // Paymaster地址 (重构)
    paymasterVerificationGasLimit: string; // 新增
    paymasterPostOpGasLimit: string;       // 新增
    paymasterData: string;      // Paymaster数据
    signature: string;          // 最终签名
}
```

**实施任务:**
- [ ] 更新SuperRelay中的UserOperation类型定义
- [ ] 更新AirAccount KMS接口参数验证
- [ ] 更新测试用例中的数据结构
- [ ] 验证与官方bundler的兼容性

### 1.2 实现标准ECDSA签名算法
```rust
// 在AirAccount TA中实现标准secp256k1签名
pub fn generate_ecdsa_signature(
    private_key: &[u8; 32],
    message_hash: &[u8; 32]
) -> Result<[u8; 65], &'static str> {
    // 使用secp256k1库实现标准ECDSA签名
    // 返回格式: [r(32) + s(32) + v(1)]
}
```

**实施任务:**
- [ ] 在dual_signature.rs中集成secp256k1算法
- [ ] 更新mock实现生成65字节标准签名
- [ ] 验证签名格式与以太坊兼容
- [ ] 更新测试验证签名长度和格式

## **阶段2: 双重验证逻辑重构** (预计3-4天)

### 2.1 SBT(NFT)资格验证
```typescript
// Sepolia链SBT合约验证
async function verifySBTOwnership(
    userAddress: string,
    sbtContract: string = "0xBfde68c232F2248114429DDD9a7c3Adbff74bD7f"
): Promise<boolean> {
    // 调用SBT合约查询用户是否持有有效SBT
    const balance = await sbtContract.balanceOf(userAddress);
    return balance > 0;
}
```

### 2.2 PNTs代币余额验证
```typescript
// Gas费用预估和PNTs余额检查
async function verifyPNTsBalance(
    userAddress: string,
    estimatedGas: bigint,
    priceInPnts: number = 1000 // 1 PNTs = 0.001 ETH
): Promise<boolean> {
    const pntsContract = "0x3e7B771d4541eC85c8137e950598Ac97553a337a";
    const requiredPnts = estimatedGas * BigInt(priceInPnts);
    const balance = await pntsContract.balanceOf(userAddress);
    return balance >= requiredPnts;
}
```

### 2.3 重新定义验证响应
```json
{
    "success": true,
    "signature": "0x[65字节ECDSA签名]",
    "userOpHash": "0x[32字节操作哈希]",
    "teeDeviceId": "[TEE设备标识符-待确认含义]",
    "verificationProof": {
        "dualSignatureMode": true,
        "paymasterVerified": true,  // SBT + PNTs余额验证通过
        "userPasskeyVerified": true, // Passkey用户意图确认
        "sbtOwnership": true,        // 新增: SBT持有状态
        "pntsBalance": "1500.0",     // 新增: PNTs余额
        "gasEstimation": "21000",    // 新增: Gas估算
        "requiredPnts": "21.0",      // 新增: 所需PNTs
        "timestamp": "2025-09-03T06:04:10.747Z"
    }
}
```

**实施任务:**
- [ ] 集成ethers.js连接Sepolia链
- [ ] 实现SBT合约余额查询
- [ ] 实现PNTs余额检查逻辑
- [ ] 更新双重验证流程
- [ ] 重构响应数据结构

## **阶段3: Paymaster合约集成** (预计2-3天)

### 3.1 Paymaster合约交互
```typescript
// 与已部署的Paymaster合约交互
const paymasterContract = {
    address: "0x3720B69B7f30D92FACed624c39B1fd317408774B",
    entrypoint: "0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789"
};

async function validateWithPaymaster(
    userOp: UserOperation
): Promise<PaymasterValidationResult> {
    // 调用Paymaster合约验证用户操作
    // 检查gas预付费情况
    // 返回paymasterData字段
}
```

### 3.2 完整的ERC-4337流程
```
用户提交UserOperation
    ↓
1. Passkey签名验证 (用户意图确认)
    ↓
2. SBT资格检查 (NFT持有验证)
    ↓  
3. PNTs余额验证 (Gas费用检查)
    ↓
4. Paymaster合约调用 (生成paymasterData)
    ↓
5. TEE TA最终签名 (硬件级安全)
    ↓
6. 提交给Bundler处理
```

**实施任务:**
- [ ] 集成Paymaster合约ABI和接口
- [ ] 实现gas估算逻辑
- [ ] 更新paymasterData生成
- [ ] 测试与EntryPoint合约兼容性

## **阶段4: 端到端测试验证** (预计1-2天)

### 4.1 创建标准化测试用例
```rust
// 更新测试用例使用真实合约数据
#[tokio::test]
async fn test_erc4337_standard_flow() {
    let user_op = UserOperation {
        sender: "0x[真实账户地址]",
        nonce: "0x1",
        factory: "0x0000000000000000000000000000000000000000",
        factoryData: "0x",
        // ... 其他标准字段
    };
    
    // 测试完整流程
    let response = test_dual_signature_with_chain_verification(user_op).await;
    
    assert!(response.success);
    assert_eq!(response.signature.len(), 132); // "0x" + 65字节 * 2
    assert!(response.verification_proof.sbt_ownership);
    assert!(response.verification_proof.pnts_balance_sufficient);
}
```

### 4.2 链上集成测试
- [ ] 测试SBT合约查询功能
- [ ] 测试PNTs余额检查
- [ ] 验证Paymaster合约交互
- [ ] 测试完整ERC-4337流程
- [ ] 性能基准测试

## **阶段5: 生产环境准备** (预计1天)

### 5.1 配置管理
```toml
# config/sepolia-production.toml
[blockchain]
network = "sepolia"
rpc_url = "https://sepolia.infura.io/v3/[PROJECT_ID]"

[contracts]
sbt_contract = "0xBfde68c232F2248114429DDD9a7c3Adbff74bD7f"
pnts_contract = "0x3e7B771d4541eC85c8137e950598Ac97553a337a"
paymaster_contract = "0x3720B69B7f30D92FACed624c39B1fd317408774B"
entry_point = "0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789"

[pricing]
pnts_to_eth_rate = 0.001
gas_price_buffer = 1.2
```

### 5.2 错误处理和监控
- [ ] 完善错误码定义
- [ ] 添加链上调用监控
- [ ] 实现交易状态跟踪
- [ ] 添加性能指标收集

## 📋 验收标准

### **功能验收**
- [ ] 支持ERC-4337 v0.6完整标准
- [ ] 生成65字节标准ECDSA签名
- [ ] 正确验证SBT持有状态
- [ ] 准确检查PNTs余额充足性
- [ ] 成功集成Paymaster合约
- [ ] 通过官方bundler测试

### **性能验收**
- [ ] 单次验证延迟 < 2秒
- [ ] 链上查询超时处理 < 10秒
- [ ] 支持并发50个请求
- [ ] 错误恢复时间 < 30秒

### **安全验收**
- [ ] TEE内签名生成
- [ ] 防重放攻击机制
- [ ] 输入参数完整验证
- [ ] 敏感数据不泄露日志

## 🚀 执行时间线

| **阶段** | **预计时间** | **关键里程碑** |
|---------|-------------|----------------|
| 阶段1 | 第1-3天 | ERC-4337标准化完成 |
| 阶段2 | 第4-7天 | 双重验证逻辑重构完成 |
| 阶段3 | 第8-10天 | Paymaster集成完成 |
| 阶段4 | 第11-12天 | 测试验证完成 |
| 阶段5 | 第13天 | 生产环境就绪 |

**总预计完成时间**: 13个工作日

## 🤔 待确认问题

1. **teeDeviceId字段含义**: 您希望这个字段表示什么？是TEE硬件唯一标识、设备序列号还是其他业务含义？

2. **SBT验证细节**: 除了简单的`balanceOf > 0`检查，是否需要验证特定的SBT类型或属性？

3. **PNTs定价策略**: 0.001 ETH的汇率是否需要支持动态调整？是否需要预言机获取实时价格？

4. **Gas估算精度**: 是否需要考虑网络拥堵情况下的gas价格波动？

5. **错误处理策略**: 当SBT或PNTs验证失败时，是否需要特殊的用户提示或降级处理？

请您确认这个计划是否符合您的预期，以及待确认问题的答案，然后我们可以开始实施Phase 1.6的改进工作。