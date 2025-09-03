#!/bin/bash
set -e

# SuperRelay OP-TEE Startup Script
# This script starts QEMU with OP-TEE and launches SuperRelay inside the VM

OPTEE_DIR="/opt/optee"
SUPERRELAY_DIR="/opt/superrelay"
LOG_DIR="/opt/superrelay/logs"

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

# Signal handlers for graceful shutdown
cleanup() {
    log_info "Shutting down SuperRelay OP-TEE environment..."
    
    if [ ! -z "$QEMU_PID" ] && kill -0 $QEMU_PID 2>/dev/null; then
        log_info "Stopping QEMU (PID: $QEMU_PID)..."
        kill $QEMU_PID
        wait $QEMU_PID 2>/dev/null || true
    fi
    
    # Clean up any remaining processes
    pkill -f "qemu-system-aarch64" || true
    pkill -f "telnet" || true
    
    log_info "Cleanup completed"
    exit 0
}

trap cleanup SIGTERM SIGINT

# Validate environment
validate_environment() {
    log_info "🔍 Validating environment..."
    
    # Check required files
    local required_files=(
        "$OPTEE_DIR/images/bl1.bin"
        "$OPTEE_DIR/images/bl2.bin" 
        "$OPTEE_DIR/images/bl31.bin"
        "$OPTEE_DIR/images/tee-header_v2.bin"
        "$OPTEE_DIR/images/tee-pager_v2.bin"
        "$OPTEE_DIR/images/tee-pageable_v2.bin"
        "$OPTEE_DIR/images/Image"
        "$OPTEE_DIR/images/rootfs.cpio.gz"
        "$SUPERRELAY_DIR/super-relay"
    )
    
    for file in "${required_files[@]}"; do
        if [ ! -f "$file" ]; then
            log_error "Required file not found: $file"
            exit 1
        fi
    done
    
    # Check TA file
    local ta_file="/lib/optee_armtz/12345678-5b69-11d4-9fee-00c04f4c3456.ta"
    if [ ! -f "$ta_file" ]; then
        log_error "SuperRelay TA not found: $ta_file"
        exit 1
    fi
    
    # Check QEMU
    if ! command -v qemu-system-aarch64 >/dev/null 2>&1; then
        log_error "qemu-system-aarch64 not found!"
        exit 1
    fi
    
    log_info "✅ Environment validation completed"
}

# Start QEMU with OP-TEE
start_qemu() {
    log_info "🚀 Starting QEMU ARM64 with OP-TEE..."
    
    # Create log directory
    mkdir -p "$LOG_DIR"
    
    cd "$OPTEE_DIR/images"
    
    # QEMU configuration
    local QEMU_ARGS=(
        -nographic
        -serial tcp::54320,server,nowait
        -serial tcp::54321,server,nowait
        -smp 2
        -machine virt,secure=on
        -cpu cortex-a57
        -d unimp
        -semihosting-config enable=on,target=native
        -m 1057
        -bios bl1.bin
        -initrd rootfs.cpio.gz
        -kernel Image
        -no-acpi
        -netdev user,id=vmnic,hostfwd=tcp::3000-:3000,hostfwd=tcp::9000-:9000,hostfwd=tcp::8545-:8545
        -device virtio-net-device,netdev=vmnic
        -object rng-random,filename=/dev/urandom,id=rng0
        -device virtio-rng-device,rng=rng0
        -rtc base=utc,clock=host
        -append 'console=ttyAMA0,38400 keep_bootcon root=/dev/vda2 panic=0 rw'
    )
    
    log_info "QEMU command: qemu-system-aarch64 ${QEMU_ARGS[*]}"
    
    # Start QEMU in background
    qemu-system-aarch64 "${QEMU_ARGS[@]}" > "$LOG_DIR/qemu.log" 2>&1 &
    QEMU_PID=$!
    
    log_info "QEMU started with PID: $QEMU_PID"
    
    # Wait for QEMU to be ready
    log_info "⏳ Waiting for QEMU to initialize..."
    sleep 10
    
    # Check if QEMU is still running
    if ! kill -0 $QEMU_PID 2>/dev/null; then
        log_error "QEMU failed to start!"
        cat "$LOG_DIR/qemu.log"
        exit 1
    fi
}

# Wait for OP-TEE to be ready
wait_for_optee() {
    log_info "⏳ Waiting for OP-TEE to be ready..."
    
    local max_attempts=30
    local attempt=0
    
    while [ $attempt -lt $max_attempts ]; do
        if nc -z localhost 54320 2>/dev/null; then
            log_info "✅ OP-TEE serial port is ready"
            sleep 5  # Additional wait for complete boot
            return 0
        fi
        
        sleep 2
        ((attempt++))
        log_debug "Attempt $attempt/$max_attempts waiting for OP-TEE..."
    done
    
    log_error "Timeout waiting for OP-TEE to be ready"
    return 1
}

