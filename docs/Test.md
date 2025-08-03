
## Common Development Commands

### Building and Testing
```bash
# Build the entire project
cargo build --all --all-features
make build

# Run all tests (includes unit, spec-integrated, and spec-modular tests)
make test

# Run only unit tests
make test-unit
# Alternative: cargo nextest run --locked --workspace --all-features --no-fail-fast

# Run spec tests for integrated mode (both v0.6 and v0.7)
make test-spec-integrated

# Run spec tests for modular mode
make test-spec-modular

# Clean build artifacts
make clean
```

### Code Quality and Formatting
```bash
# Format code with nightly Rust
make fmt
# Alternative: cargo +nightly fmt

# Lint code and check for warnings
make lint
# Alternative: cargo clippy --all --all-features --tests -- -D warnings
```

### Running the Application
```bash
# Start SuperRelay service (recommended - handles process cleanup automatically)
./scripts/start_superrelay.sh

# Run SuperRelay binary directly
cargo run --bin super-relay

# Start development environment (automated setup)
./scripts/setup_dev_env.sh

# Run demonstration
./scripts/run_demo.sh
```

### Process Management and Cleanup

**IMPORTANT**: SuperRelay uses multiple services on different ports and must properly handle process cleanup to prevent port occupation errors.

#### Automated Process Cleanup
The startup script `./scripts/start_superrelay.sh` automatically handles process cleanup:
- Kills existing processes on ports 8545 (Anvil), 3000 (SuperRelay RPC), 9000 (Swagger UI), 8080 (Metrics)
- Terminates any lingering rundler or super-relay processes
- Provides clean shutdown via SIGTERM trap

#### Manual Process Cleanup (when needed)
```bash
# Kill processes by port
lsof -ti:8545 | xargs kill -9   # Anvil
lsof -ti:3000 | xargs kill -9   # SuperRelay RPC
lsof -ti:9000 | xargs kill -9   # Swagger UI
lsof -ti:8080 | xargs kill -9   # Metrics

# Kill by process name
pkill -f "rundler"
pkill -f "super-relay"
pkill -f "anvil"

# Check for remaining processes
ps aux | grep -E "(rundler|super-relay|anvil)"
```

#### Development Best Practices
1. **Always use the startup script**: `./scripts/start_superrelay.sh` handles all process lifecycle management
2. **Clean shutdown**: Use Ctrl+C to trigger cleanup trap function
3. **Port conflict resolution**: The script automatically resolves port conflicts before starting
4. **Environment isolation**: Each startup creates a clean environment state

## Monitoring and Metrics

### Prometheus Metrics Access

**Service URLs**:
- **Prometheus Metrics**: http://localhost:8080/metrics - Raw metrics data
- **Health Check**: http://localhost:9000/health - Service health status  
- **Swagger UI**: http://localhost:9000/swagger-ui/ - API documentation and testing

### Viewing Metrics

#### Quick Check
```bash
# Check if service is running
curl -s http://localhost:8080/metrics | head -5 || echo "❌ Service not running"

# View key metrics
curl -s http://localhost:8080/metrics | grep -E "(pool_|rpc_|builder_)" | head -10

# Check health status
curl -s http://localhost:9000/health | jq . 2>/dev/null || echo "Health endpoint not available"
```

#### Real-time Monitoring (macOS Compatible)
```bash
# Create monitoring script
cat > monitor_metrics.sh << 'EOF'
#!/bin/bash
echo "🔍 SuperRelay Real-time Monitoring"
echo "Press Ctrl+C to exit"
echo "=========================="

while true; do
  if curl -s http://localhost:8080/metrics > /dev/null 2>&1; then
    echo -e "\n📊 $(date '+%H:%M:%S') - Metrics Update"
    
    # Pool metrics
    echo "📦 Pool:"
    curl -s http://localhost:8080/metrics | grep "pool_" | head -3
    
    # RPC metrics
    echo "🌐 RPC:"
    curl -s http://localhost:8080/metrics | grep "rpc_" | head -3
    
    # Health status
    echo "🏥 Health:"
    curl -s http://localhost:9000/health 2>/dev/null || echo "Health endpoint not available"
    
  else
    echo "❌ SuperRelay service not running"
    echo "Please start with: ./scripts/start_superrelay.sh"
  fi
  
  sleep 5
done
EOF

chmod +x monitor_metrics.sh
./monitor_metrics.sh
```

#### Key Metric Categories
- **Pool Metrics**: `pool_userops_seen_total`, `pool_userops_added_total`, `pool_userops_removed_total`
- **Builder Metrics**: `builder_bundles_submitted_total`, `builder_bundle_gas_used`, `builder_transaction_fee`
- **RPC Metrics**: `rpc_requests_total`, `rpc_request_duration`, `rpc_errors_total`
- **Paymaster Metrics**: `paymaster_sponsorships_total`, `paymaster_gas_sponsored`

