# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

SuperRelay is an enterprise-grade Account Abstraction Paymaster solution built on top of Rundler (Alchemy's ERC-4337 bundler). It provides gas sponsorship services for decentralized applications through a modular, high-performance architecture.

**Key Technologies**: Rust (workspace), ERC-4337, Ethereum, Account Abstraction, JSON-RPC API, Swagger UI

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

### Running Specific Tests
```bash
# Run tests for specific crate
cargo test -p rundler-paymaster-relay

# Run spec tests for specific version
./test/spec-tests/local/run-spec-tests-v0_6.sh
./test/spec-tests/remote/run-spec-tests-v0_7.sh
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

## 🔒 SuperRelay Claude工作规范 (基于Jason Cursor Rules 0.16)

### 📁 文档体系结构 (严格遵守)
所有项目文档必须在 `docs/` 目录下，遵循以下结构：

- `docs/Solution.md` - **用户输入文档，绝对不可修改**
- `docs/Design.md` - 架构设计文档
- `docs/Features.md` - 核心功能描述
- `docs/Plan.md` - 开发计划
- `docs/Changes.md` - 版本迭代记录
- `docs/Deploy.md` - 部署运维文档
- `docs/Install.md` - 用户安装文档
- `docs/UserCaseTest.md` - 用户测试用例
- `docs/comprehensive.md` - 综合评估报告

### 🔄 开发流程 (严格执行)
```
输入(Solution) -> 设计(Design) -> 拆解(Features) -> 计划(Plan) -> 开发迭代(Changes) -> 测试验证 -> 规范验证
```

### 📊 版本管理规范
- 初始版本: `0.1.0`
- 递增规则: `0.1.1 -> 0.1.2 -> 0.1.12 -> 0.2.1`
- 每次更新 `Changes.md` 必须更新版本号
- 在明确开始 0.2.0 开发前，都在完成 0.1.x 版本

### ⚠️ 核心开发约束
**最小影响范围原则**:
- 禁止擅自优化和扩张功能范围
- 缩小影响范围，再缩小影响范围
- 只针对提出的问题，使用最少代码修改
- 严格遵守指令，禁止修改任何不相关代码
- 禁止任何不相关优化

**模块化原则**:
- 新增功能能独立模块就不要在原有主流程文件完成
- 每个修改都说清楚为何这样做

### 🔧 SuperRelay特定技术约束

**安全第一原则**:
- **私钥管理**: 测试(.env) -> 生产(环境变量) -> 未来(硬件API)
- **输入验证**: 所有RPC输入必须经过 `validation.rs`
- **速率限制**: Token bucket算法，`config.toml`可配置
- **错误处理**: 使用 `ErrorObjectOwned`，不暴露敏感信息

**Rust项目质量要求**:
```bash
# 每次完成todo后必须执行
cargo check --workspace
cargo test --workspace
./scripts/security_check.sh
./scripts/format.sh  # git提交前必须运行
git commit  # 禁止使用 --no-verify
```

**架构约束**:
- **Rundler集成**: 扩展不修改，保持向后兼容
- **PaymasterRelay**: `Arc<T>`共享状态，异步优先
- **配置驱动**: 所有参数通过 `config.toml` 控制
- **模块化扩展**: 如Security_filter独立crate

### 📝 代码质量标准
```rust
// ✅ 必须遵循的RPC方法模式
pub async fn safe_rpc_method(&self, input: Input) -> Result<Output, ErrorObjectOwned> {
    // 1. Input validation first
    self.validator.validate_input(&input)?;

    // 2. Rate limiting check
    if !self.rate_limiter.check_rate_limit(client_ip) {
        return Err(rate_limit_error());
    }

    // 3. Business logic
    let result = self.process(input).await?;

    Ok(result)
}

// ❌ 绝对禁止
pub fn unsafe_method(&self) -> String {
    self.private_key.unwrap() // 暴露敏感数据 + panic风险
}
```

### 🧪 测试和验证要求
**测试层级**:
- **用户视角**: 产品Features验证
- **产品方案视角**: 业务流程测试
- **系统视角**: 技术组件测试

**自动化要求**:
- Rust项目: `cargo build && cargo test`
- 每次修改后运行并确认无错误
- 编译测试部署指令写入 `DEPLOY.md`

### 🏗️ 架构设计原则
- **少侵入**: 最小化对原有代码的修改
- **隔离原有**: 新功能独立模块实现
- **高效通信**: 组件间清晰的接口设计
- **可扩展**: 支持新增安全过滤等全局模块
- **结构清晰**: 业务组件和技术组件分离
- **数据统一**: 统一的数据结构和传递格式
- **安全检查**: 全流程安全验证
- **容错重试**: 无状态可重复操作

### 🔍 失败处理策略
如果同样思路三次对话后还是失败：
1. **反思**: 分析失败原因
2. **改变思路**: 尝试其他技术方案
3. **拆分问题**: 将复杂问题分解为更小问题

### 📋 里程碑管理
- **小版本完成** (0.1.11->0.1.12): 更新 `Changes.md`
- **大版本完成** (0.1->0.2): 完成相关文档更新
- **功能完成**: 及时记录到版本迭代中

### 🎯 SuperRelay特定目标
- **编译时间**: < 2分钟 full workspace build
- **RPC响应**: < 200ms p95延迟
- **内存使用**: < 500MB稳态运行
- **安全扫描**: 0 critical/high issues
- **测试覆盖**: > 80% 核心业务逻辑

### 💡 Claude执行方式
每次执行都必须：
1. 检查是否需要更新 `docs/` 目录文档
2. 遵循最小影响范围原则
3. 完成测试和格式化
4. 更新 `Changes.md` 和版本号
5. 说明修改原因和影响范围