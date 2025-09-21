// 简单的余额检查脚本
require('dotenv').config();
const { ethers } = require("ethers");

const SEPOLIA_RPC = process.env.NODE_HTTP;
const PNT_TOKEN_ADDRESS = process.env.PNT_TOKEN_ADDRESS || "0x3e7B771d4541eC85c8137e950598Ac97553a337a";
const SIMPLE_ACCOUNT_ADDRESS = process.env.SIMPLE_ACCOUNT_ADDRESS || "0xC33733449b3f3052E80E8a6ac0145bB3FA87dd6b";

const ERC20_ABI = [
    "function balanceOf(address account) public view returns (uint256)",
    "function name() public view returns (string)",
    "function symbol() public view returns (string)"
];

async function checkBalances() {
    console.log("🔍 检查账户余额和连接");
    console.log("=======================");
    console.log(`RPC: ${SEPOLIA_RPC}`);
    console.log(`PNT Token: ${PNT_TOKEN_ADDRESS}`);
    console.log(`SimpleAccount: ${SIMPLE_ACCOUNT_ADDRESS}`);
    console.log("");

    try {
        const provider = new ethers.providers.JsonRpcProvider(SEPOLIA_RPC);

        // 测试基本连接
        console.log("📡 测试 RPC 连接...");
        const blockNumber = await provider.getBlockNumber();
        console.log(`✅ 当前区块: ${blockNumber}`);

        // 检查 ETH 余额
        console.log("\n💰 检查 ETH 余额...");
        const ethBalance = await provider.getBalance(SIMPLE_ACCOUNT_ADDRESS);
        console.log(`SimpleAccount ETH: ${ethers.utils.formatEther(ethBalance)} ETH`);

        // 检查 PNT 代币合约
        console.log("\n🪙 检查 PNT 代币...");
        const pntToken = new ethers.Contract(PNT_TOKEN_ADDRESS, ERC20_ABI, provider);

        try {
            const name = await pntToken.name();
            const symbol = await pntToken.symbol();
            console.log(`代币名称: ${name}`);
            console.log(`代币符号: ${symbol}`);
        } catch (error) {
            console.log("⚠️ 无法获取代币信息:", error.message);
        }

        try {
            const balance = await pntToken.balanceOf(SIMPLE_ACCOUNT_ADDRESS);
            console.log(`PNT 余额: ${ethers.utils.formatEther(balance)} PNT`);
        } catch (error) {
            console.log("⚠️ 无法获取 PNT 余额:", error.message);
        }

        // 检查合约是否存在
        console.log("\n🔍 检查合约代码...");
        const code = await provider.getCode(PNT_TOKEN_ADDRESS);
        if (code === "0x") {
            console.log("❌ PNT 代币合约不存在或地址错误");
        } else {
            console.log(`✅ PNT 代币合约存在 (代码长度: ${code.length} 字符)`);
        }

        const simpleAccountCode = await provider.getCode(SIMPLE_ACCOUNT_ADDRESS);
        if (simpleAccountCode === "0x") {
            console.log("❌ SimpleAccount 合约不存在或地址错误");
        } else {
            console.log(`✅ SimpleAccount 存在 (代码长度: ${simpleAccountCode.length} 字符)`);
        }

    } catch (error) {
        console.error("❌ 检查失败:", error.message);
    }
}

checkBalances();