#### External Prometheus Setup (Optional)
```yaml
# prometheus.yml
global:
  scrape_interval: 15s

scrape_configs:
  - job_name: 'super-relay'
    static_configs:
      - targets: ['localhost:8080']
```

```bash
# Start Prometheus with Docker
docker run -p 9090:9090 -v $(pwd)/prometheus.yml:/etc/prometheus/prometheus.yml prom/prometheus

# Access Prometheus UI at http://localhost:9090
```

### Development Environment Setup
```bash
# Setup complete development environment
./scripts/setup_dev_env.sh

# Start local Anvil test network
./scripts/start_anvil.sh

# Fund paymaster accounts
./scripts/fund_paymaster.sh

# Start development server
./scripts/start_dev_server.sh

# Run end-to-end tests
./scripts/test_e2e.sh
```

## Architecture Overview

### Workspace Structure
This is a Rust workspace with multiple crates organized under `/crates/` and binary targets under `/bin/`:

**Core Binaries**:
- `bin/rundler/` - Main Rundler ERC-4337 bundler (extends Alchemy's Rundler)
- `bin/super-relay/` - SuperRelay service binary

**Key Crates**:
- `crates/paymaster-relay/` - **Core PaymasterRelay service** with API, policy engine, and signer management
- `crates/rpc/` - JSON-RPC server implementation with ERC-4337 APIs
- `crates/pool/` - UserOperation mempool management
- `crates/builder/` - Bundle creation and transaction building
- `crates/sim/` - UserOperation simulation and validation
- `crates/provider/` - Ethereum provider abstractions and integrations
- `crates/contracts/` - Smart contract bindings and utilities
- `crates/types/` - Shared type definitions across the system
- `crates/signer/` - Cryptographic signing infrastructure

### Key Design Patterns

1. **Modular Extension Architecture**: SuperPaymaster extends Rundler without modifying core functionality
2. **Service-Oriented Design**: Each crate provides specific services with clear interfaces
3. **Multi-Version Support**: Supports both EntryPoint v0.6 and v0.7 specifications
4. **Policy-Based Access Control**: Configurable policy engine for transaction filtering
5. **Comprehensive API Surface**: JSON-RPC, REST API with Swagger UI, and Prometheus metrics

### Core Components Integration

**PaymasterRelayService** (`crates/paymaster-relay/src/service.rs`):
- Orchestrates user operation sponsorship workflow
- Integrates with PolicyEngine for access control
- Manages SignerManager for cryptographic operations
- Coordinates with Rundler's pool for transaction submission

**API Layer**:
- JSON-RPC API (port 3000) - Primary ERC-4337 interface
- Swagger UI (port 9000) - Interactive API documentation and testing
- Prometheus metrics (port 8080) - Performance monitoring

## Configuration

### Environment Variables
```bash
PAYMASTER_PRIVATE_KEY="0x..." # Paymaster account private key
NODE_HTTP="http://localhost:8545" # Ethereum node endpoint
```

### Configuration Files
- `config/config.toml` - Main service configuration
- `config/paymaster-policies.toml` - Policy engine rules
- `config/production.toml` - Production environment settings
- `bin/rundler/chain_specs/` - Network-specific configurations

## Service Ports and Endpoints

| Service | Port | Purpose |
|---------|------|---------|
| JSON-RPC API | 3000 | Main ERC-4337 API service |
| Swagger UI | 9000 | Interactive API documentation |
| Prometheus Metrics | 8080 | Performance monitoring |

**Key Endpoints**:
- `http://localhost:3000` - JSON-RPC API
- `http://localhost:9000/swagger-ui/` - Interactive API explorer
- `http://localhost:9000/health` - Service health check
- `http://localhost:8080/metrics` - Prometheus metrics

## Testing Strategy

### Test Structure
- **Unit Tests**: Individual crate functionality (`cargo nextest run`)
- **Integration Tests**: Cross-crate interaction testing
- **Spec Tests**: ERC-4337 compliance testing (v0.6 and v0.7)
- **Demo Tests**: End-to-end user scenario validation

### Running ERC-4337 Spec Tests

#### Quick Run (Recommended)
```bash
# Run all spec tests
make test-spec-integrated

# Run specific version tests
make test-spec-integrated-v0_6  # EntryPoint v0.6 tests
make test-spec-integrated-v0_7  # EntryPoint v0.7 tests
```

#### Manual Run
```bash
# Navigate to test directory
cd test/spec-tests/local

# Run v0.6 tests (EntryPoint: 0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789)
./run-spec-tests-v0_6.sh

# Run v0.7 tests (EntryPoint: 0x0000000071727De22E5E9d8BAf0edAc6f37da032)
./run-spec-tests-v0_7.sh
```

#### Test Environment Setup (First Time)
```bash
# Install PDM (Python dependency manager)
pip install pdm

# Install v0.6 test dependencies
cd test/spec-tests/v0_6/bundler-spec-tests
pdm install && pdm run update-deps

# Install v0.7 test dependencies
cd ../../../v0_7/bundler-spec-tests
pdm install && pdm run update-deps
```

#### Test Coverage
- **RPC API Tests**: eth_sendUserOperation, eth_estimateUserOperationGas, etc.
- **Bundle Processing**: Bundle validation, gas estimation, transaction simulation
- **Reputation System**: Entity reputation tracking and throttling
- **Operation Banning**: Invalid operation detection and banning logic
- **Multi-Version Support**: Both EntryPoint v0.6 and v0.7 compliance

### Running Specific Tests
```bash
# Run tests for specific crate
cargo test -p rundler-paymaster-relay

# Run unit tests only
make test-unit

# Run with specific test pattern
cd test/spec-tests/v0_6/bundler-spec-tests
pdm run pytest -rA --url http://localhost:3000 --entry-point 0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789 --ethereum-node http://localhost:8545 -k "test_eth_sendUserOperation"
```

## Development Workflows

### Adding New Features
1. Identify target crate (typically `crates/paymaster-relay/` for Paymaster features)
2. Follow existing module structure and patterns
3. Add appropriate tests in `tests/` directory
4. Update configuration files if needed
5. Run full test suite before committing

### Debugging and Troubleshooting
- Enable tracing logs for detailed operation flow
- Use `./scripts/test_simple.sh` for quick validation
- Check service health endpoints for status
- Monitor Prometheus metrics for performance issues

### Working with Contracts
- Contract artifacts in `crates/contracts/contracts/out/`
- Supports both v0.6 and v0.7 EntryPoint contracts
- Use Foundry for contract compilation and testing

## Dependencies and External Integration

**Key Dependencies**:
- `alloy-*` - Ethereum interaction and primitives
- `tokio` - Async runtime
- `jsonrpsee` - JSON-RPC server implementation
- `axum` - HTTP server for REST API
- `ethers` - Ethereum library (legacy support)
- `tonic` - gRPC framework for internal services

**External Services**:
- Ethereum node (Anvil for development, mainnet/L2s for production)
- EntryPoint smart contracts (v0.6: `0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789`)


⏺ 现在我为你详细介绍如何运行 ERC-4337 spec 测试和查看 Prometheus 监控指标：

  🧪 运行 ERC-4337 Spec 测试

  1. 快速运行方式 (推荐)

  # 运行所有 spec 测试
  make test-spec-integrated

  # 或者单独运行
  make test-spec-integrated-v0_6  # EntryPoint v0.6 测试
  make test-spec-integrated-v0_7  # EntryPoint v0.7 测试

  2. 手动运行方式

  # 进入测试目录
  cd test/spec-tests/local

  # 运行 v0.6 测试 
  ./run-spec-tests-v0_6.sh

  # 运行 v0.7 测试
  ./run-spec-tests-v0_7.sh

  3. 测试环境准备

  在首次运行前，需要安装 Python 依赖：

  # 安装 PDM (Python 依赖管理器)
  pip install pdm

  # 安装 v0.6 测试依赖
  cd test/spec-tests/v0_6/bundler-spec-tests
  pdm install && pdm run update-deps

  # 安装 v0.7 测试依赖  
  cd ../../../v0_7/bundler-spec-tests
  pdm install && pdm run update-deps

  4. 测试说明

  - v0.6 测试：使用 EntryPoint 0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789
  - v0.7 测试：使用 EntryPoint 0x0000000071727De22E5E9d8BAf0edAc6f37da032
  - 测试内容：RPC API、Bundle 处理、声誉系统、操作禁用等
  - 自动化：脚本会自动启动 Anvil + Rundler，运行测试后清理

  📊 查看 Prometheus 监控指标

  1. 启动 SuperRelay 服务

  # 启动服务 (会自动启动监控)
  ./scripts/start_superrelay.sh

  2. 访问监控端点

  | 服务                 | URL                               | 说明        |
  |--------------------|-----------------------------------|-----------|
  | Prometheus Metrics | http://localhost:8080/metrics     | 原始指标数据    |
  | Health Check       | http://localhost:9000/health      | 服务健康状态    |
  | Swagger UI         | http://localhost:9000/swagger-ui/ | API 文档和测试 |

  3. 查看关键指标

  # 直接查看所有指标
  curl http://localhost:8080/metrics

  # 过滤特定指标
  curl http://localhost:8080/metrics | grep pool_

  # 查看服务健康状态
  curl http://localhost:9000/health

  4. 主要监控指标类别

⏺ Bash(curl -s http://localhost:8080/metrics 2>/dev/null | grep "^#" | head -10 ||
      echo "服务未运行，显示示例指标")
  ⎿  (No content)

⏺ 如果服务未运行，以下是主要的监控指标类别：

  Pool 指标：
  - pool_userops_seen_total - 收到的 UserOperation 总数
  - pool_userops_added_total - 添加到池中的 UserOperation 数
  - pool_userops_removed_total - 从池中移除的数量

  Builder 指标：
  - builder_bundles_submitted_total - 提交的 Bundle 总数
  - builder_bundle_gas_used - Bundle Gas 使用量
  - builder_transaction_fee - 交易费用

  RPC 指标：
  - rpc_requests_total - RPC 请求总数
  - rpc_request_duration - 请求处理时间
  - rpc_errors_total - RPC 错误数量

  Paymaster 指标：
  - paymaster_sponsorships_total - 赞助的操作数量
  - paymaster_gas_sponsored - 赞助的 Gas 总量

  5. 完整监控工作流

  # 1. 启动服务
  ./scripts/start_superrelay.sh

  # 2. 在另一个终端查看实时指标
  watch -n 2 'curl -s http://localhost:8080/metrics | grep -E 
  "(pool_|builder_|rpc_)" | tail -10'

  # 3. 运行一些测试产生数据
  make test-spec-integrated-v0_6

  # 4. 检查健康状态
  curl http://localhost:9000/health | jq

  6. 设置外部 Prometheus 服务器

  如果想使用完整的 Prometheus + Grafana 监控：

  # prometheus.yml
  global:
    scrape_interval: 15s

  scrape_configs:
    - job_name: 'super-relay'
      static_configs:
        - targets: ['localhost:8080']

  # 启动 Prometheus
  docker run -p 9090:9090 -v $(pwd)/prometheus.yml:/etc/prometheus/prometheus.yml
  prom/prometheus

  # 访问 Prometheus UI
  open http://localhost:9090

  这样你就可以全面地运行 ERC-4337 规范测试并监控 SuperRelay 的性能指标了！



🔧 macOS 正确的监控命令

  1. 不使用 watch 的替代方案

  # 方案 A: 使用 while 循环 (推荐)
  while true; do
    clear
    echo "=== SuperRelay Metrics $(date) ==="
    curl -s http://localhost:8080/metrics | grep -E "(pool_|builder_|rpc_)" | tail
  -10
    sleep 2
  done

  2. 安装 watch 命令 (可选)

  # 使用 Homebrew 安装 watch
  brew install watch

  # 然后就可以使用原来的命令
  watch -n 2 'curl -s http://localhost:8080/metrics | grep -E 
  "(pool_|builder_|rpc_)" | tail -10'

  3. 更实用的监控脚本

  # 创建一个监控脚本
  cat > monitor_metrics.sh << 'EOF'
  #!/bin/bash
  echo "🔍 SuperRelay 实时监控"
  echo "按 Ctrl+C 退出"
  echo "=========================="

  while true; do
    if curl -s http://localhost:8080/metrics > /dev/null 2>&1; then
      echo -e "\n📊 $(date '+%H:%M:%S') - 指标更新"

      # Pool 指标
      echo "📦 Pool:"
      curl -s http://localhost:8080/metrics | grep "pool_" | head -3

      # RPC 指标  
      echo "🌐 RPC:"
      curl -s http://localhost:8080/metrics | grep "rpc_" | head -3

      # 健康状态
      echo "🏥 Health:"
      curl -s http://localhost:9000/health 2>/dev/null || echo "Health endpoint not 
  available"

    else
      echo "❌ SuperRelay 服务未运行"
      echo "请先运行：./scripts/start_superrelay.sh"
    fi

    sleep 5
  done
  EOF

  chmod +x monitor_metrics.sh
  ./monitor_metrics.sh

  4. 简单的一次性检查

  # 检查服务是否运行
  curl -s http://localhost:8080/metrics | head -5 || echo "❌ 服务未运行"

  # 查看关键指标
  echo "📊 Pool 指标："
  curl -s http://localhost:8080/metrics | grep pool_ | head -3

  echo "📊 RPC 指标："
  curl -s http://localhost:8080/metrics | grep rpc_ | head -3

  5. 检查服务状态的快捷命令

  # 添加到你的 .zshrc 或 .bashrc
  alias check-superrelay='curl -s http://localhost:9000/health | jq . 2>/dev/null ||
   echo "SuperRelay not running"'
  alias metrics-superrelay='curl -s http://localhost:8080/metrics | grep -E 
  "(pool_|rpc_|builder_)" | head -10'

  # 使用
  check-superrelay
  metrics-superrelay

  谢谢你指出这个错误！macOS 确实没有预装 watch 命令，上面的替代方案更加实用。
