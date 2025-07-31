//! Global config account for Pump program

use borsh::{BorshDeserialize, BorshSerialize};
use solana_sdk::pubkey::Pubkey;

/// Global configuration account (matching pumpfun-rs structure)
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct GlobalAccount {
    /// Unique identifier for the global account
    pub discriminator: u64,
    /// Whether the global account has been initialized
    pub initialized: bool,
    /// Authority that can modify global settings
    pub authority: Pubkey,
    /// Account that receives fees
    pub fee_recipient: Pubkey,
    /// Initial virtual token reserves
    pub initial_virtual_token_reserves: u64,
    /// Initial virtual SOL reserves
    pub initial_virtual_sol_reserves: u64,
    /// Initial actual token reserves
    pub initial_real_token_reserves: u64,
    /// Total supply of tokens
    pub token_total_supply: u64,
    /// Fee in basis points
    pub fee_basis_points: u64,
    /// Authority that can withdraw funds
    pub withdraw_authority: Pubkey,
    /// Flag to enable pool migration
    pub enable_migrate: bool,
    /// Fee for migrating pools
    pub pool_migration_fee: u64,
    /// Fee for creators in basis points
    pub creator_fee_basis_points: u64,
    /// Array of public keys for fee recipients
    pub fee_recipients: [Pubkey; 7],
    /// Authority that sets the creator of the token
    pub set_creator_authority: Pubkey,
}

impl GlobalAccount {
    /// Calculate fee amount
    pub fn calculate_fee(&self, trade_value: u64) -> u64 {
        (trade_value as u128 * self.fee_basis_points as u128 / 10000) as u64
    }

    /// Calculate initial market cap for a new token using Pump constants
    pub fn get_initial_market_cap_sol(&self) -> u64 {
    
        const INITIAL_VIRTUAL_TOKEN_RESERVES: u128 = 1_073_000_000_000_000;
        const INITIAL_VIRTUAL_SOL_RESERVES: u128 = 30_000_000_000; 
        const TOKEN_TOTAL_SUPPLY: u128 = 1_000_000_000_000_000; 

        // Market cap 
        ((TOKEN_TOTAL_SUPPLY * INITIAL_VIRTUAL_SOL_RESERVES) / INITIAL_VIRTUAL_TOKEN_RESERVES) as u64
    }

    /// Calculates the initial amount of tokens
    pub fn get_initial_buy_price(&self, amount: u64) -> u64 {
        if amount == 0 {
            return 0;
        }

        let n: u128 = (self.initial_virtual_sol_reserves as u128)
            * (self.initial_virtual_token_reserves as u128);
        let i: u128 = (self.initial_virtual_sol_reserves as u128) + (amount as u128);
        let r: u128 = n / i + 1;
        let s: u128 = (self.initial_virtual_token_reserves as u128) - r;

        if s < (self.initial_real_token_reserves as u128) {
            s as u64
        } else {
            self.initial_real_token_reserves
        }
    }
}