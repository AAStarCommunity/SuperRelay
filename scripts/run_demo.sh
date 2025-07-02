#!/bin/bash

# SuperPaymaster Demo Runner
# Automated demonstration of SuperPaymaster capabilities

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
MAGENTA='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Configuration
DEMO_DIR="demo"
DEMO_SCRIPT="superPaymasterDemo.js"

echo -e "${BLUE}🎭 SuperPaymaster Demo Runner${NC}"
echo -e "${BLUE}==============================${NC}"

# Function to check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Check prerequisites
check_prerequisites() {
    echo -e "\n${CYAN}🔍 Checking prerequisites...${NC}"
    
    # Check Node.js
    if command_exists node; then
        local node_version=$(node --version)
        echo -e "${GREEN}✅ Node.js: $node_version${NC}"
    else
        echo -e "${RED}❌ Node.js not found. Please install Node.js 16+${NC}"
        exit 1
    fi
    
    # Check npm
    if command_exists npm; then
        local npm_version=$(npm --version)
        echo -e "${GREEN}✅ npm: $npm_version${NC}"
    else
        echo -e "${RED}❌ npm not found. Please install npm${NC}"
        exit 1
    fi
    
    # Check if SuperRelay is running
    if curl -s http://localhost:3000/health >/dev/null 2>&1; then
        echo -e "${GREEN}✅ SuperRelay service is running${NC}"
    else
        echo -e "${YELLOW}⚠️  SuperRelay service not detected at localhost:3000${NC}"
        echo -e "${YELLOW}   Starting SuperRelay automatically...${NC}"
        if [ -f "scripts/restart_super_relay.sh" ]; then
            ./scripts/restart_super_relay.sh
        else
            echo -e "${RED}❌ SuperRelay not running and restart script not found${NC}"
            echo -e "${CYAN}💡 Please run: ./scripts/restart_super_relay.sh${NC}"
            exit 1
        fi
    fi
    
    # Check if Anvil is running
    if curl -s http://localhost:8545 >/dev/null 2>&1; then
        echo -e "${GREEN}✅ Anvil test network is running${NC}"
    else
        echo -e "${RED}❌ Anvil test network not running at localhost:8545${NC}"
        echo -e "${CYAN}💡 Please start Anvil: anvil${NC}"
        exit 1
    fi
}

# Setup demo environment
setup_demo() {
    echo -e "\n${CYAN}🔧 Setting up demo environment...${NC}"
    
    # Create demo directory if it doesn't exist
    if [ ! -d "$DEMO_DIR" ]; then
        mkdir -p "$DEMO_DIR"
        echo -e "${GREEN}✅ Created demo directory${NC}"
    fi
    
    # Install dependencies if package.json exists
    if [ -f "$DEMO_DIR/package.json" ]; then
        echo -e "${CYAN}📦 Installing demo dependencies...${NC}"
        cd "$DEMO_DIR"
        npm install --silent
        cd ..
        echo -e "${GREEN}✅ Dependencies installed${NC}"
    fi
    
    # Check if demo script exists
    if [ ! -f "$DEMO_DIR/$DEMO_SCRIPT" ]; then
        echo -e "${RED}❌ Demo script not found: $DEMO_DIR/$DEMO_SCRIPT${NC}"
        exit 1
    fi
}

# Run funding to ensure accounts are ready
ensure_funding() {
    echo -e "\n${CYAN}💰 Ensuring accounts are funded...${NC}"
    
    if [ -f "scripts/fund_paymaster.sh" ]; then
        ./scripts/fund_paymaster.sh auto-rebalance
        echo -e "${GREEN}✅ Account funding completed${NC}"
    else
        echo -e "${YELLOW}⚠️  Funding script not found, continuing with demo...${NC}"
    fi
}

# Run the demo
run_demo() {
    echo -e "\n${MAGENTA}🎬 Starting SuperPaymaster Demo...${NC}"
    echo -e "${MAGENTA}===================================${NC}"
    
    cd "$DEMO_DIR"
    
    # Run the demo with proper error handling
    if node "$DEMO_SCRIPT"; then
        echo -e "\n${GREEN}🎉 Demo completed successfully!${NC}"
        return 0
    else
        echo -e "\n${RED}❌ Demo encountered errors${NC}"
        return 1
    fi
}

