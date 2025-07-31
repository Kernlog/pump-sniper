//! Events

use crate::accounts::{TokenInfo, BondingCurveAccount};
use crate::common::MarketData;
use solana_sdk::pubkey::Pubkey;

#[derive(Debug, Clone)]
pub enum SniperEvent {
    TokenCreated(TokenInfo),
    BondingCurveUpdated {
        bonding_curve: Pubkey,
        data: BondingCurveAccount,
    },
    MarketCapUpdated(MarketData),
    BuyTriggered {
        token_info: TokenInfo,
        market_cap: u64,
        buy_amount: u64,
    },
    BuyExecuted {
        token_info: TokenInfo,
        transaction_signature: String,
        amount_spent: u64,
        tokens_received: u64,
    },
    BuyFailed {
        token_info: TokenInfo,
        error: String,
        retry_count: u32,
    },
    ConnectionStatusChanged {
        connected: bool,
        endpoint: String,
    },
    StatsUpdate {
        tokens_tracked: usize,
        successful_buys: usize,
        failed_buys: usize,
        uptime_seconds: u64,
    },
}

impl SniperEvent {
    pub fn event_type(&self) -> &'static str {
        match self {
            SniperEvent::TokenCreated(_) => "token_created",
            SniperEvent::BondingCurveUpdated { .. } => "bonding_curve_updated",
            SniperEvent::MarketCapUpdated(_) => "market_cap_updated",
            SniperEvent::BuyTriggered { .. } => "buy_triggered",
            SniperEvent::BuyExecuted { .. } => "buy_executed",
            SniperEvent::BuyFailed { .. } => "buy_failed",
            SniperEvent::ConnectionStatusChanged { .. } => "connection_status_changed",
            SniperEvent::StatsUpdate { .. } => "stats_update",
        }
    }

    pub fn is_critical(&self) -> bool {
        matches!(
            self,
            SniperEvent::BuyTriggered { .. } | 
            SniperEvent::BuyExecuted { .. } | 
            SniperEvent::BuyFailed { .. }
        )
    }
}