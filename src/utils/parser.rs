//! Transaction parsing utils

use crate::{
    accounts::TokenInfo,
    constants::CREATE_DISCRIMINATOR,
    instructions::CreateInstruction,
    utils::pda::derive_bonding_curve_pda,
};
use solana_sdk::pubkey::Pubkey;
use tracing::{error, warn};
use yellowstone_grpc_proto::prelude::SubscribeUpdateTransactionInfo;

/// Check if tx contains a create instruction
pub fn is_create_transaction(transaction: &SubscribeUpdateTransactionInfo) -> bool {
    if let Some(ref transaction_data) = transaction.transaction {
        if let Some(ref message) = transaction_data.message {
            for instruction in &message.instructions {
                if instruction.data.len() >= 8 {
                    let discriminator = &instruction.data[0..8];
                    if discriminator == CREATE_DISCRIMINATOR {
                        return true;
                    }
                }
            }
        }
    }
    false
}

/// Parse token creation data from transaction
pub fn parse_token_creation(
    transaction: &SubscribeUpdateTransactionInfo,
    signature: String,
) -> Option<TokenInfo> {
    if let Some(ref transaction_data) = transaction.transaction {
        if let Some(ref message) = transaction_data.message {
            // Find create instruction and extract data
            for (index, instruction) in message.instructions.iter().enumerate() {
                if instruction.data.len() >= 8 {
                    let discriminator = &instruction.data[0..8];
                    if discriminator == CREATE_DISCRIMINATOR {
                        // Parse instruction data
                        match CreateInstruction::from_bytes(&instruction.data) {
                            Ok(create_data) => {
                                // Extract mint from instruction accounts
                                if let Some(mint) = extract_mint_from_instruction(message, index) {
                                    // Derive bonding curve PDA
                                    match derive_bonding_curve_pda(&mint) {
                                        Ok(bonding_curve) => {
                                            return Some(TokenInfo::new(
                                                mint,
                                                create_data.name,
                                                create_data.symbol,
                                                create_data.creator,
                                                create_data.uri,
                                                bonding_curve,
                                                signature,
                                            ));
                                        }
                                        Err(e) => {
                                            error!("Failed to derive bonding curve PDA: {}", e);
                                        }
                                    }
                                } else {
                                    warn!("Failed to extract mint from create instruction");
                                }
                            }
                            Err(e) => {
                                error!("Failed to parse create instruction: {}", e);
                            }
                        }
                    }
                }
            }
        }
    }
    None
}

/// Extract mint pubkey from instruction accounts
fn extract_mint_from_instruction(
    message: &yellowstone_grpc_proto::prelude::Message,
    instruction_index: usize,
) -> Option<Pubkey> {
    if let Some(instruction) = message.instructions.get(instruction_index) {
        if let Some(&account_index) = instruction.accounts.first() {
            if let Some(account_key) = message.account_keys.get(account_index as usize) {
                return Pubkey::try_from(account_key.as_slice()).ok();
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_discriminator() {
        assert_eq!(CREATE_DISCRIMINATOR.len(), 8);
        // Verify discriminator matches expected value
        assert_eq!(CREATE_DISCRIMINATOR, [24, 30, 200, 40, 5, 28, 7, 119]);
    }
}