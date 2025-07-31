//! Monitor  - Track Pump token creations and mcs

use anyhow::Result;
use asuga_trial::{
    common::{Config, SniperEvent, StreamClient, MarketData},
    utils::{TransactionExecutor, PriceFetcher},
    accounts::TokenInfo,
};
use std::collections::HashMap;
use tokio::sync::mpsc;
use tracing::{error, info, warn};
use std::time::{Duration, Instant};

struct TokenTracker {
    token_info: TokenInfo,
    initial_market_cap_usd: f64,
    current_market_cap_usd: f64,
    first_seen: Instant,
    last_updated: Instant,
}

impl TokenTracker {
    fn new(token_info: TokenInfo, market_cap_usd: f64) -> Self {
        let now = Instant::now();
        Self {
            token_info,
            initial_market_cap_usd: market_cap_usd,
            current_market_cap_usd: market_cap_usd,
            first_seen: now,
            last_updated: now,
        }
    }

    fn update_market_cap(&mut self, market_cap_usd: f64) {
        self.current_market_cap_usd = market_cap_usd;
        self.last_updated = Instant::now();
    }

    fn age_seconds(&self) -> u64 {
        self.first_seen.elapsed().as_secs()
    }

    fn market_cap_change_percent(&self) -> f64 {
        if self.initial_market_cap_usd == 0.0 {
            return 0.0;
        }
        ((self.current_market_cap_usd - self.initial_market_cap_usd) / self.initial_market_cap_usd) * 100.0
    }
}

struct MonitorBot {
    tracked_tokens: HashMap<String, TokenTracker>,
    event_receiver: mpsc::UnboundedReceiver<SniperEvent>,
    transaction_executor: TransactionExecutor,
    price_fetcher: PriceFetcher,
    start_time: Instant,
}

impl MonitorBot {
    fn new(
        event_receiver: mpsc::UnboundedReceiver<SniperEvent>,
        config: Config,
    ) -> Self {
        let transaction_executor = TransactionExecutor::new(config);
        let price_fetcher = PriceFetcher::new();
        
        Self {
            tracked_tokens: HashMap::new(),
            event_receiver,
            transaction_executor,
            price_fetcher,
            start_time: Instant::now(),
        }
    }

