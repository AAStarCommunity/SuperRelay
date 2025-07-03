#!/bin/bash
# Stop Anvil local blockchain

set -e

echo "🛑 Stopping Anvil Local Blockchain"
echo "================================="

# Check if PID file exists
if [ -f ".anvil.pid" ]; then
    ANVIL_PID=$(cat .anvil.pid)
    
    # Check if process is still running
    if kill -0 $ANVIL_PID 2>/dev/null; then
        echo "🔍 Found Anvil process with PID: $ANVIL_PID"
        echo "⏹️  Stopping Anvil..."
        kill $ANVIL_PID
        
        # Wait for process to stop
        local count=0
        while kill -0 $ANVIL_PID 2>/dev/null && [ $count -lt 10 ]; do
            sleep 1
            count=$((count + 1))
        done
        
        # Force kill if still running
        if kill -0 $ANVIL_PID 2>/dev/null; then
            echo "⚠️  Force killing Anvil process..."
            kill -9 $ANVIL_PID
        fi
        
        echo "✅ Anvil stopped successfully"
    else
        echo "⚠️  Anvil process not running (PID $ANVIL_PID not found)"
    fi
    
    # Remove PID file
    rm -f .anvil.pid
else
    echo "⚠️  No Anvil PID file found"
fi

# Also try to kill any anvil processes by name
if pgrep anvil >/dev/null 2>&1; then
    echo "🔍 Found running anvil processes, stopping them..."
    pkill anvil || true
    sleep 2
    
    if pgrep anvil >/dev/null 2>&1; then
        echo "⚠️  Force killing remaining anvil processes..."
        pkill -9 anvil || true
    fi
fi

# Clean up any processes using port 8545
if lsof -Pi :8545 -sTCP:LISTEN -t >/dev/null 2>&1; then
    echo "🧹 Cleaning up processes using port 8545..."
    lsof -Pi :8545 -sTCP:LISTEN -t | xargs kill -9 2>/dev/null || true
fi

echo "🧹 Cleanup complete"
echo ""
echo "💡 To check if any blockchain processes are still running:"
echo "   ps aux | grep anvil"
echo "   lsof -i :8545" 