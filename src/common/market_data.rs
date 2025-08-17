//! Market data structures and calcs

use crate::accounts::{BondingCurveAccount, TokenInfo};

/// Market data for a token
#[derive(Debug, Clone)]
pub struct MarketData {
    /// Token info
    pub token_info: TokenInfo,
    /// Bonding curve account data
    pub bonding_curve_data: BondingCurveAccount,
    /// Current market cap in SOL lamports
    pub current_market_cap_sol: u64,
    /// Last update timestamp
    pub last_updated: u64,
    /// Price per token in SOL lamports
    pub price_per_token_sol: u64,
    /// vol
    pub volume_24h: Option<u64>,
}

impl MarketData {
    /// Create new market data from token info and bonding curve
    pub fn new(token_info: TokenInfo, bonding_curve_data: BondingCurveAccount) -> Self {
        let current_market_cap_sol = bonding_curve_data.get_market_cap_sol();
        let price_per_token_sol = if bonding_curve_data.token_total_supply > 0 {
            current_market_cap_sol / bonding_curve_data.token_total_supply
        } else {
            0
        };

        Self {
            token_info,
            bonding_curve_data,
            current_market_cap_sol,
            last_updated: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            price_per_token_sol,
            volume_24h: None,
        }
    }

    /// Get market cap in SOL for display
    pub fn market_cap_sol_display(&self) -> f64 {
        self.current_market_cap_sol as f64 / 1e9
    }

    /// Get price per token in SOL for display
    pub fn price_per_token_sol_display(&self) -> f64 {
        self.price_per_token_sol as f64 / 1e9
    }

    /// Get bonding curve progress (0-100%)
    pub fn curve_progress(&self) -> f64 {
        self.bonding_curve_data.get_curve_progress()
    }

    /// Check if token meets market cap threshold
    pub fn meets_threshold(&self, threshold_sol: u64) -> bool {
        self.current_market_cap_sol >= threshold_sol
    }

    /// Get age since last update in seconds
    pub fn age_seconds(&self) -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
            - self.last_updated
    }

    /// Check if data is stale (older than threshold)
    pub fn is_stale(&self, max_age_seconds: u64) -> bool {
        self.age_seconds() > max_age_seconds
    }

    /// Update market data with new bonding curve data
    pub fn update(&mut self, bonding_curve_data: BondingCurveAccount) {
        self.bonding_curve_data = bonding_curve_data;
        self.current_market_cap_sol = self.bonding_curve_data.get_market_cap_sol();
        self.price_per_token_sol = if self.bonding_curve_data.token_total_supply > 0 {
            self.current_market_cap_sol / self.bonding_curve_data.token_total_supply
        } else {
            0
        };
        self.last_updated = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
    }
}
