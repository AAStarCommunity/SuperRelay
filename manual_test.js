#!/usr/bin/env node

const { ethers } = require('ethers');
const axios = require('axios');

const SUPER_RELAY_URL = 'http://localhost:3000';
const RPC_URL = 'http://localhost:8545';

console.log('🧪 Manual SuperRelay Test - REAL Data Collection');
console.log('================================================\n');

async function runManualTest() {
    try {
        // 1. Test SuperRelay Health
        console.log('1️⃣ SuperRelay Health Check:');
        const health = await axios.get(`${SUPER_RELAY_URL}/health`).catch(() => null);
        console.log(`   Status: ${health ? health.data : 'NOT RESPONDING'}\n`);

        // 2. Test EntryPoint Support
        console.log('2️⃣ EntryPoint Support Check:');
        const entryPoints = await axios.post(SUPER_RELAY_URL, {
            jsonrpc: '2.0',
            id: 1,
            method: 'eth_supportedEntryPoints',
            params: []
        }).catch(e => ({ data: { error: e.message } }));
        console.log('   Response:', JSON.stringify(entryPoints.data, null, 2));
        
        // 3. Test Chain Connection
        console.log('\n3️⃣ Chain Connection:');
        const chainId = await axios.post(RPC_URL, {
            jsonrpc: '2.0',
            id: 1,
            method: 'eth_chainId',
            params: []
        }).catch(e => ({ data: { error: e.message } }));
        console.log('   Chain ID:', chainId.data);

        // 4. Test Account Balance
        console.log('\n4️⃣ Test Account Balance:');
        const balance = await axios.post(RPC_URL, {
            jsonrpc: '2.0',
            id: 1,
            method: 'eth_getBalance',
            params: ['0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266', 'latest']
        }).catch(e => ({ data: { error: e.message } }));
        console.log('   Balance (Wei):', balance.data);
        if (balance.data.result) {
            const ethBalance = ethers.formatEther(balance.data.result);
            console.log(`   Balance (ETH): ${ethBalance}`);
        }

        // 5. Create Sample UserOperation
        console.log('\n5️⃣ Sample UserOperation Structure:');
        const sampleUserOp = {
            sender: '0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266',
            nonce: '0x0',
            initCode: '0x',
            callData: '0x',
            callGasLimit: '0x5208',
            verificationGasLimit: '0x5208', 
            preVerificationGas: '0x5208',
            maxFeePerGas: '0x3b9aca00',
            maxPriorityFeePerGas: '0x3b9aca00',
            paymasterAndData: '0x',
            signature: '0x'
        };
        console.log('   Structure:', JSON.stringify(sampleUserOp, null, 2));

        // 6. Test Available API Methods
        console.log('\n6️⃣ Available API Methods Test:');
        const methods = ['pm_getChainId', 'pm_getSupportedEntryPoints', 'eth_chainId'];
        
        for (const method of methods) {
            const response = await axios.post(SUPER_RELAY_URL, {
                jsonrpc: '2.0',
                id: 1,
                method: method,
                params: []
            }).catch(e => ({ data: { error: e.response?.data?.error || e.message } }));
            
            console.log(`   ${method}:`, response.data.result || response.data.error);
        }
        
    } catch (error) {
        console.error('❌ Test failed:', error.message);
    }
}

runManualTest();