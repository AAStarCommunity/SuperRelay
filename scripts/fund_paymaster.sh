#!/bin/bash

# Enhanced SuperPaymaster Account Funding Management
# Automated EntryPoint deposit and balance monitoring

set -e

# Configuration
ANVIL_URL="http://localhost:8545"
FUNDER_PRIVATE_KEY="0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"  # First anvil account
PAYMASTER_ADDRESS="0x70997970C51812dc3A010C7d01b50e0d17dc79C8"  # Second anvil account
ENTRYPOINT_ADDRESS="0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789"  # Standard EntryPoint v0.6 address
MIN_BALANCE_ETH="1.0"
MIN_ENTRYPOINT_DEPOSIT_ETH="0.5"
TARGET_ENTRYPOINT_DEPOSIT_ETH="2.0"
FUNDING_AMOUNT_ETH="5.0"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Get EntryPoint address
get_entrypoint_address() {
    if [ -f "../.entrypoint_address" ]; then
        cat ../.entrypoint_address
    elif [ -f ".entrypoint_address" ]; then
        cat .entrypoint_address
    else
        # Default EntryPoint v0.6 address (standard deployed)
        echo "0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789"
    fi
}

ENTRYPOINT_ADDRESS=$(get_entrypoint_address)

# Convert ETH to Wei (18 decimals)
eth_to_wei() {
    local eth_amount="$1"
    python3 -c "print(int(float('${eth_amount}') * 10**18))"
}

# Convert Wei to ETH
wei_to_eth() {
    local wei_amount="$1"
    # Clean the input - remove whitespace
    wei_amount=$(echo "$wei_amount" | tr -d ' \n\r\t')

    # Handle empty or invalid values
    if [ -z "$wei_amount" ] || [ "$wei_amount" = "0" ]; then
        echo "0.0"
        return
    fi

    # Convert decimal wei to ETH
    python3 -c "
import sys
try:
    val = '${wei_amount}'.strip()
    # If it's hex, convert to decimal first
    if val.startswith('0x'):
        val = int(val, 16)
    else:
        val = int(val)
    result = float(val) / 10**18
    print(f'{result:.6f}')
except Exception as e:
    print('0.0')
" 2>/dev/null || echo "0.0"
}

# Check balance
check_balance() {
    local address="$1"
    # cast balance returns decimal wei value, not hex
    local balance_wei=$(cast balance "$address" --rpc-url "$ANVIL_URL" 2>&1 | grep -E '^[0-9]+$' | head -1 || echo "0")
    local balance_eth=$(wei_to_eth "$balance_wei")
    echo "$balance_eth"
}

# Check EntryPoint deposit
check_entrypoint_deposit() {
    local paymaster="$1"
    local entrypoint="$2"

    # Call deposits(address) on EntryPoint contract
    local deposit_output=$(cast call "$entrypoint" "deposits(address)(uint256)" "$paymaster" --rpc-url "$ANVIL_URL" 2>&1)
    # Extract the decimal value (last line, first field)
    local deposit_wei=$(echo "$deposit_output" | tail -1 | awk '{print $1}' | grep -E '^[0-9]+$' || echo "0")
    local deposit_eth=$(wei_to_eth "$deposit_wei")
    echo "$deposit_eth"
}

# Fund account with ETH
fund_account() {
    local target_address="$1"
    local amount_eth="$2"
    local amount_wei=$(eth_to_wei "$amount_eth")

    echo -e "${BLUE}💰 Funding $target_address with $amount_eth ETH...${NC}"

    cast send --private-key "$FUNDER_PRIVATE_KEY" \
        --rpc-url "$ANVIL_URL" \
        --value "$amount_wei" \
        "$target_address" \
        > /dev/null 2>&1

    if [ $? -eq 0 ]; then
        echo -e "${GREEN}✅ Successfully funded $target_address${NC}"
        return 0
    else
        echo -e "${RED}❌ Failed to fund $target_address${NC}"
        return 1
    fi
}

# Deposit to EntryPoint
deposit_to_entrypoint() {
    local paymaster="$1"
    local entrypoint="$2"
    local amount_eth="$3"
    local amount_wei=$(eth_to_wei "$amount_eth")

    echo -e "${BLUE}🏦 Depositing $amount_eth ETH to EntryPoint for paymaster...${NC}"

    # First fund the paymaster account if needed
    local current_balance=$(check_balance "$paymaster")
    local required_balance=$(python3 -c "print(float('${amount_eth}') + 0.1)")  # Add 0.1 ETH for gas

    if [ "$(python3 -c "print(1 if float('${current_balance}') < float('${required_balance}') else 0)")" = "1" ]; then
        echo -e "${YELLOW}⚠️  Paymaster needs more ETH for deposit. Funding first...${NC}"
        fund_account "$paymaster" "$FUNDING_AMOUNT_ETH"
    fi

    # Deposit to EntryPoint using depositTo(address)
    cast send --private-key "$FUNDER_PRIVATE_KEY" \
        --rpc-url "$ANVIL_URL" \
        --value "$amount_wei" \
        "$entrypoint" \
        "depositTo(address)" \
        "$paymaster" \
        > /dev/null 2>&1

    if [ $? -eq 0 ]; then
        echo -e "${GREEN}✅ Successfully deposited to EntryPoint${NC}"
        return 0
    else
        echo -e "${RED}❌ Failed to deposit to EntryPoint${NC}"
        return 1
    fi
}

