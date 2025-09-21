// ERC-4337 PNT Token Transfer Test with Bundler
// 这是成功完成 5 PNT 转账的测试脚本

require('dotenv').config();
const { ethers } = require("ethers");

// 网络配置
const SEPOLIA_RPC = process.env.NODE_HTTP || "https://eth-sepolia.g.alchemy.com/v2/YOUR_API_KEY";
const BUNDLER_URL = process.env.BUNDLER_URL || "https://rundler-superrelay.fly.dev";
const CHAIN_ID = 11155111; // Sepolia

// 合约地址
const ENTRYPOINT_ADDRESS = "0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789";
const FACTORY_ADDRESS = "0x9406Cc6185a346906296840746125a0E44976454";
const PNT_TOKEN_ADDRESS = process.env.PNT_TOKEN_ADDRESS || "0x3e7B771d4541eC85c8137e950598Ac97553a337a";

// 账户配置
const ACCOUNT_A_PRIVATE_KEY = process.env.PRIVATE_KEY_A || process.env.PRIVATE_KEY;
const ACCOUNT_A_ADDRESS = "0x451caD1e2FCA26dE9faf715a549c4f336085c1AF";
const SIMPLE_ACCOUNT_ADDRESS = "0x6ff9A269085C79001e647b3D56C9176841A19935";
const CONTRACT_ACCOUNT_A = "0x6ff9A269085C79001e647b3D56C9176841A19935";

// 检查必需的环境变量
if (!ACCOUNT_A_PRIVATE_KEY) {
    console.error("❌ 错误: 缺少私钥环境变量");
    console.error("请设置 PRIVATE_KEY_A 或 PRIVATE_KEY 环境变量");
    console.error("示例: export PRIVATE_KEY_A=0xYOUR_PRIVATE_KEY");
    console.error("或者创建 .env 文件并添加 PRIVATE_KEY_A=0xYOUR_PRIVATE_KEY");
    process.exit(1);
}

// ABI 定义
const SIMPLE_ACCOUNT_ABI = [
    "function execute(address dest, uint256 value, bytes calldata func) external",
    "function getNonce() public view returns (uint256)",
    "function owner() public view returns (address)"
];

const ERC20_ABI = [
    "function transfer(address to, uint256 amount) public returns (bool)",
    "function balanceOf(address account) public view returns (uint256)",
    "function decimals() public view returns (uint8)"
];

const SIMPLE_ACCOUNT_FACTORY_ABI = [
    "function createAccount(address owner, uint256 salt) public returns (address)",
    "function getAddress(address owner, uint256 salt) public view returns (address)"
];

// 初始化 Provider
const provider = new ethers.providers.JsonRpcProvider(SEPOLIA_RPC);
const wallet = new ethers.Wallet(ACCOUNT_A_PRIVATE_KEY, provider);

// 合约实例
const pntToken = new ethers.Contract(PNT_TOKEN_ADDRESS, ERC20_ABI, provider);
const factory = new ethers.Contract(FACTORY_ADDRESS, SIMPLE_ACCOUNT_FACTORY_ABI, provider);

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
 * 为 SimpleAccount v0.6 签名 UserOperation
 * 关键发现：v0.6 使用 Ethereum Signed Message 格式，而非 EIP-712
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
        headers: {
            'Content-Type': 'application/json',
        },
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
 * 等待 UserOperation 被包含在区块中
 */
async function waitForUserOpReceipt(userOpHash) {
    console.log(`等待 UserOperation 被包含: ${userOpHash}`);

    for (let i = 0; i < 60; i++) { // 等待最多 5 分钟
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
            if (result.result) {
                return result.result;
            }
        } catch (error) {
            console.log(`检查收据失败: ${error.message}`);
        }

        await new Promise(resolve => setTimeout(resolve, 5000)); // 等待 5 秒
    }
    throw new Error('UserOperation 超时未被包含');
}

/**
 * 主测试函数：PNT 代币转账
 */
