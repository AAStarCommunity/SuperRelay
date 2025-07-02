#!/bin/bash

# Generate Chain Spec with deployed EntryPoint address

set -e

echo "📝 Generating custom chain spec..."

# Get deployed EntryPoint address
if [ -f ".entrypoint_address" ]; then
    ENTRYPOINT_ADDRESS=$(cat .entrypoint_address)
    echo "📍 Using EntryPoint: $ENTRYPOINT_ADDRESS"
else
    echo "❌ EntryPoint address not found. Please deploy EntryPoint first."
    exit 1
fi

# Generate custom chain spec
cat > bin/rundler/chain_specs/local_dev.toml << EOF
base = "ethereum"

name = "Local Development Chain"
id = 31337

# Use our deployed EntryPoint address for v0.6
entry_point_address_v0_6 = "$ENTRYPOINT_ADDRESS"

bundle_max_send_interval_millis = 250
block_gas_limit = 30000000

# Dev-friendly settings
min_max_priority_fee_per_gas = 1000000
EOF

echo "✅ Chain spec generated: bin/rundler/chain_specs/local_dev.toml"
echo "🔧 EntryPoint address: $ENTRYPOINT_ADDRESS" 