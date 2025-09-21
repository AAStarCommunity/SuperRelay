// PNT 代币转账专用测试脚本
// 专门针对 PNT 代币转账功能的测试，包含余额检查和转账验证

require('dotenv').config();
const { ethers } = require("ethers");

// 导入共享函数
const {
    signUserOpForSimpleAccount,
    getUserOpHash,
    sendUserOperation,
    waitForUserOpReceipt
} = require('./testTransferWithBundler');

// 网络和合约配置
const SEPOLIA_RPC = process.env.NODE_HTTP || "https://eth-sepolia.g.alchemy.com/v2/YOUR_API_KEY";
const BUNDLER_URL = process.env.BUNDLER_URL || "https://rundler-superrelay.fly.dev";
const CHAIN_ID = 11155111;

const ENTRYPOINT_ADDRESS = "0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789";
const PNT_TOKEN_ADDRESS = process.env.PNT_TOKEN_ADDRESS || "0x3e7B771d4541eC85c8137e950598Ac97553a337a";
const SIMPLE_ACCOUNT_ADDRESS = process.env.SIMPLE_ACCOUNT_ADDRESS || "0xC33733449b3f3052E80E8a6ac0145bB3FA87dd6b";

// 账户配置
const SENDER_PRIVATE_KEY = process.env.PRIVATE_KEY_A || process.env.PRIVATE_KEY;

// 检查必需的环境变量
if (!SENDER_PRIVATE_KEY) {
    console.error("❌ 错误: 缺少私钥环境变量");
    console.error("请设置 PRIVATE_KEY_A 或 PRIVATE_KEY 环境变量");
    console.error("或者创建 .env 文件");
    process.exit(1);
}
const RECEIVER_ADDRESS = process.env.EOA_ADDRESS || "0x7fe8e808dbDF6e4F7fBC605E38d2F140D54F208a"; // EOA Address

// ABI
const SIMPLE_ACCOUNT_ABI = [
    "function execute(address dest, uint256 value, bytes calldata func) external",
    "function getNonce() public view returns (uint256)"
];

const ERC20_ABI = [
    "function transfer(address to, uint256 amount) public returns (bool)",
    "function balanceOf(address account) public view returns (uint256)",
    "function decimals() public view returns (uint8)",
    "function name() public view returns (string)",
    "function symbol() public view returns (string)"
];

/**
 * 获取 PNT 代币信息
 */
async function getPNTTokenInfo(provider) {
    const pntToken = new ethers.Contract(PNT_TOKEN_ADDRESS, ERC20_ABI, provider);

    try {
        const [name, symbol, decimals] = await Promise.all([
            pntToken.name(),
            pntToken.symbol(),
            pntToken.decimals()
        ]);

        return { name, symbol, decimals, address: PNT_TOKEN_ADDRESS };
    } catch (error) {
        console.log("获取代币信息失败:", error.message);
        return { name: "Unknown", symbol: "PNT", decimals: 18, address: PNT_TOKEN_ADDRESS };
    }
}

/**
 * 检查账户余额
 */
async function checkBalances(provider, addresses) {
    const pntToken = new ethers.Contract(PNT_TOKEN_ADDRESS, ERC20_ABI, provider);
    const balances = {};

    for (const [name, address] of Object.entries(addresses)) {
        try {
            const balance = await pntToken.balanceOf(address);
            balances[name] = {
                address,
                balance,
                formatted: ethers.utils.formatEther(balance)
            };
        } catch (error) {
            console.log(`获取 ${name} 余额失败:`, error.message);
            balances[name] = {
                address,
                balance: ethers.BigNumber.from(0),
                formatted: "0.0"
            };
        }
    }

    return balances;
}

/**
 * 构建 PNT 转账 UserOperation
 */
