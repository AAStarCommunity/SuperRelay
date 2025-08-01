#!/bin/bash
# Setup development environment for SuperPaymaster project
# This script helps new developers set up their environment quickly

set -e

echo "🚀 Setting up SuperPaymaster Development Environment"
echo "=================================================="

# Check if running on macOS or Linux
OS=$(uname -s)
echo "📱 Detected OS: $OS"

# Function to check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Function to install with homebrew (macOS)
install_with_brew() {
    if command_exists brew; then
        echo "🍺 Installing $1 with Homebrew..."
        brew install "$1"
    else
        echo "❌ Homebrew not found. Please install Homebrew first:"
        echo "   /bin/bash -c \"\$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)\""
        exit 1
    fi
}

# Function to install with apt (Ubuntu/Debian)
install_with_apt() {
    echo "📦 Installing $1 with apt..."
    sudo apt update
    sudo apt install -y "$1"
}

# 1. Check and install Rust
echo "🦀 Checking Rust installation..."
if command_exists rustc; then
    RUST_VERSION=$(rustc --version)
    echo "✅ Rust found: $RUST_VERSION"
else
    echo "📥 Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source ~/.cargo/env
    echo "✅ Rust installed successfully"
fi

# Update Rust to latest stable
echo "🔄 Updating Rust to latest stable..."
rustup update stable
rustup default stable

# 2. Check and install Foundry (for anvil, cast, forge)
echo "⚒️  Checking Foundry installation..."
if command_exists anvil && command_exists cast && command_exists forge; then
    FOUNDRY_VERSION=$(anvil --version | head -n1)
    echo "✅ Foundry found: $FOUNDRY_VERSION"
else
    echo "📥 Installing Foundry..."
    curl -L https://foundry.paradigm.xyz | bash
    source ~/.bashrc 2>/dev/null || source ~/.zshrc 2>/dev/null || true
    ~/.foundry/bin/foundryup

    # Add to PATH if not already there
    if ! echo $PATH | grep -q ~/.foundry/bin; then
        echo 'export PATH="$HOME/.foundry/bin:$PATH"' >> ~/.bashrc
        echo 'export PATH="$HOME/.foundry/bin:$PATH"' >> ~/.zshrc 2>/dev/null || true
        export PATH="$HOME/.foundry/bin:$PATH"
    fi
    echo "✅ Foundry installed successfully"
fi

# 3. Check and install Node.js (for demo and testing)
echo "📦 Checking Node.js installation..."
if command_exists node; then
    NODE_VERSION=$(node --version)
    echo "✅ Node.js found: $NODE_VERSION"
else
    echo "📥 Installing Node.js..."
    if [[ "$OS" == "Darwin" ]]; then
        install_with_brew node
    elif [[ "$OS" == "Linux" ]]; then
        # Install Node.js LTS
        curl -fsSL https://deb.nodesource.com/setup_lts.x | sudo -E bash -
        install_with_apt nodejs
    fi
    echo "✅ Node.js installed successfully"
fi

# 4. Check and install additional tools
echo "🔧 Checking additional tools..."

# Git
if ! command_exists git; then
    echo "📥 Installing Git..."
    if [[ "$OS" == "Darwin" ]]; then
        install_with_brew git
    elif [[ "$OS" == "Linux" ]]; then
        install_with_apt git
    fi
fi

# curl
if ! command_exists curl; then
    echo "📥 Installing curl..."
    if [[ "$OS" == "Darwin" ]]; then
        install_with_brew curl
    elif [[ "$OS" == "Linux" ]]; then
        install_with_apt curl
    fi
fi

# jq (for JSON processing)
if ! command_exists jq; then
    echo "📥 Installing jq..."
    if [[ "$OS" == "Darwin" ]]; then
        install_with_brew jq
    elif [[ "$OS" == "Linux" ]]; then
        install_with_apt jq
    fi
fi

