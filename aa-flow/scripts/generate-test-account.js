// 生成测试账户配置
require('dotenv').config();
const { ethers } = require("ethers");

const SEPOLIA_RPC = process.env.NODE_HTTP;
const FACTORY_ADDRESS = "0x9406Cc6185a346906296840746125a0E44976454";

const SIMPLE_ACCOUNT_FACTORY_ABI = [
    "function createAccount(address owner, uint256 salt) public returns (address)",
    "function getAddress(address owner, uint256 salt) public view returns (address)"
];

async function generateTestAccount() {
    console.log("🔧 生成测试账户配置...");

    // 1. 生成新的随机私钥
    const wallet = ethers.Wallet.createRandom();
    console.log(`\n🔑 新生成的测试私钥:`);
    console.log(`私钥: ${wallet.privateKey}`);
    console.log(`EOA 地址: ${wallet.address}`);

    // 2. 计算对应的 SimpleAccount 地址
    const provider = new ethers.providers.JsonRpcProvider(SEPOLIA_RPC);
    const factory = new ethers.Contract(FACTORY_ADDRESS, SIMPLE_ACCOUNT_FACTORY_ABI, provider);

    const salt = 0; // 使用 salt = 0
    const simpleAccountAddress = await factory.getAddress(wallet.address, salt);

    console.log(`\n🏠 对应的 SimpleAccount 地址:`);
    console.log(`SimpleAccount: ${simpleAccountAddress}`);

    // 3. 生成更新的 .env 配置
    console.log(`\n📝 更新 .env 配置:`);
    console.log(`PRIVATE_KEY_A="${wallet.privateKey}"`);
    console.log(`PRIVATE_KEY="${wallet.privateKey}"`);
    console.log(`EOA_ADDRESS="${wallet.address}"`);
    console.log(`SIMPLE_ACCOUNT_ADDRESS="${simpleAccountAddress}"`);

    // 4. 提醒需要做的事情
    console.log(`\n⚠️  接下来需要做的事情:`);
    console.log(`1. 更新 .env 文件中的私钥和地址`);
    console.log(`2. 向 EOA 地址发送一些 Sepolia ETH (用于 gas)`);
    console.log(`3. 部署 SimpleAccount (或向其发送 PNT 代币)`);
    console.log(`4. 运行测试`);

    console.log(`\n🚰 获取测试 ETH:`);
    console.log(`- Sepolia Faucet: https://sepoliafaucet.com/`);
    console.log(`- 发送到 EOA: ${wallet.address}`);

    return {
        privateKey: wallet.privateKey,
        eoaAddress: wallet.address,
        simpleAccountAddress
    };
}

if (require.main === module) {
    generateTestAccount()
        .then(result => {
            console.log("\n✅ 测试账户生成完成！");
        })
        .catch(error => {
            console.error("❌ 生成失败:", error);
        });
}

module.exports = { generateTestAccount };