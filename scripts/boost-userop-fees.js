#!/usr/bin/env node

// UserOperation 费用提升脚本
// 为滞留在 mempool 中的 UserOperation 提高费用

const { ethers } = require("ethers");

// 配置
const SEPOLIA_RPC = process.env.NODE_HTTP || "https://eth-sepolia.g.alchemy.com/v2/YOUR_API_KEY";
const BUNDLER_URL = process.env.BUNDLER_URL || "https://rundler-superrelay.fly.dev";
const CHAIN_ID = 11155111;
const ENTRYPOINT_ADDRESS = "0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789";

// 需要提升费用的 UserOperation 信息
const STUCK_USEROP = {
    hash: "0x9574de239acbaf0f42fe338f71342315dfdd02ecef104add24ae18fa7cc580fd",
    sender: "0x6ff9A269085C79001e647b3D56C9176841A19935", // 从日志推断
    // 需要完整的 UserOp 数据来重新提交
};

// 当前费用 (从日志中获取)
const CURRENT_FEES = {
    maxFeePerGas: "100005075",        // 当前费用
    maxPriorityFeePerGas: "100000000" // 当前优先费用
};

// 建议的新费用 (增加缓冲)
const SUGGESTED_FEES = {
    maxFeePerGas: "100020000",        // 增加约 15,000 wei (0.015 Gwei)
    maxPriorityFeePerGas: "100000000" // 保持不变
};

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
 * 签名 UserOperation
 */
async function signUserOpForSimpleAccount(userOp, privateKey, entryPointAddress, chainId) {
    const wallet = new ethers.Wallet(privateKey);
    const userOpHash = getUserOpHash(userOp, entryPointAddress, chainId);
    const signature = await wallet.signMessage(ethers.utils.arrayify(userOpHash));
    return signature;
}

/**
 * 发送 UserOperation 到 Bundler
 */
async function sendUserOperation(userOp) {
    const response = await fetch(BUNDLER_URL, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
            jsonrpc: '2.0',
            method: 'eth_sendUserOperation',
            params: [userOp, ENTRYPOINT_ADDRESS],
            id: 1,
        }),
    });

    const result = await response.json();
    if (result.error) {
        throw new Error(`Bundler error: ${result.error.message}`);
    }
    return result.result;
}

/**
 * 检查当前 UserOperation 状态
 */
async function checkUserOpStatus(userOpHash) {
    console.log(`🔍 检查 UserOperation 状态: ${userOpHash}`);

    const response = await fetch(BUNDLER_URL, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
            jsonrpc: '2.0',
            method: 'eth_getUserOperationReceipt',
            params: [userOpHash],
            id: 1,
        }),
    });

    const result = await response.json();

    if (result.result) {
        console.log("✅ UserOperation 已完成！");
        console.log(`   交易哈希: ${result.result.transactionHash}`);
        console.log(`   区块号: ${result.result.blockNumber}`);
        return { status: 'completed', receipt: result.result };
    } else if (result.result === null) {
        console.log("⏳ UserOperation 仍在 mempool 中等待打包");
        return { status: 'pending' };
    } else {
        console.log("❓ 未知状态:", result);
        return { status: 'unknown' };
    }
}

/**
 * 创建费用提升的 UserOperation
 */
function createBoostedUserOp(originalUserOp, newFees) {
    const boostedUserOp = {
        ...originalUserOp,
        maxFeePerGas: ethers.utils.hexlify(newFees.maxFeePerGas),
        maxPriorityFeePerGas: ethers.utils.hexlify(newFees.maxPriorityFeePerGas),
        signature: "0x" // 需要重新计算
    };

    console.log("💰 费用提升对比:");
    console.log(`原费用: ${ethers.utils.formatUnits(originalUserOp.maxFeePerGas, "gwei")} Gwei`);
    console.log(`新费用: ${ethers.utils.formatUnits(newFees.maxFeePerGas, "gwei")} Gwei`);

    const increase = ethers.BigNumber.from(newFees.maxFeePerGas).sub(originalUserOp.maxFeePerGas);
    console.log(`增加: ${ethers.utils.formatUnits(increase, "gwei")} Gwei`);

    return boostedUserOp;
}

/**
 * 显示使用说明
 */
function showUsage() {
    console.log("UserOperation 费用提升工具");
    console.log("");
    console.log("用法:");
    console.log("  node boost-userop-fees.js check");
    console.log("  node boost-userop-fees.js boost <private_key>");
    console.log("");
    console.log("示例:");
    console.log("  node boost-userop-fees.js check");
    console.log("  node boost-userop-fees.js boost 0xYOUR_PRIVATE_KEY");
    console.log("");
    console.log("注意:");
    console.log("  - 需要原 UserOperation 的完整数据才能重新提交");
    console.log("  - 私钥必须是 UserOperation sender 的 owner");
    console.log("  - 只有相同 nonce 的操作会替换旧操作");
}

/**
 * 主函数
 */
async function main() {
    const args = process.argv.slice(2);

    if (args.length === 0) {
        showUsage();
        return;
    }

    const command = args[0];

    console.log("⛽ UserOperation 费用提升工具");
    console.log("==============================");
    console.log("");

    if (command === 'check') {
        // 检查状态
        await checkUserOpStatus(STUCK_USEROP.hash);

    } else if (command === 'boost') {
        if (args.length < 2) {
            console.error("❌ 缺少私钥参数");
            showUsage();
            return;
        }

        const privateKey = args[1];

        try {
            // 1. 检查当前状态
            const status = await checkUserOpStatus(STUCK_USEROP.hash);

            if (status.status === 'completed') {
                console.log("✅ UserOperation 已完成，无需提升费用");
                return;
            }

            // 2. 这里需要构造完整的 UserOperation
            console.log("❌ 需要完整的 UserOperation 数据才能重新提交");
            console.log("");
            console.log("要获取完整数据，需要:");
            console.log("1. 从原始发送者获取 UserOperation 的完整参数");
            console.log("2. 或者创建一个新的转账 UserOperation");
            console.log("");
            console.log("💡 建议: 运行测试脚本创建新的转账操作:");
            console.log("   cd aa-flow && npm run test:pnt 1");

        } catch (error) {
            console.error("❌ 处理失败:", error.message);
        }

    } else {
        console.error("❌ 未知命令:", command);
        showUsage();
    }
}

// 运行脚本
if (require.main === module) {
    main().catch(error => {
        console.error("脚本执行失败:", error);
        process.exit(1);
    });
}

module.exports = {
    checkUserOpStatus,
    createBoostedUserOp,
    signUserOpForSimpleAccount,
    sendUserOperation
};