# Auto-rebalance function
auto_rebalance() {
    local paymaster="$1"
    local entrypoint="$2"

    echo -e "${BLUE}🔄 Auto-rebalancing paymaster accounts...${NC}"

    # Check current balances
    local balance_eth=$(check_balance "$paymaster")
    local deposit_eth=$(check_entrypoint_deposit "$paymaster" "$entrypoint")

    echo -e "📊 Current Status:"
    echo -e "   💰 Paymaster Balance: ${balance_eth} ETH"
    echo -e "   🏦 EntryPoint Deposit: ${deposit_eth} ETH"

    local needs_rebalance=false

    # Check if balance is too low
    if [ "$(python3 -c "print(1 if float('${balance_eth}') < float('${MIN_BALANCE_ETH}') else 0)")" = "1" ]; then
        echo -e "${YELLOW}⚠️  Paymaster balance below minimum (${MIN_BALANCE_ETH} ETH)${NC}"
        fund_account "$paymaster" "$FUNDING_AMOUNT_ETH"
        needs_rebalance=true
    fi

    # Check if EntryPoint deposit is too low
    if [ "$(python3 -c "print(1 if float('${deposit_eth}') < float('${MIN_ENTRYPOINT_DEPOSIT_ETH}') else 0)")" = "1" ]; then
        echo -e "${YELLOW}⚠️  EntryPoint deposit below minimum (${MIN_ENTRYPOINT_DEPOSIT_ETH} ETH)${NC}"
        deposit_to_entrypoint "$paymaster" "$entrypoint" "$TARGET_ENTRYPOINT_DEPOSIT_ETH"
        needs_rebalance=true
    fi

    if [ "$needs_rebalance" = false ]; then
        echo -e "${GREEN}✅ All balances are sufficient${NC}"
    fi

    return 0
}

# Monitor mode - continuous monitoring
monitor_mode() {
    local interval_seconds="${1:-60}"  # Default 60 seconds

    echo -e "${BLUE}👁️  Starting continuous monitoring (checking every ${interval_seconds}s)${NC}"
    echo -e "${YELLOW}Press Ctrl+C to stop monitoring${NC}"

    while true; do
        echo -e "\n${BLUE}[$(date)] Checking balances...${NC}"
        auto_rebalance "$PAYMASTER_ADDRESS" "$ENTRYPOINT_ADDRESS"
        sleep "$interval_seconds"
    done
}

# Status report
status_report() {
    echo -e "${BLUE}📋 SuperPaymaster Financial Status Report${NC}"
    echo -e "==========================================="

    # Check if Anvil is running
    if ! curl -s -X POST -H "Content-Type: application/json" \
        --data '{"jsonrpc":"2.0","method":"eth_chainId","params":[],"id":1}' \
        "$ANVIL_URL" > /dev/null 2>&1; then
        echo -e "\n${RED}❌ Error: Anvil is not running on $ANVIL_URL${NC}"
        echo -e "${YELLOW}💡 Please start Anvil first:${NC}"
        echo -e "   anvil --host 0.0.0.0 --port 8545 --accounts 10 --balance 10000"
        return 1
    fi

    # Network info
    echo -e "\n${BLUE}🌐 Network Information:${NC}"
    echo -e "   RPC URL: $ANVIL_URL"
    echo -e "   EntryPoint: $ENTRYPOINT_ADDRESS"
    echo -e "   Paymaster: $PAYMASTER_ADDRESS"

    # Balances
    echo -e "\n${BLUE}💰 Account Balances:${NC}"
    local funder_balance=$(check_balance "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266")
    local paymaster_balance=$(check_balance "$PAYMASTER_ADDRESS")
    local entrypoint_deposit=$(check_entrypoint_deposit "$PAYMASTER_ADDRESS" "$ENTRYPOINT_ADDRESS")

    echo -e "   🏦 Funder Account: ${funder_balance} ETH"
    echo -e "   💳 Paymaster Account: ${paymaster_balance} ETH"
    echo -e "   🏛️  EntryPoint Deposit: ${entrypoint_deposit} ETH"

    # Health check
    echo -e "\n${BLUE}🏥 Health Status:${NC}"
    local health_ok=true

    if [ "$(python3 -c "print(1 if float('${paymaster_balance}') >= float('${MIN_BALANCE_ETH}') else 0)")" = "1" ]; then
        echo -e "   ✅ Paymaster balance: OK (>= ${MIN_BALANCE_ETH} ETH)"
    else
        echo -e "   ❌ Paymaster balance: LOW (< ${MIN_BALANCE_ETH} ETH)"
        health_ok=false
    fi

    if [ "$(python3 -c "print(1 if float('${entrypoint_deposit}') >= float('${MIN_ENTRYPOINT_DEPOSIT_ETH}') else 0)")" = "1" ]; then
        echo -e "   ✅ EntryPoint deposit: OK (>= ${MIN_ENTRYPOINT_DEPOSIT_ETH} ETH)"
    else
        echo -e "   ❌ EntryPoint deposit: LOW (< ${MIN_ENTRYPOINT_DEPOSIT_ETH} ETH)"
        health_ok=false
    fi

    echo -e "\n${BLUE}📊 Overall Status:${NC}"
    if [ "$health_ok" = true ]; then
        echo -e "   ${GREEN}🟢 HEALTHY - All balances sufficient${NC}"
    else
        echo -e "   ${RED}🔴 ATTENTION NEEDED - Some balances are low${NC}"
        echo -e "   💡 Run: $0 auto-rebalance"
    fi
}

