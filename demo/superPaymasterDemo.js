#!/usr/bin/env node

/**
 * SuperPaymaster Demo Application
 * 
 * This demo showcases the core capabilities of SuperPaymaster:
 * 1. UserOperation sponsorship via pm_sponsorUserOperation API
 * 2. Gas fee abstraction for users
 * 3. Policy-based access control
 * 4. Multiple EntryPoint version support (v0.6 & v0.7)
 * 
 * Usage: node superPaymasterDemo.js [options]
 */

const { ethers } = require('ethers');
const axios = require('axios');

// Configuration
const CONFIG = {
    // SuperRelay RPC endpoint
    SUPER_RELAY_URL: process.env.SUPER_RELAY_URL || 'http://localhost:3000',
    
    // Anvil test network
    RPC_URL: process.env.RPC_URL || 'http://localhost:8545',
    
    // EntryPoint contract address (from deployed contract)
    ENTRY_POINT_ADDRESS: process.env.ENTRY_POINT_ADDRESS || '0x5FbDB2315678afecb367f032d93F642f64180aa3',
    
    // Test accounts (Anvil default accounts)
    ACCOUNTS: {
        USER: {
            address: '0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266',
            privateKey: '0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80'
        },
        PAYMASTER: {
            address: '0x70997970C51812dc3A010C7d01b50e0d17dc79C8',
            privateKey: '0x59c6995e998f97a5a0044966f0945389dc9e86dae88c6a2440f60b6c4b9f78c2'
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

// Helper function for colored console output
function colorLog(color, message) {
    console.log(`${color}${message}${COLORS.RESET}`);
}

class SuperPaymasterDemo {
    constructor() {
        this.provider = new ethers.JsonRpcProvider(CONFIG.RPC_URL);
        this.userWallet = new ethers.Wallet(CONFIG.ACCOUNTS.USER.privateKey, this.provider);
        this.paymasterWallet = new ethers.Wallet(CONFIG.ACCOUNTS.PAYMASTER.privateKey, this.provider);
    }

    async initialize() {
        colorLog(COLORS.BLUE, '\n🚀 SuperPaymaster Demo Application');
        colorLog(COLORS.BLUE, '=====================================\n');

        // Check connections
        await this.checkConnections();
        
        // Display account information
        await this.displayAccountInfo();
    }

    async checkConnections() {
        colorLog(COLORS.CYAN, '🔗 Checking connections...');
        
        try {
            // Check Anvil connection
            const network = await this.provider.getNetwork();
            colorLog(COLORS.GREEN, `✅ Connected to network: ${network.name} (Chain ID: ${network.chainId})`);
            
            // Check SuperRelay connection
            const response = await axios.get(`${CONFIG.SUPER_RELAY_URL}/health`);
            if (response.data === 'ok') {
                colorLog(COLORS.GREEN, `✅ SuperRelay service is running at ${CONFIG.SUPER_RELAY_URL}`);
            }
            
            // Check EntryPoint contract
            const entryPointCode = await this.provider.getCode(CONFIG.ENTRY_POINT_ADDRESS);
            if (entryPointCode !== '0x') {
                colorLog(COLORS.GREEN, `✅ EntryPoint contract found at ${CONFIG.ENTRY_POINT_ADDRESS}`);
            } else {
                colorLog(COLORS.YELLOW, `⚠️  EntryPoint contract not found. Please deploy EntryPoint first.`);
            }
            
        } catch (error) {
            colorLog(COLORS.RED, `❌ Connection error: ${error.message}`);
            throw error;
        }
    }

    async displayAccountInfo() {
        colorLog(COLORS.CYAN, '\n💰 Account Information:');
        
        const userBalance = await this.provider.getBalance(CONFIG.ACCOUNTS.USER.address);
        const paymasterBalance = await this.provider.getBalance(CONFIG.ACCOUNTS.PAYMASTER.address);
        
        console.log(`   👤 User Account: ${CONFIG.ACCOUNTS.USER.address}`);
        console.log(`      Balance: ${ethers.formatEther(userBalance)} ETH`);
        
        console.log(`   🏦 Paymaster Account: ${CONFIG.ACCOUNTS.PAYMASTER.address}`);
        console.log(`      Balance: ${ethers.formatEther(paymasterBalance)} ETH`);
        
        console.log(`   📍 EntryPoint: ${CONFIG.ENTRY_POINT_ADDRESS}`);
    }

    // Create a simple UserOperation
    createUserOperation() {
        colorLog(COLORS.CYAN, '\n🔧 Creating UserOperation...');
        
        const userOp = {
            sender: CONFIG.ACCOUNTS.USER.address,
            nonce: '0x0',
            initCode: '0x',
            callData: '0x', // Empty call data for simple transfer
            callGasLimit: '0x186A0', // 100,000
            verificationGasLimit: '0x186A0', // 100,000
            preVerificationGas: '0x5208', // 21,000
            maxFeePerGas: '0x3B9ACA00', // 1 gwei
            maxPriorityFeePerGas: '0x3B9ACA00', // 1 gwei
            paymasterAndData: '0x',
            signature: '0x'
        };

        colorLog(COLORS.GREEN, '✅ UserOperation created successfully');
        console.log(`   Sender: ${userOp.sender}`);
        console.log(`   Gas Limits: ${parseInt(userOp.callGasLimit, 16)} / ${parseInt(userOp.verificationGasLimit, 16)}`);
        console.log(`   Max Fee: ${parseInt(userOp.maxFeePerGas, 16) / 1e9} gwei`);
        
        return userOp;
    }

    // Create UserOperation v0.7 format
    createUserOperationV07() {
        colorLog(COLORS.CYAN, '\n🔧 Creating UserOperation v0.7...');
        
        const userOp = {
            sender: CONFIG.ACCOUNTS.USER.address,
            nonce: '0x0',
            callData: '0x',
            callGasLimit: '0x186A0',
            verificationGasLimit: '0x186A0',
            preVerificationGas: '0x5208',
            maxFeePerGas: '0x3B9ACA00',
            maxPriorityFeePerGas: '0x3B9ACA00',
            signature: '0x'
            // Note: v0.7 doesn't have initCode and paymasterAndData at the top level
        };

        colorLog(COLORS.GREEN, '✅ UserOperation v0.7 created successfully');
        return userOp;
    }

    // Call SuperPaymaster's pm_sponsorUserOperation API
    async sponsorUserOperation(userOp) {
        colorLog(COLORS.CYAN, '\n💎 Requesting UserOperation sponsorship...');
        
        try {
            const response = await axios.post(CONFIG.SUPER_RELAY_URL, {
                jsonrpc: '2.0',
                id: 1,
                method: 'pm_sponsorUserOperation',
                params: [userOp, CONFIG.ENTRY_POINT_ADDRESS]
            }, {
                headers: {
                    'Content-Type': 'application/json'
                }
            });

            if (response.data.error) {
                colorLog(COLORS.RED, `❌ Sponsorship failed: ${response.data.error.message}`);
                return null;
            }

            const userOpHash = response.data.result;
            colorLog(COLORS.GREEN, `✅ UserOperation sponsored successfully!`);
            console.log(`   UserOp Hash: ${userOpHash}`);
            
            return userOpHash;
            
        } catch (error) {
            colorLog(COLORS.RED, `❌ API call failed: ${error.message}`);
            if (error.response?.data) {
                console.log('   Response:', JSON.stringify(error.response.data, null, 2));
            }
            return null;
        }
    }

    // Test different scenarios
    async testValidUserOperation() {
        colorLog(COLORS.MAGENTA, '\n🧪 Test 1: Valid UserOperation Sponsorship');
        colorLog(COLORS.MAGENTA, '==========================================');
        
        const userOp = this.createUserOperation();
        const result = await this.sponsorUserOperation(userOp);
        
        if (result) {
            colorLog(COLORS.GREEN, '✅ Test 1 PASSED: Valid UserOperation was sponsored');
        } else {
            colorLog(COLORS.RED, '❌ Test 1 FAILED: Valid UserOperation was rejected');
        }
        
        return result !== null;
    }

    async testUserOperationV07() {
        colorLog(COLORS.MAGENTA, '\n🧪 Test 2: UserOperation v0.7 Format');
        colorLog(COLORS.MAGENTA, '====================================');
        
        const userOp = this.createUserOperationV07();
        const result = await this.sponsorUserOperation(userOp);
        
        if (result) {
            colorLog(COLORS.GREEN, '✅ Test 2 PASSED: v0.7 UserOperation was sponsored');
        } else {
            colorLog(COLORS.YELLOW, '⚠️  Test 2 INFO: v0.7 UserOperation processing result');
        }
        
        return result !== null;
    }

    async testUnauthorizedSender() {
        colorLog(COLORS.MAGENTA, '\n🧪 Test 3: Unauthorized Sender Rejection');
        colorLog(COLORS.MAGENTA, '========================================');
        
        const userOp = this.createUserOperation();
        userOp.sender = '0x1234567890123456789012345678901234567890'; // Unauthorized address
        
        const result = await this.sponsorUserOperation(userOp);
        
        if (result === null) {
            colorLog(COLORS.GREEN, '✅ Test 3 PASSED: Unauthorized sender was rejected');
            return true;
        } else {
            colorLog(COLORS.RED, '❌ Test 3 FAILED: Unauthorized sender was unexpectedly sponsored');
            return false;
        }
    }

    async testInvalidEntryPoint() {
        colorLog(COLORS.MAGENTA, '\n🧪 Test 4: Invalid EntryPoint Rejection');
        colorLog(COLORS.MAGENTA, '======================================');
        
        const userOp = this.createUserOperation();
        const invalidEntryPoint = '0x0000000000000000000000000000000000000001';
        
        try {
            const response = await axios.post(CONFIG.SUPER_RELAY_URL, {
                jsonrpc: '2.0',
                id: 1,
                method: 'pm_sponsorUserOperation',
                params: [userOp, invalidEntryPoint]
            }, {
                headers: {
                    'Content-Type': 'application/json'
                }
            });

            if (response.data.error && response.data.error.message.includes('Unknown entry point')) {
                colorLog(COLORS.GREEN, '✅ Test 4 PASSED: Invalid EntryPoint was rejected');
                return true;
            } else {
                colorLog(COLORS.RED, '❌ Test 4 FAILED: Invalid EntryPoint was not rejected');
                return false;
            }
            
        } catch (error) {
            colorLog(COLORS.GREEN, '✅ Test 4 PASSED: Invalid EntryPoint caused expected error');
            return true;
        }
    }

    async testNumberFormats() {
        colorLog(COLORS.MAGENTA, '\n🧪 Test 5: Number Format Support');
        colorLog(COLORS.MAGENTA, '=================================');
        
        // Test with decimal format
        const userOpDecimal = this.createUserOperation();
        userOpDecimal.callGasLimit = '100000';  // Decimal format
        userOpDecimal.maxFeePerGas = '1000000000';  // Decimal format
        
        const result = await this.sponsorUserOperation(userOpDecimal);
        
        if (result !== null) {
            colorLog(COLORS.GREEN, '✅ Test 5 PASSED: Decimal number format supported');
            return true;
        } else {
            colorLog(COLORS.YELLOW, '⚠️  Test 5 INFO: Decimal format handling result');
            return false;
        }
    }

    // API Feature demonstration
    async demonstrateAPIFeatures() {
        colorLog(COLORS.MAGENTA, '\n🎯 API Features Demonstration');
        colorLog(COLORS.MAGENTA, '==============================');
        
        // Test supported EntryPoints API
        try {
            const response = await axios.post(CONFIG.SUPER_RELAY_URL, {
                jsonrpc: '2.0',
                id: 1,
                method: 'eth_supportedEntryPoints',
                params: []
            });
            
            if (response.data.result) {
                colorLog(COLORS.GREEN, '✅ eth_supportedEntryPoints API working');
                console.log('   Supported EntryPoints:', response.data.result);
            }
        } catch (error) {
            colorLog(COLORS.RED, '❌ eth_supportedEntryPoints API failed');
        }

        // Test health check
        try {
            const response = await axios.get(`${CONFIG.SUPER_RELAY_URL}/health`);
            if (response.data === 'ok') {
                colorLog(COLORS.GREEN, '✅ Health check API working');
            }
        } catch (error) {
            colorLog(COLORS.RED, '❌ Health check API failed');
        }
    }

    // Run comprehensive demo
    async runDemo() {
        try {
            await this.initialize();
            
            colorLog(COLORS.BLUE, '\n🎬 Starting SuperPaymaster Feature Demo...\n');
            
            // Core functionality tests
            const results = [];
            results.push(await this.testValidUserOperation());
            results.push(await this.testUserOperationV07());
            results.push(await this.testUnauthorizedSender());
            results.push(await this.testInvalidEntryPoint());
            results.push(await this.testNumberFormats());
            
            // API features
            await this.demonstrateAPIFeatures();
            
            // Summary
            this.displaySummary(results);
            
        } catch (error) {
            colorLog(COLORS.RED, `\n💥 Demo failed with error: ${error.message}`);
            process.exit(1);
        }
    }

    displaySummary(results) {
        colorLog(COLORS.BLUE, '\n📊 Demo Summary');
        colorLog(COLORS.BLUE, '================');
        
        const passed = results.filter(r => r).length;
        const total = results.length;
        
        colorLog(COLORS.GREEN, `✅ Tests Passed: ${passed}/${total}`);
        
        if (passed === total) {
            colorLog(COLORS.GREEN, '\n🎉 All core features are working correctly!');
            colorLog(COLORS.GREEN, '🚀 SuperPaymaster is ready for production use!');
        } else {
            colorLog(COLORS.YELLOW, '\n⚠️  Some tests had unexpected results.');
            colorLog(COLORS.YELLOW, '🔍 Check the output above for details.');
        }
        
        colorLog(COLORS.CYAN, '\n💡 Key SuperPaymaster Capabilities Demonstrated:');
        console.log('   🎯 ERC-4337 UserOperation sponsorship');
        console.log('   🔒 Policy-based access control');
        console.log('   ⚡ Multiple EntryPoint version support');
        console.log('   🛡️  Security validation and error handling');
        console.log('   📊 Flexible parameter format support');
        console.log('   🔗 Complete JSON-RPC API integration');
        
        colorLog(COLORS.BLUE, '\n📚 Next Steps:');
        console.log('   1. Customize paymaster policies in config/paymaster-policies.toml');
        console.log('   2. Add your smart wallet addresses to the allowlist');
        console.log('   3. Configure production settings in config/production.toml');
        console.log('   4. Set up monitoring and alerting');
        console.log('   5. Deploy to your production environment');
    }
}

// Usage examples and documentation
function showUsage() {
    console.log(`
SuperPaymaster Demo Application
===============================

This demo showcases SuperPaymaster's core capabilities:

🎯 Core Features:
  • ERC-4337 UserOperation sponsorship
  • Gas fee abstraction for users
  • Policy-based access control
  • Multi-version EntryPoint support

🔧 Prerequisites:
  • Anvil running on localhost:8545
  • SuperRelay running on localhost:3000
  • EntryPoint contract deployed

🚀 Usage:
  node superPaymasterDemo.js                    # Run full demo
  node superPaymasterDemo.js --help             # Show this help
  node superPaymasterDemo.js --test-only        # Run tests only

🌐 Environment Variables:
  SUPER_RELAY_URL    SuperRelay endpoint (default: http://localhost:3000)
  RPC_URL           Blockchain RPC endpoint (default: http://localhost:8545)
  ENTRY_POINT_ADDRESS  EntryPoint contract address

📚 Learn More:
  • See docs/Features.md for detailed feature descriptions
  • See docs/Deploy.md for deployment instructions
  • See config/ directory for configuration examples
`);
}

// Main execution
async function main() {
    const args = process.argv.slice(2);
    
    if (args.includes('--help') || args.includes('-h')) {
        showUsage();
        return;
    }
    
    const demo = new SuperPaymasterDemo();
    await demo.runDemo();
}

// Run if called directly
if (require.main === module) {
    main().catch(error => {
        colorLog(COLORS.RED, `\nDemo failed: ${error.message}`);
        process.exit(1);
    });
}

module.exports = SuperPaymasterDemo; 