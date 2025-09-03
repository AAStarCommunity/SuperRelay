#!/bin/bash
# SuperRelay OP-TEE Three-Phase Build and Deployment Script
# Builds all components for Docker, Cloud, and Hardware deployment

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
MAGENTA='\033[0;35m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

log_debug() {
    echo -e "${BLUE}[DEBUG]${NC} $1"
}

log_header() {
    echo -e "${CYAN}${1}${NC}"
}

log_phase() {
    echo -e "${MAGENTA}=== $1 ===${NC}"
}

# Configuration
RUST_VERSION="1.70"
OPTEE_VERSION="3.22.0"
QEMU_ARCH="aarch64"
TARGET_ARCH="aarch64-unknown-linux-gnu"

# Build phases
PHASE_1_BUILD="false"
PHASE_2_BUILD="false"
PHASE_3_BUILD="false"
BUILD_TA="false"
VERBOSE="false"
CLEAN_BUILD="false"

# Usage information
usage() {
    cat << EOF
SuperRelay OP-TEE Three-Phase Builder

Usage: $0 [OPTIONS] [PHASES]

Phases:
    phase1      Build Phase 1: Docker + QEMU + OP-TEE
    phase2      Build Phase 2: Cloud ARM platform  
    phase3      Build Phase 3: NXP i.MX 93 hardware
    ta          Build Trusted Application only
    all         Build all phases (default)

Options:
    -h, --help          Show this help message
    -v, --verbose       Enable verbose output
    -c, --clean         Clean build (remove existing artifacts)
    --rust-version VER  Rust version to use (default: ${RUST_VERSION})
    --optee-version VER OP-TEE version to use (default: ${OPTEE_VERSION})
    --target-arch ARCH  Target architecture (default: ${TARGET_ARCH})

Examples:
    $0 all                      # Build all phases
    $0 phase1 phase2            # Build phases 1 and 2
    $0 ta                       # Build TA only
    $0 --clean all              # Clean build all phases
    $0 --verbose phase3         # Verbose build of phase 3

Environment Variables:
    CARGO_BUILD_JOBS           Number of parallel build jobs
    RUST_TARGET_PATH          Custom target path
    OPTEE_PLATFORM            OP-TEE platform (default: vexpress-qemu_virt)

EOF
}

