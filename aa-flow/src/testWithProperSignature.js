// ERC-4337 签名验证测试脚本
// 专门用于测试不同签名方法的正确性

const { ethers } = require("ethers");

// 配置
const ENTRYPOINT_ADDRESS = "0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789";
const CHAIN_ID = 11155111; // Sepolia
const PRIVATE_KEY = process.env.PRIVATE_KEY_A;

/**
 * 计算 UserOperation Hash
 */
function getUserOpHash(userOp, entryPointAddress, chainId) {
    const packedUserOp = ethers.utils.defaultAbiCoder.encode([
        "address", "uint256", "bytes32", "bytes32",
        "uint256", "uint256", "uint256", "uint256",
        "uint256", "bytes32"
    ], [
        userOp.sender,
        userOp.nonce,
        ethers.utils.keccak256(userOp.initCode),
        ethers.utils.keccak256(userOp.callData),
        userOp.callGasLimit,
        userOp.verificationGasLimit,
        userOp.preVerificationGas,
        userOp.maxFeePerGas,
        userOp.maxPriorityFeePerGas,
        ethers.utils.keccak256(userOp.paymasterAndData)
    ]);

    const encoded = ethers.utils.defaultAbiCoder.encode([
        "bytes32", "address", "uint256"
    ], [
        ethers.utils.keccak256(packedUserOp),
        entryPointAddress,
        chainId
    ]);

    return ethers.utils.keccak256(encoded);
}

/**
 * 方法 1: EIP-712 签名 (在 v0.6 中不工作)
 */
async function signUserOpEIP712(userOp, privateKey, entryPointAddress, chainId) {
    console.log("🔐 尝试 EIP-712 签名...");

    const wallet = new ethers.Wallet(privateKey);

    const domain = {
        name: "SimpleAccount",
        version: "1",
        chainId: chainId,
        verifyingContract: entryPointAddress
    };

    const types = {
        UserOperation: [
            { name: "sender", type: "address" },
            { name: "nonce", type: "uint256" },
            { name: "initCode", type: "bytes" },
            { name: "callData", type: "bytes" },
            { name: "callGasLimit", type: "uint256" },
            { name: "verificationGasLimit", type: "uint256" },
            { name: "preVerificationGas", type: "uint256" },
            { name: "maxFeePerGas", type: "uint256" },
            { name: "maxPriorityFeePerGas", type: "uint256" },
            { name: "paymasterAndData", type: "bytes" }
        ]
    };

    try {
        const signature = await wallet._signTypedData(domain, types, userOp);
        console.log("✅ EIP-712 签名成功");
        return signature;
    } catch (error) {
        console.log("❌ EIP-712 签名失败:", error.message);
        throw error;
    }
}

/**
 * 方法 2: Ethereum Signed Message 签名 (v0.6 正确方法)
 */
async function signUserOpMessage(userOp, privateKey, entryPointAddress, chainId) {
    console.log("🔐 使用 Ethereum Signed Message 签名...");

    const wallet = new ethers.Wallet(privateKey);
    const userOpHash = getUserOpHash(userOp, entryPointAddress, chainId);

    try {
        const signature = await wallet.signMessage(ethers.utils.arrayify(userOpHash));
        console.log("✅ Message 签名成功");
        return signature;
    } catch (error) {
        console.log("❌ Message 签名失败:", error.message);
        throw error;
    }
}

/**
 * 方法 3: 直接签名原始哈希 (低级方法)
 */
async function signUserOpRaw(userOp, privateKey, entryPointAddress, chainId) {
    console.log("🔐 直接签名原始哈希...");

    const wallet = new ethers.Wallet(privateKey);
    const userOpHash = getUserOpHash(userOp, entryPointAddress, chainId);

    try {
        const signature = await wallet.signMessage(userOpHash);
        console.log("✅ 原始哈希签名成功");
        return signature;
    } catch (error) {
        console.log("❌ 原始哈希签名失败:", error.message);
        throw error;
    }
}

/**
 * 验证签名的有效性
 */
function verifySignature(userOp, signature, expectedSigner, entryPointAddress, chainId) {
    console.log("\n🔍 验证签名有效性...");

    try {
        const userOpHash = getUserOpHash(userOp, entryPointAddress, chainId);

        // 模拟 SimpleAccount v0.6 的验证逻辑
        // 使用 toEthSignedMessageHash() 格式
        const messageHash = ethers.utils.hashMessage(ethers.utils.arrayify(userOpHash));
        const recoveredAddress = ethers.utils.recoverAddress(messageHash, signature);

        console.log(`原始 UserOp Hash: ${userOpHash}`);
        console.log(`Message Hash: ${messageHash}`);
        console.log(`期望签名者: ${expectedSigner}`);
        console.log(`恢复的地址: ${recoveredAddress}`);

        const isValid = recoveredAddress.toLowerCase() === expectedSigner.toLowerCase();
        console.log(`签名验证: ${isValid ? '✅ 有效' : '❌ 无效'}`);

        return isValid;
    } catch (error) {
        console.log("❌ 验证签名时出错:", error.message);
        return false;
    }
}

