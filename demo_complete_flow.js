/**
 * Phase 1 完整数据流程演示
 * 从前端 JavaScript 到 TEE TA 的真实数据传输
 */

const { ethers } = require('ethers');

async function demonstrateCompleteFlow() {
    console.log('🚀 Phase 1 完整数据流程演示');
    console.log('================================');
    
    // ========== 步骤 1: 前端 JavaScript 构造 UserOperation ==========
    console.log('\n📱 步骤 1: 前端构造 UserOperation');
    console.log('----------------------------------');
    
    const userOperation = {
        sender: "0x742d35cc6634c0532925a3b8d4521fb8d0000001",
        nonce: "0x1", // 用户账户的 nonce
        initCode: "0x", // 账户已存在，无需初始化代码
        callData: "0xb61d27f60000000000000000000000001234567890123456789012345678901234567890000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000000",
        callGasLimit: "0x5208", // 21000 gas
        verificationGasLimit: "0x5208", // 21000 gas  
        preVerificationGas: "0x5208", // 21000 gas
        maxFeePerGas: "0x3b9aca00", // 1 gwei
        maxPriorityFeePerGas: "0x3b9aca00", // 1 gwei
        paymasterAndData: "0x" // 无 Paymaster
    };
    
    console.log('UserOperation 原始数据:');
    console.log(JSON.stringify(userOperation, null, 2));
    
    // 解析 callData 字段
    console.log('\n📋 CallData 解析:');
    console.log('  方法签名: 0xb61d27f6 (execute)');
    console.log('  目标地址: 0x1234567890123456789012345678901234567890');
    console.log('  转账金额: 0 ETH');
    console.log('  数据长度: 0 bytes');
    
    // ========== 步骤 2: 计算 UserOperation Hash ==========
    console.log('\n🔐 步骤 2: 计算 UserOperation Hash');
    console.log('------------------------------------');
    
    // 模拟 ERC-4337 UserOperation Hash 计算
    const entryPointAddress = "0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789";
    const chainId = 11155111; // Sepolia
    
    console.log('Hash 计算参数:');
    console.log(`  EntryPoint: ${entryPointAddress}`);
    console.log(`  Chain ID: ${chainId}`);
    console.log(`  编码方式: Standard ABI Encoding (非 packed)`);
    
    // 计算各个字段的 hash
    const initCodeHash = ethers.keccak256('0x'); // 空 initCode
    const callDataHash = ethers.keccak256(userOperation.callData);
    const paymasterHash = ethers.keccak256('0x'); // 空 paymaster
    
    console.log('\n字段 Hash 值:');
    console.log(`  initCodeHash: ${initCodeHash}`);
    console.log(`  callDataHash: ${callDataHash}`);
    console.log(`  paymasterHash: ${paymasterHash}`);
    
    // Standard ABI Encoding
    const userOpHashInput = ethers.AbiCoder.defaultAbiCoder().encode(
        ['address', 'uint256', 'bytes32', 'bytes32', 'uint256', 'uint256', 'uint256', 'uint256', 'uint256', 'bytes32'],
        [
            userOperation.sender,
            userOperation.nonce,
            initCodeHash,
            callDataHash,
            userOperation.callGasLimit,
            userOperation.verificationGasLimit,
            userOperation.preVerificationGas,
            userOperation.maxFeePerGas,
            userOperation.maxPriorityFeePerGas,
            paymasterHash
        ]
    );
    
    const userOpHash = ethers.keccak256(userOpHashInput);
    console.log(`\nUserOperation Hash (第一层): ${userOpHash}`);
    
    // 最终 Hash (加入 EntryPoint 和 ChainID)
    const finalHashInput = ethers.AbiCoder.defaultAbiCoder().encode(
        ['bytes32', 'address', 'uint256'],
        [userOpHash, entryPointAddress, chainId]
    );
    
    const finalUserOpHash = ethers.keccak256(finalHashInput);
    console.log(`最终 UserOperation Hash: ${finalUserOpHash}`);
    
    // ========== 步骤 3: 用户 Passkey 认证和签名 ==========
    console.log('\n🔑 步骤 3: 用户 Passkey 认证和签名');
    console.log('------------------------------------');
    
    console.log('WebAuthn 认证流程:');
    console.log('  1. 浏览器调用 navigator.credentials.get()');
    console.log('  2. 用户进行生物识别验证 (指纹/面部识别)');
    console.log('  3. 设备生成 WebAuthn 签名');
    
    const userPasskeyData = {
        credentialId: "test-credential-id-phase1-enhanced",
        challengeMessage: `Sign UserOperation: ${finalUserOpHash.slice(2)}`,
        authenticatorSignature: "passkey_signature_" + ethers.keccak256(finalUserOpHash).slice(2, 66),
        clientDataJSON: {
            type: "webauthn.get",
            challenge: Buffer.from(finalUserOpHash.slice(2), 'hex').toString('base64url'),
            origin: "https://dapp.example.com",
            crossOrigin: false
        }
    };
    
    console.log('\nPasskey 签名数据:');
    console.log(`  Credential ID: ${userPasskeyData.credentialId}`);
    console.log(`  Challenge Message: ${userPasskeyData.challengeMessage.slice(0, 50)}...`);
    console.log(`  Passkey Signature: ${userPasskeyData.authenticatorSignature.slice(0, 50)}...`);
    
    // ========== 步骤 4: Paymaster 业务验证和签名 ==========
    console.log('\n💳 步骤 4: Paymaster 业务验证和签名');
    console.log('--------------------------------------');
    
    const paymasterWallet = ethers.Wallet.createRandom();
    console.log(`Paymaster 地址: ${paymasterWallet.address}`);
    
    // 业务验证数据
    const businessValidation = {
        balance: "2.5", // ETH 余额
        membershipLevel: "platinum", // 会员等级
        approvedAt: Math.floor(Date.now() / 1000),
        riskScore: 0.1 // 风险评分
    };
    
    console.log('\n业务验证数据:');
    console.log(JSON.stringify(businessValidation, null, 2));
    
    // Paymaster 签名计算 (solidityPackedKeccak256)
    const accountId = "passkey_user_test-phase1_airaccount_dev";
    const nonce = Math.floor(Date.now() / 1000) % 1000000;
    const timestamp = Math.floor(Date.now() / 1000);
    const userSigHash = ethers.keccak256(ethers.toUtf8Bytes(userPasskeyData.authenticatorSignature));
    
    console.log('\nPaymaster 签名参数:');
    console.log(`  UserOp Hash: ${finalUserOpHash}`);
    console.log(`  Account ID: ${accountId}`);
    console.log(`  User Sig Hash: ${userSigHash}`);
    console.log(`  Nonce: ${nonce}`);
    console.log(`  Timestamp: ${timestamp}`);
    
    // solidityPackedKeccak256 编码
    const packedMessage = ethers.solidityPackedKeccak256(
        ['bytes32', 'string', 'bytes32', 'uint64', 'uint64'],
        [finalUserOpHash, accountId, userSigHash, nonce, timestamp]
    );
    
    const paymasterSignature = await paymasterWallet.signMessage(ethers.getBytes(packedMessage));
    
    console.log(`\nPaymaster 消息 Hash: ${packedMessage}`);
    console.log(`Paymaster 签名: ${paymasterSignature}`);
    
    // ========== 步骤 5: 构造完整请求发送到 AirAccount CA ==========
    console.log('\n🌐 步骤 5: 发送到 AirAccount CA (Node.js)');
    console.log('----------------------------------------------');
    
    const kmsRequest = {
        userOperation: userOperation,
        accountId: accountId,
        signatureFormat: "erc4337",
        userSignature: userPasskeyData.authenticatorSignature,
        userPublicKey: "0x04deadbeefcafebabe1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef12",
        businessValidation: businessValidation,
        nonce: nonce,
        timestamp: timestamp
    };
    
    const requestHeaders = {
        "Content-Type": "application/json",
        "x-paymaster-address": paymasterWallet.address.toLowerCase(),
        "x-paymaster-signature": paymasterSignature
    };
    
    console.log('HTTP 请求详情:');
    console.log(`  URL: POST http://localhost:3002/kms/sign-user-operation`);
    console.log('  Headers:', JSON.stringify(requestHeaders, null, 2));
    console.log('  Body:', JSON.stringify(kmsRequest, null, 2));
    
    // ========== 步骤 6: AirAccount CA 处理流程 ==========
    console.log('\n🔧 步骤 6: AirAccount CA 内部处理');
    console.log('----------------------------------');
    
    console.log('CA 验证流程:');
    console.log('  1. 验证 Paymaster 签名');
    console.log('     - 重新计算 solidityPackedKeccak256');
    console.log('     - 使用 ethers.verifyMessage 恢复地址');
    console.log('     - 对比 Header 中的 Paymaster 地址');
    
    console.log('\n  2. 验证用户 Passkey 签名');
    console.log('     - 查找账户关联的 Passkey 凭证');
    console.log('     - 验证 WebAuthn 签名格式');
    console.log('     - 确认用户身份真实性');
    
    console.log('\n  3. 双重验证通过后，准备 TEE 签名');
    console.log(`     - UserOp Hash: ${finalUserOpHash}`);
    console.log(`     - Account ID: ${accountId}`);
    console.log('     - 签名类型: ECDSA_SECP256K1');
    
    // ========== 步骤 7: TEE TA 密钥管理和签名 ==========
    console.log('\n🔐 步骤 7: TEE TA 密钥管理和签名');
    console.log('----------------------------------');
    
    console.log('TEE TA 内部流程:');
    console.log('  1. 安全环境初始化');
    console.log('     - OP-TEE OS 启动');
    console.log('     - TA (Trusted Application) 加载');
    console.log('     - 硬件随机数生成器初始化');
    
    console.log('\n  2. 密钥对生成/恢复');
    console.log('     - 查询账户关联的密钥对');
    console.log('     - 如果不存在，生成新的 ECDSA 密钥对');
    console.log('     - 私钥存储在 TEE 安全存储中');
    console.log('     - 公钥导出给 CA');
    
    console.log('\n  3. 消息签名');
    console.log(`     - 接收消息 Hash: ${finalUserOpHash}`);
    console.log('     - 使用私钥进行 ECDSA 签名');
    console.log('     - 签名格式: {r, s, v} (65 bytes)');
    
    // 模拟 TEE 签名结果
    const teeDeviceId = `tee_${Math.floor(Date.now() / 1000)}`;
    const teeSignature = `0x${ethers.randomBytes(65).map(b => b.toString(16).padStart(2, '0')).join('')}`;
    
    console.log('\nTEE 签名结果:');
    console.log(`  TEE Device ID: ${teeDeviceId}`);
    console.log(`  TEE 签名: ${teeSignature.slice(0, 20)}...${teeSignature.slice(-20)}`);
    
    // ========== 步骤 8: 响应数据结构 ==========
    console.log('\n📤 步骤 8: 最终响应数据');
    console.log('------------------------');
    
    const finalResponse = {
        success: true,
        signature: teeSignature,
        userOpHash: finalUserOpHash,
        teeDeviceId: teeDeviceId,
        verificationProof: {
            paymasterVerified: true,
            userPasskeyVerified: true,
            dualSignatureMode: true,
            timestamp: new Date().toISOString(),
            chainId: chainId,
            entryPoint: entryPointAddress
        },
        metadata: {
            accountId: accountId,
            paymasterAddress: paymasterWallet.address.toLowerCase(),
            businessValidation: businessValidation,
            processingTime: "847ms"
        }
    };
    
    console.log('最终响应:');
    console.log(JSON.stringify(finalResponse, null, 2));
    
    // ========== 步骤 9: 系统边界和安全保证 ==========
    console.log('\n🛡️  步骤 9: 系统边界和安全保证');
    console.log('----------------------------------');
    
    console.log('安全边界:');
    console.log('  1. 前端边界:');
    console.log('     - 用户浏览器 ↔ 用户设备 Secure Enclave');
    console.log('     - WebAuthn API 保证生物识别真实性');
    
    console.log('\n  2. 网络边界:');
    console.log('     - HTTPS/TLS 1.3 加密传输');
    console.log('     - Paymaster 签名防篡改');
    
    console.log('\n  3. 服务边界:');
    console.log('     - Node.js CA (用户空间) ↔ TEE TA (安全世界)');
    console.log('     - Linux Kernel ↔ OP-TEE OS');
    console.log('     - ARM Normal World ↔ ARM Secure World');
    
    console.log('\n  4. 硬件边界:');
    console.log('     - CPU ↔ TEE 协处理器');
    console.log('     - 主内存 ↔ TEE 安全内存');
    console.log('     - 硬件随机数生成器');
    
    console.log('\n安全保证:');
    console.log('  ✅ 私钥永不离开 TEE 安全环境');
    console.log('  ✅ 双重签名防止单点故障');
    console.log('  ✅ Passkey 确保用户真实意图');
    console.log('  ✅ Paymaster 确保业务规则合规');
    console.log('  ✅ 硬件级别的防篡改保护');
    
    console.log('\n🎯 完整数据流程演示结束');
    console.log('===========================');
}

// 如果直接运行此文件
if (require.main === module) {
    demonstrateCompleteFlow().catch(console.error);
}

module.exports = { demonstrateCompleteFlow };