# Validation functions
check_build_dependencies() {
    log_info "🔍 Checking build dependencies..."
    
    local missing_deps=()
    
    # Check Rust
    if ! command -v rustc >/dev/null 2>&1; then
        missing_deps+=("rust")
    else
        local rust_version=$(rustc --version | awk '{print $2}')
        log_debug "Found Rust: $rust_version"
    fi
    
    # Check Cargo
    if ! command -v cargo >/dev/null 2>&1; then
        missing_deps+=("cargo")
    fi
    
    # Check cross-compilation tools
    if ! command -v aarch64-linux-gnu-gcc >/dev/null 2>&1; then
        log_warn "aarch64-linux-gnu-gcc not found - may need for cross compilation"
    fi
    
    # Check Docker (for Phase 1)
    if [[ "$PHASE_1_BUILD" == "true" ]] && ! command -v docker >/dev/null 2>&1; then
        missing_deps+=("docker")
    fi
    
    # Check build tools
    local build_tools=("make" "cmake" "pkg-config")
    for tool in "${build_tools[@]}"; do
        if ! command -v "$tool" >/dev/null 2>&1; then
            log_warn "$tool not found - may be needed for some builds"
        fi
    done
    
    if [ ${#missing_deps[@]} -ne 0 ]; then
        log_error "Missing dependencies: ${missing_deps[*]}"
        log_error "Please install missing dependencies and try again"
        exit 1
    fi
    
    log_info "✅ Build dependencies check passed"
}

# Setup build environment
setup_build_environment() {
    log_info "⚙️  Setting up build environment..."
    
    # Add ARM64 target if not present
    if ! rustup target list --installed | grep -q "$TARGET_ARCH"; then
        log_info "Adding Rust target: $TARGET_ARCH"
        rustup target add "$TARGET_ARCH"
    fi
    
    # Set cross-compilation environment
    export CC_aarch64_unknown_linux_gnu=aarch64-linux-gnu-gcc
    export CXX_aarch64_unknown_linux_gnu=aarch64-linux-gnu-g++
    export AR_aarch64_unknown_linux_gnu=aarch64-linux-gnu-ar
    export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc
    
    # Set build parallelism
    export CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-$(nproc)}"
    
    # Enable OP-TEE features
    export RUSTFLAGS="${RUSTFLAGS} -C target-cpu=cortex-a55"
    
    log_info "✅ Build environment configured"
    log_debug "Target: $TARGET_ARCH"
    log_debug "Build jobs: $CARGO_BUILD_JOBS"
}

# Clean build artifacts
clean_build_artifacts() {
    if [[ "$CLEAN_BUILD" == "true" ]]; then
        log_info "🧹 Cleaning build artifacts..."
        
        # Clean Rust artifacts
        cargo clean
        
        # Clean TA build artifacts
        if [ -d "$PROJECT_ROOT/ta/super_relay_ta" ]; then
            make -C "$PROJECT_ROOT/ta/super_relay_ta" clean 2>/dev/null || true
        fi
        
        # Clean Docker artifacts (Phase 1)
        if [[ "$PHASE_1_BUILD" == "true" ]]; then
            docker system prune -f >/dev/null 2>&1 || true
        fi
        
        log_info "✅ Build artifacts cleaned"
    fi
}

# Build Trusted Application
build_trusted_application() {
    if [[ "$BUILD_TA" == "true" ]]; then
        log_phase "Building SuperRelay Trusted Application"
        
        local ta_dir="$PROJECT_ROOT/ta/super_relay_ta"
        
        if [ ! -d "$ta_dir" ]; then
            log_error "TA directory not found: $ta_dir"
            return 1
        fi
        
        cd "$ta_dir"
        
        # Set OP-TEE build environment
        export PLATFORM="${OPTEE_PLATFORM:-vexpress-qemu_virt}"
        export CROSS_COMPILE="aarch64-linux-gnu-"
        
        # Check for OP-TEE development kit
        if [ -z "$TA_DEV_KIT_DIR" ]; then
            log_warn "TA_DEV_KIT_DIR not set - using default"
            export TA_DEV_KIT_DIR="/opt/optee/optee_os/out/arm/export-ta_arm64"
        fi
        
        log_info "Building TA with platform: $PLATFORM"
        
        # Build the TA
        if ! make CROSS_COMPILE="$CROSS_COMPILE" PLATFORM="$PLATFORM" -j"$CARGO_BUILD_JOBS"; then
            log_error "TA build failed"
            return 1
        fi
        
        # Verify TA was built
        local ta_file="out/12345678-5b69-11d4-9fee-00c04f4c3456.ta"
        if [ -f "$ta_file" ]; then
            local ta_size=$(stat -f%z "$ta_file" 2>/dev/null || stat -c%s "$ta_file" 2>/dev/null || echo "unknown")
            log_info "✅ TA built successfully (size: $ta_size bytes)"
            log_debug "TA file: $PWD/$ta_file"
        else
            log_error "❌ TA build failed - output file not found"
            return 1
        fi
        
        cd "$PROJECT_ROOT"
        log_info "✅ Trusted Application build completed"
    fi
}

# Build SuperRelay for specific phase
build_superrelay() {
    local phase="$1"
    local features="$2"
    local profile="$3"
    
    log_info "🏗️  Building SuperRelay for $phase..."
    log_info "Features: $features"
    log_info "Profile: $profile"
    
    cd "$PROJECT_ROOT"
    
    local cargo_args=(
        "build"
        "--target=$TARGET_ARCH"
        "--profile=$profile"
    )
    
    if [[ -n "$features" ]]; then
        cargo_args+=("--features=$features")
    fi
    
    if [[ "$VERBOSE" == "true" ]]; then
        cargo_args+=("--verbose")
    fi
    
    # Build SuperRelay
    if ! cargo "${cargo_args[@]}"; then
        log_error "SuperRelay build failed for $phase"
        return 1
    fi
    
    # Verify binary
    local binary_path="target/$TARGET_ARCH/$profile/super-relay"
    if [ -f "$binary_path" ]; then
        local binary_size=$(stat -f%z "$binary_path" 2>/dev/null || stat -c%s "$binary_path" 2>/dev/null || echo "unknown")
        log_info "✅ SuperRelay built successfully (size: $binary_size bytes)"
        log_debug "Binary: $PWD/$binary_path"
    else
        log_error "❌ SuperRelay build failed - binary not found"
        return 1
    fi
    
    log_info "✅ SuperRelay build completed for $phase"
}

# Phase 1: Docker + QEMU + OP-TEE
build_phase1() {
    log_phase "Phase 1: Docker + QEMU + OP-TEE Build"
    
    # Build TA
    BUILD_TA="true"
    build_trusted_application
    
    # Build SuperRelay with OP-TEE features for development
    build_superrelay "Phase 1" "optee-kms" "dev"
    
    # Build Docker image
    log_info "🐳 Building Docker image for Phase 1..."
    
    cd "$PROJECT_ROOT"
    
    if ! "$SCRIPT_DIR/build_optee_env.sh" build; then
        log_error "Docker image build failed"
        return 1
    fi
    
    log_info "✅ Phase 1 build completed"
    
    # Test instructions
    log_info "🧪 To test Phase 1:"
    log_info "   $SCRIPT_DIR/build_optee_env.sh run --dev-mode"
    log_info "   curl http://localhost:3000/health"
}

# Phase 2: Cloud ARM platform
build_phase2() {
    log_phase "Phase 2: Cloud ARM Platform Build"
    
    # Build TA for cloud deployment
    BUILD_TA="true"
    build_trusted_application
    
    # Build SuperRelay with optimizations for cloud
    build_superrelay "Phase 2" "optee-kms" "release"
    
    # Build cloud-optimized Docker image
    log_info "🌐 Building cloud Docker image for Phase 2..."
    
    cd "$PROJECT_ROOT"
    
    # Create cloud-specific Dockerfile if it doesn't exist
    local cloud_dockerfile="docker/Dockerfile.optee-cloud"
    if [ ! -f "$cloud_dockerfile" ]; then
        log_info "Creating cloud-specific Dockerfile..."
        sed 's/phase1-dev/phase2-cloud/g' docker/Dockerfile.optee-qemu > "$cloud_dockerfile"
    fi
    
    # Build cloud image
    docker build -f "$cloud_dockerfile" -t superrelay-optee:phase2-cloud . || {
        log_error "Cloud Docker image build failed"
        return 1
    }
    
    log_info "✅ Phase 2 build completed"
    
    # Deployment instructions
    log_info "🚀 To deploy Phase 2:"
    log_info "   kubectl apply -f k8s/superrelay-optee-phase2.yaml"
    log_info "   kubectl get pods -n superrelay-optee"
}

# Phase 3: NXP i.MX 93 hardware
build_phase3() {
    log_phase "Phase 3: NXP i.MX 93 Hardware Build"
    
    # Build TA for hardware deployment
    BUILD_TA="true"
    build_trusted_application
    
    # Build SuperRelay with hardware optimizations
    build_superrelay "Phase 3" "optee-kms,imx93-hardware" "release"
    
    # Create hardware deployment package
    log_info "📦 Creating hardware deployment package..."
    
    local deploy_dir="$PROJECT_ROOT/deploy/imx93"
    mkdir -p "$deploy_dir"
    
    # Copy binary
    cp "target/$TARGET_ARCH/release/super-relay" "$deploy_dir/"
    
    # Copy TA
    if [ -f "ta/super_relay_ta/out/12345678-5b69-11d4-9fee-00c04f4c3456.ta" ]; then
        cp "ta/super_relay_ta/out/12345678-5b69-11d4-9fee-00c04f4c3456.ta" "$deploy_dir/"
    fi
    
    # Copy configuration
    cp "config/imx93-config.toml" "$deploy_dir/"
    
    # Copy deployment script
    cp "scripts/deploy_imx93_hardware.sh" "$deploy_dir/"
    chmod +x "$deploy_dir/deploy_imx93_hardware.sh"
    
    # Create deployment archive
    local archive_name="superrelay-imx93-$(date +%Y%m%d_%H%M%S).tar.gz"
    cd "$PROJECT_ROOT/deploy"
    tar -czf "$archive_name" imx93/
    
    log_info "✅ Phase 3 build completed"
    log_info "📦 Deployment package: deploy/$archive_name"
    
    # Deployment instructions
    log_info "🏭 To deploy Phase 3:"
    log_info "   1. Copy package to i.MX 93 board"
    log_info "   2. Extract: tar -xzf $archive_name"
    log_info "   3. Run: sudo ./imx93/deploy_imx93_hardware.sh full-deploy"
}

# Generate build report
generate_build_report() {
    log_header "=== Build Report ==="
    
    local report_file="$PROJECT_ROOT/build_report_$(date +%Y%m%d_%H%M%S).txt"
    
    {
        echo "SuperRelay OP-TEE Build Report"
        echo "Generated: $(date)"
        echo "=============================="
        echo ""
        
        echo "Build Configuration:"
        echo "- Rust version: $(rustc --version 2>/dev/null || echo 'Not found')"
        echo "- Target architecture: $TARGET_ARCH"
        echo "- OP-TEE version: $OPTEE_VERSION"
        echo "- Build jobs: $CARGO_BUILD_JOBS"
        echo ""
        
        echo "Build Results:"
        
        if [[ "$PHASE_1_BUILD" == "true" ]]; then
            echo "- Phase 1 (Docker + QEMU): ✅ Built"
            if [ -f "target/$TARGET_ARCH/dev/super-relay" ]; then
                local size=$(stat -f%z "target/$TARGET_ARCH/dev/super-relay" 2>/dev/null || stat -c%s "target/$TARGET_ARCH/dev/super-relay" 2>/dev/null)
                echo "  Binary size: $size bytes"
            fi
        fi
        
        if [[ "$PHASE_2_BUILD" == "true" ]]; then
            echo "- Phase 2 (Cloud ARM): ✅ Built"
            if [ -f "target/$TARGET_ARCH/release/super-relay" ]; then
                local size=$(stat -f%z "target/$TARGET_ARCH/release/super-relay" 2>/dev/null || stat -c%s "target/$TARGET_ARCH/release/super-relay" 2>/dev/null)
                echo "  Binary size: $size bytes"
            fi
        fi
        
        if [[ "$PHASE_3_BUILD" == "true" ]]; then
            echo "- Phase 3 (i.MX 93 Hardware): ✅ Built"
            if [ -f "target/$TARGET_ARCH/release/super-relay" ]; then
                local size=$(stat -f%z "target/$TARGET_ARCH/release/super-relay" 2>/dev/null || stat -c%s "target/$TARGET_ARCH/release/super-relay" 2>/dev/null)
                echo "  Binary size: $size bytes"
            fi
            if [ -d "deploy/imx93" ]; then
                echo "  Deployment package: ✅ Created"
            fi
        fi
        
        if [[ "$BUILD_TA" == "true" ]]; then
            echo "- Trusted Application: ✅ Built"
            if [ -f "ta/super_relay_ta/out/12345678-5b69-11d4-9fee-00c04f4c3456.ta" ]; then
                local size=$(stat -f%z "ta/super_relay_ta/out/12345678-5b69-11d4-9fee-00c04f4c3456.ta" 2>/dev/null || stat -c%s "ta/super_relay_ta/out/12345678-5b69-11d4-9fee-00c04f4c3456.ta" 2>/dev/null)
                echo "  TA size: $size bytes"
            fi
        fi
        
        echo ""
        echo "Next Steps:"
        if [[ "$PHASE_1_BUILD" == "true" ]]; then
            echo "- Test Phase 1: ./scripts/build_optee_env.sh run"
        fi
        if [[ "$PHASE_2_BUILD" == "true" ]]; then
            echo "- Deploy Phase 2: kubectl apply -f k8s/superrelay-optee-phase2.yaml"
        fi
        if [[ "$PHASE_3_BUILD" == "true" ]]; then
            echo "- Deploy Phase 3: Transfer deploy package to i.MX 93 and run deployment script"
        fi
        
    } > "$report_file"
    
    # Display report
    cat "$report_file"
    log_info "📄 Build report saved: $report_file"
}

# Main execution
main() {
    # Parse arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            -h|--help)
                usage
                exit 0
                ;;
            -v|--verbose)
                VERBOSE="true"
                shift
                ;;
            -c|--clean)
                CLEAN_BUILD="true"
                shift
                ;;
            --rust-version)
                RUST_VERSION="$2"
                shift 2
                ;;
            --optee-version)
                OPTEE_VERSION="$2"
                shift 2
                ;;
            --target-arch)
                TARGET_ARCH="$2"
                shift 2
                ;;
            phase1)
                PHASE_1_BUILD="true"
                shift
                ;;
            phase2)
                PHASE_2_BUILD="true"
                shift
                ;;
            phase3)
                PHASE_3_BUILD="true"
                shift
                ;;
            ta)
                BUILD_TA="true"
                shift
                ;;
            all)
                PHASE_1_BUILD="true"
                PHASE_2_BUILD="true"
                PHASE_3_BUILD="true"
                BUILD_TA="true"
                shift
                ;;
            *)
                log_error "Unknown option: $1"
                usage
                exit 1
                ;;
        esac
    done
    
    # Default to all if no phases specified
    if [[ "$PHASE_1_BUILD" == "false" && "$PHASE_2_BUILD" == "false" && "$PHASE_3_BUILD" == "false" && "$BUILD_TA" == "false" ]]; then
        PHASE_1_BUILD="true"
        PHASE_2_BUILD="true"
        PHASE_3_BUILD="true"
        BUILD_TA="true"
    fi
    
    export VERBOSE
    
    log_header "🚀 SuperRelay OP-TEE Three-Phase Builder"
    
    # Check dependencies
    check_build_dependencies
    
    # Setup environment
    setup_build_environment
    
    # Clean if requested
    clean_build_artifacts
    
    # Execute builds
    if [[ "$PHASE_1_BUILD" == "true" ]]; then
        build_phase1
    fi
    
    if [[ "$PHASE_2_BUILD" == "true" ]]; then
        build_phase2
    fi
    
    if [[ "$PHASE_3_BUILD" == "true" ]]; then
        build_phase3
    fi
    
    if [[ "$BUILD_TA" == "true" && "$PHASE_1_BUILD" == "false" && "$PHASE_2_BUILD" == "false" && "$PHASE_3_BUILD" == "false" ]]; then
        # Build TA only
        build_trusted_application
    fi
    
    # Generate report
    generate_build_report
    
    log_info "🎉 All builds completed successfully!"
}

# Trap signals for cleanup
trap 'log_info "Build script interrupted"; exit 1' INT TERM

# Execute main function
main "$@"