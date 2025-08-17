//! Pump bot

pub mod accounts;
pub mod common;
pub mod constants;
pub mod error;
pub mod instructions;
pub mod utils;

pub use accounts::{BondingCurveAccount, TokenInfo};
pub use common::{Config, MarketData, SniperEvent};
pub use error::SniperError;

use anyhow::Result;
use common::{Config as StreamConfig, StreamClient};
use solana_sdk::{pubkey::Pubkey, signature::Keypair, signer::Signer};
use std::collections::{HashMap, HashSet};
use tokio::sync::mpsc;
use tracing::{error, info};
use utils::PriceFetcher;

pub struct Sniper {
    config: StreamConfig,
    tracked_tokens: HashMap<String, TokenInfo>,
    bought_tokens: HashSet<String>,
    bonding_curve_cache: HashMap<Pubkey, BondingCurveAccount>,
    event_receiver: mpsc::UnboundedReceiver<SniperEvent>,
    event_sender: mpsc::UnboundedSender<SniperEvent>,
    transaction_executor: utils::TransactionExecutor,
    price_fetcher: PriceFetcher,
    wallet: Option<Keypair>,
    test_mode_single_buy: bool,
    has_bought_once: bool,
}

impl Sniper {
    pub async fn new(config: StreamConfig) -> Result<Self, SniperError> {
        config.validate()?;

        let (event_sender, event_receiver) = mpsc::unbounded_channel();
        let transaction_executor = utils::TransactionExecutor::new(config.clone());
        let price_fetcher = PriceFetcher::new();

        Ok(Self {
            config,
            tracked_tokens: HashMap::new(),
            bought_tokens: HashSet::new(),
            bonding_curve_cache: HashMap::new(),
            event_receiver,
            event_sender,
            transaction_executor,
            price_fetcher,
            wallet: None,
            test_mode_single_buy: false,
            has_bought_once: false,
        })
    }

    pub fn set_wallet(&mut self, wallet: Keypair) {
        info!("Wallet configured: {}", wallet.pubkey());
        self.wallet = Some(wallet);
    }

    pub fn enable_test_mode(&mut self) {
        info!("TEST MODE ENABLED: Will stop after first successful buy");
        self.test_mode_single_buy = true;
    }

    pub async fn start(&mut self) -> Result<(), SniperError> {
        info!("Starting Pump Sniper Bot");
        info!(
            "Market cap threshold: ${:.2} USD",
            self.config.market_cap_threshold_usd_display()
        );
        info!(
            "Buy amount: {:.2} SOL",
            self.config.buy_amount_sol_display()
        );
        info!(
            "Priority fee: {:.3} SOL",
            self.config.priority_fee_sol_display()
        );

        if self.wallet.is_none() {
            return Err(SniperError::InvalidConfig(
                "No wallet configured".to_string(),
            ));
        }

        if let Err(e) = self.price_fetcher.get_sol_price_usd().await {
            error!("Failed to fetch initial SOL price: {}", e);
        }

        let mut stream_client = StreamClient::new(self.config.clone(), self.event_sender.clone());

        tokio::spawn(async move {
            if let Err(e) = stream_client.start().await {
                error!("gRPC streaming failed: {}", e);
            }
        });

        self.process_events().await
    }

    async fn process_events(&mut self) -> Result<(), SniperError> {
        while let Some(event) = self.event_receiver.recv().await {
            if let Err(e) = self.handle_event(event).await {
                error!("Error handling event: {}", e);
            }
        }
        Ok(())
    }

    async fn handle_event(&mut self, event: SniperEvent) -> Result<(), SniperError> {
        match event {
            SniperEvent::TokenCreated(token_info) => self.handle_token_creation(token_info).await,
            SniperEvent::BondingCurveUpdated {
                bonding_curve,
                data,
            } => self.handle_bonding_curve_update(bonding_curve, data).await,
            SniperEvent::MarketCapUpdated(market_data) => {
                self.handle_market_cap_update(market_data).await
            }
            SniperEvent::BuyTriggered {
                token_info,
                market_cap,
                buy_amount,
            } => {
                self.handle_buy_trigger(token_info, market_cap, buy_amount)
                    .await
            }
            _ => Ok(()),
        }
    }

    async fn handle_token_creation(&mut self, token_info: TokenInfo) -> Result<(), SniperError> {
        info!("TOKEN: {} ({})", token_info.symbol, token_info.mint);

        self.tracked_tokens
            .insert(token_info.mint.to_string(), token_info.clone());

        self.check_market_cap(token_info).await
    }

    async fn handle_bonding_curve_update(
        &mut self,
        bonding_curve: Pubkey,
        data: BondingCurveAccount,
    ) -> Result<(), SniperError> {
        self.bonding_curve_cache.insert(bonding_curve, data);
        for token_info in self.tracked_tokens.clone().values() {
            if token_info.bonding_curve == bonding_curve {
                if let Some(cached_data) = self.bonding_curve_cache.get(&bonding_curve) {
                    let market_data = MarketData::new(token_info.clone(), cached_data.clone());

                    // instant check, no RPC
                    match self
                        .price_fetcher
                        .calculate_market_cap_usd(market_data.current_market_cap_sol)
                        .await
                    {
                        Ok(market_cap_usd) => {
                            if market_cap_usd >= self.config.market_cap_threshold_usd
                                && !self.bought_tokens.contains(&token_info.mint.to_string())
                            {
                                if self.test_mode_single_buy && self.has_bought_once {
                                    return Ok(());
                                }

                                info!(
                                    "INSTANT BUY: {} ${:.0}K",
                                    token_info.symbol,
                                    market_cap_usd / 1000.0
                                );

                                let _ = self.event_sender.send(SniperEvent::BuyTriggered {
                                    token_info: token_info.clone(),
                                    market_cap: market_data.current_market_cap_sol,
                                    buy_amount: self.config.buy_amount_sol,
                                });
                            }
                        }
                        Err(e) => {
                            error!("Price fetch failed for {}: {}", token_info.symbol, e);
                        }
                    }
                }
                break;
            }
        }

        Ok(())
    }