async function buildPNTTransferUserOp(provider, fromAddress, toAddress, amount, privateKey) {
    console.log("\n🔧 构建 PNT 转账 UserOperation...");

    // 1. 编码 ERC20 transfer 调用
    const pntToken = new ethers.Contract(PNT_TOKEN_ADDRESS, ERC20_ABI, provider);
    const transferData = pntToken.interface.encodeFunctionData("transfer", [toAddress, amount]);

    // 2. 编码 SimpleAccount execute 调用
    const simpleAccount = new ethers.Contract(fromAddress, SIMPLE_ACCOUNT_ABI, provider);
    const executeData = simpleAccount.interface.encodeFunctionData("execute", [
        PNT_TOKEN_ADDRESS,
        0, // value = 0 ETH
        transferData
    ]);

    // 3. 获取 nonce
    const nonce = await simpleAccount.getNonce();
    console.log(`当前 nonce: ${nonce}`);

    // 4. 获取 Gas 价格
    const feeData = await provider.getFeeData();
    const maxFeePerGas = feeData.maxFeePerGas || ethers.utils.parseUnits("100", "gwei");
    const maxPriorityFeePerGas = feeData.maxPriorityFeePerGas || ethers.utils.parseUnits("2", "gwei");

    console.log(`Max Fee Per Gas: ${ethers.utils.formatUnits(maxFeePerGas, "gwei")} Gwei`);
    console.log(`Max Priority Fee: ${ethers.utils.formatUnits(maxPriorityFeePerGas, "gwei")} Gwei`);

    // 5. 构建 UserOperation
    const userOp = {
        sender: fromAddress,
        nonce: ethers.utils.hexlify(nonce),
        initCode: "0x",
        callData: executeData,
        callGasLimit: "0x15F90", // 90000 gas
        verificationGasLimit: "0x15F90", // 90000 gas
        preVerificationGas: "0xAF2C", // 44844 gas (minimum required)
        maxFeePerGas: ethers.utils.hexlify(maxFeePerGas),
        maxPriorityFeePerGas: ethers.utils.hexlify(maxPriorityFeePerGas),
        paymasterAndData: "0x",
        signature: "0x"
    };

    // 6. 计算签名
    console.log("🔐 计算签名...");
    const signature = await signUserOpForSimpleAccount(
        userOp,
        privateKey,
        ENTRYPOINT_ADDRESS,
        CHAIN_ID
    );
    userOp.signature = signature;

    console.log(`✅ UserOperation 构建完成`);
    console.log(`  Sender: ${userOp.sender}`);
    console.log(`  Nonce: ${userOp.nonce}`);
    console.log(`  Call Gas: ${parseInt(userOp.callGasLimit)}`);
    console.log(`  Verification Gas: ${parseInt(userOp.verificationGasLimit)}`);

    return userOp;
}

/**
 * 执行 PNT 转账测试
 */
async function executePNTTransfer(transferAmount, customReceiver = null) {
    console.log("🚀 开始 PNT 代币转账测试");
    console.log("============================");

    const provider = new ethers.providers.JsonRpcProvider(SEPOLIA_RPC);
    const receiverAddress = customReceiver || RECEIVER_ADDRESS;

    try {
        // 1. 获取代币信息
        console.log("\n📋 PNT 代币信息:");
        const tokenInfo = await getPNTTokenInfo(provider);
        console.log(`名称: ${tokenInfo.name}`);
        console.log(`符号: ${tokenInfo.symbol}`);
        console.log(`精度: ${tokenInfo.decimals}`);
        console.log(`地址: ${tokenInfo.address}`);

        // 2. 检查初始余额
        console.log("\n💰 检查初始余额:");
        const addresses = {
            "SimpleAccount": SIMPLE_ACCOUNT_ADDRESS,
            "接收者": receiverAddress
        };

        const initialBalances = await checkBalances(provider, addresses);
        Object.entries(initialBalances).forEach(([name, info]) => {
            console.log(`${name}: ${info.formatted} PNT (${info.address})`);
        });

        // 3. 验证转账金额
        const amount = ethers.utils.parseEther(transferAmount.toString());
        const senderBalance = initialBalances["SimpleAccount"].balance;

        if (senderBalance.lt(amount)) {
            throw new Error(`余额不足: 需要 ${ethers.utils.formatEther(amount)} PNT, 但只有 ${ethers.utils.formatEther(senderBalance)} PNT`);
        }

        console.log(`\n💸 转账金额: ${ethers.utils.formatEther(amount)} PNT`);

        // 4. 构建并发送 UserOperation
        const userOp = await buildPNTTransferUserOp(
            provider,
            SIMPLE_ACCOUNT_ADDRESS,
            receiverAddress,
            amount,
            SENDER_PRIVATE_KEY
        );

        console.log("\n📤 发送 UserOperation 到 Bundler...");
        const userOpHash = await sendUserOperation(userOp);
        console.log(`UserOperation Hash: ${userOpHash}`);

        // 5. 等待确认
        console.log("\n⏳ 等待交易确认...");
        const receipt = await waitForUserOpReceipt(userOpHash);

        console.log("✅ 转账成功！");
        console.log(`交易哈希: ${receipt.transactionHash}`);
        console.log(`区块号: ${receipt.blockNumber}`);
        console.log(`Gas 使用: ${receipt.gasUsed}`);

        // 6. 检查最终余额
        console.log("\n💰 检查最终余额:");
        const finalBalances = await checkBalances(provider, addresses);
        Object.entries(finalBalances).forEach(([name, info]) => {
            console.log(`${name}: ${info.formatted} PNT`);
        });

        // 7. 计算余额变化
        console.log("\n📊 余额变化:");
        const senderDiff = initialBalances["SimpleAccount"].balance.sub(finalBalances["SimpleAccount"].balance);
        const receiverDiff = finalBalances["接收者"].balance.sub(initialBalances["接收者"].balance);

        console.log(`发送者减少: ${ethers.utils.formatEther(senderDiff)} PNT`);
        console.log(`接收者增加: ${ethers.utils.formatEther(receiverDiff)} PNT`);

        // 8. 验证转账
        const transferSuccess = senderDiff.eq(amount) && receiverDiff.eq(amount);
        console.log(`\n🎯 转账验证: ${transferSuccess ? '✅ 成功' : '❌ 失败'}`);

        if (!transferSuccess) {
            console.log(`期望转账: ${ethers.utils.formatEther(amount)} PNT`);
            console.log(`实际发送者减少: ${ethers.utils.formatEther(senderDiff)} PNT`);
            console.log(`实际接收者增加: ${ethers.utils.formatEther(receiverDiff)} PNT`);
        }

        return {
            success: transferSuccess,
            userOpHash,
            transactionHash: receipt.transactionHash,
            blockNumber: receipt.blockNumber,
            gasUsed: receipt.gasUsed,
            transferAmount: ethers.utils.formatEther(amount),
            balanceChanges: {
                sender: {
                    before: ethers.utils.formatEther(initialBalances["SimpleAccount"].balance),
                    after: ethers.utils.formatEther(finalBalances["SimpleAccount"].balance),
                    change: ethers.utils.formatEther(senderDiff)
                },
                receiver: {
                    before: ethers.utils.formatEther(initialBalances["接收者"].balance),
                    after: ethers.utils.formatEther(finalBalances["接收者"].balance),
                    change: ethers.utils.formatEther(receiverDiff)
                }
            }
        };

    } catch (error) {
        console.error("❌ PNT 转账失败:", error.message);
        throw error;
    }
}

