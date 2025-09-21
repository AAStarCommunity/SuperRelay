// A 到 B 的 PNT 代币转账测试
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

// 账户配置
const PRIVATE_KEY = process.env.PRIVATE_KEY_A || process.env.PRIVATE_KEY;
const SIMPLE_ACCOUNT_A = process.env.SIMPLE_ACCOUNT_A || "0x7D7a0D3239285faE78F9c364D81bb1E3bc555BC6";
const SIMPLE_ACCOUNT_B = process.env.SIMPLE_ACCOUNT_B || "0x27243FAc2c0bEf46F143a705708dC4A7eD476854";

// 检查必需的环境变量
if (!PRIVATE_KEY) {
    console.error("❌ 错误: 缺少私钥环境变量");
    console.error("请设置 PRIVATE_KEY_A 或 PRIVATE_KEY 环境变量");
    process.exit(1);
}

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
 * 检查账户余额
 */
async function checkBalances(provider) {
    const pntToken = new ethers.Contract(PNT_TOKEN_ADDRESS, ERC20_ABI, provider);

    try {
        const [balanceA, balanceB, name, symbol] = await Promise.all([
            pntToken.balanceOf(SIMPLE_ACCOUNT_A),
            pntToken.balanceOf(SIMPLE_ACCOUNT_B),
            pntToken.name(),
            pntToken.symbol()
        ]);

        return {
            tokenInfo: { name, symbol },
            accountA: {
                address: SIMPLE_ACCOUNT_A,
                balance: balanceA,
                formatted: ethers.utils.formatEther(balanceA)
            },
            accountB: {
                address: SIMPLE_ACCOUNT_B,
                balance: balanceB,
                formatted: ethers.utils.formatEther(balanceB)
            }
        };
    } catch (error) {
        console.error("获取余额失败:", error.message);
        throw error;
    }
}

/**
 * 检查账户是否已部署
 */
async function checkAccountDeployment(provider, address) {
    const code = await provider.getCode(address);
    return code !== "0x";
}

/**
 * 执行 A 到 B 的转账测试
 */
