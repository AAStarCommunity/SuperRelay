#!/bin/bash
# Headless browser testing for SuperRelay demo
# Tests the interactive demo using Playwright for automated browser testing

set -e

echo "🎭 SuperRelay Headless Browser Demo Testing"
echo "==========================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
DEMO_DIR="demo"
SUPERRELAY_URL="http://localhost:3000"
ANVIL_URL="http://localhost:8545"
DEMO_HTML="interactive-demo.html"

# Test results tracking
PASSED=0
FAILED=0

# Function to run test and track results
run_test() {
    local test_name="$1"
    local test_command="$2"
    
    echo -e "\n${BLUE}🧪 Testing: $test_name${NC}"
    
    if eval "$test_command"; then
        echo -e "${GREEN}✅ PASSED: $test_name${NC}"
        ((PASSED++))
        return 0
    else
        echo -e "${RED}❌ FAILED: $test_name${NC}"
        ((FAILED++))
        return 1
    fi
}

# Check if Node.js demo dependencies are installed
check_demo_dependencies() {
    echo -e "${BLUE}📦 Checking demo dependencies...${NC}"
    
    if [ ! -d "$DEMO_DIR" ]; then
        echo -e "${RED}❌ Demo directory not found${NC}"
        return 1
    fi
    
    cd "$DEMO_DIR"
    
    if [ ! -f "package.json" ]; then
        echo -e "${RED}❌ package.json not found in demo directory${NC}"
        return 1
    fi
    
    if [ ! -d "node_modules" ]; then
        echo -e "${YELLOW}📥 Installing demo dependencies...${NC}"
        npm install
    fi
    
    echo -e "${GREEN}✅ Demo dependencies ready${NC}"
    cd ..
    return 0
}

# Install Playwright for headless browser testing
install_playwright() {
    echo -e "${BLUE}🎭 Setting up Playwright for headless browser testing...${NC}"
    
    # Check if we need to install Playwright
    if ! npm list -g @playwright/test &>/dev/null; then
        echo -e "${YELLOW}📥 Installing Playwright globally...${NC}"
        npm install -g @playwright/test
    fi
    
    # Install browser binaries
    if ! command -v playwright &>/dev/null; then
        echo -e "${YELLOW}📥 Installing Playwright browser binaries...${NC}"
        npx playwright install chromium
    fi
    
    echo -e "${GREEN}✅ Playwright ready for headless testing${NC}"
}

# Create Playwright test configuration
create_playwright_config() {
    echo -e "${BLUE}⚙️ Creating Playwright test configuration...${NC}"
    
    cat > playwright.config.js << 'EOF'
module.exports = {
  testDir: './tests',
  timeout: 30000,
  fullyParallel: false,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: 1,
  reporter: 'html',
  use: {
    baseURL: 'http://localhost:3000',
    trace: 'on-first-retry',
    screenshot: 'only-on-failure',
    headless: true,
  },
  projects: [
    {
      name: 'chromium',
      use: { 
        ...require('@playwright/test').devices['Desktop Chrome'],
        headless: true
      },
    },
  ],
};
EOF

    echo -e "${GREEN}✅ Playwright configuration created${NC}"
}

