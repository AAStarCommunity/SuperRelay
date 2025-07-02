#!/bin/bash

# Paymaster Funding Management Script
# Ensures paymaster has sufficient balance in EntryPoint contract

set -e

# Configuration
PAYMASTER_PRIVATE_KEY="0x59c6995e998f97a5a0044966f0945389dc9e86dae88c6a2440f60b6c4b9f78c2"
PAYMASTER_ADDRESS="0x70997970C51812dc3A010C7d01b50e0d17dc79C8"
FUNDER_PRIVATE_KEY="0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
RPC_URL="http://localhost:8545"

echo "💰 Paymaster Funding Management"
echo "================================"

# Function to get EntryPoint address
get_entrypoint_address() {
    if [ -f ".entrypoint_address" ]; then
        cat .entrypoint_address
    else
        echo "❌ EntryPoint address file not found. Please deploy EntryPoint first."
        exit 1
    fi
}

# Function to check paymaster account balance
check_paymaster_balance() {
    local balance=$(cast balance $PAYMASTER_ADDRESS --rpc-url $RPC_URL)
    echo "📊 Paymaster account balance: $(cast --to-dec $balance | awk '{printf "%.2f", $1/1e18}') ETH"
    
    # If balance is less than 10 ETH, fund the account
    if [ $(echo "$balance < 10000000000000000000" | python3 -c "import sys; print(int(float(input()) < 10000000000000000000))") -eq 1 ]; then
        echo "💸 Funding paymaster account..."
        cast send --private-key $FUNDER_PRIVATE_KEY \
            --rpc-url $RPC_URL \
            --value 50ether \
            $PAYMASTER_ADDRESS > /dev/null
        echo "✅ Paymaster account funded with 50 ETH"
    fi
}

# Function to check EntryPoint deposit
check_entrypoint_deposit() {
    local entrypoint=$1
    echo "🔍 Checking EntryPoint deposit for paymaster..."
    
    # Check if EntryPoint contract exists
    local code=$(cast code $entrypoint --rpc-url $RPC_URL)
    if [ "$code" = "0x" ]; then
        echo "❌ EntryPoint contract not found at $entrypoint"
        return 1
    fi
    
    # Get current deposit (using balanceOf function)
    local deposit=$(cast call $entrypoint "balanceOf(address)" $PAYMASTER_ADDRESS --rpc-url $RPC_URL 2>/dev/null || echo "0x0")
    local deposit_decimal=$(cast --to-dec $deposit 2>/dev/null || echo "0")
    
    echo "📊 Current EntryPoint deposit: $(echo "$deposit_decimal" | awk '{printf "%.2f", $1/1e18}') ETH"
    
    # Check if deposit is sufficient (2 ETH minimum)
    if [ $(echo "$deposit_decimal < 2000000000000000000" | python3 -c "import sys; print(int(float(input()) < 2000000000000000000))") -eq 1 ]; then
        echo "💰 Depositing funds to EntryPoint..."
        
        # Deposit to EntryPoint using depositTo function
        cast send --private-key $PAYMASTER_PRIVATE_KEY \
            --rpc-url $RPC_URL \
            --value 5ether \
            --gas-limit 200000 \
            $entrypoint \
            "depositTo(address)" \
            $PAYMASTER_ADDRESS
            
        echo "✅ Deposited 5 ETH to EntryPoint"
        
        # Verify deposit
        sleep 2
        local new_deposit=$(cast call $entrypoint "balanceOf(address)" $PAYMASTER_ADDRESS --rpc-url $RPC_URL)
        local new_deposit_decimal=$(cast --to-dec $new_deposit)
        echo "📊 New EntryPoint deposit: $(echo "$new_deposit_decimal" | awk '{printf "%.2f", $1/1e18}') ETH"
    else
        echo "✅ EntryPoint deposit is sufficient"
    fi
}

# Function to show deposit info
show_deposit_info() {
    local entrypoint=$1
    echo ""
    echo "📊 Paymaster Financial Status"
    echo "============================="
    
    local account_balance=$(cast balance $PAYMASTER_ADDRESS --rpc-url $RPC_URL)
    echo "🏦 Account Balance: $(cast --to-dec $account_balance | awk '{printf "%.2f", $1/1e18}') ETH"
    
    local deposit=$(cast call $entrypoint "balanceOf(address)" $PAYMASTER_ADDRESS --rpc-url $RPC_URL 2>/dev/null || echo "0x0")
    local deposit_decimal=$(cast --to-dec $deposit 2>/dev/null || echo "0")
    echo "💳 EntryPoint Deposit: $(echo "$deposit_decimal" | awk '{printf "%.2f", $1/1e18}') ETH"
    
    echo "📍 EntryPoint Address: $entrypoint"
    echo "🔑 Paymaster Address: $PAYMASTER_ADDRESS"
}

# Main execution
main() {
    echo "🚀 Starting paymaster funding check..."
    
    # Get EntryPoint address
    local entrypoint=$(get_entrypoint_address)
    echo "📍 Using EntryPoint: $entrypoint"
    
    # Check and fund paymaster account if needed
    check_paymaster_balance
    
    # Check and fund EntryPoint deposit if needed
    check_entrypoint_deposit $entrypoint
    
    # Show final status
    show_deposit_info $entrypoint
    
    echo ""
    echo "🎉 Paymaster funding management complete!"
}

# Run main function
main "$@" 