// 为 UserOperation 支付费用的实用工具
// 可以通过提高费用或使用 Paymaster 来帮助 UserOperation 被打包

const { ethers } = require("ethers");

// 导入现有的工具函数
const {
    signUserOpForSimpleAccount,
    sendUserOperation,
    waitForUserOpReceipt
} = require('./testTransferWithBundler');

// 配置
const SEPOLIA_RPC = process.env.NODE_HTTP || "https://eth-sepolia.g.alchemy.com/v2/YOUR_API_KEY";
const BUNDLER_URL = process.env.BUNDLER_URL || "https://rundler-superrelay.fly.dev";
const CHAIN_ID = 11155111;
const ENTRYPOINT_ADDRESS = "0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789";

/**
 * 检查 UserOperation 状态
 */
async function checkUserOpStatus(userOpHash) {
    console.log(`🔍 检查 UserOperation: ${userOpHash}`);

    try {
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
            console.log(`   Gas 使用: ${result.result.gasUsed}`);
            return { status: 'completed', receipt: result.result };
        } else if (result.result === null) {
            console.log("⏳ UserOperation 仍在等待打包");
            return { status: 'pending' };
        } else {
            console.log("❓ 状态未知:", result);
            return { status: 'unknown', data: result };
        }
    } catch (error) {
        console.error("❌ 检查状态失败:", error.message);
        return { status: 'error', error: error.message };
    }
}

/**
 * 创建费用提升的转账 UserOperation
 * 这会创建一个新的转账操作，使用更高的 gas 费用
 */
async function createHighFeeTransfer(params) {
    const {
        fromAddress,
        toAddress,
        amount,
        privateKey,
        feeMultiplier = 1.2 // 费用增加 20%
    } = params;

    console.log("💰 创建高费用转账 UserOperation...");
    console.log(`从: ${fromAddress}`);
    console.log(`到: ${toAddress}`);
    console.log(`金额: ${ethers.utils.formatEther(amount)} PNT`);
    console.log(`费用倍数: ${feeMultiplier}x`);

    const provider = new ethers.providers.JsonRpcProvider(SEPOLIA_RPC);

    // PNT 代币合约
    const PNT_TOKEN_ADDRESS = "0x7D1AfA7B718fb893dB30A3aBc0Cfc608AaCfeBB0";
    const ERC20_ABI = [
        "function transfer(address to, uint256 amount) public returns (bool)"
    ];
    const SIMPLE_ACCOUNT_ABI = [
        "function execute(address dest, uint256 value, bytes calldata func) external",
        "function getNonce() public view returns (uint256)"
    ];

    // 编码转账数据
    const pntToken = new ethers.Contract(PNT_TOKEN_ADDRESS, ERC20_ABI, provider);
    const transferData = pntToken.interface.encodeFunctionData("transfer", [toAddress, amount]);

    const simpleAccount = new ethers.Contract(fromAddress, SIMPLE_ACCOUNT_ABI, provider);
    const executeData = simpleAccount.interface.encodeFunctionData("execute", [
        PNT_TOKEN_ADDRESS,
        0,
        transferData
    ]);

    // 获取 nonce 和 gas 价格
    const nonce = await simpleAccount.getNonce();
    const feeData = await provider.getFeeData();

    // 提高费用
    const baseFee = feeData.maxFeePerGas || ethers.utils.parseUnits("100", "gwei");
    const priorityFee = feeData.maxPriorityFeePerGas || ethers.utils.parseUnits("2", "gwei");

    const boostedMaxFee = baseFee.mul(Math.floor(feeMultiplier * 100)).div(100);
    const boostedPriorityFee = priorityFee.mul(Math.floor(feeMultiplier * 100)).div(100);

    console.log(`原 Gas 费用: ${ethers.utils.formatUnits(baseFee, "gwei")} Gwei`);
    console.log(`新 Gas 费用: ${ethers.utils.formatUnits(boostedMaxFee, "gwei")} Gwei`);

    // 构建 UserOperation
    const userOp = {
        sender: fromAddress,
        nonce: ethers.utils.hexlify(nonce),
        initCode: "0x",
        callData: executeData,
        callGasLimit: "0x15F90", // 90000
        verificationGasLimit: "0x15F90", // 90000
        preVerificationGas: "0x5208", // 21000
        maxFeePerGas: ethers.utils.hexlify(boostedMaxFee),
        maxPriorityFeePerGas: ethers.utils.hexlify(boostedPriorityFee),
        paymasterAndData: "0x",
        signature: "0x"
    };

    // 签名
    console.log("🔐 计算签名...");
    const signature = await signUserOpForSimpleAccount(
        userOp,
        privateKey,
        ENTRYPOINT_ADDRESS,
        CHAIN_ID
    );
    userOp.signature = signature;

    return userOp;
}