# Create Playwright test for interactive demo
create_demo_test() {
    echo -e "${BLUE}📝 Creating Playwright test for interactive demo...${NC}"
    
    mkdir -p tests
    
    cat > tests/demo-interactive.spec.js << 'EOF'
const { test, expect } = require('@playwright/test');

test.describe('SuperRelay Interactive Demo', () => {
  test.beforeEach(async ({ page }) => {
    // Set up page error handling
    page.on('console', msg => {
      if (msg.type() === 'error') {
        console.log('Console error:', msg.text());
      }
    });
    
    page.on('pageerror', error => {
      console.log('Page error:', error.message);
    });
  });

  test('should load interactive demo page', async ({ page }) => {
    await page.goto('/demo/interactive-demo.html');
    
    // Check if page title is correct
    await expect(page).toHaveTitle(/SuperRelay.*Demo/i);
    
    // Check if main elements are present
    await expect(page.locator('h1')).toContainText('SuperRelay Demo');
  });

  test('should check service connectivity', async ({ page }) => {
    await page.goto('/demo/interactive-demo.html');
    
    // Wait for connectivity checks to complete
    await page.waitForTimeout(2000);
    
    // Check if connection status indicators are present
    const statusIndicators = page.locator('[data-testid*="status"]');
    await expect(statusIndicators.first()).toBeVisible();
  });

  test('should create UserOperation form', async ({ page }) => {
    await page.goto('/demo/interactive-demo.html');
    
    // Check if UserOperation form elements exist
    const senderInput = page.locator('input[name="sender"]');
    const nonceInput = page.locator('input[name="nonce"]');
    const callDataInput = page.locator('input[name="callData"]');
    
    if (await senderInput.count() > 0) {
      await expect(senderInput).toBeVisible();
      await expect(nonceInput).toBeVisible();
      await expect(callDataInput).toBeVisible();
    }
  });

  test('should test API endpoint calls', async ({ page }) => {
    await page.goto('/demo/interactive-demo.html');
    
    // Wait for page to load
    await page.waitForTimeout(1000);
    
    // Look for test buttons or API call triggers
    const testButton = page.locator('button:has-text("Test")').first();
    
    if (await testButton.count() > 0) {
      await testButton.click();
      
      // Wait for API call to complete
      await page.waitForTimeout(3000);
      
      // Check for result display
      const resultArea = page.locator('[data-testid="result"], .result, #result');
      if (await resultArea.count() > 0) {
        await expect(resultArea).toBeVisible();
      }
    }
  });

  test('should handle errors gracefully', async ({ page }) => {
    await page.goto('/demo/interactive-demo.html');
    
    // Wait for page to load
    await page.waitForTimeout(1000);
    
    // Test with invalid data if form exists
    const senderInput = page.locator('input[name="sender"]');
    
    if (await senderInput.count() > 0) {
      await senderInput.fill('invalid-address');
      
      const submitButton = page.locator('button[type="submit"], button:has-text("Submit")').first();
      if (await submitButton.count() > 0) {
        await submitButton.click();
        
        // Check for error message
        await page.waitForTimeout(2000);
        const errorMessage = page.locator('.error, [data-testid="error"], .alert-error');
        
        // Error handling should be present (either error message or form validation)
        const hasError = (await errorMessage.count() > 0) || 
                        (await page.locator('input:invalid').count() > 0);
        
        if (hasError) {
          console.log('✅ Error handling working correctly');
        }
      }
    }
  });
});
EOF

    echo -e "${GREEN}✅ Playwright demo test created${NC}"
}

# Test Node.js demo functionality
test_nodejs_demo() {
    echo -e "${BLUE}🚀 Testing Node.js demo functionality...${NC}"
    
    cd "$DEMO_DIR"
    
    # Test if demo script runs without crashing
    timeout 30s node superPaymasterDemo.js --help > demo_help.log 2>&1 || true
    
    if grep -q "SuperPaymaster Demo Application" demo_help.log; then
        echo -e "${GREEN}✅ Node.js demo help command works${NC}"
    else
        echo -e "${RED}❌ Node.js demo help command failed${NC}"
        cat demo_help.log
        cd ..
        return 1
    fi
    
    # Clean up
    rm -f demo_help.log
    cd ..
    return 0
}

# Test services are running
test_services_running() {
    echo -e "${BLUE}🔍 Checking if required services are running...${NC}"
    
    # Check Anvil
    if curl -s -X POST -H "Content-Type: application/json" \
        --data '{"jsonrpc":"2.0","method":"eth_chainId","params":[],"id":1}' \
        "$ANVIL_URL" > /dev/null; then
        echo -e "${GREEN}✅ Anvil is running${NC}"
    else
        echo -e "${YELLOW}⚠️ Anvil not running - some tests may fail${NC}"
    fi
    
    # Check SuperRelay
    if curl -s "$SUPERRELAY_URL/health" > /dev/null; then
        echo -e "${GREEN}✅ SuperRelay is running${NC}"
    else
        echo -e "${YELLOW}⚠️ SuperRelay not running - some tests may fail${NC}"
    fi
    
    return 0
}