    async fn run(&mut self) -> Result<()> {
        info!("Starting Pump.Fun Monitor Mode");
        info!("Continuous tracking of all tokens with live market cap updates");
        info!("Watching for new token creations...\n");

        // Print header
        self.print_header();

        let mut last_update_check = Instant::now();
        let mut last_display_refresh = Instant::now();
        
        loop {
            // Check for new events (non-blocking)
            while let Ok(event) = self.event_receiver.try_recv() {
                match event {
                    SniperEvent::TokenCreated(token_info) => {
                        self.handle_new_token(token_info).await;
                    }
                    SniperEvent::MarketCapUpdated(market_data) => {
                        self.handle_market_cap_update(market_data).await;
                    }
                    _ => {}
                }
            }

            // Update all tracked tokens every 3 seconds
            if last_update_check.elapsed() >= Duration::from_secs(3) {
                self.update_all_market_caps().await;
                last_update_check = Instant::now();
            }

            // Refresh display every 2 seconds
            if last_display_refresh.elapsed() >= Duration::from_secs(2) {
                self.refresh_display();
                last_display_refresh = Instant::now();
            }

            // Small delay to prevent busy waiting
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }

    fn print_header(&self) {
        println!("{}", "=".repeat(120));
        println!("{:^120}", "PUMP.FUN TOKEN MONITOR");
        println!("{}", "=".repeat(120));
        println!(
            "{:<45} {:<15} {:<12} {:<12} {:<12} {:<8} {:<10}",
            "TOKEN (SYMBOL)", "MINT ADDRESS", "INITIAL MC", "CURRENT MC", "CHANGE %", "AGE (s)", "STATUS"
        );
        println!("{}", "-".repeat(120));
    }

    async fn handle_new_token(&mut self, token_info: TokenInfo) {
        info!("New token detected: {} ({})", token_info.name, token_info.symbol);
        
        // Small delay to allow bonding curve to be created
        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
        
        // Try to get actual bonding curve data first (current state)
        match self.transaction_executor.fetch_bonding_curve_data(&token_info.bonding_curve).await {
            Ok(bonding_curve_data) => {
                let market_cap_sol = bonding_curve_data.get_market_cap_sol();
                
                match self.price_fetcher.calculate_market_cap_usd(market_cap_sol).await {
                    Ok(market_cap_usd) => {
                        // Add to tracking
                        let tracker = TokenTracker::new(token_info.clone(), market_cap_usd);
                        self.tracked_tokens.insert(token_info.mint.to_string(), tracker);
                        
                        info!(
                            "{} added to tracking - Initial MC: ${:.2}",
                            token_info.symbol,
                            market_cap_usd
                        );
                    }
                    Err(e) => {
                        error!("Failed to calculate market cap in USD for {}: {}", token_info.symbol, e);
                    }
                }
            }
            Err(e) => {
                // Fallback: use global account initial values and add to tracking
                match self.transaction_executor.fetch_global_account().await {
                    Ok(global_account) => {
                        let market_cap_sol = global_account.get_initial_market_cap_sol();
                        
                        match self.price_fetcher.calculate_market_cap_usd(market_cap_sol).await {
                            Ok(market_cap_usd) => {
                                let tracker = TokenTracker::new(token_info.clone(), market_cap_usd);
                                self.tracked_tokens.insert(token_info.mint.to_string(), tracker);
                                
                                info!(
                                    "{} added to tracking (fallback) - Initial MC: ${:.2}",
                                    token_info.symbol,
                                    market_cap_usd
                                );
                            }
                            Err(e2) => {
                                error!("Failed to add {} to tracking: bonding curve error: {}, price error: {}", 
                                       token_info.symbol, e, e2);
                            }
                        }
                    }
                    Err(e2) => {
                        error!("Failed to add {} to tracking: bonding: {}, global: {}", 
                               token_info.symbol, e, e2);
                    }
                }
            }
        }
    }

    async fn handle_market_cap_update(&mut self, market_data: MarketData) {
        let mint_str = market_data.token_info.mint.to_string();
        
        if let Some(tracker) = self.tracked_tokens.get_mut(&mint_str) {
            // Convert SOL market cap to USD
            match self.price_fetcher.calculate_market_cap_usd(market_data.current_market_cap_sol).await {
                Ok(market_cap_usd) => {
                    let old_market_cap = tracker.current_market_cap_usd;
                    tracker.update_market_cap(market_cap_usd);
                    
                    // Only print if there's a significant change (>1% or >$50)
                    let change_percent = ((market_cap_usd - old_market_cap) / old_market_cap).abs() * 100.0;
                    let change_usd = (market_cap_usd - old_market_cap).abs();
                    
                    if change_percent > 1.0 || change_usd > 50.0 {
                        let change_str = if tracker.current_market_cap_usd > tracker.initial_market_cap_usd {
                            format!("🟢+{:.2}%", tracker.market_cap_change_percent())
                        } else {
                            format!("🔴{:.2}%", tracker.market_cap_change_percent())
                        };
                        
                        println!(
                            "{:<45} {:<15} {:<12.2} {:<12.2} {:<12} {:<8} {:<10}",
                            format!("{} ({})", 
                                truncate_string(&tracker.token_info.name, 25),
                                &tracker.token_info.symbol
                            ),
                            truncate_string(&mint_str, 15),
                            tracker.initial_market_cap_usd,
                            tracker.current_market_cap_usd,
                            change_str,
                            tracker.age_seconds(),
                            "📈 UPDATE"
                        );
                    }
                }
                Err(e) => {
                    error!("❌ Failed to calculate market cap in USD for update: {}", e);
                }
            }
        }
    }

    /// Update market caps for all tracked tokens
    async fn update_all_market_caps(&mut self) {
        for (_mint, tracker) in self.tracked_tokens.iter_mut() {
            // Skip if updated recently (within 1 second)
            if tracker.last_updated.elapsed() < Duration::from_secs(1) {
                continue;
            }

            match self.transaction_executor.fetch_bonding_curve_data(&tracker.token_info.bonding_curve).await {
                Ok(bonding_curve_data) => {
                    let market_cap_sol = bonding_curve_data.get_market_cap_sol();
                    
                    match self.price_fetcher.calculate_market_cap_usd(market_cap_sol).await {
                        Ok(market_cap_usd) => {
                            let old_market_cap = tracker.current_market_cap_usd;
                            tracker.update_market_cap(market_cap_usd);
                            
                            // Log significant changes (>5% or >$100)
                            let change_percent = ((market_cap_usd - old_market_cap) / old_market_cap).abs() * 100.0;
                            let change_usd = (market_cap_usd - old_market_cap).abs();
                            
                            if change_percent > 5.0 || change_usd > 100.0 {
                                info!(
                                    "{} market cap updated: ${:.2} ({}{}%)",
                                    tracker.token_info.symbol,
                                    market_cap_usd,
                                    if market_cap_usd > old_market_cap { "+" } else { "" },
                                    (market_cap_usd - old_market_cap) / old_market_cap * 100.0
                                );
                            }
                        }
                        Err(e) => {
                            // Don't spam errors, just continue
                            if tracker.age_seconds() % 30 == 0 {
                                error!("Failed to calculate USD market cap for {}: {}", tracker.token_info.symbol, e);
                            }
                        }
                    }
                }
                Err(e) => {
                    // Don't spam errors, just continue
                    if tracker.age_seconds() % 30 == 0 {
                        error!("Failed to fetch bonding curve for {}: {}", tracker.token_info.symbol, e);
                    }
                }
            }
            
            // Small delay between requests to avoid rate limits
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    }

    /// Refresh the display with current token data
    fn refresh_display(&self) {
        if self.tracked_tokens.is_empty() {
            return;
        }

        // Clear screen and reprint header
        print!("\x1B[2J\x1B[1;1H"); // Clear screen and move cursor to top
        self.print_header();

        // Sort tokens by market cap (descending)
        let mut tokens: Vec<_> = self.tracked_tokens.iter().collect();
        tokens.sort_by(|a, b| b.1.current_market_cap_usd.partial_cmp(&a.1.current_market_cap_usd).unwrap());

        // Print all tracked tokens
        for (mint, tracker) in tokens {
            let change_str = if tracker.current_market_cap_usd > tracker.initial_market_cap_usd {
                format!("+{:.2}%", tracker.market_cap_change_percent())
            } else if tracker.current_market_cap_usd < tracker.initial_market_cap_usd {
                format!("{:.2}%", tracker.market_cap_change_percent())
            } else {
                "0.00%".to_string()
            };

            // Determine status based on market cap
            let status = if tracker.current_market_cap_usd >= 8000.0 {
                "BUY!"
            } else if tracker.current_market_cap_usd >= 7000.0 {
                "CLOSE"
            } else if tracker.age_seconds() < 5 {
                "NEW"
            } else {
                "TRACKING"
            };

            println!(
                "{:<45} {:<15} {:<12.2} {:<12.2} {:<12} {:<8} {:<10}",
                format!("{} ({})", 
                    truncate_string(&tracker.token_info.name, 25),
                    &tracker.token_info.symbol
                ),
                truncate_string(mint, 15),
                tracker.initial_market_cap_usd,
                tracker.current_market_cap_usd,
                change_str,
                tracker.age_seconds(),
                status
            );
        }

        // Print status footer
        self.print_status();
    }

    fn print_status(&self) {
        let uptime = self.start_time.elapsed().as_secs();
        let tracked_count = self.tracked_tokens.len();
        
        println!("{}", "-".repeat(120));
        println!(
            "Status: {} tokens tracked | Uptime: {}s | Last update: {} | Threshold: $8000",
            tracked_count,
            uptime,
            chrono::Utc::now().format("%H:%M:%S")
        );
        println!("{}", "-".repeat(120));
    }
}

fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    // Load configuration
    let config = Config::from_env().unwrap_or_else(|_| {
        warn!("Failed to load config from environment, using defaults");
        Config::default()
    });

    // Validate config
    if let Err(e) = config.validate() {
        error!("Invalid configuration: {}", e);
        return Err(e.into());
    }

    info!("Configuration loaded:");
    info!("  gRPC Endpoint: {}", config.grpc_endpoint);
    info!("  RPC Endpoint: {}", config.rpc_endpoint);
    info!("  Market Cap Threshold: ${:.2} USD", config.market_cap_threshold_usd_display());

    // Create event channel
    let (event_sender, event_receiver) = mpsc::unbounded_channel();

    // Start gRPC streaming
    let mut stream_client = StreamClient::new(config.clone(), event_sender);
    
    // Start monitor bot
    let mut monitor = MonitorBot::new(event_receiver, config);
    
    // Run both concurrently
    tokio::select! {
        result = stream_client.start() => {
            error!("gRPC stream ended: {:?}", result);
        }
        result = monitor.run() => {
            error!("Monitor ended: {:?}", result);
        }
    }

    Ok(())
}