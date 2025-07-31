#!/bin/bash

# Pump.Fun Monitor Test Script
echo "🚀 Starting Pump.Fun Token Monitor"
echo "This will track new token creations and their market caps in real-time"
echo "Press Ctrl+C to stop"
echo ""

# Set environment variables for testing
export RUST_LOG=info
export GRPC_ENDPOINT=""
export RPC_ENDPOINT=""
export MARKET_CAP_THRESHOLD_USD="8000"           # $8k USD
export MAX_BUY_AMOUNT_SOL="100000000"            # 0.1 SOL in lamports
export MAX_SLIPPAGE_BPS="500"                    # 5%

# Run the monitor
cargo run --bin monitor