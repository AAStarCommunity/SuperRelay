# SuperRelay v2.0 Development Plan

## Project Overview
SuperRelay v2.0 is a complete redesign of the ERC-4337 paymaster service as a truly non-invasive extension to Rundler. The goal is to provide gas sponsorship functionality while maintaining 0 lines of original Rundler code modification.

## Development Phases

### ✅ Phase 1: Foundation (COMPLETED)
**Duration**: 3 days  
**Status**: ✅ COMPLETED

#### Milestone 1: Basic RPC ✅
- [x] Clean crate structure (`crates/paymaster-relay`)
- [x] Basic RPC server with health check
- [x] Integration tests
- [x] **Evidence**: `curl http://localhost:3002` health check returns "ok"

**Deliverables Completed**:
- ✅ `crates/paymaster-relay/` crate with full module structure
- ✅ `bin/super-relay/` standalone binary 
- ✅ All RPC methods functional: `pm_health`, `pm_getChainId`, `pm_getSupportedEntryPoints`
- ✅ Integration tests passing (4/4 tests)
- ✅ TOML-based configuration system

### Phase 2: Core Paymaster Logic (PLANNED)
**Duration**: 5 days  
**Status**: ⏸️ PAUSED (Documentation phase)

#### Milestone 2: Policy Engine (3 days)
- [ ] TOML policy configuration parser
- [ ] Address allowlist/denylist validation  
- [ ] Gas limit enforcement
- [ ] Rate limiting per address
- [ ] **Evidence**: Policy rejection works correctly

#### Milestone 3: UserOperation Signing (2 days)
- [ ] Private key management (env vars)
- [ ] EIP-712 UserOperation hash calculation
- [ ] Paymaster signature generation
- [ ] Support for both EntryPoint v0.6 and v0.7
- [ ] **Evidence**: Valid paymaster signatures generated

### Phase 3: Rundler Integration (PLANNED)
**Duration**: 4 days

#### Milestone 4: HTTP Client Integration (2 days)
- [ ] HTTP client for Rundler communication
- [ ] `eth_sendUserOperation` call forwarding
- [ ] Error handling and retries
- [ ] Connection pooling
- [ ] **Evidence**: UserOperations successfully submitted to Rundler

#### Milestone 5: End-to-End Flow (2 days)  
- [ ] Complete `pm_sponsorUserOperation` implementation
- [ ] Policy validation → Signing → Submission pipeline
- [ ] Error propagation and logging
- [ ] **Evidence**: Full sponsor flow works end-to-end

### Phase 4: Production Readiness (PLANNED)
**Duration**: 3 days

#### Milestone 6: Monitoring & Stats (1 day)
- [ ] Usage statistics collection
- [ ] Prometheus metrics (optional)
- [ ] Structured logging improvements
- [ ] **Evidence**: Statistics API returns accurate data

#### Milestone 7: Deployment & Testing (2 days)
- [ ] Docker container build
- [ ] Performance testing under load
- [ ] Integration with live Anvil testnet
- [ ] **Evidence**: Handles 25+ ops/second sustained throughput

### Phase 5: Documentation & Polish (PLANNED)
**Duration**: 2 days

#### Milestone 8: Final Documentation
- [ ] Complete API documentation
- [ ] Deployment guides
- [ ] Configuration examples
- [ ] Troubleshooting guides

## Success Criteria
- ✅ **Non-invasive**: 0 lines of original Rundler code modified
- ✅ **Isolated**: Clean module boundaries with `crates/paymaster-relay`
- ✅ **Configurable**: TOML-based policy and service configuration
- [ ] **Performant**: <1ms policy validation, ~380ms end-to-end processing
- [ ] **Reliable**: 90%+ test coverage, comprehensive error handling
- [ ] **Production-ready**: Docker deployment, monitoring, logging

## Architecture Principles
1. **Non-invasive Design**: Extend, don't modify Rundler
2. **HTTP Communication**: Services communicate via HTTP/RPC instead of memory coupling  
3. **Configuration-driven**: Policies and behavior controlled via TOML files
4. **Modular**: Clean separation between paymaster logic and bundler logic
5. **Observable**: Comprehensive logging, metrics, and health checks

## Current Status: Phase 1 Complete ✅
- RPC server fully functional
- All basic endpoints working  
- Integration tests passing
- Ready for Phase 2 implementation

**Next Step**: Implement Policy Engine (Milestone 2) when development resumes.