/**
 * 批量转账测试
 */
async function batchPNTTransferTest() {
    console.log("🔄 批量 PNT 转账测试");
    console.log("=====================");

    const transfers = [
        { amount: 1, receiver: "0x6ff9A269085C79001e647b3D56C9176841A19935" },
        { amount: 2.5, receiver: "0x6ff9A269085C79001e647b3D56C9176841A19935" },
        { amount: 5, receiver: "0x6ff9A269085C79001e647b3D56C9176841A19935" }
    ];

    const results = [];

    for (let i = 0; i < transfers.length; i++) {
        const transfer = transfers[i];
        console.log(`\n--- 转账 ${i + 1}/${transfers.length} ---`);

        try {
            const result = await executePNTTransfer(transfer.amount, transfer.receiver);
            results.push({ ...transfer, ...result });
            console.log(`✅ 转账 ${i + 1} 完成`);

            // 等待一段时间再进行下一个转账
            if (i < transfers.length - 1) {
                console.log("⏸️  等待 10 秒后进行下一个转账...");
                await new Promise(resolve => setTimeout(resolve, 10000));
            }
        } catch (error) {
            console.log(`❌ 转账 ${i + 1} 失败:`, error.message);
            results.push({ ...transfer, error: error.message });
        }
    }

    console.log("\n📊 批量转账结果总结:");
    results.forEach((result, index) => {
        if (result.error) {
            console.log(`转账 ${index + 1}: ❌ 失败 - ${result.error}`);
        } else {
            console.log(`转账 ${index + 1}: ✅ 成功 - ${result.transferAmount} PNT (${result.transactionHash})`);
        }
    });

    return results;
}

// 主程序
if (require.main === module) {
    const args = process.argv.slice(2);

    if (args.length === 0) {
        console.log("用法:");
        console.log("  node testPNTTransferFixed.js <amount> [receiver]");
        console.log("  node testPNTTransferFixed.js batch");
        console.log("");
        console.log("示例:");
        console.log("  node testPNTTransferFixed.js 5");
        console.log("  node testPNTTransferFixed.js 2.5 0x742d35Cc6634C0532925a3b8D0A40F4b7F3");
        console.log("  node testPNTTransferFixed.js batch");
        process.exit(0);
    }

    if (args[0] === 'batch') {
        batchPNTTransferTest()
            .then(results => {
                console.log("\n🎉 批量测试完成！");
                process.exit(0);
            })
            .catch(error => {
                console.error("批量测试失败:", error);
                process.exit(1);
            });
    } else {
        const amount = parseFloat(args[0]);
        const receiver = args[1];

        if (isNaN(amount) || amount <= 0) {
            console.error("❌ 无效的转账金额");
            process.exit(1);
        }

        executePNTTransfer(amount, receiver)
            .then(result => {
                console.log("\n🎉 转账测试完成！");
                console.log(JSON.stringify(result, null, 2));
                process.exit(0);
            })
            .catch(error => {
                console.error("转账测试失败:", error);
                process.exit(1);
            });
    }
}

module.exports = {
    executePNTTransfer,
    batchPNTTransferTest,
    buildPNTTransferUserOp,
    checkBalances,
    getPNTTokenInfo
};