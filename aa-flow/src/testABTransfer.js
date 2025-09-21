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
 * 计算 preVerificationGas 的精确算法
 */
function calculatePreVerificationGas(userOp) {
    const FIXED_GAS_OVERHEAD = 21000;  // 基础交易 gas
    const PER_USER_OP_OVERHEAD = 18300; // 每个 UserOp 的固定开销
    const PER_USER_OP_WORD = 4;         // 每个字的开销
    const ZERO_BYTE_COST = 4;           // 零字节成本
    const NON_ZERO_BYTE_COST = 16;      // 非零字节成本
    const SAFETY_BUFFER = 1000;         // 安全缓冲

    // 序列化 UserOperation 用于计算字节数
    const userOpData = JSON.stringify({
        sender: userOp.sender || '',
        nonce: userOp.nonce || '0x0',
        initCode: userOp.initCode || '0x',
        callData: userOp.callData || '0x',
        callGasLimit: userOp.callGasLimit || '0x0',
        verificationGasLimit: userOp.verificationGasLimit || '0x0',
        preVerificationGas: '0x0', // 临时值
        maxFeePerGas: userOp.maxFeePerGas || '0x0',
        maxPriorityFeePerGas: userOp.maxPriorityFeePerGas || '0x0',
        paymasterAndData: userOp.paymasterAndData || '0x',
        signature: userOp.signature || '0x'
    });

    // 计算字节成本
    let byteCost = 0;
    const bytes = Buffer.from(userOpData, 'utf8');

    for (const byte of bytes) {
        if (byte === 0) {
            byteCost += ZERO_BYTE_COST;
        } else {
            byteCost += NON_ZERO_BYTE_COST;
        }
    }

    // 计算总的 preVerificationGas
    const totalGas = FIXED_GAS_OVERHEAD +
                    PER_USER_OP_OVERHEAD +
                    Math.ceil(bytes.length / 32) * PER_USER_OP_WORD +
                    byteCost +
                    SAFETY_BUFFER;

    console.log(`🧮 PreVerificationGas 计算:`);
    console.log(`  - 基础交易 gas: ${FIXED_GAS_OVERHEAD}`);
    console.log(`  - UserOp 固定开销: ${PER_USER_OP_OVERHEAD}`);
    console.log(`  - 数据长度: ${bytes.length} bytes`);
    console.log(`  - 字节成本: ${byteCost}`);
    console.log(`  - 安全缓冲: ${SAFETY_BUFFER}`);
    console.log(`  - 计算总量: ${totalGas} (0x${totalGas.toString(16)})`);

    return ethers.utils.hexlify(totalGas);
}

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

        // 为 Sepolia 测试网使用非常低的 gas 价格
        const minMaxFee = ethers.BigNumber.from("100100000"); // 100100000 wei ≈ 0.1001 gwei (bundler 最低要求)
        const sepoliaMaxFee = ethers.utils.parseUnits("0.2", "gwei"); // 0.2 gwei，适合测试网
        // 忽略网络返回的过高价格，直接使用测试网合理价格
        const maxFeePerGas = sepoliaMaxFee.gt(minMaxFee) ? sepoliaMaxFee : ethers.utils.parseUnits("0.2", "gwei");

        // 使用测试网合理的优先费用
        const minPriorityFee = ethers.BigNumber.from("100000000"); // 100000000 wei ≈ 0.1 gwei (bundler 最低要求)
        const sepoliaPriorityFee = ethers.utils.parseUnits("0.1", "gwei"); // 0.1 gwei，刚好满足要求
        // 直接使用最低要求，不依赖网络返回值
        const maxPriorityFeePerGas = sepoliaPriorityFee;

        console.log(`💰 Gas 费用设置:`);
        console.log(`  - Max Fee Per Gas: ${ethers.utils.formatUnits(maxFeePerGas, "gwei")} Gwei (${maxFeePerGas.toString()} wei)`);
        console.log(`  - maxFeePerGas 最低要求: ${ethers.utils.formatUnits(minMaxFee, "gwei")} Gwei (${minMaxFee.toString()} wei)`);
        console.log(`  - Max Priority Fee: ${ethers.utils.formatUnits(maxPriorityFeePerGas, "gwei")} Gwei (${maxPriorityFeePerGas.toString()} wei)`);
        console.log(`  - maxPriorityFeePerGas 最低要求: ${ethers.utils.formatUnits(minPriorityFee, "gwei")} Gwei (${minPriorityFee.toString()} wei)`);

        // 先构建基础的 UserOperation（不含 preVerificationGas）
        const baseUserOp = {
            sender: SIMPLE_ACCOUNT_A,
            nonce: ethers.utils.hexlify(nonce),
            initCode: "0x",
            callData: executeData,
            callGasLimit: "0x15F90", // 90000 gas
            verificationGasLimit: "0x15F90", // 90000 gas
            maxFeePerGas: ethers.utils.hexlify(maxFeePerGas),
            maxPriorityFeePerGas: ethers.utils.hexlify(maxPriorityFeePerGas),
            paymasterAndData: "0x",
            signature: "0x"
        };

        // 动态计算 preVerificationGas
        const calculatedPreVerificationGas = calculatePreVerificationGas(baseUserOp);

        // 构建完整的 UserOperation
        const userOp = {
            ...baseUserOp,
            preVerificationGas: calculatedPreVerificationGas
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

        // 生成浏览器链接
        const jiffyScanUrl = `https://jiffyscan.xyz/userOpHash/${userOpHash}?network=sepolia`;
        const etherscanUrl = receipt.transactionHash
            ? `https://sepolia.etherscan.io/tx/${receipt.transactionHash}`
            : null;

        console.log(`\\n🔗 浏览器链接:`);
        console.log(`JiffyScan (UserOp): ${jiffyScanUrl}`);
        if (etherscanUrl) {
            console.log(`Etherscan (Tx): ${etherscanUrl}`);
        }

        return {
            success: transferSuccess,
            userOpHash,
            jiffyScanUrl,
            etherscanUrl,
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