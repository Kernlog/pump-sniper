//! Create token instruction

use crate::constants::CREATE_DISCRIMINATOR;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_sdk::pubkey::Pubkey;

/// Create token instruction data
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct CreateInstruction {
    pub name: String,
    pub symbol: String,
    pub uri: String,
    pub creator: Pubkey,
}

impl CreateInstruction {
    /// Get instruction discriminator
    pub const fn discriminator() -> [u8; 8] {
        CREATE_DISCRIMINATOR
    }

    /// Serialize instruction data with discriminator
    pub fn data(&self) -> Vec<u8> {
        let mut data = Vec::with_capacity(256);
        data.extend_from_slice(&Self::discriminator());
        self.serialize(&mut data).unwrap();
        data
    }

    /// Parse instruction data from bytes
    pub fn from_bytes(data: &[u8]) -> Result<Self, std::io::Error> {
        if data.len() < 8 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Data too short",
            ));
        }

        let discriminator = &data[0..8];
        if discriminator != Self::discriminator() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid discriminator",
            ));
        }

        Self::try_from_slice(&data[8..])
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
    }
}
