# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0-alpha] - 2025-08-17

### Added
- Initial alpha release of Pump Sniper
- Real-time token monitoring via Yellowstone gRPC streaming
- Automated trading execution based on market cap thresholds
- Two operational modes:
  - Monitor: Live tracking and visualization of token launches
  - Sniper: Automated trading bot with configurable parameters
- Bonding curve mathematics implementation for accurate price calculations
- Transaction priority fee optimization for faster execution
- Slippage protection and retry mechanisms
- Test mode for single-buy validation
- Comprehensive configuration via environment variables
- Memory-efficient token caching and state management
- Support for concurrent multi-token tracking

### Technical Features
- Direct Solana RPC node integration
- SPL Token program interaction
- Pump.Fun platform specific account parsing
- Real-time market cap calculation in USD
- Transaction building and signing
- Wallet management and balance checking

### Documentation
- Comprehensive README with setup instructions
- Apache 2.0 licensing
- GitHub Actions workflow for automated releases
- Contributing guidelines

### Known Limitations
- Alpha release - use at your own risk
- Requires Yellowstone gRPC access
- Limited to Pump.Fun platform tokens
- No sell functionality (buy only)
- Single wallet support per instance

[0.1.0-alpha]: https://github.com/kernlog/pump-sniper/releases/tag/v0.1.0-alpha