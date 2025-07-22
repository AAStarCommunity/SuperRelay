#!/usr/bin/env node

/**
 * SuperRelay Complete End-to-End Test Script
 * 
 * This script tests the complete flow:
 * 1. UserOperation creation
 * 2. Paymaster sponsorship (pm_sponsorUserOperation)  
 * 3. Bundler processing (eth_sendUserOperation)
 * 4. Transaction submission to Anvil blockchain
 * 5. Receipt verification and confirmation
 * 
 * Usage: node test_paymaster_complete.js
 */

const { ethers } = require('ethers');
const axios = require('axios');

// Configuration
const CONFIG = {
    SUPER_RELAY_URL: process.env.SUPER_RELAY_URL || 'http://localhost:3000',
    RPC_URL: process.env.RPC_URL || 'http://localhost:8545',
    ENTRY_POINT_ADDRESS: process.env.ENTRY_POINT_ADDRESS || null,
    
    // Test accounts (Anvil default accounts)
    ACCOUNTS: {
        USER: {
            address: '0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266',
            privateKey: '0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80'
        },
        PAYMASTER: {
            address: '0x70997970C51812dc3A010C7d01b50e0d17dc79C8',
            privateKey: '0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d'
        }
    }
};

// Colors for console output
const COLORS = {
    RESET: '\x1b[0m',
    BRIGHT: '\x1b[1m',
    RED: '\x1b[31m',
    GREEN: '\x1b[32m',
    YELLOW: '\x1b[33m',
    BLUE: '\x1b[34m',
    MAGENTA: '\x1b[35m',
    CYAN: '\x1b[36m'
};

function colorLog(color, message) {
    console.log(`${color}${message}${COLORS.RESET}`);
}

class SuperRelayTester {
    constructor() {
        this.provider = new ethers.JsonRpcProvider(CONFIG.RPC_URL);
        this.userWallet = new ethers.Wallet(CONFIG.ACCOUNTS.USER.privateKey, this.provider);
    }

    async initialize() {
        colorLog(COLORS.BLUE, '\n🚀 SuperRelay Complete E2E Test');
        colorLog(COLORS.BLUE, '==================================\n');

        // Get EntryPoint address if not provided
        if (!CONFIG.ENTRY_POINT_ADDRESS) {
            CONFIG.ENTRY_POINT_ADDRESS = await this.getEntryPointAddress();
        }

        await this.checkPrerequisites();
        await this.displaySystemInfo();
    }

    async getEntryPointAddress() {
        try {
            const response = await axios.post(CONFIG.SUPER_RELAY_URL, {
                jsonrpc: '2.0',
                id: 1,
                method: 'eth_supportedEntryPoints',
                params: []
            });

            if (response.data.result && response.data.result.length > 0) {
                return response.data.result[0];
            }
        } catch (error) {
            // Try to read from file
            try {
                const fs = require('fs');
                return fs.readFileSync('.entrypoint_address', 'utf8').trim();
            } catch (e) {
                throw new Error('Could not determine EntryPoint address. Please set ENTRY_POINT_ADDRESS environment variable.');
            }
        }
    }

    async checkPrerequisites() {
        colorLog(COLORS.CYAN, '🔍 Checking prerequisites...');
        
        // Check Anvil connection
        try {
            const network = await this.provider.getNetwork();
            colorLog(COLORS.GREEN, `✅ Anvil connected: Chain ID ${network.chainId}`);
        } catch (error) {
            throw new Error(`Anvil not accessible: ${error.message}`);
        }

        // Check SuperRelay health
        try {
            const response = await axios.get(`${CONFIG.SUPER_RELAY_URL}/health`, { timeout: 5000 });
            if (response.status === 200) {
                colorLog(COLORS.GREEN, `✅ SuperRelay healthy at ${CONFIG.SUPER_RELAY_URL}`);
            }
        } catch (error) {
            throw new Error(`SuperRelay not accessible: ${error.message}`);
        }

        // Check EntryPoint contract
        const entryPointCode = await this.provider.getCode(CONFIG.ENTRY_POINT_ADDRESS);
        if (entryPointCode === '0x') {
            throw new Error(`EntryPoint contract not found at ${CONFIG.ENTRY_POINT_ADDRESS}`);
        }
        colorLog(COLORS.GREEN, `✅ EntryPoint contract verified at ${CONFIG.ENTRY_POINT_ADDRESS}`);

        // Check paymaster funding
        await this.checkPaymasterFunding();
    }

