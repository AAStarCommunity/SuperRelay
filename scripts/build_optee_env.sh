#!/bin/bash
# SuperRelay OP-TEE Environment Builder
# Phase 1: Docker + QEMU + OP-TEE Setup Script

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
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

# Configuration
DOCKER_IMAGE_NAME="superrelay-optee"
DOCKER_TAG="phase1-dev"
CONTAINER_NAME="superrelay-optee-dev"

# Usage information
usage() {
    cat << EOF
SuperRelay OP-TEE Environment Builder

Usage: $0 [OPTIONS] [COMMAND]

Commands:
    build       Build Docker image with OP-TEE
    run         Start development container
    clean       Clean up containers and images
    logs        Show container logs
    shell       Enter container shell
    status      Show container status
    rebuild     Clean and rebuild everything

Options:
    -h, --help          Show this help message
    -v, --verbose       Enable verbose output
    -d, --detach        Run container in detached mode
    --no-cache          Build without using cache
    --dev-mode          Enable development mode (bind mounts)

Examples:
    $0 build                    # Build the OP-TEE image
    $0 run --dev-mode          # Start with development bind mounts
    $0 shell                   # Enter running container
    $0 rebuild --no-cache      # Full rebuild without cache

Environment Variables:
    OPTEE_DEBUG_LEVEL          OP-TEE debug level (default: 2)
    SUPERRELAY_CONFIG          SuperRelay config file path
    DOCKER_BUILDKIT           Enable Docker BuildKit (recommended: 1)

EOF
}