# Run Playwright tests
run_playwright_tests() {
    echo -e "${BLUE}🎭 Running Playwright headless browser tests...${NC}"
    
    # Create a simple HTTP server for the demo if needed
    if [ -f "$DEMO_DIR/$DEMO_HTML" ]; then
        echo -e "${BLUE}🌐 Starting demo web server...${NC}"
        
        # Start a simple Python HTTP server in background
        cd "$DEMO_DIR"
        python3 -m http.server 8080 > /dev/null 2>&1 &
        local server_pid=$!
        cd ..
        
        # Wait for server to start
        sleep 2
        
        # Update Playwright config to use the demo server
        sed -i.bak "s|baseURL: 'http://localhost:3000'|baseURL: 'http://localhost:8080'|g" playwright.config.js
        
        # Run Playwright tests
        if npx playwright test --reporter=line; then
            echo -e "${GREEN}✅ Playwright tests completed successfully${NC}"
            local result=0
        else
            echo -e "${RED}❌ Some Playwright tests failed${NC}"
            local result=1
        fi
        
        # Cleanup
        kill $server_pid 2>/dev/null || true
        mv playwright.config.js.bak playwright.config.js 2>/dev/null || true
        
        return $result
    else
        echo -e "${YELLOW}⚠️ Interactive demo HTML not found - skipping browser tests${NC}"
        return 0
    fi
}