# Usage function
usage() {
    echo "Usage: $0 [command] [options]"
    echo ""
    echo "Commands:"
    echo "  status                    Show financial status report"
    echo "  fund <address> <amount>   Fund an address with ETH"
    echo "  deposit <amount>          Deposit ETH to EntryPoint for paymaster"
    echo "  auto-rebalance           Auto-rebalance all accounts"
    echo "  monitor [interval]       Start continuous monitoring (default: 60s)"
    echo "  emergency-fund           Emergency funding for all accounts"
    echo ""
    echo "Examples:"
    echo "  $0 status"
    echo "  $0 fund 0x123... 1.5"
    echo "  $0 deposit 2.0"
    echo "  $0 auto-rebalance"
    echo "  $0 monitor 30"
}

# Emergency funding
emergency_fund() {
    echo -e "${RED}🚨 EMERGENCY FUNDING MODE${NC}"
    echo -e "=========================="

    echo -e "${YELLOW}💰 Funding paymaster with ${FUNDING_AMOUNT_ETH} ETH...${NC}"
    fund_account "$PAYMASTER_ADDRESS" "$FUNDING_AMOUNT_ETH"

    echo -e "${YELLOW}🏦 Depositing ${TARGET_ENTRYPOINT_DEPOSIT_ETH} ETH to EntryPoint...${NC}"
    deposit_to_entrypoint "$PAYMASTER_ADDRESS" "$ENTRYPOINT_ADDRESS" "$TARGET_ENTRYPOINT_DEPOSIT_ETH"

    echo -e "${GREEN}🎯 Emergency funding complete!${NC}"
    status_report
}

# Main function
main() {
    local command="${1:-status}"

    case "$command" in
        "status")
            status_report
            ;;
        "fund")
            if [ $# -lt 3 ]; then
                echo "Usage: $0 fund <address> <amount_eth>"
                exit 1
            fi
            fund_account "$2" "$3"
            ;;
        "deposit")
            if [ $# -lt 2 ]; then
                echo "Usage: $0 deposit <amount_eth>"
                exit 1
            fi
            deposit_to_entrypoint "$PAYMASTER_ADDRESS" "$ENTRYPOINT_ADDRESS" "$2"
            ;;
        "auto-rebalance")
            auto_rebalance "$PAYMASTER_ADDRESS" "$ENTRYPOINT_ADDRESS"
            ;;
        "monitor")
            local interval="${2:-60}"
            monitor_mode "$interval"
            ;;
        "emergency-fund")
            emergency_fund
            ;;
        "help"|"-h"|"--help")
            usage
            ;;
        *)
            echo "Unknown command: $command"
            usage
            exit 1
            ;;
    esac
}

# Check dependencies
check_dependencies() {
    if ! command -v cast >/dev/null 2>&1; then
        echo -e "${RED}❌ Error: 'cast' command not found. Please install Foundry.${NC}"
        echo "Install with: curl -L https://foundry.paradigm.xyz | bash && foundryup"
        exit 1
    fi

    if ! command -v python3 >/dev/null 2>&1; then
        echo -e "${RED}❌ Error: 'python3' command not found.${NC}"
        exit 1
    fi
}

# Initialize
check_dependencies

# Run main function
main "$@"