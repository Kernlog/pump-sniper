//! Bonding curve account for Pump tokens

use borsh::{BorshDeserialize, BorshSerialize};
use solana_sdk::pubkey::Pubkey;
use crate::error::SniperError;

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct BondingCurveAccount {
    pub discriminator: u64,
    pub virtual_token_reserves: u64,
    pub virtual_sol_reserves: u64,
    pub real_token_reserves: u64,
    pub real_sol_reserves: u64,
    pub token_total_supply: u64,
    pub complete: bool,
    pub creator: Pubkey,
}

impl BondingCurveAccount {
    pub fn get_market_cap_sol(&self) -> u64 {
        if self.virtual_token_reserves == 0 {
            return 0;
        }

        ((self.token_total_supply as u128) * (self.virtual_sol_reserves as u128)
            / (self.virtual_token_reserves as u128)) as u64
    }

    pub fn get_buy_price(&self, sol_amount: u64) -> Result<u64, SniperError> {
        if self.complete {
            return Err(SniperError::BondingCurveComplete);
        }

        if sol_amount == 0 {
            return Ok(0);
        }

        let n: u128 = (self.virtual_sol_reserves as u128) * (self.virtual_token_reserves as u128);
        let i: u128 = (self.virtual_sol_reserves as u128) + (sol_amount as u128);
        let r: u128 = n / i + 1;
        let s: u128 = (self.virtual_token_reserves as u128) - r;

        let token_amount = s as u64;
        Ok(if token_amount < self.real_token_reserves {
            token_amount
        } else {
            self.real_token_reserves
        })
    }

    pub fn get_sell_price(&self, token_amount: u64, fee_basis_points: u64) -> Result<u64, SniperError> {
        if self.complete {
            return Err(SniperError::BondingCurveComplete);
        }

        if token_amount == 0 {
            return Ok(0);
        }

        let n: u128 = ((token_amount as u128) * (self.virtual_sol_reserves as u128))
            / ((self.virtual_token_reserves as u128) + (token_amount as u128));

        let fee: u128 = (n * (fee_basis_points as u128)) / 10000;
        Ok((n - fee) as u64)
    }

    pub fn has_sufficient_liquidity(&self, sol_amount: u64) -> bool {
        !self.complete && sol_amount <= self.real_sol_reserves
    }

    pub fn get_curve_progress(&self) -> f64 {
        if self.token_total_supply == 0 {
            return 0.0;
        }
        
        let tokens_sold = self.token_total_supply - self.real_token_reserves;
        (tokens_sold as f64 / self.token_total_supply as f64) * 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_bonding_curve() -> BondingCurveAccount {
        BondingCurveAccount {
            discriminator: 1,
            virtual_token_reserves: 1_000_000_000,
            virtual_sol_reserves: 30_000_000_000,
            real_token_reserves: 800_000_000,
            real_sol_reserves: 0,
            token_total_supply: 1_000_000_000,
            complete: false,
            creator: Pubkey::new_unique(),
        }
    }

    #[test]
    fn test_market_cap_calculation() {
        let curve = create_test_bonding_curve();
        assert_eq!(curve.get_market_cap_sol(), 30_000_000_000);
    }

    #[test]
    fn test_buy_price_calculation() {
        let curve = create_test_bonding_curve();
        let tokens = curve.get_buy_price(1_000_000_000).unwrap();
        assert!(tokens > 0);
        assert!(tokens <= curve.real_token_reserves);
    }

    #[test]
    fn test_curve_progress() {
        let curve = create_test_bonding_curve();
        let progress = curve.get_curve_progress();
        assert_eq!(progress, 20.0);
    }
}