// 部署 SimpleAccount A 和 B
require('dotenv').config();
const { ethers } = require("ethers");

// 网络和合约配置
const SEPOLIA_RPC = process.env.NODE_HTTP;
const FACTORY_ADDRESS = process.env.FACTORY_ADDRESS || "0x9406Cc6185a346906296840746125a0E44976454";
const PRIVATE_KEY = process.env.PRIVATE_KEY;

// 账户配置
const EOA_ADDRESS = process.env.EOA_ADDRESS;
const SIMPLE_ACCOUNT_A = process.env.SIMPLE_ACCOUNT_A;
const SIMPLE_ACCOUNT_B = process.env.SIMPLE_ACCOUNT_B;

const SIMPLE_ACCOUNT_FACTORY_ABI = [
    "function createAccount(address owner, uint256 salt) public returns (address)",
    "function getAddress(address owner, uint256 salt) public view returns (address)"
];

async function deployAccounts() {
    console.log("🚀 开始部署 SimpleAccount A 和 B");
    console.log("================================");

    const provider = new ethers.providers.JsonRpcProvider(SEPOLIA_RPC);
    const wallet = new ethers.Wallet(PRIVATE_KEY, provider);
    const factory = new ethers.Contract(FACTORY_ADDRESS, SIMPLE_ACCOUNT_FACTORY_ABI, wallet);

    console.log(`EOA 地址: ${EOA_ADDRESS}`);
    console.log(`Factory 地址: ${FACTORY_ADDRESS}`);

    try {
        // 1. 检查 EOA 余额
        const balance = await provider.getBalance(EOA_ADDRESS);
        console.log(`\n💰 EOA ETH 余额: ${ethers.utils.formatEther(balance)} ETH`);

        if (balance.lt(ethers.utils.parseEther("0.01"))) {
            throw new Error("EOA 余额不足，需要至少 0.01 ETH 用于部署");
        }

        // 2. 检查当前部署状态
        const [codeA, codeB] = await Promise.all([
            provider.getCode(SIMPLE_ACCOUNT_A),
            provider.getCode(SIMPLE_ACCOUNT_B)
        ]);

        const isADeployed = codeA !== "0x";
        const isBDeployed = codeB !== "0x";

        console.log(`\n🏠 当前部署状态:`);
        console.log(`账户 A: ${isADeployed ? '✅ 已部署' : '❌ 未部署'} (${SIMPLE_ACCOUNT_A})`);
        console.log(`账户 B: ${isBDeployed ? '✅ 已部署' : '❌ 未部署'} (${SIMPLE_ACCOUNT_B})`);

        // 3. 部署账户 A (salt = 0)
        if (!isADeployed) {
            console.log(`\n🔧 部署账户 A...`);
            const txA = await factory.createAccount(EOA_ADDRESS, 0);
            console.log(`交易哈希: ${txA.hash}`);

            const receiptA = await txA.wait();
            console.log(`✅ 账户 A 部署成功！区块: ${receiptA.blockNumber}`);

            // 验证地址
            const deployedA = await factory.getAddress(EOA_ADDRESS, 0);
            if (deployedA.toLowerCase() !== SIMPLE_ACCOUNT_A.toLowerCase()) {
                throw new Error(`地址不匹配: 期望 ${SIMPLE_ACCOUNT_A}, 实际 ${deployedA}`);
            }
        } else {
            console.log(`\n✅ 账户 A 已存在，跳过部署`);
        }

        // 4. 部署账户 B (salt = 1)
        if (!isBDeployed) {
            console.log(`\n🔧 部署账户 B...`);
            const txB = await factory.createAccount(EOA_ADDRESS, 1);
            console.log(`交易哈希: ${txB.hash}`);

            const receiptB = await txB.wait();
            console.log(`✅ 账户 B 部署成功！区块: ${receiptB.blockNumber}`);

            // 验证地址
            const deployedB = await factory.getAddress(EOA_ADDRESS, 1);
            if (deployedB.toLowerCase() !== SIMPLE_ACCOUNT_B.toLowerCase()) {
                throw new Error(`地址不匹配: 期望 ${SIMPLE_ACCOUNT_B}, 实际 ${deployedB}`);
            }
        } else {
            console.log(`\n✅ 账户 B 已存在，跳过部署`);
        }

        // 5. 最终验证
        console.log(`\n🔍 最终验证...`);
        const [finalCodeA, finalCodeB] = await Promise.all([
            provider.getCode(SIMPLE_ACCOUNT_A),
            provider.getCode(SIMPLE_ACCOUNT_B)
        ]);

        const finalADeployed = finalCodeA !== "0x";
        const finalBDeployed = finalCodeB !== "0x";

        console.log(`账户 A: ${finalADeployed ? '✅ 部署成功' : '❌ 部署失败'}`);
        console.log(`账户 B: ${finalBDeployed ? '✅ 部署成功' : '❌ 部署失败'}`);

        if (finalADeployed && finalBDeployed) {
            console.log(`\n🎉 所有账户部署完成！现在可以进行转账测试：`);
            console.log(`npm run test:ab 10`);
        }

        return {
            accountA: {
                address: SIMPLE_ACCOUNT_A,
                deployed: finalADeployed
            },
            accountB: {
                address: SIMPLE_ACCOUNT_B,
                deployed: finalBDeployed
            }
        };

    } catch (error) {
        console.error("❌ 部署失败:", error.message);
        throw error;
    }
}

if (require.main === module) {
    deployAccounts()
        .then(result => {
            console.log("\n✅ 部署完成！");
            process.exit(0);
        })
        .catch(error => {
            console.error("部署失败:", error);
            process.exit(1);
        });
}

module.exports = { deployAccounts };