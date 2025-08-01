# Deploy Guide

This guide provides instructions for deploying, initializing, and maintaining the SuperRelay project, intended for operators and developers.

## Part 1: First-Time Environment Setup

Follow these steps only once to prepare your machine for development.

### 1. Install Core Tools

- **Rust Toolchain**:
  ```bash
  # Install Rust and rustup
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  source "$HOME/.cargo/env"
  # Add the nightly toolchain required by format checks
  rustup toolchain install nightly
  ```

- **Foundry**:
  ```bash
  # Install Foundry (anvil, cast, forge)
  curl -L https://foundry.paradigm.xyz | bash
  source "$HOME/.bashrc"
  foundryup
  ```

- **Commit Verification Tool**:
  ```bash
  # Install cocogitto for Conventional Commit validation
  cargo install cocogitto
  ```

### 2. Clone and Build the Project

```bash
# Clone the repository and its submodules
git clone --recurse-submodules https://github.com/AAStarCommunity/SuperRelay.git
cd SuperRelay

# Build the project for the first time.
# This will also install the git hooks managed by cargo-husky.
cargo build
```

---

## Part 2: Daily Development Workflow

Use these commands for everyday development.

### 1. Start the Local Development Environment

We have a single, unified script to set up the entire local environment.

```bash
# This script starts Anvil, deploys EntryPoint, and starts the SuperRelay service.
# It also provides a health check at the end.
./scripts/start_dev_server.sh
```
The script will automatically handle:
- Starting a fresh `anvil` node on `http://localhost:8545`.
- Deploying the `EntryPoint` contract.
- Funding the default paymaster wallet.
- Starting the `SuperRelay` service on `http://localhost:3000`.

### 2. Verify Services Manually

After the script finishes, you can manually check the different services:

- **Check Anvil Node**:
  ```bash
  curl -X POST -H "Content-Type: application/json" --data '{"jsonrpc":"2.0","id":1,"method":"eth_blockNumber","params":[]}' http://localhost:8545
  ```
  *Expected: A JSON response with a `result` field, e.g., `{"jsonrpc":"2.0","result":"0x0","id":1}`.*

- **Check SuperRelay Health**:
  ```bash
  curl http://localhost:3000/health
  ```
  *Expected: `{"status":"ok"}` or similar healthy status.*

- **Check SuperRelay Sponsored Chains**:
  ```bash
  curl -X POST -H "Content-Type: application/json" --data '{"jsonrpc":"2.0","id":1,"method":"eth_supportedEntryPoints","params":[]}' http://localhost:3000
  ```
  *Expected: A JSON response with a result array containing the EntryPoint address.*

- **Explore Interactive API Docs**:
  - Open your browser and navigate to **`http://localhost:9000/swagger-ui/`**.

### 3. Running Tests

- **Run all Rust tests**:
  ```bash
  cargo test
  ```

- **Run contract tests (IMPORTANT)**:
  `forge test` must be run from within the specific contract directory.
  ```bash
  # For v0.6 contracts
  cd crates/contracts/contracts/v0_6 && forge test && cd ../../..

  # For v0_7 contracts
  cd crates/contracts/contracts/v0_7 && forge test && cd ../../..
  ```

### 4. Stopping the Environment

- Simply press `Ctrl+C` in the terminal where you ran `./scripts/start_dev_server.sh`. The script is designed to shut down all background processes (anvil, super-relay) gracefully.

---
## Common Questions & Troubleshooting

- **Why redeploy EntryPoint every time?**
  - The `start_dev_server.sh` script starts a fresh `anvil` instance for each run. This guarantees a clean, predictable state for every test session, preventing results from one session from interfering with the next. This deterministic approach is a best practice for reliable testing.

- **Why does `fund_paymaster.sh` not call `cast` directly?**
  - The script acts as a wrapper to provide context and usability. It loads environment variables (like private keys and RPC URLs from `.env`), performs calculations (e.g., converting ETH to wei), and provides user-friendly subcommands (`status`, `auto-rebalance`). This makes funding safer and easier than using raw `cast` commands.

- **What if `git commit` fails?**
  - This is likely due to the pre-commit hooks. Check the error message.
    - `rustfmt`: Run `cargo +nightly fmt --all`.
    - `clippy`: Fix the warnings reported by the linter.
    - `cocogitto`: Ensure your commit message follows the [Conventional Commits](https://www.conventionalcommits.org/) standard (e.g., `feat: add new feature`).

- **What if `forge test` fails?**
  - Make sure you are in the correct directory (e.g., `crates/contracts/contracts/v0_6`) before running the command. The project contains multiple, independent Foundry projects.