/**
 * 执行费用赞助转账
 */
async function sponsorTransfer(params) {
    console.log("🎁 执行费用赞助转账");
    console.log("====================");

    try {
        // 1. 创建高费用 UserOperation
        const userOp = await createHighFeeTransfer(params);

        // 2. 发送到 Bundler
        console.log("\n📤 发送到 Bundler...");
        const userOpHash = await sendUserOperation(userOp);
        console.log(`UserOperation Hash: ${userOpHash}`);

        // 3. 等待确认
        console.log("\n⏳ 等待确认...");
        const receipt = await waitForUserOpReceipt(userOpHash);

        console.log("\n✅ 费用赞助转账成功！");
        console.log(`交易哈希: ${receipt.transactionHash}`);
        console.log(`Gas 使用: ${receipt.gasUsed}`);

        return {
            success: true,
            userOpHash,
            transactionHash: receipt.transactionHash,
            gasUsed: receipt.gasUsed
        };

    } catch (error) {
        console.error("❌ 费用赞助失败:", error.message);
        throw error;
    }
}

/**
 * 为某人支付小额转账费用的便捷函数
 */
async function paySmallTransferFee(recipientAddress, amountPNT = "1", sponsorPrivateKey) {
    console.log("💝 支付小额转账费用");
    console.log("===================");
    console.log(`为 ${recipientAddress} 支付 ${amountPNT} PNT 的转账费用`);

    const amount = ethers.utils.parseEther(amountPNT);
    const fromAddress = "0x6ff9A269085C79001e647b3D56C9176841A19935"; // SimpleAccount

    const params = {
        fromAddress,
        toAddress: recipientAddress,
        amount,
        privateKey: sponsorPrivateKey,
        feeMultiplier: 1.5 // 增加 50% 费用确保快速打包
    };

    return await sponsorTransfer(params);
}

/**
 * 显示使用帮助
 */
function showHelp() {
    console.log("UserOperation 费用支付工具");
    console.log("============================");
    console.log("");
    console.log("功能:");
    console.log("1. 检查 UserOperation 状态");
    console.log("2. 创建高费用转账 (确保快速打包)");
    console.log("3. 为他人支付小额转账费用");
    console.log("");
    console.log("用法:");
    console.log("  node payForUserOp.js check <userop_hash>");
    console.log("  node payForUserOp.js pay <recipient_address> <amount_pnt> <private_key>");
    console.log("  node payForUserOp.js sponsor <from> <to> <amount> <private_key> [fee_multiplier]");
    console.log("");
    console.log("示例:");
    console.log("  # 检查 UserOperation 状态");
    console.log("  node payForUserOp.js check 0x9574de239acbaf0f42fe338f71342315dfdd02ecef104add24ae18fa7cc580fd");
    console.log("");
    console.log("  # 为某人支付 2 PNT 转账费用 (高费用确保快速打包)");
    console.log("  node payForUserOp.js pay 0x742d35Cc6634C0532925a3b8D0A40F4b7F3 2 0xYOUR_PRIVATE_KEY");
    console.log("");
    console.log("  # 完全自定义转账");
    console.log("  node payForUserOp.js sponsor 0xFROM 0xTO 5 0xKEY 1.8");
}

// 主函数
async function main() {
    const args = process.argv.slice(2);

    if (args.length === 0) {
        showHelp();
        return;
    }

    const command = args[0];

    try {
        if (command === 'check') {
            if (args.length < 2) {
                console.error("❌ 需要 UserOperation hash");
                return;
            }
            await checkUserOpStatus(args[1]);

        } else if (command === 'pay') {
            if (args.length < 4) {
                console.error("❌ 需要: recipient_address amount_pnt private_key");
                return;
            }
            await paySmallTransferFee(args[1], args[2], args[3]);

        } else if (command === 'sponsor') {
            if (args.length < 5) {
                console.error("❌ 需要: from_address to_address amount private_key [fee_multiplier]");
                return;
            }

            const params = {
                fromAddress: args[1],
                toAddress: args[2],
                amount: ethers.utils.parseEther(args[3]),
                privateKey: args[4],
                feeMultiplier: args[5] ? parseFloat(args[5]) : 1.5
            };
            await sponsorTransfer(params);

        } else {
            console.error("❌ 未知命令:", command);
            showHelp();
        }
    } catch (error) {
        console.error("执行失败:", error.message);
        process.exit(1);
    }
}

// 运行脚本
if (require.main === module) {
    main();
}

module.exports = {
    checkUserOpStatus,
    createHighFeeTransfer,
    sponsorTransfer,
    paySmallTransferFee
};