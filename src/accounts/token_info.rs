//! Token info and metadata

use solana_sdk::pubkey::Pubkey;

/// Token info
#[derive(Debug, Clone)]
pub struct TokenInfo {
    /// Token mint
    pub mint: Pubkey,
    /// Token name
    pub name: String,
    /// Token symbol
    pub symbol: String,
    /// Token creator
    pub creator: Pubkey,
    /// Metadata URI
    pub uri: String,
    /// Bonding curve PDA
    pub bonding_curve: Pubkey,
    /// Creation transaction signature
    pub creation_signature: String,
    /// Creation timestamp
    pub created_at: u64,
}

impl TokenInfo {
    /// Create a new TokenInfo instance
    pub fn new(
        mint: Pubkey,
        name: String,
        symbol: String,
        creator: Pubkey,
        uri: String,
        bonding_curve: Pubkey,
        creation_signature: String,
    ) -> Self {
        Self {
            mint,
            name,
            symbol,
            creator,
            uri,
            bonding_curve,
            creation_signature,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }

    /// Get age of token in seconds
    pub fn age_seconds(&self) -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
            - self.created_at
    }

    /// Format for display
    pub fn display_name(&self) -> String {
        format!("{} ({})", self.name, self.symbol)
    }
}