# Validation functions
check_dependencies() {
    log_info "🔍 Checking dependencies..."

    local missing_deps=()

    # Check Docker
    if ! command -v docker >/dev/null 2>&1; then
        missing_deps+=("docker")
    fi

    # Check Docker Compose (optional but recommended)
    if ! command -v docker-compose >/dev/null 2>&1; then
        log_warn "Docker Compose not found (optional)"
    fi

    # Check system requirements
    if [[ "$(uname)" == "Darwin" ]]; then
        # macOS specific checks
        if ! docker info | grep -q "Server"; then
            missing_deps+=("docker-daemon")
        fi
    elif [[ "$(uname)" == "Linux" ]]; then
        # Linux specific checks
        if ! groups | grep -q docker; then
            log_warn "Current user not in docker group. You may need to use sudo or add user to docker group."
        fi
    fi

    if [ ${#missing_deps[@]} -ne 0 ]; then
        log_error "Missing dependencies: ${missing_deps[*]}"
        log_error "Please install missing dependencies and try again"
        exit 1
    fi

    log_info "✅ Dependencies check passed"
}

# Build functions
build_optee_image() {
    local no_cache=""
    local verbose=""

    if [[ "$1" == "--no-cache" ]]; then
        no_cache="--no-cache"
        log_info "Building without cache"
    fi

    if [[ "$VERBOSE" == "true" ]]; then
        verbose="--progress=plain"
    fi

    log_info "🏗️ Building SuperRelay OP-TEE Docker image..."
    log_info "Image: ${DOCKER_IMAGE_NAME}:${DOCKER_TAG}"

    cd "$PROJECT_ROOT"

    # Enable BuildKit for better performance
    export DOCKER_BUILDKIT=1

    # Build arguments
    local build_args=(
        --file docker/Dockerfile.optee-qemu
        --tag "${DOCKER_IMAGE_NAME}:${DOCKER_TAG}"
        --tag "${DOCKER_IMAGE_NAME}:latest"
        ${no_cache}
        ${verbose}
        --build-arg OPTEE_VERSION=3.22.0
        --build-arg RUST_VERSION=1.70
        --build-arg BUILD_DATE="$(date -u +'%Y-%m-%dT%H:%M:%SZ')"
        --build-arg VCS_REF="$(git rev-parse --short HEAD 2>/dev/null || echo 'unknown')"
        .
    )

    if ! docker build "${build_args[@]}"; then
        log_error "Docker build failed!"
        exit 1
    fi

    log_info "✅ Docker image built successfully"

    # Show image information
    docker images "${DOCKER_IMAGE_NAME}" --format "table {{.Repository}}\t{{.Tag}}\t{{.Size}}\t{{.CreatedAt}}"
}

# Container management functions
start_container() {
    local detach_mode=""
    local dev_mode="false"

    while [[ $# -gt 0 ]]; do
        case $1 in
            -d|--detach)
                detach_mode="-d"
                shift
                ;;
            --dev-mode)
                dev_mode="true"
                shift
                ;;
            *)
                shift
                ;;
        esac
    done

    log_info "🚀 Starting SuperRelay OP-TEE container..."

    # Stop existing container if running
    if docker ps -q --filter "name=${CONTAINER_NAME}" | grep -q .; then
        log_info "Stopping existing container..."
        docker stop "${CONTAINER_NAME}" >/dev/null
    fi

    # Remove existing container if exists
    if docker ps -aq --filter "name=${CONTAINER_NAME}" | grep -q .; then
        log_info "Removing existing container..."
        docker rm "${CONTAINER_NAME}" >/dev/null
    fi

    # Container run arguments
    local run_args=(
        ${detach_mode}
        --name "${CONTAINER_NAME}"
        --privileged
        --restart unless-stopped
        -p 3000:3000        # JSON-RPC API
        -p 9000:9000        # HTTP REST API
        -p 8545:8545        # Anvil (if used)
        -p 54320:54320      # QEMU console
        -p 54321:54321      # QEMU monitor
    )

    # Development mode bind mounts
    if [[ "$dev_mode" == "true" ]]; then
        log_info "Enabling development mode with bind mounts"
        run_args+=(
            -v "${PROJECT_ROOT}/config:/opt/superrelay/config:ro"
            -v "${PROJECT_ROOT}/logs:/opt/superrelay/logs"
            -v "${PROJECT_ROOT}/ta:/opt/ta:ro"
        )
    fi

    # Environment variables
    run_args+=(
        -e "OPTEE_DEBUG_LEVEL=${OPTEE_DEBUG_LEVEL:-2}"
        -e "RUST_LOG=${RUST_LOG:-info}"
        -e "RUST_BACKTRACE=1"
    )

    # Start container
    if ! docker run "${run_args[@]}" "${DOCKER_IMAGE_NAME}:${DOCKER_TAG}"; then
        log_error "Failed to start container"
        exit 1
    fi

    if [[ -z "$detach_mode" ]]; then
        log_info "✅ Container started in interactive mode"
    else
        log_info "✅ Container started in detached mode"
        show_container_info
    fi
}

show_container_info() {
    log_info "📊 Container Information:"
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "  🔒 SuperRelay OP-TEE Development Environment"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "  📊 JSON-RPC API:     http://localhost:3000"
    echo "  🌐 HTTP REST API:    http://localhost:9000"
    echo "  🏥 Health Check:     http://localhost:3000/health"
    echo "  📈 Metrics:          http://localhost:3000/metrics"
    echo "  🔧 QEMU Console:     telnet localhost 54320"
    echo "  📁 Container Name:   ${CONTAINER_NAME}"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    echo "🔑 Private keys are secured in OP-TEE environment"
    echo "🛡️ All signing operations executed in Secure World"
    echo ""

    # Show container status
    docker ps --filter "name=${CONTAINER_NAME}" --format "table {{.Names}}\t{{.Status}}\t{{.Ports}}"
}

enter_shell() {
    log_info "🐚 Entering container shell..."

    if ! docker ps --filter "name=${CONTAINER_NAME}" --filter "status=running" -q | grep -q .; then
        log_error "Container ${CONTAINER_NAME} is not running"
        log_info "Start the container first with: $0 run"
        exit 1
    fi

    docker exec -it "${CONTAINER_NAME}" /bin/bash
}

show_logs() {
    local follow=""
    local lines="100"

    while [[ $# -gt 0 ]]; do
        case $1 in
            -f|--follow)
                follow="-f"
                shift
                ;;
            -n|--lines)
                lines="$2"
                shift 2
                ;;
            *)
                shift
                ;;
        esac
    done

    log_info "📋 Showing container logs (last ${lines} lines)..."

    if ! docker ps -a --filter "name=${CONTAINER_NAME}" -q | grep -q .; then
        log_error "Container ${CONTAINER_NAME} does not exist"
        exit 1
    fi

    docker logs ${follow} --tail "${lines}" "${CONTAINER_NAME}"
}

