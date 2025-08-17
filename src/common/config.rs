//! Config

use crate::error::SniperError;

#[derive(Debug, Clone)]
pub struct Config {
    /// gRPC endpoint for streaming
    pub grpc_endpoint: String,
    /// RPC endpoint for transactions
    pub rpc_endpoint: String,
    /// Market cap threshold in USD
    pub market_cap_threshold_usd: f64,
    /// Maximum slippage tolerance (basis points)
    pub max_slippage_bps: u64,
    /// Buy amount in SOL lamports
    pub buy_amount_sol: u64,
    /// Priority fee in SOL lamports (0.005 SOL = 5_000_000 lamports)
    pub priority_fee_sol: u64,
    /// Compute unit limit for buy transactions
    pub compute_unit_limit: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            grpc_endpoint: "".to_string(),
            rpc_endpoint: "".to_string(),
            market_cap_threshold_usd: 8000.0,
            max_slippage_bps: 500,
            buy_amount_sol: 50_000_000,
            priority_fee_sol: 5_000_000,
            compute_unit_limit: 200_000,
        }
    }
}

impl Config {
    /// Load config
    pub fn from_env() -> Result<Self, SniperError> {
        let mut config = Self::default();

        if let Ok(endpoint) = std::env::var("GRPC_ENDPOINT") {
            config.grpc_endpoint = endpoint;
        }

        if let Ok(endpoint) = std::env::var("RPC_ENDPOINT") {
            config.rpc_endpoint = endpoint;
        }

        if let Ok(threshold) = std::env::var("MARKET_CAP_THRESHOLD_USD") {
            config.market_cap_threshold_usd = threshold.parse().map_err(|_| {
                SniperError::InvalidConfig("Invalid market cap threshold".to_string())
            })?;
        }

        if let Ok(slippage) = std::env::var("MAX_SLIPPAGE_BPS") {
            config.max_slippage_bps = slippage
                .parse()
                .map_err(|_| SniperError::InvalidConfig("Invalid slippage".to_string()))?;
        }

        if let Ok(amount) = std::env::var("BUY_AMOUNT_SOL") {
            config.buy_amount_sol = amount
                .parse()
                .map_err(|_| SniperError::InvalidConfig("Invalid buy amount".to_string()))?;
        }

        if let Ok(fee) = std::env::var("PRIORITY_FEE_SOL") {
            config.priority_fee_sol = fee
                .parse()
                .map_err(|_| SniperError::InvalidConfig("Invalid priority fee".to_string()))?;
        }

        if let Ok(limit) = std::env::var("COMPUTE_UNIT_LIMIT") {
            config.compute_unit_limit = limit.parse().map_err(|_| {
                SniperError::InvalidConfig("Invalid compute unit limit".to_string())
            })?;
        }

        Ok(config)
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<(), SniperError> {
        if self.market_cap_threshold_usd <= 0.0 {
            return Err(SniperError::InvalidConfig(
                "Market cap threshold must be positive".to_string(),
            ));
        }

        if self.max_slippage_bps > 10000 {
            return Err(SniperError::InvalidConfig(
                "Slippage cannot exceed 100%".to_string(),
            ));
        }

        if self.buy_amount_sol == 0 {
            return Err(SniperError::InvalidConfig(
                "Buy amount cannot be zero".to_string(),
            ));
        }

        if self.priority_fee_sol == 0 {
            return Err(SniperError::InvalidConfig(
                "Priority fee cannot be zero".to_string(),
            ));
        }

        Ok(())
    }

    /// Get market cap threshold in USD (for display)
    pub fn market_cap_threshold_usd_display(&self) -> f64 {
        self.market_cap_threshold_usd
    }

    /// Get buy amount in SOL (for display)
    pub fn buy_amount_sol_display(&self) -> f64 {
        self.buy_amount_sol as f64 / 1e9
    }

    /// Get priority fee in SOL (for display)
    pub fn priority_fee_sol_display(&self) -> f64 {
        self.priority_fee_sol as f64 / 1e9
    }
}
