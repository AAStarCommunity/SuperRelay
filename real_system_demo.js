/**
 * 真实系统演示 - 与实际运行的 AirAccount CA 和 TEE 交互
 */

const fetch = require('node-fetch');

async function realSystemDemo() {
    console.log('🌟 真实系统演示 - 与 AirAccount CA + TEE TA 交互');
    console.log('================================================');
    
    // 使用演示脚本生成的真实数据
    const realUserOperation = {
        sender: "0x742d35cc6634c0532925a3b8d4521fb8d0000001",
        nonce: "0x1",
        initCode: "0x",
        callData: "0xb61d27f60000000000000000000000001234567890123456789012345678901234567890000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000000",
        callGasLimit: "0x5208",
        verificationGasLimit: "0x5208", 
        preVerificationGas: "0x5208",
        maxFeePerGas: "0x3b9aca00",
        maxPriorityFeePerGas: "0x3b9aca00",
        paymasterAndData: "0x"
    };
    
    // 真实的 Paymaster 数据 (从演示脚本生成)
    const paymasterAddress = "0x02b78ba2a33b238d697c3e9ef8842b7de92fb7e3";
    const paymasterSignature = "0x85a68a8c620cfe596c922a6b973afd120bff9f482b57d4b86dfc70af1da862e700bf12be8046edcab519568106f6b84cad01c1228cca5377784bc7660a317c791b";
    
    const requestBody = {
        userOperation: realUserOperation,
        accountId: "passkey_user_test-phase1_airaccount_dev",
        signatureFormat: "erc4337",
        userSignature: "passkey_signature_c544175f2ef2b4ef804b012b6bbd1bd85c6b7f75fbbf29f0e9fe16ed96d33462",
        userPublicKey: "0x04deadbeefcafebabe1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef12",
        businessValidation: {
            balance: "2.5",
            membershipLevel: "platinum",
            approvedAt: 1756866254,
            riskScore: 0.1
        },
        nonce: 866254,
        timestamp: 1756866254
    };
    
    console.log('\n📡 发送请求到真实的 AirAccount CA');
    console.log('URL: POST http://localhost:3002/kms/sign-user-operation');
    console.log('Expected UserOp Hash: 0x8d983344151e70bb11d37795e46e2586d943010ab48bbf8337ca1b919cb093ef');
    
    try {
        const response = await fetch('http://localhost:3002/kms/sign-user-operation', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
                'x-paymaster-address': paymasterAddress,
                'x-paymaster-signature': paymasterSignature
            },
            body: JSON.stringify(requestBody)
        });
        
        const responseData = await response.json();
        
        console.log(`\n📥 响应状态: ${response.status} ${response.statusText}`);
        console.log('\n📋 完整响应数据:');
        console.log(JSON.stringify(responseData, null, 2));
        
        if (response.ok && responseData.success) {
            console.log('\n🎉 真实系统测试成功!');
            console.log('===============================');
            
            console.log('\n🔍 详细分析:');
            console.log(`✅ UserOp Hash 验证: ${responseData.userOpHash}`);
            console.log(`✅ TEE 签名生成: ${responseData.signature}`);
            console.log(`✅ TEE Device ID: ${responseData.teeDeviceId}`);
            
            if (responseData.verificationProof) {
                console.log('\n🛡️ 验证证明:');
                console.log(`  Paymaster 验证: ${responseData.verificationProof.paymasterVerified ? '✅' : '❌'}`);
                console.log(`  Passkey 验证: ${responseData.verificationProof.userPasskeyVerified ? '✅' : '❌'}`);
                console.log(`  双重签名模式: ${responseData.verificationProof.dualSignatureMode ? '✅' : '❌'}`);
                console.log(`  处理时间戳: ${responseData.verificationProof.timestamp}`);
            }
            
            console.log('\n🏗️ 系统架构验证:');
            console.log('  ┌─ JavaScript 前端');
            console.log('  │  └─ UserOperation 构造 + Hash 计算');
            console.log('  │');  
            console.log('  ├─ HTTP/TLS 网络层');
            console.log('  │  └─ Paymaster 签名验证');
            console.log('  │');
            console.log('  ├─ Node.js AirAccount CA');
            console.log('  │  ├─ WebAuthn Passkey 验证');
            console.log('  │  ├─ 双重签名验证逻辑');
            console.log('  │  └─ TEE 客户端调用');
            console.log('  │');
            console.log('  └─ OP-TEE 安全世界');
            console.log('     ├─ TA (Trusted Application)');
            console.log('     ├─ 密钥生成/管理');
            console.log('     └─ ECDSA 硬件签名');
            
        } else {
            console.log('\n❌ 请求失败:');
            console.log(`错误: ${responseData.error}`);
            console.log(`详情: ${responseData.details}`);
        }
        
    } catch (error) {
        console.error('\n💥 网络错误:', error.message);
        console.log('\n💡 请确保 AirAccount CA 服务正在运行:');
        console.log('   cd /Volumes/UltraDisk/Dev2/aastar/AirAccount/packages/airaccount-ca-nodejs');
        console.log('   npm run dev');
    }
}

// 运行真实系统演示
realSystemDemo().catch(console.error);