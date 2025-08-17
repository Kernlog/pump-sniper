# Pump Sniper

A high-performance Solana token monitoring and automated trading bot designed for tracking new token launches on Pump.Fun and executing trades based on market capitalization thresholds.

## Overview

Pump Sniper is a specialized trading bot that monitors the Solana blockchain for new token creations on the Pump.Fun platform. It automatically identifies and purchases tokens when they reach predefined market capitalization thresholds, enabling traders to capture early opportunities in newly launched tokens.

The system consists of two primary components:
- **Monitor**: Real-time token tracking and market cap visualization
- **Sniper**: Automated trading execution based on configurable parameters

## Features

### Core Functionality
- Real-time token creation monitoring via gRPC streaming
- Instant market capitalization calculation and tracking
- Automated buy execution when tokens meet threshold criteria
- Configurable trading parameters and risk management
- Multi-token concurrent tracking capabilities
- Transaction priority fee optimization for faster execution

### Technical Capabilities
- Direct integration with Solana RPC nodes
- Yellowstone gRPC protocol support for low-latency data streaming
- Bonding curve mathematics for accurate price calculations
- Slippage protection and transaction retry mechanisms
- Memory-efficient token caching and state management

## Architecture

### Project Structure
```
pump-sniper/
├── src/
│   ├── accounts/        # Solana account structures
│   ├── bin/            # Executable binaries
│   │   ├── monitor.rs  # Token monitoring tool
│   │   └── sniper.rs   # Automated trading bot
│   ├── common/         # Shared components
│   ├── constants/      # Program constants
│   ├── error/          # Error handling
│   ├── instructions/   # Transaction builders
│   └── utils/          # Helper functions
├── Cargo.toml          # Rust dependencies
└── test_*.sh           # Testing scripts
```

### Components

#### Monitor Mode
The monitor provides real-time visibility into token launches:
- Displays token name, symbol, and mint address
- Shows initial and current market capitalization
- Calculates percentage changes since launch
- Updates market data every 3 seconds
- Visual status indicators for buy signals

#### Sniper Mode
The automated trading bot executes trades based on:
- Market cap threshold detection
- Configurable buy amounts
- Priority fee settings for transaction speed
- Slippage tolerance parameters
- Test mode for single-buy validation

## Installation

### Prerequisites
- Rust 1.70 or higher
- Solana CLI tools
- Access to Solana RPC endpoint
- Yellowstone gRPC endpoint access

### Build from Source
```bash
git clone https://github.com/kernlog/pump-sniper.git
cd pump-sniper
cargo build --release
```

## Configuration

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `GRPC_ENDPOINT` | Yellowstone gRPC endpoint URL | Required |
| `RPC_ENDPOINT` | Solana RPC node URL | Required |
| `MARKET_CAP_THRESHOLD_USD` | Market cap trigger in USD | 8000 |
| `BUY_AMOUNT_SOL` | Purchase amount in lamports | 50000000 |
| `PRIORITY_FEE_SOL` | Transaction priority fee | 100000 |
| `COMPUTE_UNIT_LIMIT` | Compute units for transactions | 75000 |
| `MAX_SLIPPAGE_BPS` | Maximum slippage in basis points | 500 |
| `WALLET_PRIVATE_KEY` | Base58 encoded private key | Required for sniper |

### Configuration Files
Create a `.env` file in the project root:
```env
GRPC_ENDPOINT=your_grpc_endpoint
RPC_ENDPOINT=your_rpc_endpoint
MARKET_CAP_THRESHOLD_USD=8000
BUY_AMOUNT_SOL=50000000
WALLET_PRIVATE_KEY=your_base58_private_key
```

## Usage

### Running the Monitor
```bash
# Basic monitoring
cargo run --bin monitor

# With custom configuration
MARKET_CAP_THRESHOLD_USD=10000 cargo run --bin monitor

# Using the test script
./test_monitor.sh
```

### Running the Sniper
```bash
# Ensure wallet is configured
export WALLET_PRIVATE_KEY=your_base58_private_key

# Run sniper
cargo run --bin sniper

# Test mode (stops after first buy)
TEST_MODE=true cargo run --bin sniper

# Using the test script
./test_sniper.sh
```

### Command Line Options
The monitor displays a real-time dashboard:
```
====================================================================================================
                                    PUMP.FUN TOKEN MONITOR
====================================================================================================
TOKEN (SYMBOL)               MINT ADDRESS    INITIAL MC   CURRENT MC   CHANGE %    AGE (s)  STATUS
----------------------------------------------------------------------------------------------------
MyToken (MTK)                4Xm9...         1250.50      8500.75      +580.00%    45       BUY!
```

## Trading Strategy

### Market Cap Threshold
The bot monitors tokens and triggers buy orders when:
1. Token market cap reaches the configured USD threshold
2. Token has not been previously purchased
3. Sufficient wallet balance exists

### Risk Management
- **Slippage Protection**: Configurable maximum slippage percentage
- **Buy Amount Limits**: Fixed SOL amounts per trade
- **Single Purchase Logic**: Prevents duplicate buys of same token
- **Test Mode**: Allows validation with single trade execution

### Transaction Execution
1. Detects token meeting threshold criteria
2. Calculates optimal transaction parameters
3. Builds and signs transaction with priority fee
4. Submits to network with retry logic
5. Confirms transaction success

## Technical Details

### Bonding Curve Mathematics
The system implements Pump.Fun's bonding curve formula:
- Virtual token reserves calculation
- Real-time price derivation
- Market cap computation based on curve state

### Data Streaming
Utilizes Yellowstone gRPC for:
- Account update subscriptions
- Transaction monitoring
- Minimal latency data delivery

### Performance Optimizations
- Concurrent token tracking
- Cached market data
- Batch RPC requests
- Efficient memory management

## Development

### Running Tests
```bash
cargo test
```

### Building Documentation
```bash
cargo doc --open
```

### Code Structure
- **Accounts**: Borsh-serialized Solana account structures
- **Instructions**: Transaction instruction builders
- **Utils**: Helper functions for PDA derivation, price calculations
- **Common**: Shared configuration and event handling

## Troubleshooting

### Common Issues

**Connection Errors**
- Verify gRPC endpoint accessibility
- Check RPC node availability
- Ensure network connectivity

**Transaction Failures**
- Increase priority fee for congested network
- Adjust slippage tolerance
- Verify wallet balance

**Missing Tokens**
- Confirm gRPC subscription filters
- Check program ID configuration
- Verify account parsing logic

## Performance Metrics

### Typical Latencies
- Token detection: <20ms from creation
- Market cap calculation: <5ms
- Transaction submission: <30ms
- Confirmation time: 400ms

### Resource Usage
- Memory: ~100MB baseline
- CPU: <5% during monitoring
- Network: Varies with token activity

## Disclaimer

This software is provided for educational and research purposes. Cryptocurrency trading involves substantial risk of loss. Users should:
- Understand the risks involved in automated trading
- Never invest more than they can afford to lose
- Conduct thorough research before trading
- Comply with local regulations and tax obligations

## License

This project is released under the Apache License. See LICENSE file for details.

## Contributing

Contributions are welcome. Please:
1. Fork the repository
2. Create a feature branch
3. Commit changes with clear messages
4. Submit a pull request with description

## Support

For issues, questions, or improvements:
- Open an issue on GitHub
- Review existing documentation
- Check closed issues for solutions