/**
 * 主测试函数
 */
async function testSignatureMethods() {
    console.log("🧪 ERC-4337 签名方法测试");
    console.log("=========================");

    // 模拟 UserOperation
    const sampleUserOp = {
        sender: "0x6ff9A269085C79001e647b3D56C9176841A19935",
        nonce: "0x1",
        initCode: "0x",
        callData: "0xb61d27f60000000000000000000000007d1afa7b718fb893db30a3abc0cfc608aacfebb000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000002444a9059cbb0000000000000000000000006ff9a269085c79001e647b3d56c9176841a199350000000000000000000000000000000000000000000000004563918244f40000",
        callGasLimit: "0x11170",
        verificationGasLimit: "0x11170",
        preVerificationGas: "0x5208",
        maxFeePerGas: "0x174876e800",
        maxPriorityFeePerGas: "0x174876e800",
        paymasterAndData: "0x"
    };

    const wallet = new ethers.Wallet(PRIVATE_KEY);
    const expectedSigner = wallet.address;

    console.log(`\n测试钱包地址: ${expectedSigner}`);

    const results = {};

    // 测试方法 1: EIP-712
    console.log("\n" + "=".repeat(50));
    console.log("测试方法 1: EIP-712 签名");
    console.log("=".repeat(50));
    try {
        const eip712Signature = await signUserOpEIP712(
            sampleUserOp, PRIVATE_KEY, ENTRYPOINT_ADDRESS, CHAIN_ID
        );
        const eip712Valid = verifySignature(
            sampleUserOp, eip712Signature, expectedSigner, ENTRYPOINT_ADDRESS, CHAIN_ID
        );
        results.eip712 = { signature: eip712Signature, valid: eip712Valid };
    } catch (error) {
        console.log("EIP-712 测试失败:", error.message);
        results.eip712 = { error: error.message };
    }

    // 测试方法 2: Ethereum Signed Message
    console.log("\n" + "=".repeat(50));
    console.log("测试方法 2: Ethereum Signed Message");
    console.log("=".repeat(50));
    try {
        const messageSignature = await signUserOpMessage(
            sampleUserOp, PRIVATE_KEY, ENTRYPOINT_ADDRESS, CHAIN_ID
        );
        const messageValid = verifySignature(
            sampleUserOp, messageSignature, expectedSigner, ENTRYPOINT_ADDRESS, CHAIN_ID
        );
        results.message = { signature: messageSignature, valid: messageValid };
    } catch (error) {
        console.log("Message 签名测试失败:", error.message);
        results.message = { error: error.message };
    }

    // 测试方法 3: 原始哈希
    console.log("\n" + "=".repeat(50));
    console.log("测试方法 3: 原始哈希签名");
    console.log("=".repeat(50));
    try {
        const rawSignature = await signUserOpRaw(
            sampleUserOp, PRIVATE_KEY, ENTRYPOINT_ADDRESS, CHAIN_ID
        );
        const rawValid = verifySignature(
            sampleUserOp, rawSignature, expectedSigner, ENTRYPOINT_ADDRESS, CHAIN_ID
        );
        results.raw = { signature: rawSignature, valid: rawValid };
    } catch (error) {
        console.log("原始哈希测试失败:", error.message);
        results.raw = { error: error.message };
    }

    // 总结结果
    console.log("\n" + "=".repeat(50));
    console.log("📊 测试结果总结");
    console.log("=".repeat(50));

    Object.entries(results).forEach(([method, result]) => {
        if (result.error) {
            console.log(`${method}: ❌ 失败 - ${result.error}`);
        } else {
            console.log(`${method}: ${result.valid ? '✅ 有效' : '❌ 无效'}`);
            console.log(`  签名: ${result.signature.slice(0, 20)}...`);
        }
    });

    // 确定推荐方法
    console.log("\n🎯 结论:");
    if (results.message && results.message.valid) {
        console.log("✅ 推荐使用 Ethereum Signed Message 方法 (wallet.signMessage)");
        console.log("   这是 SimpleAccount v0.6 的正确签名格式");
    } else if (results.eip712 && results.eip712.valid) {
        console.log("✅ EIP-712 方法有效");
    } else {
        console.log("❌ 所有方法都失败了，需要进一步调试");
    }

    return results;
}

// 运行测试
if (require.main === module) {
    testSignatureMethods()
        .then(results => {
            console.log("\n🎉 签名测试完成！");
            process.exit(0);
        })
        .catch(error => {
            console.error("测试失败:", error);
            process.exit(1);
        });
}

module.exports = {
    signUserOpEIP712,
    signUserOpMessage,
    signUserOpRaw,
    verifySignature,
    getUserOpHash,
    testSignatureMethods
};