async function testABTransfer(transferAmount) {
    console.log("🚀 开始 A → B PNT 代币转账测试");
    console.log("===================================");
    console.log(`发送方 (A): ${SIMPLE_ACCOUNT_A}`);
    console.log(`接收方 (B): ${SIMPLE_ACCOUNT_B}`);
    console.log(`转账金额: ${transferAmount} PNT`);

    const provider = new ethers.providers.JsonRpcProvider(SEPOLIA_RPC);

    try {
        // 1. 检查代币信息和余额
        console.log("\\n📊 检查初始状态...");
        const balances = await checkBalances(provider);

        console.log(`代币: ${balances.tokenInfo.name} (${balances.tokenInfo.symbol})`);
        console.log(`账户 A 余额: ${balances.accountA.formatted} PNT`);
        console.log(`账户 B 余额: ${balances.accountB.formatted} PNT`);

        // 2. 检查账户部署状态
        const [isADeployed, isBDeployed] = await Promise.all([
            checkAccountDeployment(provider, SIMPLE_ACCOUNT_A),
            checkAccountDeployment(provider, SIMPLE_ACCOUNT_B)
        ]);

        console.log(`\\n🏠 账户部署状态:`);
        console.log(`账户 A: ${isADeployed ? '✅ 已部署' : '❌ 未部署'}`);
        console.log(`账户 B: ${isBDeployed ? '✅ 已部署' : '❌ 未部署'}`);

        if (!isADeployed) {
            throw new Error("账户 A 未部署，无法进行转账");
        }

        // 3. 验证余额
        const amount = ethers.utils.parseEther(transferAmount.toString());
        if (balances.accountA.balance.lt(amount)) {
            throw new Error(`账户 A 余额不足: 需要 ${transferAmount} PNT, 但只有 ${balances.accountA.formatted} PNT`);
        }

        // 4. 构建 UserOperation
        console.log("\\n🔧 构建 UserOperation...");

        // 编码 ERC20 transfer 调用
        const pntToken = new ethers.Contract(PNT_TOKEN_ADDRESS, ERC20_ABI, provider);
        const transferData = pntToken.interface.encodeFunctionData("transfer", [SIMPLE_ACCOUNT_B, amount]);

        // 编码 SimpleAccount execute 调用
        const simpleAccount = new ethers.Contract(SIMPLE_ACCOUNT_A, SIMPLE_ACCOUNT_ABI, provider);
        const executeData = simpleAccount.interface.encodeFunctionData("execute", [
            PNT_TOKEN_ADDRESS,
            0, // value = 0 ETH
            transferData
        ]);

        // 获取 nonce
        const nonce = await simpleAccount.getNonce();
        console.log(`当前 nonce: ${nonce}`);

        // 获取 Gas 价格
        const feeData = await provider.getFeeData();
        const maxFeePerGas = feeData.maxFeePerGas || ethers.utils.parseUnits("100", "gwei");
        const maxPriorityFeePerGas = feeData.maxPriorityFeePerGas || ethers.utils.parseUnits("2", "gwei");

        console.log(`Max Fee Per Gas: ${ethers.utils.formatUnits(maxFeePerGas, "gwei")} Gwei`);

        // 构建 UserOperation
        const userOp = {
            sender: SIMPLE_ACCOUNT_A,
            nonce: ethers.utils.hexlify(nonce),
            initCode: "0x",
            callData: executeData,
            callGasLimit: "0x15F90", // 90000 gas
            verificationGasLimit: "0x15F90", // 90000 gas
            preVerificationGas: "0xAF3C", // 44868 gas (minimum required)
            maxFeePerGas: ethers.utils.hexlify(maxFeePerGas),
            maxPriorityFeePerGas: ethers.utils.hexlify(maxPriorityFeePerGas),
            paymasterAndData: "0x",
            signature: "0x"
        };

        // 5. 计算签名
        console.log("🔐 计算签名...");
        const signature = await signUserOpForSimpleAccount(
            userOp,
            PRIVATE_KEY,
            ENTRYPOINT_ADDRESS,
            CHAIN_ID
        );
        userOp.signature = signature;

        console.log("✅ UserOperation 构建完成");

        // 6. 发送 UserOperation
        console.log("\\n📤 发送 UserOperation 到 Bundler...");
        const userOpHash = await sendUserOperation(userOp);
        console.log(`UserOperation Hash: ${userOpHash}`);

        // 7. 等待确认
        console.log("\\n⏳ 等待交易确认...");
        const receipt = await waitForUserOpReceipt(userOpHash);

        console.log("✅ 转账成功！");
        console.log(`交易哈希: ${receipt.transactionHash}`);
        console.log(`区块号: ${receipt.blockNumber}`);
        console.log(`Gas 使用: ${receipt.gasUsed}`);

        // 8. 检查最终余额
        console.log("\\n📊 检查最终余额...");
        const finalBalances = await checkBalances(provider);

        console.log(`账户 A 余额: ${finalBalances.accountA.formatted} PNT`);
        console.log(`账户 B 余额: ${finalBalances.accountB.formatted} PNT`);

        // 9. 计算余额变化
        const senderDiff = balances.accountA.balance.sub(finalBalances.accountA.balance);
        const receiverDiff = finalBalances.accountB.balance.sub(balances.accountB.balance);

        console.log("\\n📈 余额变化:");
        console.log(`账户 A 减少: ${ethers.utils.formatEther(senderDiff)} PNT`);
        console.log(`账户 B 增加: ${ethers.utils.formatEther(receiverDiff)} PNT`);

        // 10. 验证转账
        const transferSuccess = senderDiff.eq(amount) && receiverDiff.eq(amount);
        console.log(`\\n🎯 转账验证: ${transferSuccess ? '✅ 成功' : '❌ 失败'}`);

        return {
            success: transferSuccess,
            userOpHash,
            transactionHash: receipt.transactionHash,
            blockNumber: receipt.blockNumber,
            gasUsed: receipt.gasUsed,
            transferAmount: ethers.utils.formatEther(amount),
            balanceChanges: {
                accountA: {
                    before: ethers.utils.formatEther(balances.accountA.balance),
                    after: ethers.utils.formatEther(finalBalances.accountA.balance),
                    change: ethers.utils.formatEther(senderDiff)
                },
                accountB: {
                    before: ethers.utils.formatEther(balances.accountB.balance),
                    after: ethers.utils.formatEther(finalBalances.accountB.balance),
                    change: ethers.utils.formatEther(receiverDiff)
                }
            }
        };

    } catch (error) {
        console.error("❌ A → B 转账失败:", error.message);
        throw error;
    }
}

// 主程序
if (require.main === module) {
    const args = process.argv.slice(2);

    if (args.length === 0) {
        console.log("用法:");
        console.log("  node testABTransfer.js <amount>");
        console.log("");
        console.log("示例:");
        console.log("  node testABTransfer.js 10");
        console.log("  node testABTransfer.js 2.5");
        process.exit(0);
    }

    const amount = parseFloat(args[0]);

    if (isNaN(amount) || amount <= 0) {
        console.error("❌ 无效的转账金额");
        process.exit(1);
    }

    testABTransfer(amount)
        .then(result => {
            console.log("\\n🎉 A → B 转账测试完成！");
            console.log(JSON.stringify(result, null, 2));
            process.exit(0);
        })
        .catch(error => {
            console.error("A → B 转账测试失败:", error);
            process.exit(1);
        });
}

module.exports = { testABTransfer, checkBalances };