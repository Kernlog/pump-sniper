//! Sell token instruction

use borsh::{BorshDeserialize, BorshSerialize};

/// Sell token instruction data
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct SellInstruction {
    pub amount: u64,
    pub min_sol_output: u64,
}

impl SellInstruction {
    /// Serialize instruction data with discriminator
    pub fn data(&self) -> Vec<u8> {
        let mut data = Vec::with_capacity(256);
        // TODO: Add sell discriminator when needed
        self.serialize(&mut data).unwrap();
        data
    }
}
