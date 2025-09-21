// 生成 A、B 两个测试账户，使用同一个私钥控制
require('dotenv').config();
const { ethers } = require("ethers");

const SEPOLIA_RPC = process.env.NODE_HTTP;
const FACTORY_ADDRESS = "0x9406Cc6185a346906296840746125a0E44976454";

const SIMPLE_ACCOUNT_FACTORY_ABI = [
    "function createAccount(address owner, uint256 salt) public returns (address)",
    "function getAddress(address owner, uint256 salt) public view returns (address)"
];

async function generateABAccounts() {
    console.log("🔧 生成 A、B 两个测试账户...");

    // 1. 生成一个私钥控制两个账户
    const wallet = ethers.Wallet.createRandom();
    console.log(`\n🔑 共享私钥 (控制账户 A 和 B):`);
    console.log(`私钥: ${wallet.privateKey}`);
    console.log(`EOA 地址: ${wallet.address}`);

    // 2. 计算对应的 SimpleAccount 地址 (使用不同的 salt)
    const provider = new ethers.providers.JsonRpcProvider(SEPOLIA_RPC);
    const factory = new ethers.Contract(FACTORY_ADDRESS, SIMPLE_ACCOUNT_FACTORY_ABI, provider);

    const saltA = 0; // 账户 A 使用 salt = 0
    const saltB = 1; // 账户 B 使用 salt = 1

    const accountA = await factory.getAddress(wallet.address, saltA);
    const accountB = await factory.getAddress(wallet.address, saltB);

    console.log(`\n🏠 账户地址:`);
    console.log(`账户 A (SimpleAccount): ${accountA}`);
    console.log(`账户 B (SimpleAccount): ${accountB}`);

    // 3. 生成 .env 配置
    console.log(`\n📝 .env 配置:`);
    console.log(`# Test accounts controlled by same private key`);
    console.log(`PRIVATE_KEY="${wallet.privateKey}"`);
    console.log(`PRIVATE_KEY_A="${wallet.privateKey}"`);
    console.log(`PRIVATE_KEY_B="${wallet.privateKey}"`);
    console.log(``);
    console.log(`# EOA Address (owner of both SimpleAccounts)`);
    console.log(`EOA_ADDRESS="${wallet.address}"`);
    console.log(``);
    console.log(`# SimpleAccount addresses`);
    console.log(`SIMPLE_ACCOUNT_A="${accountA}"`);
    console.log(`SIMPLE_ACCOUNT_B="${accountB}"`);

    // 4. 生成资金需求说明
    console.log(`\n💰 资金需求:`);
    console.log(`请向以下地址发送资金:`);
    console.log(``);
    console.log(`1. EOA 地址 (用于 gas 费用):`);
    console.log(`   地址: ${wallet.address}`);
    console.log(`   需要: 0.05 ETH (Sepolia 测试网)`);
    console.log(`   用途: 支付 UserOperation 的 gas 费用`);
    console.log(``);
    console.log(`2. SimpleAccount A (发送方):`);
    console.log(`   地址: ${accountA}`);
    console.log(`   需要: 100 PNT 代币`);
    console.log(`   用途: 测试转账的发送方`);
    console.log(``);
    console.log(`3. SimpleAccount B (接收方):`);
    console.log(`   地址: ${accountB}`);
    console.log(`   需要: 0 PNT (初始为空，接收转账)`);
    console.log(`   用途: 测试转账的接收方`);

    // 5. 获取测试资金的方式
    console.log(`\n🚰 获取测试资金:`);
    console.log(`ETH (Sepolia):`);
    console.log(`- Sepolia Faucet: https://sepoliafaucet.com/`);
    console.log(`- Alchemy Faucet: https://sepoliafaucets.alchemy.com/sepolia`);
    console.log(`- 发送到: ${wallet.address}`);
    console.log(``);
    console.log(`PNT 代币:`);
    console.log(`- 合约地址: 0x3e7B771d4541eC85c8137e950598Ac97553a337a`);
    console.log(`- 发送到: ${accountA} (SimpleAccount A)`);

    // 6. 测试步骤
    console.log(`\n🧪 测试步骤:`);
    console.log(`1. 更新 .env 文件`);
    console.log(`2. 向 EOA 发送 0.05 ETH`);
    console.log(`3. 向 SimpleAccount A 发送 100 PNT`);
    console.log(`4. 运行: npm run test:ab 10  (转账 10 PNT 从 A 到 B)`);

    return {
        privateKey: wallet.privateKey,
        eoaAddress: wallet.address,
        accountA,
        accountB,
        saltA,
        saltB
    };
}

if (require.main === module) {
    generateABAccounts()
        .then(result => {
            console.log("\n✅ A、B 测试账户生成完成！");
            console.log("\n📋 摘要:");
            console.log(`EOA: ${result.eoaAddress}`);
            console.log(`账户 A: ${result.accountA}`);
            console.log(`账户 B: ${result.accountB}`);
        })
        .catch(error => {
            console.error("❌ 生成失败:", error);
        });
}

module.exports = { generateABAccounts };