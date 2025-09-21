// 向 EntryPoint 合约存款脚本
require('dotenv').config();
const { ethers } = require("ethers");

// 网络和合约配置
const SEPOLIA_RPC = process.env.NODE_HTTP || "https://eth-sepolia.g.alchemy.com/v2/YOUR_API_KEY";
const CHAIN_ID = 11155111;

const ENTRYPOINT_ADDRESS = "0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789";
const SIMPLE_ACCOUNT_A = process.env.SIMPLE_ACCOUNT_A || "0x7D7a0D3239285faE78F9c364D81bb1E3bc555BC6";

// 私钥配置 (使用 EOA 私钥，不是 SimpleAccount 私钥)
const EOA_PRIVATE_KEY = process.env.PRIVATE_KEY_A || process.env.PRIVATE_KEY;

// 检查必需的环境变量
if (!EOA_PRIVATE_KEY) {
    console.error("❌ 错误: 缺少私钥环境变量");
    console.error("请设置 PRIVATE_KEY_A 或 PRIVATE_KEY 环境变量");
    process.exit(1);
}

// EntryPoint ABI (只包含存款相关函数)
const ENTRYPOINT_ABI = [
    "function depositTo(address account) external payable",
    "function getDepositInfo(address account) external view returns (uint256 deposit, bool staked, uint112 stake, uint32 unstakeDelaySec, uint48 withdrawTime)",
    "function balanceOf(address account) external view returns (uint256)"
];

/**
 * 检查 EntryPoint 中的存款余额
 */
async function checkEntryPointBalance(provider, account) {
    try {
        const entryPoint = new ethers.Contract(ENTRYPOINT_ADDRESS, ENTRYPOINT_ABI, provider);

        const balance = await entryPoint.balanceOf(account);
        const depositInfo = await entryPoint.getDepositInfo(account);

        console.log(`📊 EntryPoint 存款信息:`);
        console.log(`  账户: ${account}`);
        console.log(`  余额: ${ethers.utils.formatEther(balance)} ETH (${balance.toString()} wei)`);
        console.log(`  存款: ${ethers.utils.formatEther(depositInfo.deposit)} ETH`);
        console.log(`  是否质押: ${depositInfo.staked}`);

        return { balance, depositInfo };
    } catch (error) {
        console.error('检查 EntryPoint 余额失败:', error);
        throw error;
    }
}

/**
 * 向 EntryPoint 存款
 */
async function depositToEntryPoint(amount) {
    try {
        console.log(`🚀 开始向 EntryPoint 存款`);
        console.log(`===========================`);
        console.log(`EntryPoint: ${ENTRYPOINT_ADDRESS}`);
        console.log(`受益账户: ${SIMPLE_ACCOUNT_A}`);
        console.log(`存款金额: ${amount} ETH`);
        console.log(``);

        // 连接到网络
        const provider = new ethers.providers.JsonRpcProvider(SEPOLIA_RPC);
        const wallet = new ethers.Wallet(EOA_PRIVATE_KEY, provider);

        console.log(`💰 检查 EOA 账户余额...`);
        const eoaBalance = await wallet.getBalance();
        console.log(`EOA 地址: ${wallet.address}`);
        console.log(`EOA 余额: ${ethers.utils.formatEther(eoaBalance)} ETH`);

        const depositAmount = ethers.utils.parseEther(amount);

        if (eoaBalance.lt(depositAmount)) {
            throw new Error(`EOA 余额不足: 需要 ${amount} ETH，但只有 ${ethers.utils.formatEther(eoaBalance)} ETH`);
        }

        // 检查存款前的状态
        console.log(`\n📊 存款前状态:`);
        const beforeDeposit = await checkEntryPointBalance(provider, SIMPLE_ACCOUNT_A);

        // 创建 EntryPoint 合约实例
        const entryPoint = new ethers.Contract(ENTRYPOINT_ADDRESS, ENTRYPOINT_ABI, wallet);

        console.log(`\n💸 执行存款交易...`);

        // 估算 gas
        const gasEstimate = await entryPoint.estimateGas.depositTo(SIMPLE_ACCOUNT_A, {
            value: depositAmount
        });

        console.log(`Gas 估算: ${gasEstimate.toString()}`);

        // 执行存款
        const tx = await entryPoint.depositTo(SIMPLE_ACCOUNT_A, {
            value: depositAmount,
            gasLimit: gasEstimate.mul(120).div(100) // 增加 20% 缓冲
        });

        console.log(`交易已发送: ${tx.hash}`);
        console.log(`⏳ 等待交易确认...`);

        const receipt = await tx.wait();

        if (receipt.status === 1) {
            console.log(`✅ 存款成功！`);
            console.log(`区块号: ${receipt.blockNumber}`);
            console.log(`Gas 使用: ${receipt.gasUsed.toString()}`);
        } else {
            throw new Error('交易失败');
        }

        // 检查存款后的状态
        console.log(`\n📊 存款后状态:`);
        const afterDeposit = await checkEntryPointBalance(provider, SIMPLE_ACCOUNT_A);

        // 计算变化
        const balanceIncrease = afterDeposit.balance.sub(beforeDeposit.balance);
        console.log(`\n📈 存款变化:`);
        console.log(`增加: ${ethers.utils.formatEther(balanceIncrease)} ETH`);
        console.log(`预期: ${amount} ETH`);
        console.log(`匹配: ${balanceIncrease.eq(depositAmount) ? '✅' : '❌'}`);

        console.log(`\n🎉 EntryPoint 存款完成！`);

        return {
            success: true,
            txHash: tx.hash,
            blockNumber: receipt.blockNumber,
            gasUsed: receipt.gasUsed.toString(),
            balanceBefore: ethers.utils.formatEther(beforeDeposit.balance),
            balanceAfter: ethers.utils.formatEther(afterDeposit.balance),
            deposited: ethers.utils.formatEther(balanceIncrease)
        };

    } catch (error) {
        console.error(`❌ EntryPoint 存款失败:`, error.message);
        throw error;
    }
}

// 主函数
async function main() {
    const args = process.argv.slice(2);

    if (args.length === 0) {
        console.log(`用法:`);
        console.log(`  node depositToEntryPoint.js <金额(ETH)>`);
        console.log(``);
        console.log(`示例:`);
        console.log(`  node depositToEntryPoint.js 0.01`);
        console.log(`  node depositToEntryPoint.js 0.005`);
        console.log(``);
        console.log(`检查当前余额:`);
        console.log(`  node depositToEntryPoint.js check`);
        return;
    }

    if (args[0] === 'check') {
        const provider = new ethers.providers.JsonRpcProvider(SEPOLIA_RPC);
        await checkEntryPointBalance(provider, SIMPLE_ACCOUNT_A);
        return;
    }

    const amount = args[0];

    if (isNaN(parseFloat(amount)) || parseFloat(amount) <= 0) {
        console.error(`❌ 错误: 无效的金额 "${amount}"`);
        console.error(`请提供有效的 ETH 金额，例如: 0.01`);
        process.exit(1);
    }

    try {
        const result = await depositToEntryPoint(amount);
        console.log(JSON.stringify(result, null, 2));
    } catch (error) {
        console.error('存款失败:', error);
        process.exit(1);
    }
}

// 执行主函数
if (require.main === module) {
    main().catch(error => {
        console.error('脚本执行失败:', error);
        process.exit(1);
    });
}

module.exports = { depositToEntryPoint, checkEntryPointBalance };