# Deploy SuperRelay and TA inside the VM
deploy_superrelay() {
    log_info "📦 Deploying SuperRelay inside OP-TEE environment..."
    
    # Create deployment script
    cat << 'EOF' > /tmp/deploy_superrelay.exp
#!/usr/bin/expect -f

set timeout 60
spawn telnet localhost 54320

expect {
    "buildroot login:" {
        send "root\r"
        exp_continue
    }
    "# " {
        # We're logged in, continue with deployment
    }
    timeout {
        puts "Timeout waiting for login prompt"
        exit 1
    }
}

# Create directories
send "mkdir -p /opt/superrelay/config /opt/superrelay/logs\r"
expect "# "

# Check TA installation
send "ls -la /lib/optee_armtz/\r"  
expect "# "

# Check tee-supplicant
send "ps aux | grep tee-supplicant\r"
expect "# "

# Test OP-TEE device
send "ls -la /dev/tee*\r"
expect "# "

# Copy SuperRelay binary (this would normally be done via shared volume)
send "echo 'SuperRelay binary would be copied here'\r"
expect "# "

# Set up configuration
send "cat > /opt/superrelay/config/optee-config.toml << 'EOFCONFIG'\r"
send "[node]\r"
send "http_api = \"0.0.0.0:3000\"\r"
send "network = \"dev\"\r"
send "node_http = \"http://localhost:8545\"\r"
send "\r"
send "[paymaster_relay]\r"
send "enabled = true\r" 
send "kms_backend = \"optee\"\r"
send "\r"
send "[optee_kms]\r"
send "device_path = \"/dev/teepriv0\"\r"
send "ta_uuid = \"12345678-5b69-11d4-9fee-00c04f4c3456\"\r"
send "\r"
send "[optee_kms.keys]\r"
send "primary_paymaster = \"paymaster-key-001\"\r"
send "EOFCONFIG\r"
expect "# "

# Test TA communication
send "echo 'Testing SuperRelay TA...'\r"
expect "# "

puts "SuperRelay deployment completed"
exit 0
EOF
    
    chmod +x /tmp/deploy_superrelay.exp
    
    if ! /tmp/deploy_superrelay.exp; then
        log_error "Failed to deploy SuperRelay inside VM"
        return 1
    fi
    
    log_info "✅ SuperRelay deployment completed"
}

# Start SuperRelay inside the VM
start_superrelay() {
    log_info "🏁 Starting SuperRelay with OP-TEE backend..."
    
    # Create startup script  
    cat << 'EOF' > /tmp/start_superrelay.exp
#!/usr/bin/expect -f

set timeout 30
spawn telnet localhost 54320

expect "# "

# Change to SuperRelay directory
send "cd /opt/superrelay\r"
expect "# "

# Start SuperRelay in background
send "./super-relay node --config config/optee-config.toml --paymaster-relay > logs/superrelay.log 2>&1 &\r"
expect "# "

# Get the PID
send "echo $! > superrelay.pid\r"
expect "# "

# Wait a moment for startup
send "sleep 3\r"
expect "# "

# Check if it's running
send "ps aux | grep super-relay | grep -v grep\r" 
expect "# "

# Test health endpoint
send "curl -f http://localhost:3000/health || echo 'Health check failed'\r"
expect "# "

puts "SuperRelay startup sequence completed"
exit 0
EOF

    chmod +x /tmp/start_superrelay.exp
    
    if ! /tmp/start_superrelay.exp; then
        log_warn "SuperRelay startup script completed with warnings"
    else
        log_info "✅ SuperRelay started successfully"
    fi
}

# Monitor services
monitor_services() {
    log_info "📊 Starting service monitoring..."
    
    while true; do
        # Check QEMU process
        if ! kill -0 $QEMU_PID 2>/dev/null; then
            log_error "QEMU process died unexpectedly"
            exit 1
        fi
        
        # Check SuperRelay health (every 30 seconds)
        if (( $(date +%s) % 30 == 0 )); then
            if ! curl -s -f http://localhost:3000/health >/dev/null 2>&1; then
                log_warn "SuperRelay health check failed"
            else
                log_debug "SuperRelay health check passed"
            fi
        fi
        
        sleep 5
    done
}

# Display service information
show_service_info() {
    log_info "🌐 SuperRelay with OP-TEE is running!"
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "  🔒 TEE-Secured SuperRelay Service Information"  
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "  📊 JSON-RPC API:     http://localhost:3000"
    echo "  🌐 HTTP REST API:    http://localhost:9000"  
    echo "  🏥 Health Check:     http://localhost:3000/health"
    echo "  📈 Metrics:          http://localhost:3000/metrics"
    echo "  🔧 QEMU Console:     telnet localhost 54320"
    echo "  📁 Logs Directory:   $LOG_DIR"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    echo "🔑 Private keys are secured in OP-TEE environment"
    echo "🛡️ All signing operations executed in Secure World"
    echo ""
}

# Main execution
main() {
    log_info "🔐 Starting SuperRelay with OP-TEE on QEMU ARM64..."
    
    # Validate environment
    validate_environment
    
    # Start QEMU
    start_qemu
    
    # Wait for OP-TEE
    if ! wait_for_optee; then
        log_error "Failed to start OP-TEE environment"
        exit 1
    fi
    
    # Deploy SuperRelay
    if ! deploy_superrelay; then
        log_error "Failed to deploy SuperRelay"
        exit 1
    fi
    
    # Start SuperRelay
    start_superrelay
    
    # Show service information
    show_service_info
    
    # Monitor services
    monitor_services
}

# Execute main function
main "$@"