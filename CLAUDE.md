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
# Start full SuperPaymaster service (requires environment setup)
cargo run --bin rundler -- node \
  --paymaster.enabled \
  --paymaster.private_key=$PAYMASTER_PRIVATE_KEY \
  --paymaster.policy_file=config/paymaster-policies.toml \
  --node_http=$NODE_HTTP \
  --unsafe \
  --network=dev \
  --rpc.api=eth,debug,admin,rundler,paymaster

# Run SuperRelay binary directly
cargo run --bin super-relay

# Start development environment (automated setup)
./scripts/setup_dev_env.sh

# Run demonstration
./scripts/run_demo.sh
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