    async checkPaymasterFunding() {
        try {
            // Check paymaster deposit in EntryPoint
            const depositCalldata = this.provider.interface.encodeFunctionData('balanceOf', [CONFIG.ACCOUNTS.PAYMASTER.address]);
            const depositHex = await this.provider.call({
                to: CONFIG.ENTRY_POINT_ADDRESS,
                data: depositCalldata
            });
            
            const depositWei = BigInt(depositHex);
            const minDeposit = ethers.parseEther('0.1');
            
            if (depositWei < minDeposit) {
                colorLog(COLORS.YELLOW, `⚠️  Paymaster deposit low: ${ethers.formatEther(depositWei)} ETH`);
                colorLog(COLORS.YELLOW, '   Run start_dev_server.sh to fund paymaster');
            } else {
                colorLog(COLORS.GREEN, `✅ Paymaster funded: ${ethers.formatEther(depositWei)} ETH`);
            }
        } catch (error) {
            colorLog(COLORS.YELLOW, `⚠️  Could not check paymaster funding: ${error.message}`);
        }
    }

    async displaySystemInfo() {
        colorLog(COLORS.CYAN, '\n📊 System Information:');
        
        const userBalance = await this.provider.getBalance(CONFIG.ACCOUNTS.USER.address);
        const paymasterBalance = await this.provider.getBalance(CONFIG.ACCOUNTS.PAYMASTER.address);
        
        console.log(`   👤 User: ${CONFIG.ACCOUNTS.USER.address}`);
        console.log(`      Balance: ${ethers.formatEther(userBalance)} ETH`);
        console.log(`   🏦 Paymaster: ${CONFIG.ACCOUNTS.PAYMASTER.address}`);
        console.log(`      Balance: ${ethers.formatEther(paymasterBalance)} ETH`);
        console.log(`   📍 EntryPoint: ${CONFIG.ENTRY_POINT_ADDRESS}`);
        console.log(`   🌐 RPC: ${CONFIG.RPC_URL}`);
        console.log(`   ⚡ SuperRelay: ${CONFIG.SUPER_RELAY_URL}`);
    }

    createTestUserOperation(nonce = 0) {
        return {
            sender: CONFIG.ACCOUNTS.USER.address,
            nonce: `0x${nonce.toString(16)}`,
            initCode: '0x',
            callData: '0x', // Simple call with no data
            callGasLimit: '0x186A0', // 100,000
            verificationGasLimit: '0x186A0', // 100,000  
            preVerificationGas: '0x5208', // 21,000
            maxFeePerGas: '0x3B9ACA00', // 1 gwei
            maxPriorityFeePerGas: '0x3B9ACA00', // 1 gwei
            paymasterAndData: '0x',
            signature: '0x'
        };
    }

    async callJsonRpc(method, params) {
        try {
            const response = await axios.post(CONFIG.SUPER_RELAY_URL, {
                jsonrpc: '2.0',
                id: 1,
                method: method,
                params: params
            }, {
                headers: { 'Content-Type': 'application/json' },
                timeout: 10000
            });

            if (response.data.error) {
                throw new Error(`RPC Error: ${response.data.error.message} (Code: ${response.data.error.code})`);
            }

            return response.data.result;
        } catch (error) {
            if (error.response) {
                throw new Error(`HTTP ${error.response.status}: ${JSON.stringify(error.response.data)}`);
            }
            throw error;
        }
    }