    async fn check_market_cap(&mut self, token_info: TokenInfo) -> Result<(), SniperError> {
        // cached data first
        if let Some(cached_data) = self.bonding_curve_cache.get(&token_info.bonding_curve) {
            let market_data = MarketData::new(token_info.clone(), cached_data.clone());

            match self
                .price_fetcher
                .calculate_market_cap_usd(market_data.current_market_cap_sol)
                .await
            {
                Ok(market_cap_usd) => {
                    if market_cap_usd >= self.config.market_cap_threshold_usd {
                        info!(
                            "CACHED BUY: {} ${:.0}K",
                            token_info.symbol,
                            market_cap_usd / 1000.0
                        );

                        if !self.bought_tokens.contains(&token_info.mint.to_string()) {
                            if self.test_mode_single_buy && self.has_bought_once {
                                return Ok(());
                            }

                            let _ = self.event_sender.send(SniperEvent::BuyTriggered {
                                token_info,
                                market_cap: market_data.current_market_cap_sol,
                                buy_amount: self.config.buy_amount_sol,
                            });
                        }
                    }
                    return Ok(());
                }
                Err(e) => {
                    error!("Price fetch failed for {}: {}", token_info.symbol, e);
                    return Ok(());
                }
            }
        }

        // RPC fallback if not cached
        match self
            .transaction_executor
            .fetch_bonding_curve_data(&token_info.bonding_curve)
            .await
        {
            Ok(bonding_curve_data) => {
                let market_data = MarketData::new(token_info.clone(), bonding_curve_data);

                // cached SOL price
                match self
                    .price_fetcher
                    .calculate_market_cap_usd(market_data.current_market_cap_sol)
                    .await
                {
                    Ok(market_cap_usd) => {
                        if market_cap_usd >= self.config.market_cap_threshold_usd {
                            info!(
                                "BUY TARGET: {} ${:.0}K",
                                token_info.symbol,
                                market_cap_usd / 1000.0
                            );
                            if !self.bought_tokens.contains(&token_info.mint.to_string()) {
                                if self.test_mode_single_buy && self.has_bought_once {
                                    info!(
                                        "TEST MODE: {} at ${:.2} meets threshold but skipping (already bought once)",
                                        token_info.display_name(),
                                        market_cap_usd
                                    );
                                    return Ok(());
                                }

                                info!(
                                    "FAST BUY: {} ${:.0}K",
                                    token_info.symbol,
                                    market_cap_usd / 1000.0
                                );

                                let _ = self.event_sender.send(SniperEvent::BuyTriggered {
                                    token_info,
                                    market_cap: market_data.current_market_cap_sol,
                                    buy_amount: self.config.buy_amount_sol,
                                });
                            }
                        }
                    }
                    Err(e) => {
                        error!("Price fetch failed for {}: {}", token_info.symbol, e);
                    }
                }
            }
            Err(e) => {
                error!(
                    "Failed to fetch market data for {}: {}",
                    token_info.symbol, e
                );
            }
        }

        Ok(())
    }

    async fn handle_market_cap_update(
        &mut self,
        _market_data: MarketData,
    ) -> Result<(), SniperError> {
        Ok(())
    }

    async fn handle_buy_trigger(
        &mut self,
        token_info: TokenInfo,
        _market_cap: u64,
        buy_amount: u64,
    ) -> Result<(), SniperError> {
        let mint_str = token_info.mint.to_string();
        if self.bought_tokens.contains(&mint_str) {
            info!("Already bought {}, skipping", token_info.display_name());
            return Ok(());
        }

        if self.test_mode_single_buy && self.has_bought_once {
            info!(
                "TEST MODE: Skipping buy for {} (already bought once)",
                token_info.display_name()
            );
            return Ok(());
        }

        // prevents double buys
        self.bought_tokens.insert(mint_str.clone());

        info!(
            "Executing buy for {} - Amount: {} SOL",
            token_info.display_name(),
            buy_amount as f64 / 1e9
        );

        if let Some(wallet) = &self.wallet {
            match self
                .transaction_executor
                .execute_buy(wallet, &token_info, buy_amount)
                .await
            {
                Ok(signature) => {
                    info!(
                        "BUY SUCCESSFUL! {} - TX: {} - Amount: {} SOL",
                        token_info.display_name(),
                        signature,
                        buy_amount as f64 / 1e9
                    );

                    self.has_bought_once = true;
                    self.tracked_tokens.remove(&mint_str);
                    if self.test_mode_single_buy {
                        info!("TEST MODE: First buy completed successfully. Stopping sniper.");
                        std::process::exit(0);
                    }
                }
                Err(e) => {
                    error!("Buy failed for {}: {}", token_info.display_name(), e);
                    // allow retry
                    self.bought_tokens.remove(&mint_str);
                }
            }
        } else {
            error!("No wallet configured for buying");
            self.bought_tokens.remove(&mint_str);
        }

        Ok(())
    }

    pub fn get_stats(&self) -> (usize, usize) {
        (self.tracked_tokens.len(), 0)
    }
}

pub async fn run() -> Result<()> {
    tracing_subscriber::fmt::init();

    let config = StreamConfig::from_env().unwrap_or_default();
    let mut sniper = Sniper::new(config).await?;

    sniper.start().await?;
    Ok(())
}
