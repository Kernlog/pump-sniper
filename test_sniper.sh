#!/bin/bash

# Pump Sniper Test Script
echo "Starting Pump.Fun Sniper Bot"
echo "This will execute real trades when tokens hit $8000 market cap"
echo ""

# Set env vars
export RUST_LOG=info
export GRPC_ENDPOINT=""
export RPC_ENDPOINT="h"
export MARKET_CAP_THRESHOLD_USD="8000"     # $8k USD threshold
export BUY_AMOUNT_SOL="50000000"          # 0.05 SOL buy amount
export PRIORITY_FEE_SOL="100000"          # 0.0001 SOL priority fee  
export COMPUTE_UNIT_LIMIT="75000"         # 75k compute units
export MAX_SLIPPAGE_BPS="500"             # 5% slippage
export TEST_MODE="true"                    # Stop after first buy

# Check wallet
if [ -z "$WALLET_PRIVATE_KEY" ]; then
    echo "ERROR: WALLET_PRIVATE_KEY environment variable not set!"
    echo ""
    echo "To set your wallet:"
    echo "Run: export WALLET_PRIVATE_KEY=your_base58_private_key"
    echo ""
    exit 1
fi

echo "Configuration:"
echo "  Market Cap Threshold: $8,000 USD"
echo "  Buy Amount: 0.05 SOL"
echo "  Priority Fee: 0.0001 SOL"
echo "  Max Slippage: 5%"
echo ""

# Run the sniper
cargo run --bin sniper