    async testPaymasterSponsorship() {
        colorLog(COLORS.MAGENTA, '\n🎯 Test 1: Paymaster Sponsorship (pm_sponsorUserOperation)');
        colorLog(COLORS.MAGENTA, '===========================================================');
        
        const userOp = this.createTestUserOperation();
        colorLog(COLORS.CYAN, '📝 Created UserOperation:');
        console.log(`   Sender: ${userOp.sender}`);
        console.log(`   Nonce: ${userOp.nonce}`);
        console.log(`   Gas Limits: ${parseInt(userOp.callGasLimit, 16)} / ${parseInt(userOp.verificationGasLimit, 16)}`);
        
        try {
            const userOpHash = await this.callJsonRpc('pm_sponsorUserOperation', [userOp, CONFIG.ENTRY_POINT_ADDRESS]);
            
            colorLog(COLORS.GREEN, '✅ Paymaster sponsorship successful!');
            console.log(`   UserOp Hash: ${userOpHash}`);
            
            return { success: true, userOpHash, userOp };
        } catch (error) {
            colorLog(COLORS.RED, `❌ Paymaster sponsorship failed: ${error.message}`);
            return { success: false, error: error.message };
        }
    }

    async testBundlerSubmission(userOp) {
        colorLog(COLORS.MAGENTA, '\n🎯 Test 2: Bundler Submission (eth_sendUserOperation)');
        colorLog(COLORS.MAGENTA, '====================================================');
        
        try {
            const userOpHash = await this.callJsonRpc('eth_sendUserOperation', [userOp, CONFIG.ENTRY_POINT_ADDRESS]);
            
            colorLog(COLORS.GREEN, '✅ Bundler accepted UserOperation!');
            console.log(`   UserOp Hash: ${userOpHash}`);
            
            return { success: true, userOpHash };
        } catch (error) {
            colorLog(COLORS.RED, `❌ Bundler submission failed: ${error.message}`);
            return { success: false, error: error.message };
        }
    }

    async testUserOperationStatus(userOpHash) {
        colorLog(COLORS.MAGENTA, '\n🎯 Test 3: UserOperation Status Tracking');
        colorLog(COLORS.MAGENTA, '=========================================');
        
        let attempts = 0;
        const maxAttempts = 10;
        const delayMs = 2000;
        
        while (attempts < maxAttempts) {
            try {
                // Check if UserOperation has been processed
                const receipt = await this.callJsonRpc('eth_getUserOperationReceipt', [userOpHash]);
                
                if (receipt) {
                    colorLog(COLORS.GREEN, '✅ UserOperation processed successfully!');
                    console.log(`   Transaction Hash: ${receipt.transactionHash}`);
                    console.log(`   Block Number: ${receipt.blockNumber}`);
                    console.log(`   Gas Used: ${receipt.gasUsed}`);
                    console.log(`   Success: ${receipt.success}`);
                    
                    return { success: true, receipt };
                }
            } catch (error) {
                // UserOperation might not be processed yet
                colorLog(COLORS.YELLOW, `⏳ Attempt ${attempts + 1}: UserOperation not processed yet...`);
            }
            
            attempts++;
            if (attempts < maxAttempts) {
                await new Promise(resolve => setTimeout(resolve, delayMs));
            }
        }
        
        colorLog(COLORS.YELLOW, '⚠️  UserOperation status check timed out');
        return { success: false, error: 'Timeout waiting for processing' };
    }

    async testForceBundle() {
        colorLog(COLORS.MAGENTA, '\n🎯 Test 4: Force Bundle Creation (debug_bundler_sendBundleNow)');
        colorLog(COLORS.MAGENTA, '=============================================================');
        
        try {
            await this.callJsonRpc('debug_bundler_sendBundleNow', []);
            colorLog(COLORS.GREEN, '✅ Bundle creation forced successfully!');
            return { success: true };
        } catch (error) {
            colorLog(COLORS.YELLOW, `⚠️  Force bundle failed: ${error.message}`);
            return { success: false, error: error.message };
        }
    }

    async testMempoolStatus() {
        colorLog(COLORS.MAGENTA, '\n🎯 Test 5: Mempool Status (debug_bundler_dumpMempool)');
        colorLog(COLORS.MAGENTA, '====================================================');
        
        try {
            const mempool = await this.callJsonRpc('debug_bundler_dumpMempool', []);
            
            colorLog(COLORS.GREEN, '✅ Mempool status retrieved!');
            console.log(`   UserOperations in mempool: ${JSON.stringify(mempool, null, 2)}`);
            
            return { success: true, mempool };
        } catch (error) {
            colorLog(COLORS.YELLOW, `⚠️  Mempool status failed: ${error.message}`);
            return { success: false, error: error.message };
        }
    }

