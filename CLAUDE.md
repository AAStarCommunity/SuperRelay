# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

SuperRelay is an ERC-4337 Account Abstraction bundler with integrated Paymaster Relay service, built on top of Rundler (Alchemy's ERC-4337 bundler). It provides gas sponsorship, security filtering, and on-chain transaction submission for ERC-4337 UserOperations.

### Key Architectural Principles
- **Non-invasive Design**: 0 lines of original Rundler code modified
- **Isolation**: Independent `paymaster-relay` crate with separate API namespace (`pm`)
- **Efficient Communication**: Direct memory calls with microsecond-level response times
- **Modular Plugin Architecture**: Extensible design for future enhancements

### Performance Characteristics
- **UserOperation Processing**: <1ms latency
- **End-to-End Processing**: ~380ms average
- **Sustained Throughput**: ~25 ops/second
- **Peak Throughput**: ~45 ops/second
- **Memory Usage**: ~45MB base, ~78MB under load

## Development Commands

### Build and Test
- **Build**: `cargo build --all --all-features`
- **Clean**: `cargo clean`
- **Unit tests**: `cargo nextest run --locked --workspace --all-features --no-fail-fast`
- **All tests**: `make test` (includes unit, spec-integrated, and spec-modular tests)
- **Lint**: `cargo clippy --all --all-features --tests -- -D warnings`
- **Format**: `cargo +nightly fmt`

### Development Environment
- **Start development server**: `./scripts/start_dev_server.sh`
- **Run integration tests**: `./scripts/test_integration.sh`
- **Deploy contracts**: `./scripts/deploy_entrypoint.sh`

The development server script automatically:
1. Starts Anvil (local Ethereum node)
2. Deploys EntryPoint contracts
3. Funds the Paymaster
4. Starts SuperRelay with paymaster-relay enabled

### Service Endpoints
- **JSON-RPC API**: `http://localhost:3000` (primary API)
- **Swagger UI**: `http://localhost:9000/swagger-ui/` (API documentation)
- **Metrics**: `http://localhost:8080/metrics` (Prometheus format)

## Architecture

### Core Components
1. **Rundler Base**: ERC-4337 bundler infrastructure (pool, builder, RPC services)
2. **Paymaster Relay**: Gas sponsorship service with policy engine and signing
3. **Swagger UI**: Interactive API documentation and testing interface

### System Architecture Flow
```
Client Request → PaymasterRelayApi (pm namespace) → PaymasterRelayService →
PolicyEngine (validation) → SignerManager (signing) → Pool (mempool submission)
```

### Task Communication Pattern
- **RPC → Pool**: Submits UserOperations via `eth_sendUserOperation`
- **RPC → Builder**: Debug namespace for manual bundling control
- **Builder ↔ Pool**: Bundle coordination and UserOperation validation
- **PaymasterRelay → Pool**: Direct integration for sponsored UserOperations

### Workspace Structure
- `bin/rundler/`: Main bundler CLI application
- `bin/super-relay/`: SuperRelay specific binaries
- `bin/dashboard/`: Dashboard application
- `crates/paymaster-relay/`: Paymaster relay service implementation
- `crates/pool/`: UserOperation mempool management
- `crates/builder/`: Bundle creation and submission
- `crates/rpc/`: JSON-RPC server and API endpoints
- `crates/sim/`: Gas estimation and simulation
- `crates/signer/`: Key management and signing
- `crates/provider/`: Ethereum provider abstractions

### Key Services Integration
- **PaymasterRelayService**: Core sponsorship logic in `crates/paymaster-relay/src/service.rs`
- **PolicyEngine**: Request filtering in `crates/paymaster-relay/src/policy.rs`
- **SwaggerUI**: API documentation in `crates/paymaster-relay/src/swagger.rs`
- **Statistics**: Usage metrics in `crates/paymaster-relay/src/statistics.rs`

### Configuration
- Main config: `config/config.toml`
- Paymaster policies: `config/paymaster-policies.toml`
- Production config: `config/production.toml`

### Entry Points Support
- **EntryPoint v0.6**: `0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789`
- **EntryPoint v0.7**: Full ERC-4337 v0.7 specification support

## API Endpoints

### Paymaster Relay API (pm namespace)
- `pm_sponsorUserOperation`: **Core endpoint** - Atomic validation, policy check, signing, and mempool submission
  - Input: `UserOperation` + `EntryPoint` address
  - Output: `UserOpHash` of successfully sponsored operation
  - Processing: <1ms latency for validation and signing
- `pm_getSupportedEntryPoints`: Get supported EntryPoint addresses
- `pm_getChainId`: Get current chain ID
- `pm_getStatistics`: Get usage statistics (transactions, users, gas sponsored)

### Standard ERC-4337 Bundler API (eth namespace)
- `eth_sendUserOperation`: Submit UserOperation to mempool
- `eth_estimateUserOperationGas`: Gas estimation
- `eth_getUserOperationReceipt`: Get transaction receipt
- `eth_getUserOperationByHash`: Get UserOperation by hash
- `eth_supportedEntryPoints`: Get supported EntryPoints

### Debug APIs (debug namespace)
- `debug_bundler_clearState`: Clear bundler state for testing
- `debug_bundler_dumpMempool`: Dump current mempool contents
- `debug_bundler_sendBundleNow`: Force immediate bundle creation
- `debug_bundler_setBundlingMode`: Control bundling behavior

## Development Workflow

### Running Tests
1. Unit tests are the primary testing method
2. Spec tests validate ERC-4337 compliance for both v0.6 and v0.7
3. Integration tests use the full development environment

### Adding Features
1. Follow existing patterns in the crate structure
2. Add API endpoints in `crates/paymaster-relay/src/rpc.rs`
3. Implement business logic in `crates/paymaster-relay/src/service.rs`
4. Update Swagger documentation in `api_schemas.rs`
5. Add metrics in `crates/paymaster-relay/src/metrics.rs`

### Policy Configuration
Paymaster policies are defined in TOML files with support for:
- **AllowedSenders/DeniedSenders**: Address-based filtering
- **AllowedTargets**: Contract interaction filtering
- **MaxGasLimit**: Gas consumption limits
- **Rate Limiting**: Request frequency controls
- **Time-based Policies**: Temporal access controls
- **Custom Validation Rules**: Extensible policy framework

Example policy configuration:
```toml
[default]
type = "allowlist"
addresses = ["0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266"]
max_gas_limit = "1000000"
```

### Key Environment Variables
- `PAYMASTER_PRIVATE_KEY`: Private key for paymaster signing
- `NODE_HTTP`: Ethereum node RPC URL (default: `http://localhost:8545`)
- `RUST_LOG`: Logging configuration
- `PAYMASTER_POLICY_PATH`: Path to policy configuration file

### Important Notes
- Always run lint before committing: `cargo clippy --all --all-features --tests -- -D warnings`
- Use `cargo +nightly fmt` for code formatting
- The development server automatically handles contract deployment and funding
- Swagger UI provides interactive API testing at port 9000

## Testing Strategy

### Multi-Level Testing Pyramid
- **Unit Tests**: Core module functionality (85% coverage target >90%)
- **Integration Tests**: RPC endpoint validation (85% coverage target >90%)
- **Spec Tests**: ERC-4337 compliance validation for v0.6 and v0.7
- **E2E Tests**: Full workflow with Anvil testnet
- **Performance Tests**: Load testing and benchmarking

### Demo Applications
Interactive demo scenarios available in `demo/` directory:
- Real UserOperation handling
- Error handling validation
- Performance testing under load

## Security Considerations

### Current Security Measures
- **Private Key Security**: Environment variable loading (production: KMS integration planned)
- **Policy Enforcement**: TOML-configured validation rules
- **Input Validation**: Comprehensive UserOperation validation
- **Error Handling**: Secure error propagation

### Planned Security Enhancements
- **ARM OP-TEE KMS**: Hardware-backed key storage
- **Security Filter Module**: Rate limiting and IP whitelisting
- **Risk Assessment Engine**: Anomaly detection and scoring
- **Audit Logging**: Comprehensive security event tracking

## Deployment and Configuration

### Docker Deployment
```bash
docker run -p 3000:3000 -p 8080:8080 -p 9000:9000 \
  -e PAYMASTER_PRIVATE_KEY=your_key \
  -e NODE_HTTP=your_rpc_url \
  rundler node --paymaster.enabled=true
```

### Production Configuration
- Use `config/production.toml` for production settings
- Environment-based configuration for sensitive values
- Multi-chain support via configuration
- Prometheus metrics at `/metrics` endpoint

## Monitoring and Observability

### Current Monitoring
- **Health Check**: `/health` endpoint with status reporting
- **Basic Metrics**: API call statistics and success rates
- **Structured Logging**: Configurable levels with RUST_LOG

### Planned Enhancements
- **Prometheus Integration**: Comprehensive metrics collection
- **Real-time Dashboard**: Live system status and performance
- **Alert System**: Proactive notification for critical issues
- **Distributed Tracing**: Request flow tracking across components