//! Constants

use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

/// Pump.Fun program
pub const PUMPFUN_PROGRAM_ID: &str = "6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P";

/// Discriminators
pub const CREATE_DISCRIMINATOR: [u8; 8] = [24, 30, 200, 40, 5, 28, 7, 119];
pub const BUY_DISCRIMINATOR: [u8; 8] = [102, 6, 61, 18, 1, 218, 235, 234];

/// Seeds for PDA derivation
pub mod seeds {
    pub const GLOBAL_SEED: &[u8] = b"global";
    pub const MINT_AUTHORITY_SEED: &[u8] = b"mint-authority";
    pub const BONDING_CURVE_SEED: &[u8] = b"bonding-curve";
    pub const METADATA_SEED: &[u8] = b"metadata";
    pub const CREATOR_VAULT_SEED: &[u8] = b"creator-vault";
}

/// Program addresses
pub mod accounts {
    use super::*;

    pub fn pumpfun_program_id() -> Pubkey {
        Pubkey::from_str(PUMPFUN_PROGRAM_ID).unwrap()
    }

    pub fn mpl_token_metadata() -> Pubkey {
        Pubkey::from_str("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s").unwrap()
    }

    pub fn system_program() -> Pubkey {
        Pubkey::from_str("11111111111111111111111111111111").unwrap()
    }

    pub fn token_program() -> Pubkey {
        Pubkey::from_str("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA").unwrap()
    }

    pub fn associated_token_program() -> Pubkey {
        Pubkey::from_str("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL").unwrap()
    }

    pub fn rent_sysvar() -> Pubkey {
        Pubkey::from_str("SysvarRent111111111111111111111111111111111").unwrap()
    }

    pub fn event_authority() -> Pubkey {
        Pubkey::from_str("Ce6TQqeHC9p8KetsN6JsjHK7UTZk7nasjjnr7XxXp9F1").unwrap()
    }
}
