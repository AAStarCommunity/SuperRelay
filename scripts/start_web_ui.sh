#!/bin/bash
# SuperRelay Web UI startup script
# Independent deployment of Swagger UI and admin interfaces

set -e

echo "🌐 SuperRelay Web UI v0.1.5 Starting"
echo "📋 Independent Frontend Deployment"
echo "======================================"

# Check if Node.js is installed
if ! command -v node &> /dev/null; then
    echo "❌ Error: Node.js not found. Please install Node.js 16+ first."
    echo "💡 Install from: https://nodejs.org/"
    exit 1
fi

# Check if npm is installed
if ! command -v npm &> /dev/null; then
    echo "❌ Error: npm not found. Please install npm first."
    exit 1
fi

# Change to web-ui directory
cd web-ui

echo "📁 Working directory: $(pwd)"

# Install dependencies if needed
if [ ! -d "node_modules" ]; then
    echo "📦 Installing Web UI dependencies..."
    npm install
    echo "✅ Dependencies installed"
else
    echo "✅ Dependencies already installed"
fi

# Kill existing process on port 9000
if lsof -ti:9000 >/dev/null 2>&1; then
    echo "🛑 Killing existing process on port 9000..."
    lsof -ti:9000 | xargs kill -9 2>/dev/null || true
    sleep 1
fi

echo ""
echo "🚀 Starting Swagger UI server..."
echo "------------------------------------"
echo "💡 Web UI Components:"
echo "  • Swagger UI = Interactive API documentation"
echo "  • OpenAPI Spec = http://localhost:9000/openapi.json"
echo "  • Main Interface = http://localhost:9000/"
echo "------------------------------------"
echo ""

echo "🔧 Executing command:"
echo "  npm run serve"
echo ""

# Start the web UI server
echo "✨ Web UI server starting on port 9000..."
echo "📖 Access Swagger UI at: http://localhost:9000/"
echo "📄 OpenAPI specification: http://localhost:9000/openapi.json"
echo ""
echo "🔄 Server logs:"
echo "------------------------------------"

npm run serve