show_status() {
    log_info "📊 Container Status:"

    if docker ps --filter "name=${CONTAINER_NAME}" --filter "status=running" -q | grep -q .; then
        echo "🟢 Container is running"
        show_container_info

        # Test connectivity
        echo ""
        log_info "🔗 Testing connectivity..."

        if curl -s -f http://localhost:3000/health >/dev/null 2>&1; then
            echo "✅ SuperRelay API is responding"
        else
            echo "❌ SuperRelay API is not responding"
        fi

    elif docker ps -a --filter "name=${CONTAINER_NAME}" -q | grep -q .; then
        echo "🟡 Container exists but is not running"
        docker ps -a --filter "name=${CONTAINER_NAME}" --format "table {{.Names}}\t{{.Status}}\t{{.Ports}}"
    else
        echo "🔴 Container does not exist"
        log_info "Create and start with: $0 run"
    fi
}

clean_environment() {
    log_info "🧹 Cleaning SuperRelay OP-TEE environment..."

    # Stop and remove container
    if docker ps -q --filter "name=${CONTAINER_NAME}" | grep -q .; then
        log_info "Stopping container..."
        docker stop "${CONTAINER_NAME}" >/dev/null
    fi

    if docker ps -aq --filter "name=${CONTAINER_NAME}" | grep -q .; then
        log_info "Removing container..."
        docker rm "${CONTAINER_NAME}" >/dev/null
    fi

    # Remove images
    local remove_images="false"
    read -p "Remove Docker images? [y/N]: " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        remove_images="true"
    fi

    if [[ "$remove_images" == "true" ]]; then
        log_info "Removing Docker images..."
        docker rmi "${DOCKER_IMAGE_NAME}:${DOCKER_TAG}" 2>/dev/null || true
        docker rmi "${DOCKER_IMAGE_NAME}:latest" 2>/dev/null || true
    fi

    # Clean dangling images
    local dangling_images
    dangling_images=$(docker images -f "dangling=true" -q)
    if [[ -n "$dangling_images" ]]; then
        log_info "Cleaning dangling images..."
        docker rmi $dangling_images 2>/dev/null || true
    fi

    log_info "✅ Environment cleaned"
}

# Main execution logic
main() {
    local command=""
    local verbose="false"
    local no_cache=""
    local dev_mode=""
    local detach=""

    # Parse arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            -h|--help)
                usage
                exit 0
                ;;
            -v|--verbose)
                verbose="true"
                shift
                ;;
            --no-cache)
                no_cache="--no-cache"
                shift
                ;;
            --dev-mode)
                dev_mode="--dev-mode"
                shift
                ;;
            -d|--detach)
                detach="--detach"
                shift
                ;;
            build|run|clean|logs|shell|status|rebuild)
                command="$1"
                shift
                ;;
            *)
                log_error "Unknown option: $1"
                usage
                exit 1
                ;;
        esac
    done

    export VERBOSE="$verbose"

    # Default command
    if [[ -z "$command" ]]; then
        command="build"
    fi

    # Check dependencies
    check_dependencies

    # Execute command
    case "$command" in
        build)
            build_optee_image $no_cache
            ;;
        run)
            start_container $detach $dev_mode
            ;;
        shell)
            enter_shell
            ;;
        logs)
            show_logs "$@"
            ;;
        status)
            show_status
            ;;
        clean)
            clean_environment
            ;;
        rebuild)
            clean_environment
            build_optee_image $no_cache
            ;;
        *)
            log_error "Unknown command: $command"
            usage
            exit 1
            ;;
    esac
}

# Trap signals for cleanup
trap 'log_info "Build script interrupted"; exit 1' INT TERM

# Execute main function
main "$@"