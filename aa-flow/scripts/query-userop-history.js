// 查询 UserOperation 历史记录的工具
require('dotenv').config();
const { ethers } = require("ethers");

const BUNDLER_URL = process.env.BUNDLER_URL || "https://rundler-superrelay.fly.dev";

/**
 * 查询 UserOperation 收据
 */
async function getUserOpReceipt(userOpHash) {
    try {
        const response = await fetch(BUNDLER_URL, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify({
                jsonrpc: '2.0',
                method: 'eth_getUserOperationReceipt',
                params: [userOpHash],
                id: 1,
            }),
        });

        const result = await response.json();
        if (result.error) {
            throw new Error(`Bundler error: ${result.error.message}`);
        }
        return result.result;
    } catch (error) {
        console.error("查询失败:", error.message);
        return null;
    }
}

/**
 * 解析 UserOperation 收据
 */
function parseUserOpReceipt(receipt) {
    if (!receipt) {
        console.log("❌ 未找到收据");
        return;
    }

    console.log("📊 UserOperation 详细信息");
    console.log("========================");

    // 基本信息
    console.log(`UserOp Hash: ${receipt.userOpHash}`);
    console.log(`Sender: ${receipt.sender}`);
    console.log(`Nonce: ${receipt.nonce}`);
    console.log(`Success: ${receipt.success ? '✅' : '❌'}`);

    // Gas 信息
    const actualGasCostWei = BigInt(receipt.actualGasCost);
    const actualGasCostEth = ethers.utils.formatEther(actualGasCostWei.toString());
    console.log(`Actual Gas Cost: ${actualGasCostEth} ETH`);
    console.log(`Actual Gas Used: ${parseInt(receipt.actualGasUsed)} gas`);

    // 交易信息
    if (receipt.receipt) {
        console.log(`\n🔗 区块链信息:`);
        console.log(`Transaction Hash: ${receipt.receipt.transactionHash}`);
        console.log(`Block Number: ${parseInt(receipt.receipt.blockNumber)}`);
        console.log(`Gas Used: ${parseInt(receipt.receipt.gasUsed)} gas`);
        console.log(`Effective Gas Price: ${parseInt(receipt.receipt.effectiveGasPrice) / 1e9} Gwei`);
    }

    // 解析事件日志
    console.log(`\n📋 事件日志:`);
    receipt.logs.forEach((log, index) => {
        console.log(`\n事件 ${index + 1}:`);
        console.log(`  合约: ${log.address}`);

        // 解析 ERC20 Transfer 事件
        if (log.topics[0] === '0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef') {
            const from = '0x' + log.topics[1].slice(26);
            const to = '0x' + log.topics[2].slice(26);
            const amount = ethers.utils.formatEther(log.data);
            console.log(`  类型: ERC20 Transfer`);
            console.log(`  从: ${from}`);
            console.log(`  到: ${to}`);
            console.log(`  金额: ${amount} 代币`);
        }
        // 解析 UserOperationEvent
        else if (log.topics[0] === '0x49628fd1471006c1482da88028e9ce4dbb080b815c9b0344d39e5a8e6ec1419f') {
            console.log(`  类型: UserOperation Event`);
            console.log(`  UserOp Hash: ${log.topics[1]}`);
            console.log(`  Sender: 0x${log.topics[2].slice(26)}`);
        }
        else {
            console.log(`  类型: 其他事件`);
            console.log(`  Topic[0]: ${log.topics[0]}`);
        }
    });

    // 时间戳
    if (receipt.logs.length > 0 && receipt.logs[0].blockTimestamp) {
        const timestamp = parseInt(receipt.logs[0].blockTimestamp);
        const date = new Date(timestamp * 1000);
        console.log(`\n⏰ 执行时间: ${date.toLocaleString()}`);
    }
}

/**
 * 查询支持的 EntryPoint
 */
async function getSupportedEntryPoints() {
    try {
        const response = await fetch(BUNDLER_URL, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify({
                jsonrpc: '2.0',
                method: 'eth_supportedEntryPoints',
                params: [],
                id: 1,
            }),
        });

        const result = await response.json();
        return result.result;
    } catch (error) {
        console.error("查询 EntryPoints 失败:", error.message);
        return [];
    }
}

// 主程序
async function main() {
    const args = process.argv.slice(2);

    if (args.length === 0) {
        console.log("用法:");
        console.log("  node query-userop-history.js <userOpHash>");
        console.log("  node query-userop-history.js entrypoints");
        console.log("");
        console.log("示例:");
        console.log("  node query-userop-history.js 0xf245b530a22fc074c18c3617d0baea09a0676c333b87d3aee4127cb9de6cc4c8");
        console.log("  node query-userop-history.js entrypoints");
        return;
    }

    if (args[0] === 'entrypoints') {
        console.log("🔍 查询支持的 EntryPoints...");
        const entryPoints = await getSupportedEntryPoints();
        console.log("支持的 EntryPoints:", entryPoints);
        return;
    }

    const userOpHash = args[0];
    console.log(`🔍 查询 UserOperation: ${userOpHash}`);
    console.log(`Bundler: ${BUNDLER_URL}`);
    console.log("");

    const receipt = await getUserOpReceipt(userOpHash);
    parseUserOpReceipt(receipt);
}

if (require.main === module) {
    main().catch(console.error);
}

module.exports = {
    getUserOpReceipt,
    parseUserOpReceipt,
    getSupportedEntryPoints
};