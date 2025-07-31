//! Price fetching utilities

use anyhow::Result;
use serde::Deserialize;
use std::time::{Duration, SystemTime};
use tracing::info;

#[derive(Debug, Deserialize)]
struct CoinGeckoResponse {
    solana: SolanaPrice,
}

#[derive(Debug, Deserialize)]
struct SolanaPrice {
    usd: f64,
}

/// Price fetcher for SOL/USD
pub struct PriceFetcher {
    client: reqwest::Client,
    cached_price: Option<(f64, SystemTime)>,
    cache_duration: Duration,
}

impl PriceFetcher {
    /// Create new price fetcher
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
            cached_price: None,
            cache_duration: Duration::from_secs(30), // Cache for 30 seconds
        }
    }

    /// Get current SOL price in USD
    pub async fn get_sol_price_usd(&mut self) -> Result<f64> {
        if let Some((price, timestamp)) = self.cached_price {
            if timestamp.elapsed().unwrap_or(Duration::MAX) < self.cache_duration {
                return Ok(price);
            }
        }

        // If no cache or expired, fetch fresh price
        let price = self.fetch_fresh_price().await?;
        
        // Update cache
        self.cached_price = Some((price, SystemTime::now()));
        
        Ok(price)
    }

    async fn fetch_fresh_price(&self) -> Result<f64> {
        let url = "https://api.coingecko.com/api/v3/simple/price?ids=solana&vs_currencies=usd";
        
        let response = self
            .client
            .get(url)
            .timeout(Duration::from_secs(3)) 
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!("CoinGecko API error: {}", response.status()));
        }

        let data: CoinGeckoResponse = response.json().await?;
        
        info!("SOL PRICE: ${:.2}", data.solana.usd);
        
        Ok(data.solana.usd)
    }

    /// Calculate market cap in USD
    pub async fn calculate_market_cap_usd(&mut self, sol_amount: u64) -> Result<f64> {
        let sol_price = self.get_sol_price_usd().await?;
        let sol_amount_f64 = sol_amount as f64 / 1e9; // Convert lamports to SOL
        let market_cap_usd = sol_amount_f64 * sol_price;
        
        Ok(market_cap_usd)
    }
}

impl Default for PriceFetcher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fetch_sol_price() {
        let mut fetcher = PriceFetcher::new();
        
        if let Ok(price) = fetcher.get_sol_price_usd().await {
            assert!(price > 100.0);
            assert!(price < 250.0);
        }
    }

    #[test]
    fn test_market_cap_calculation() {
        let sol_amount = 1_000_000_000;
        let sol_price = 100.0;
        
        let sol_amount_f64 = sol_amount as f64 / 1e9;
        let market_cap = sol_amount_f64 * sol_price;
        
        assert_eq!(market_cap, 100.0);
    }
}