# Docker
if ! command_exists docker; then
    echo "📥 Installing Docker..."
    if [[ "$OS" == "Darwin" ]]; then
        echo "⚠️  Please install Docker Desktop from: https://www.docker.com/products/docker-desktop"
        echo "   Or use: brew install --cask docker"
    elif [[ "$OS" == "Linux" ]]; then
        # Install Docker
        curl -fsSL https://get.docker.com -o get-docker.sh
        sudo sh get-docker.sh
        sudo usermod -aG docker $USER
        echo "⚠️  Please log out and log back in for Docker group changes to take effect"
    fi
fi

# 5. Clone and setup project
echo "📂 Setting up project..."
if [ ! -d ".git" ]; then
    echo "❌ This script should be run from the project root directory"
    echo "   If you haven't cloned the project yet, run:"
    echo "   git clone <repository-url> && cd super-relay"
    exit 1
fi

# Initialize submodules
echo "📦 Initializing git submodules..."
git submodule update --init --recursive

# 6. Build the project
echo "🔨 Building the project..."
cargo build --release

# 7. Create necessary directories
echo "📁 Creating necessary directories..."
mkdir -p config
mkdir -p scripts
mkdir -p logs
mkdir -p data

# 8. Generate default configuration files
echo "⚙️  Generating default configuration files..."

# Create .env file for environment variables
cat > .env << 'EOF'
# SuperPaymaster Environment Configuration
# Copy this to .env.local and modify for your environment

# Network Configuration
RPC_URL=http://localhost:8545
CHAIN_ID=31337

# Paymaster Configuration
PAYMASTER_PRIVATE_KEY=0x59c6995e998f97a5a0044966f0945389dc9e86dae88c6a2440f60b6c4b9f78c2
PAYMASTER_ADDRESS=0x70997970C51812dc3A010C7d01b50e0d17dc79C8

# API Configuration
HTTP_API_HOST=0.0.0.0
HTTP_API_PORT=3000

# EntryPoint Configuration (will be updated after deployment)
ENTRY_POINT_ADDRESS=

# Logging
RUST_LOG=debug

# Database (if needed for production)
# DATABASE_URL=postgresql://user:password@localhost/super_relay

# Security (for production)
# ENABLE_CORS=true
# ALLOWED_ORIGINS=https://your-frontend.com

# Monitoring (for production)
# METRICS_PORT=9090
# HEALTH_CHECK_PORT=8080
EOF

echo "📝 Created .env file with default values"
echo "⚠️  Please copy .env to .env.local and update with your actual values"

# 9. Install demo dependencies
echo "📦 Installing demo project dependencies..."
if [ -d "demo" ]; then
    cd demo
    npm install
    cd ..
fi

# 10. Final checks
echo "✅ Development environment setup complete!"
echo ""
echo "🎯 Next Steps:"
echo "1. Copy .env to .env.local and configure your settings"
echo "2. Start local blockchain: ./scripts/start_anvil.sh"
echo "3. Deploy contracts: ./scripts/deploy_contracts.sh"
echo "4. Run tests: ./scripts/test_all.sh"
echo "5. Start super-relay: ./scripts/start_super_relay.sh"
echo ""
echo "📚 Available Scripts:"
echo "  ./scripts/start_anvil.sh       - Start local Anvil blockchain"
echo "  ./scripts/deploy_contracts.sh  - Deploy all required contracts"
echo "  ./scripts/test_rundler.sh      - Test original Rundler functionality"
echo "  ./scripts/test_super_relay.sh  - Test Super-Relay functionality"
echo "  ./scripts/start_super_relay.sh - Start Super-Relay server"
echo "  ./scripts/build_release.sh     - Build production release"
echo "  ./scripts/docker_build.sh      - Build Docker image"
echo "  ./scripts/start_swagger.sh     - Start API documentation server"
echo ""
echo "🔗 Useful Links:"
echo "  Documentation: ./docs/"
echo "  Demo Project: ./demo/"
echo "  Configuration: ./config/"
echo ""
echo "❓ Need Help?"
echo "  Check ./docs/Deploy.md for detailed deployment instructions"
echo "  Run './scripts/test_integration.sh' for a complete system test"