// 简单的 UserOperation 状态检查工具

const BUNDLER_URL = "https://rundler-superrelay.fly.dev";

/**
 * 检查 UserOperation 状态
 */
async function checkUserOpStatus(userOpHash) {
    console.log(`🔍 检查 UserOperation: ${userOpHash}`);
    console.log("========================================");

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
        console.log("📡 Bundler 响应:", JSON.stringify(result, null, 2));

        if (result.result) {
            console.log("\n✅ UserOperation 已完成！");
            console.log(`   交易哈希: ${result.result.transactionHash}`);
            console.log(`   区块号: ${result.result.blockNumber}`);
            console.log(`   Gas 使用: ${result.result.gasUsed}`);
            console.log(`   状态: SUCCESS`);

            // 显示更多详细信息
            if (result.result.logs && result.result.logs.length > 0) {
                console.log(`   事件日志: ${result.result.logs.length} 个`);
            }

            return { status: 'completed', receipt: result.result };
        } else if (result.result === null) {
            console.log("\n⏳ UserOperation 仍在等待打包");
            console.log("   状态: PENDING");
            console.log("   原因: 可能费用不足或其他验证问题");
            return { status: 'pending' };
        } else {
            console.log("\n❓ 状态未知");
            console.log("   响应:", result);
            return { status: 'unknown', data: result };
        }
    } catch (error) {
        console.error("\n❌ 检查状态失败:", error.message);
        return { status: 'error', error: error.message };
    }
}

/**
 * 主函数
 */
async function main() {
    const args = process.argv.slice(2);

    if (args.length === 0) {
        console.log("用法: node checkUserOp.js <userOpHash>");
        console.log("示例: node checkUserOp.js 0x9574de239acbaf0f42fe338f71342315dfdd02ecef104add24ae18fa7cc580fd");
        return;
    }

    const userOpHash = args[0];
    await checkUserOpStatus(userOpHash);
}

// 运行脚本
main().catch(console.error);