async function testPNTTransfer() {
    console.log("🚀 开始 ERC-4337 PNT 代币转账测试");
    console.log("=====================================");

    try {
        // 1. 检查初始余额
        console.log("\n📊 检查初始余额...");
        const senderBalance = await pntToken.balanceOf(SIMPLE_ACCOUNT_ADDRESS);
        const receiverBalance = await pntToken.balanceOf(CONTRACT_ACCOUNT_A);

        console.log(`发送方 (SimpleAccount): ${ethers.utils.formatEther(senderBalance)} PNT`);
        console.log(`接收方 (Contract A): ${ethers.utils.formatEther(receiverBalance)} PNT`);

        // 2. 准备转账数据
        const transferAmount = ethers.utils.parseEther("5"); // 5 PNT
        console.log(`\n💸 准备转账 ${ethers.utils.formatEther(transferAmount)} PNT`);

        // 编码 ERC20 transfer 调用
        const transferData = pntToken.interface.encodeFunctionData("transfer", [
            CONTRACT_ACCOUNT_A,
            transferAmount
        ]);

        // 编码 SimpleAccount execute 调用
        const simpleAccount = new ethers.Contract(SIMPLE_ACCOUNT_ADDRESS, SIMPLE_ACCOUNT_ABI, provider);
        const executeData = simpleAccount.interface.encodeFunctionData("execute", [
            PNT_TOKEN_ADDRESS,
            0, // value = 0 (no ETH transfer)
            transferData
        ]);

        // 3. 获取 nonce
        const nonce = await simpleAccount.getNonce();
        console.log(`当前 nonce: ${nonce}`);

        // 4. 获取 gas 价格
        const feeData = await provider.getFeeData();
        const maxFeePerGas = feeData.maxFeePerGas || ethers.utils.parseUnits("100", "gwei");
        const maxPriorityFeePerGas = feeData.maxPriorityFeePerGas || ethers.utils.parseUnits("2", "gwei");

        console.log(`Gas 价格: ${ethers.utils.formatUnits(maxFeePerGas, "gwei")} Gwei`);

        // 5. 构建 UserOperation
        const userOp = {
            sender: SIMPLE_ACCOUNT_ADDRESS,
            nonce: ethers.utils.hexlify(nonce),
            initCode: "0x", // 账户已存在
            callData: executeData,
            callGasLimit: "0x11170", // 70000
            verificationGasLimit: "0x11170", // 70000
            preVerificationGas: "0x5208", // 21000
            maxFeePerGas: ethers.utils.hexlify(maxFeePerGas),
            maxPriorityFeePerGas: ethers.utils.hexlify(maxPriorityFeePerGas),
            paymasterAndData: "0x",
            signature: "0x" // 暂时为空
        };

        console.log("\n🔐 计算签名...");

        // 6. 计算签名
        const signature = await signUserOpForSimpleAccount(
            userOp,
            ACCOUNT_A_PRIVATE_KEY,
            ENTRYPOINT_ADDRESS,
            CHAIN_ID
        );
        userOp.signature = signature;

        console.log(`签名: ${signature.slice(0, 20)}...`);

        // 7. 发送 UserOperation
        console.log("\n📤 发送 UserOperation 到 Bundler...");
        const userOpHash = await sendUserOperation(userOp);
        console.log(`UserOperation Hash: ${userOpHash}`);

        // 8. 等待确认
        console.log("\n⏳ 等待交易确认...");
        const receipt = await waitForUserOpReceipt(userOpHash);

        console.log("✅ 交易成功！");
        console.log(`交易哈希: ${receipt.transactionHash}`);
        console.log(`Gas 使用: ${receipt.gasUsed}`);

        // 9. 检查最终余额
        console.log("\n📊 检查最终余额...");
        const finalSenderBalance = await pntToken.balanceOf(SIMPLE_ACCOUNT_ADDRESS);
        const finalReceiverBalance = await pntToken.balanceOf(CONTRACT_ACCOUNT_A);

        console.log(`发送方 (SimpleAccount): ${ethers.utils.formatEther(finalSenderBalance)} PNT`);
        console.log(`接收方 (Contract A): ${ethers.utils.formatEther(finalReceiverBalance)} PNT`);

        // 10. 验证转账
        const senderDiff = senderBalance.sub(finalSenderBalance);
        const receiverDiff = finalReceiverBalance.sub(receiverBalance);

        console.log("\n🎯 转账验证:");
        console.log(`发送方减少: ${ethers.utils.formatEther(senderDiff)} PNT`);
        console.log(`接收方增加: ${ethers.utils.formatEther(receiverDiff)} PNT`);

        if (senderDiff.eq(transferAmount) && receiverDiff.eq(transferAmount)) {
            console.log("✅ 转账金额匹配！");
        } else {
            console.log("❌ 转账金额不匹配！");
        }

        return {
            success: true,
            userOpHash,
            transactionHash: receipt.transactionHash,
            gasUsed: receipt.gasUsed,
            balanceChanges: {
                sender: {
                    before: ethers.utils.formatEther(senderBalance),
                    after: ethers.utils.formatEther(finalSenderBalance)
                },
                receiver: {
                    before: ethers.utils.formatEther(receiverBalance),
                    after: ethers.utils.formatEther(finalReceiverBalance)
                }
            }
        };

    } catch (error) {
        console.error("❌ 测试失败:", error.message);
        throw error;
    }
}

// 运行测试
if (require.main === module) {
    testPNTTransfer()
        .then(result => {
            console.log("\n🎉 测试完成！", result);
            process.exit(0);
        })
        .catch(error => {
            console.error("测试失败:", error);
            process.exit(1);
        });
}

module.exports = {
    testPNTTransfer,
    signUserOpForSimpleAccount,
    getUserOpHash,
    sendUserOperation,
    waitForUserOpReceipt
};