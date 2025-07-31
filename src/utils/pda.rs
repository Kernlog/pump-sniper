//! PDA utils

use crate::{constants::{accounts, seeds}, error::SniperError};
use solana_sdk::pubkey::Pubkey;

/// Derive bonding curve PDA for mint
pub fn derive_bonding_curve_pda(mint: &Pubkey) -> Result<Pubkey, SniperError> {
    // Use try_find_program_address to handle potential errs
    let seeds: &[&[u8]; 2] = &[seeds::BONDING_CURVE_SEED, mint.as_ref()];
    let program_id = &accounts::pumpfun_program_id();
    
    match Pubkey::try_find_program_address(seeds, program_id) {
        Some((bonding_curve, _bump)) => Ok(bonding_curve),
        None => Err(SniperError::RpcError(format!(
            "Failed to derive bonding curve PDA for mint: {}",
            mint
        ))),
    }
}

/// Derive global config PDA
pub fn derive_global_pda() -> Result<Pubkey, SniperError> {
    let (global, _bump) = Pubkey::find_program_address(
        &[seeds::GLOBAL_SEED],
        &accounts::pumpfun_program_id(),
    );
    Ok(global)
}

/// Derive mint auth PDA
pub fn derive_mint_authority_pda() -> Result<Pubkey, SniperError> {
    let (mint_authority, _bump) = Pubkey::find_program_address(
        &[seeds::MINT_AUTHORITY_SEED],
        &accounts::pumpfun_program_id(),
    );
    Ok(mint_authority)
}

/// Derive metadata PDA for a mint
pub fn derive_metadata_pda(mint: &Pubkey) -> Result<Pubkey, SniperError> {
    let (metadata, _bump) = Pubkey::find_program_address(
        &[
            seeds::METADATA_SEED,
            accounts::mpl_token_metadata().as_ref(),
            mint.as_ref(),
        ],
        &accounts::mpl_token_metadata(),
    );
    Ok(metadata)
}

/// Derive creator vault PDA
pub fn derive_creator_vault_pda(creator: &Pubkey) -> Result<Pubkey, SniperError> {
    let (creator_vault, _bump) = Pubkey::find_program_address(
        &[seeds::CREATOR_VAULT_SEED, creator.as_ref()],
        &accounts::pumpfun_program_id(),
    );
    Ok(creator_vault)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derive_bonding_curve_pda() {
        let mint = Pubkey::new_unique();
        let bonding_curve = derive_bonding_curve_pda(&mint).unwrap();
        
        // Should be valid and deterministic
        assert_ne!(bonding_curve, Pubkey::default());
        
        let bonding_curve2 = derive_bonding_curve_pda(&mint).unwrap();
        assert_eq!(bonding_curve, bonding_curve2);
    }

    #[test]
    fn test_derive_global_pda() {
        let global = derive_global_pda().unwrap();
        assert_ne!(global, Pubkey::default());
        
        let global2 = derive_global_pda().unwrap();
        assert_eq!(global, global2);
    }
}