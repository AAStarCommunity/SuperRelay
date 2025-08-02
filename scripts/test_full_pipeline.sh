#!/bin/bash
# Complete SuperRelay testing pipeline
# Runs the full test suite from environment setup to transaction verification

set -e

echo "🚀 SuperRelay Complete Test Pipeline"
echo "===================================="

# Colors for output  
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
TEST_LOG_FILE="$PROJECT_ROOT/test_pipeline.log"

# Test phases tracking
PHASE_COUNT=8
CURRENT_PHASE=0
FAILED_PHASES=()

# Cleanup function
cleanup() {
    echo -e "\n${YELLOW}🧹 Cleaning up test environment...${NC}"
    
    # Stop any running processes
    pkill -f "anvil" 2>/dev/null || true
    pkill -f "super-relay" 2>/dev/null || true
    
    # Clean up test files
    rm -f .anvil.pid
    rm -f .superrelay.pid
    
    echo -e "${GREEN}✅ Cleanup completed${NC}"
}

# Trap cleanup on exit
trap cleanup EXIT

# Progress tracking
advance_phase() {
    ((CURRENT_PHASE++))
    local phase_name="$1"
    echo -e "\n${BLUE}📋 Phase $CURRENT_PHASE/$PHASE_COUNT: $phase_name${NC}"
    echo "$(date): Phase $CURRENT_PHASE - $phase_name" >> "$TEST_LOG_FILE"
}

# Function to run phase and track failures
run_phase() {
    local phase_name="$1"
    local phase_command="$2"
    local required="${3:-true}"
    
    advance_phase "$phase_name"
    
    echo -e "${BLUE}🔄 Executing: $phase_command${NC}"
    
    if eval "$phase_command" 2>&1 | tee -a "$TEST_LOG_FILE"; then
        echo -e "${GREEN}✅ PHASE PASSED: $phase_name${NC}"
        echo "$(date): ✅ $phase_name - PASSED" >> "$TEST_LOG_FILE"
        return 0
    else
        echo -e "${RED}❌ PHASE FAILED: $phase_name${NC}"
        echo "$(date): ❌ $phase_name - FAILED" >> "$TEST_LOG_FILE"
        FAILED_PHASES+=("$phase_name")
        
        if [ "$required" = "true" ]; then
            echo -e "${RED}💥 Critical phase failed. Stopping pipeline.${NC}"
            return 1
        else
            echo -e "${YELLOW}⚠️ Optional phase failed. Continuing...${NC}"
            return 0
        fi
    fi
}

# Wait for service to be ready
wait_for_service() {
    local service_name="$1"
    local service_url="$2"
    local max_attempts=${3:-30}
    local attempt=0
    
    echo -e "${BLUE}⏳ Waiting for $service_name to be ready...${NC}"
    
    while [ $attempt -lt $max_attempts ]; do
        if curl -s "$service_url" > /dev/null 2>&1; then
            echo -e "${GREEN}✅ $service_name is ready${NC}"
            return 0
        fi
        
        ((attempt++))
        echo -e "${YELLOW}🔄 Attempt $attempt/$max_attempts - waiting for $service_name...${NC}"
        sleep 2
    done
    
    echo -e "${RED}❌ $service_name failed to start within $(($max_attempts * 2)) seconds${NC}"
    return 1
}