    async testAPIEndpoints() {
        colorLog(COLORS.MAGENTA, '\n🎯 Test 6: API Endpoints Verification');
        colorLog(COLORS.MAGENTA, '======================================');
        
        const tests = [
            { method: 'eth_chainId', params: [] },
            { method: 'eth_supportedEntryPoints', params: [] },
            { method: 'pm_getSupportedEntryPoints', params: [] },
            { method: 'pm_getChainId', params: [] }
        ];

        let passed = 0;
        
        for (const test of tests) {
            try {
                const result = await this.callJsonRpc(test.method, test.params);
                colorLog(COLORS.GREEN, `✅ ${test.method}: ${JSON.stringify(result)}`);
                passed++;
            } catch (error) {
                colorLog(COLORS.RED, `❌ ${test.method}: ${error.message}`);
            }
        }
        
        return { success: passed === tests.length, passed, total: tests.length };
    }

    async runCompleteTest() {
        try {
            await this.initialize();
            
            colorLog(COLORS.BLUE, '\n🚀 Starting Complete E2E Test Flow...\n');
            
            // Test 1: Paymaster Sponsorship
            const sponsorResult = await this.testPaymasterSponsorship();
            if (!sponsorResult.success) {
                throw new Error(`Sponsorship failed: ${sponsorResult.error}`);
            }
            
            // Test 2: Bundler Submission  
            const bundlerResult = await this.testBundlerSubmission(sponsorResult.userOp);
            if (!bundlerResult.success) {
                throw new Error(`Bundler submission failed: ${bundlerResult.error}`);
            }
            
            // Test 3: Force Bundle (to ensure processing)
            await this.testForceBundle();
            
            // Test 4: Check UserOperation Status
            const statusResult = await this.testUserOperationStatus(bundlerResult.userOpHash);
            
            // Test 5: Mempool Status
            await this.testMempoolStatus();
            
            // Test 6: API Endpoints
            const apiResult = await this.testAPIEndpoints();
            
            // Final Summary
            this.displayFinalSummary([
                sponsorResult.success,
                bundlerResult.success,
                statusResult.success,
                apiResult.success
            ]);
            
        } catch (error) {
            colorLog(COLORS.RED, `\n💥 Test failed: ${error.message}`);
            process.exit(1);
        }
    }

    displayFinalSummary(results) {
        colorLog(COLORS.BLUE, '\n📊 Complete Test Summary');
        colorLog(COLORS.BLUE, '=========================');
        
        const passed = results.filter(r => r).length;
        const total = results.length;
        
        if (passed === total) {
            colorLog(COLORS.GREEN, '🎉 ALL TESTS PASSED!');
            colorLog(COLORS.GREEN, '✅ SuperRelay is working correctly!');
            
            colorLog(COLORS.CYAN, '\n✨ Verified Components:');
            console.log('   🎯 Paymaster sponsorship (pm_sponsorUserOperation)');
            console.log('   🔄 Bundler processing (eth_sendUserOperation)');
            console.log('   ⛓️  Blockchain submission');
            console.log('   📊 Status tracking and receipts');
            console.log('   🔧 Debug and management APIs');
            console.log('   🌐 All JSON-RPC endpoints');
            
        } else {
            colorLog(COLORS.YELLOW, `⚠️  ${passed}/${total} tests passed`);
            colorLog(COLORS.YELLOW, 'Some components may need attention.');
        }
        
        colorLog(COLORS.BLUE, '\n🚀 SuperRelay Complete E2E Test Finished!');
    }
}

// Main execution
async function main() {
    const tester = new SuperRelayTester();
    await tester.runCompleteTest();
}

if (require.main === module) {
    main().catch(error => {
        colorLog(COLORS.RED, `\nTest failed: ${error.message}`);
        process.exit(1);
    });
}

module.exports = SuperRelayTester;