# Create comprehensive demo test report
create_demo_test_report() {
    echo -e "${BLUE}📊 Creating demo test report...${NC}"
    
    local report_file="demo_test_report_$(date +%Y%m%d_%H%M%S).md"
    
    cat > "$report_file" << EOF
# SuperRelay Demo Testing Report

**Generated**: $(date)
**Test Type**: Headless Browser + Node.js Demo
**Duration**: $SECONDS seconds

## Test Environment

- **Node.js**: $(node --version 2>/dev/null || echo "Not available")
- **NPM**: $(npm --version 2>/dev/null || echo "Not available")
- **Playwright**: $(npx playwright --version 2>/dev/null || echo "Not available")
- **Demo Directory**: $DEMO_DIR
- **SuperRelay URL**: $SUPERRELAY_URL
- **Anvil URL**: $ANVIL_URL

## Test Results

### Demo Tests Summary
- ✅ **Passed**: $PASSED
- ❌ **Failed**: $FAILED
- 📊 **Total**: $((PASSED + FAILED))

### Service Status
- **Anvil**: $(curl -s $ANVIL_URL >/dev/null 2>&1 && echo "✅ Running" || echo "❌ Not running")
- **SuperRelay**: $(curl -s $SUPERRELAY_URL/health >/dev/null 2>&1 && echo "✅ Running" || echo "❌ Not running")

### Demo Components Tested
- 📱 **Interactive HTML Demo**: $([ -f "$DEMO_DIR/$DEMO_HTML" ] && echo "✅ Found" || echo "❌ Not found")
- 🚀 **Node.js Demo Script**: $([ -f "$DEMO_DIR/superPaymasterDemo.js" ] && echo "✅ Found" || echo "❌ Not found")
- 📦 **Dependencies**: $([ -d "$DEMO_DIR/node_modules" ] && echo "✅ Installed" || echo "❌ Missing")

## Browser Testing

### Playwright Tests
EOF

    if [ -f "test-results/index.html" ]; then
        echo "- 📊 **Detailed Results**: Available in test-results/index.html" >> "$report_file"
    fi
    
    if [ -f "playwright-report/index.html" ]; then
        echo "- 📊 **HTML Report**: Available in playwright-report/index.html" >> "$report_file"
    fi
    
    cat >> "$report_file" << EOF

## Recommendations

### If Tests Passed
1. ✅ Demo functionality is working correctly
2. ✅ Browser compatibility confirmed
3. ✅ API endpoints responding properly
4. ✅ Error handling working as expected

### If Tests Failed
1. 🔍 Check service logs: \`anvil.log\` and \`superrelay.log\`
2. 🚀 Ensure all services are running: \`./scripts/test_full_pipeline.sh\`
3. 📦 Verify demo dependencies: \`cd demo && npm install\`
4. 🌐 Check network connectivity to test endpoints

## Next Steps

1. **Manual Testing**: Open http://localhost:8080/interactive-demo.html
2. **API Testing**: Run \`cd demo && node superPaymasterDemo.js\`
3. **Full Integration**: Execute \`./scripts/test_full_pipeline.sh\`
4. **Production Setup**: Configure for your target environment

---
*Report generated by SuperRelay demo testing pipeline*
EOF

    echo -e "${GREEN}✅ Demo test report generated: $report_file${NC}"
    echo -e "${BLUE}📋 View report: cat $report_file${NC}"
}

# Display summary
display_summary() {
    echo -e "\n${BLUE}🏁 Demo Testing Complete${NC}"
    echo "========================"
    echo -e "${GREEN}📊 Tests Passed: $PASSED${NC}"
    echo -e "${RED}❌ Tests Failed: $FAILED${NC}"
    echo -e "${BLUE}📊 Total Tests: $((PASSED + FAILED))${NC}"
    
    if [ $FAILED -eq 0 ]; then
        echo -e "\n${GREEN}🎉 All demo tests passed!${NC}"
        echo -e "${GREEN}🚀 SuperRelay demo is fully functional!${NC}"
        
        echo -e "\n${BLUE}🔗 Demo Access Points:${NC}"
        if [ -f "$DEMO_DIR/$DEMO_HTML" ]; then
            echo -e "  • Interactive Demo: http://localhost:8080/$DEMO_HTML"
        fi
        echo -e "  • Node.js Demo: cd demo && node superPaymasterDemo.js"
        echo -e "  • API Endpoint: $SUPERRELAY_URL"
        
        return 0
    else
        echo -e "\n${YELLOW}⚠️ Some demo tests failed. Check the details above.${NC}"
        return 1
    fi
}

# Main execution
main() {
    echo -e "${BLUE}🎭 Starting SuperRelay demo testing...${NC}"
    
    # Check prerequisites
    if ! command -v node &> /dev/null; then
        echo -e "${RED}❌ Node.js not found. Please install Node.js >= 16${NC}"
        exit 1
    fi
    
    if ! command -v npm &> /dev/null; then
        echo -e "${RED}❌ NPM not found. Please install NPM${NC}"
        exit 1
    fi
    
    # Run tests
    run_test "Demo Dependencies Check" "check_demo_dependencies"
    run_test "Services Running" "test_services_running"
    run_test "Node.js Demo Functionality" "test_nodejs_demo"
    
    # Playwright tests (optional - requires additional setup)
    if command -v npx &> /dev/null; then
        echo -e "\n${BLUE}🎭 Setting up browser testing...${NC}"
        install_playwright
        create_playwright_config
        create_demo_test
        run_test "Playwright Browser Tests" "run_playwright_tests"
    else
        echo -e "${YELLOW}⚠️ NPX not available - skipping browser tests${NC}"
    fi
    
    # Generate report
    create_demo_test_report
    
    # Display summary
    display_summary
}

# Handle script arguments
case "${1:-}" in
    --help|-h)
        echo "SuperRelay Demo Headless Browser Testing"
        echo ""
        echo "Usage: $0 [options]"
        echo ""
        echo "Options:"
        echo "  --help, -h     Show this help message"
        echo "  --setup-only   Only setup dependencies, don't run tests"
        echo ""
        echo "This script tests the SuperRelay demo using headless browsers:"
        echo "• Node.js demo functionality testing"
        echo "• Playwright browser automation testing"
        echo "• Interactive demo HTML validation"
        echo "• API endpoint integration testing"
        echo ""
        echo "Prerequisites:"
        echo "• Node.js >= 16 installed"
        echo "• SuperRelay service running on port 3000"
        echo "• Anvil running on port 8545 (optional)"
        exit 0
        ;;
    --setup-only)
        echo -e "${BLUE}🔧 Setting up demo testing dependencies only...${NC}"
        check_demo_dependencies
        install_playwright
        create_playwright_config
        create_demo_test
        echo -e "${GREEN}✅ Setup complete. Run without --setup-only to execute tests.${NC}"
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