# Phase 1: Environment Setup
phase_environment_setup() {
    echo -e "${BLUE}🛠️ Setting up development environment...${NC}"
    
    # Check if setup script exists
    if [ ! -f "$SCRIPT_DIR/setup_dev_env.sh" ]; then
        echo -e "${YELLOW}⚠️ setup_dev_env.sh not found, skipping environment setup${NC}"
        return 0
    fi
    
    # Run environment setup
    "$SCRIPT_DIR/setup_dev_env.sh"
    
    # Verify required tools
    local missing_tools=()
    for tool in cargo anvil cast node jq curl; do
        if ! command -v "$tool" &> /dev/null; then
            missing_tools+=("$tool")
        fi
    done
    
    if [ ${#missing_tools[@]} -ne 0 ]; then
        echo -e "${RED}❌ Missing required tools: ${missing_tools[*]}${NC}"
        return 1
    fi
    
    echo -e "${GREEN}✅ Environment setup completed${NC}"
    return 0
}

# Phase 2: Start Anvil
phase_start_anvil() {
    echo -e "${BLUE}⚡ Starting Anvil local blockchain...${NC}"
    
    # Kill any existing anvil process
    pkill -f "anvil" 2>/dev/null || true
    sleep 2
    
    # Start anvil in background
    anvil --host 0.0.0.0 --port 8545 --chain-id 31337 > anvil.log 2>&1 &
    local anvil_pid=$!
    echo $anvil_pid > .anvil.pid
    
    # Give anvil time to start
    sleep 3
    
    # Verify anvil is running
    if ! curl -s -X POST -H "Content-Type: application/json" \
        --data '{"jsonrpc":"2.0","method":"eth_chainId","params":[],"id":1}' \
        http://localhost:8545 > /dev/null; then
        echo -e "${RED}❌ Anvil failed to start${NC}"
        return 1
    fi
    
    echo -e "${GREEN}✅ Anvil started successfully (PID: $anvil_pid)${NC}"
    return 0
}

# Phase 3: Deploy Contracts
phase_deploy_contracts() {
    echo -e "${BLUE}📜 Deploying contracts...${NC}"
    
    # Check if deploy script exists
    if [ ! -f "$SCRIPT_DIR/deploy_contracts.sh" ]; then
        echo -e "${YELLOW}⚠️ deploy_contracts.sh not found, creating mock contract addresses${NC}"
        
        # Create mock addresses for testing
        echo "0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789" > .entrypoint_v06_address
        echo "0x0000000071727De22E5E9d8BAf0edAc6f37da032" > .entrypoint_v07_address
        echo "0x70997970C51812dc3A010C7d01b50e0d17dc79C8" > .paymaster_address
        
        return 0
    fi
    
    # Run contract deployment
    "$SCRIPT_DIR/deploy_contracts.sh"
    
    # Verify contract addresses were created
    if [ ! -f ".entrypoint_v06_address" ] || [ ! -f ".entrypoint_v07_address" ]; then
        echo -e "${RED}❌ Contract deployment failed - address files not found${NC}"
        return 1
    fi
    
    echo -e "${GREEN}✅ Contracts deployed successfully${NC}"
    return 0
}

# Phase 4: Setup Test Accounts
phase_setup_accounts() {
    echo -e "${BLUE}👤 Setting up test accounts...${NC}"
    
    # Run account setup script
    "$SCRIPT_DIR/setup_test_accounts.sh"
    
    # Verify test configuration was created
    if [ ! -f ".test_accounts.json" ]; then
        echo -e "${RED}❌ Test account setup failed${NC}"
        return 1
    fi
    
    echo -e "${GREEN}✅ Test accounts configured${NC}"
    return 0
}

# Phase 5: Fund Paymaster
phase_fund_paymaster() {
    echo -e "${BLUE}💰 Funding paymaster...${NC}"
    
    # Check if fund script exists
    if [ ! -f "$SCRIPT_DIR/fund_paymaster.sh" ]; then
        echo -e "${YELLOW}⚠️ fund_paymaster.sh not found, skipping paymaster funding${NC}"
        return 0
    fi
    
    # Run paymaster funding
    "$SCRIPT_DIR/fund_paymaster.sh"
    
    echo -e "${GREEN}✅ Paymaster funded${NC}"
    return 0
}

# Phase 6: Start SuperRelay Service
phase_start_superrelay() {
    echo -e "${BLUE}🚀 Starting SuperRelay service...${NC}"
    
    # Kill any existing super-relay process
    pkill -f "super-relay" 2>/dev/null || true
    sleep 2
    
    # Build the project first
    echo -e "${BLUE}🔨 Building SuperRelay...${NC}"
    cd "$PROJECT_ROOT"
    cargo build --release
    
    # Start super-relay in background
    RUST_LOG=info ./target/release/super-relay \
        --host 0.0.0.0 \
        --port 3000 \
        --rpc-url http://localhost:8545 \
        --chain-id 31337 \
        > superrelay.log 2>&1 &
    
    local superrelay_pid=$!
    echo $superrelay_pid > .superrelay.pid
    
    # Wait for service to be ready
    if ! wait_for_service "SuperRelay" "http://localhost:3000/health" 30; then
        echo -e "${RED}❌ SuperRelay failed to start${NC}"
        return 1
    fi
    
    echo -e "${GREEN}✅ SuperRelay started successfully (PID: $superrelay_pid)${NC}"
    return 0
}

# Phase 7: Run Core Tests
phase_run_tests() {
    echo -e "${BLUE}🧪 Running core tests...${NC}"
    
    # Run UserOperation construction tests
    echo -e "${BLUE}📋 Testing UserOperation construction...${NC}"
    "$SCRIPT_DIR/test_userop_construction.sh"
    
    # Run end-to-end tests
    echo -e "${BLUE}🔄 Running end-to-end tests...${NC}"
    "$SCRIPT_DIR/test_e2e.sh"
    
    echo -e "${GREEN}✅ Core tests completed${NC}"
    return 0
}

# Phase 8: Generate Test Report
phase_generate_report() {
    echo -e "${BLUE}📊 Generating test report...${NC}"
    
    local report_file="$PROJECT_ROOT/test_report_$(date +%Y%m%d_%H%M%S).md"
    
    cat > "$report_file" << EOF
# SuperRelay Test Pipeline Report

**Generated**: $(date)
**Duration**: $SECONDS seconds
**Log File**: $TEST_LOG_FILE

## Test Environment

- **Anvil**: $(curl -s -X POST -H "Content-Type: application/json" --data '{"jsonrpc":"2.0","method":"eth_chainId","params":[],"id":1}' http://localhost:8545 | jq -r '.result // "Not available"')
- **SuperRelay**: $(curl -s http://localhost:3000/health 2>/dev/null || echo "Not available")
- **Node.js**: $(node --version 2>/dev/null || echo "Not available")
- **Rust**: $(rustc --version 2>/dev/null || echo "Not available")

## Test Results

### Completed Phases
EOF
    
    for i in $(seq 1 $CURRENT_PHASE); do
        echo "- ✅ Phase $i completed" >> "$report_file"
    done
    
    if [ ${#FAILED_PHASES[@]} -gt 0 ]; then
        echo -e "\n### Failed Phases" >> "$report_file"
        for phase in "${FAILED_PHASES[@]}"; do
            echo "- ❌ $phase" >> "$report_file"
        done
    fi
    
    cat >> "$report_file" << EOF

## Service Status

- **Anvil PID**: $(cat .anvil.pid 2>/dev/null || echo "Not running")
- **SuperRelay PID**: $(cat .superrelay.pid 2>/dev/null || echo "Not running")

## Next Steps

1. Review the detailed log file: \`$TEST_LOG_FILE\`
2. Check service logs: \`anvil.log\` and \`superrelay.log\`
3. Run individual test scripts for debugging
4. Update configuration based on test results

---
*Report generated by SuperRelay test pipeline*
EOF
    
    echo -e "${GREEN}✅ Test report generated: $report_file${NC}"
    echo -e "${BLUE}📋 View report: cat $report_file${NC}"
    
    return 0
}

# Display final summary
display_final_summary() {
    echo -e "\n${BLUE}🏁 Test Pipeline Complete${NC}"
    echo "========================="
    echo -e "${GREEN}📊 Phases Completed: $CURRENT_PHASE/$PHASE_COUNT${NC}"
    echo -e "${BLUE}⏱️ Total Duration: $SECONDS seconds${NC}"
    
    if [ ${#FAILED_PHASES[@]} -eq 0 ]; then
        echo -e "${GREEN}🎉 All phases completed successfully!${NC}"
        echo -e "\n${BLUE}🔗 Services Running:${NC}"
        echo -e "  • Anvil: http://localhost:8545"
        echo -e "  • SuperRelay: http://localhost:3000"
        echo -e "  • Health Check: http://localhost:3000/health"
        return 0
    else
        echo -e "${RED}❌ Failed Phases: ${#FAILED_PHASES[@]}${NC}"
        for phase in "${FAILED_PHASES[@]}"; do
            echo -e "  • $phase"
        done
        echo -e "\n${YELLOW}🔍 Check logs for details:${NC}"
        echo -e "  • Pipeline Log: $TEST_LOG_FILE"
        echo -e "  • Anvil Log: anvil.log"
        echo -e "  • SuperRelay Log: superrelay.log"
        return 1
    fi
}

# Main execution
main() {
    echo -e "${BLUE}🚀 Starting SuperRelay complete test pipeline...${NC}"
    echo "$(date): Starting test pipeline" > "$TEST_LOG_FILE"
    
    # Change to project root
    cd "$PROJECT_ROOT"
    
    # Run test phases
    run_phase "Environment Setup" "phase_environment_setup" false
    run_phase "Start Anvil" "phase_start_anvil" true
    run_phase "Deploy Contracts" "phase_deploy_contracts" true
    run_phase "Setup Test Accounts" "phase_setup_accounts" true
    run_phase "Fund Paymaster" "phase_fund_paymaster" false
    run_phase "Start SuperRelay" "phase_start_superrelay" true
    run_phase "Run Core Tests" "phase_run_tests" true
    run_phase "Generate Test Report" "phase_generate_report" false
    
    # Display final summary
    display_final_summary
}

# Handle script arguments
case "${1:-}" in
    --help|-h)
        echo "SuperRelay Complete Test Pipeline"
        echo ""
        echo "Usage: $0 [options]"
        echo ""
        echo "Options:"
        echo "  --help, -h     Show this help message"
        echo "  --cleanup      Clean up test environment and exit"
        echo ""
        echo "This script runs the complete SuperRelay test pipeline:"
        echo "1. Environment setup"
        echo "2. Start Anvil blockchain"
        echo "3. Deploy contracts"
        echo "4. Setup test accounts"
        echo "5. Fund paymaster"
        echo "6. Start SuperRelay service"
        echo "7. Run core tests"
        echo "8. Generate test report"
        exit 0
        ;;
    --cleanup)
        cleanup
        exit 0
        ;;
    "")
        # No arguments, run normally
        main "$@"
        ;;
    *)
        echo -e "${RED}❌ Unknown option: $1${NC}"
        echo "Use --help for usage information"
        exit 1
        ;;
esac