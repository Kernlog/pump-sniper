//! Buy token instruction

use crate::{
    constants::{accounts, BUY_DISCRIMINATOR},
    error::SniperError,
    utils::pda::{derive_bonding_curve_pda, derive_global_pda},
};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::Keypair,
    signer::Signer,
};
use spl_associated_token_account::get_associated_token_address;

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct BuyInstruction {
    pub amount: u64,
    pub max_sol_cost: u64,
}

impl BuyInstruction {
    pub const fn discriminator() -> [u8; 8] {
        BUY_DISCRIMINATOR
    }

    pub fn data(&self) -> Vec<u8> {
        let mut data = Vec::with_capacity(256);
        data.extend_from_slice(&Self::discriminator());
        self.serialize(&mut data).unwrap();
        data
    }

    pub fn create_instruction(
        &self,
        payer: &Keypair,
        mint: &Pubkey,
        fee_recipient: &Pubkey,
        creator: &Pubkey,
    ) -> Result<Instruction, SniperError> {
        // derive PDAs
        let bonding_curve = derive_bonding_curve_pda(mint)?;
        let global_pda = derive_global_pda()?;
        let creator_vault = derive_creator_vault_pda(creator)?;

        let instruction = Instruction::new_with_bytes(
            accounts::pumpfun_program_id(),
            &self.data(),
            vec![
                // Global config PDA
                AccountMeta::new_readonly(global_pda, false),
                // Fee recipient
                AccountMeta::new(*fee_recipient, false),
                // Token mint
                AccountMeta::new_readonly(*mint, false),
                // Bonding curve
                AccountMeta::new(bonding_curve, false),
                // Bonding curve token account
                AccountMeta::new(get_associated_token_address(&bonding_curve, mint), false),
                // User's token account
                AccountMeta::new(get_associated_token_address(&payer.pubkey(), mint), false),
                // Payer
                AccountMeta::new(payer.pubkey(), true),
                // System program
                AccountMeta::new_readonly(accounts::system_program(), false),
                // Token program
                AccountMeta::new_readonly(accounts::token_program(), false),
                // Creator vault
                AccountMeta::new(creator_vault, false),
                // Event authority
                AccountMeta::new_readonly(accounts::event_authority(), false),
                // Pump.fun program
                AccountMeta::new_readonly(accounts::pumpfun_program_id(), false),
            ],
        );

        Ok(instruction)
    }
}

fn derive_creator_vault_pda(creator: &Pubkey) -> Result<Pubkey, SniperError> {
    use crate::constants::seeds::CREATOR_VAULT_SEED;

    let (creator_vault, _bump) = Pubkey::find_program_address(
        &[CREATOR_VAULT_SEED, creator.as_ref()],
        &accounts::pumpfun_program_id(),
    );

    Ok(creator_vault)
}