# Display demo information
show_demo_info() {
    echo -e "\n${BLUE}📚 SuperPaymaster Demo Information${NC}"
    echo -e "${BLUE}===================================${NC}"
    
    echo -e "\n${CYAN}🎯 What this demo demonstrates:${NC}"
    echo "   • ERC-4337 UserOperation sponsorship"
    echo "   • Gas fee abstraction for users"
    echo "   • Policy-based access control"
    echo "   • Multiple EntryPoint version support (v0.6 & v0.7)"
    echo "   • JSON-RPC API integration"
    echo "   • Error handling and validation"
    
    echo -e "\n${CYAN}🧪 Test scenarios included:${NC}"
    echo "   1. Valid UserOperation sponsorship"
    echo "   2. UserOperation v0.7 format support"
    echo "   3. Unauthorized sender rejection"
    echo "   4. Invalid EntryPoint rejection"
    echo "   5. Number format flexibility"
    echo "   6. API feature demonstration"
    
    echo -e "\n${CYAN}🔧 System requirements:${NC}"
    echo "   • Node.js 16+ with npm"
    echo "   • Anvil test network running"
    echo "   • SuperRelay service running"
    echo "   • EntryPoint contract deployed"
    
    echo -e "\n${CYAN}📁 Demo files:${NC}"
    echo "   • demo/superPaymasterDemo.js - Main demo script"
    echo "   • demo/package.json - Dependencies"
    echo "   • scripts/run_demo.sh - This runner script"
}

# Interactive mode
interactive_mode() {
    echo -e "\n${CYAN}🎮 Interactive Demo Mode${NC}"
    echo -e "${CYAN}========================${NC}"
    
    while true; do
        echo -e "\n${YELLOW}What would you like to do?${NC}"
        echo "1. Run full demo"
        echo "2. Check system status"
        echo "3. Show demo information"
        echo "4. Fund accounts"
        echo "5. Run tests only"
        echo "6. Exit"
        
        read -p "Enter your choice (1-6): " choice
        
        case $choice in
            1)
                run_demo
                ;;
            2)
                check_prerequisites
                ;;
            3)
                show_demo_info
                ;;
            4)
                ensure_funding
                ;;
            5)
                cd "$DEMO_DIR"
                node "$DEMO_SCRIPT" --test-only
                cd ..
                ;;
            6)
                echo -e "${GREEN}👋 Goodbye!${NC}"
                exit 0
                ;;
            *)
                echo -e "${RED}❌ Invalid choice. Please enter 1-6.${NC}"
                ;;
        esac
    done
}

# Quick demo mode
quick_demo() {
    echo -e "\n${MAGENTA}⚡ Quick Demo Mode${NC}"
    echo -e "${MAGENTA}=================${NC}"
    
    # Run essential checks
    if ! command_exists node; then
        echo -e "${RED}❌ Node.js required but not found${NC}"
        exit 1
    fi
    
    # Quick setup
    setup_demo
    
    # Quick funding check
    if [ -f "scripts/fund_paymaster.sh" ]; then
        ./scripts/fund_paymaster.sh status
    fi
    
    # Run demo
    run_demo
}

# Usage information
usage() {
    echo "Usage: $0 [option]"
    echo ""
    echo "Options:"
    echo "  run               Run full demo with all checks"
    echo "  quick             Quick demo mode (minimal checks)"
    echo "  interactive       Interactive mode with menu"
    echo "  info              Show demo information"
    echo "  check             Check prerequisites only"
    echo "  setup             Setup demo environment only"
    echo "  help              Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0                # Run full demo (default)"
    echo "  $0 quick          # Quick demo"
    echo "  $0 interactive    # Interactive mode"
    echo "  $0 info           # Show information"
}

# Main function
main() {
    local command="${1:-run}"
    
    case "$command" in
        "run")
            check_prerequisites
            setup_demo
            ensure_funding
            run_demo
            ;;
        "quick")
            quick_demo
            ;;
        "interactive")
            check_prerequisites
            setup_demo
            interactive_mode
            ;;
        "info")
            show_demo_info
            ;;
        "check")
            check_prerequisites
            ;;
        "setup")
            setup_demo
            ;;
        "help"|"-h"|"--help")
            usage
            ;;
        *)
            echo -e "${RED}❌ Unknown command: $command${NC}"
            usage
            exit 1
            ;;
    esac
}

# Run main function
main "$@" 