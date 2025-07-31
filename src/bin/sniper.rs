//! Sniper with wallet setup

use anyhow::Result;
use asuga_trial::{common::Config, Sniper};
use solana_sdk::{signature::Keypair, signer::Signer};
use std::env;
use tracing::{error, info};

#[tokio::main]
async fn main() -> Result<()> {
    // Init logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    // Load config
    let config = Config::from_env().unwrap_or_else(|e| {
        error!("Failed to load config: {}", e);
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
    info!("  Buy Amount: {:.3} SOL", config.buy_amount_sol_display());
    info!("  Priority Fee: {:.3} SOL", config.priority_fee_sol_display());
    info!("  Slippage: {}%", config.max_slippage_bps as f64 / 100.0);

    // Load wallet from env
    let wallet = load_wallet_from_env()?;
    info!("Wallet loaded: {}", wallet.pubkey());
    
    // Check wallet balance
    let rpc_client = solana_client::rpc_client::RpcClient::new(&config.rpc_endpoint);
    match rpc_client.get_balance(&wallet.pubkey()) {
        Ok(balance) => {
            info!("Wallet balance: {:.6} SOL", balance as f64 / 1e9);
            if balance < config.buy_amount_sol + config.priority_fee_sol + 10_000_000 {
                error!("Insufficient balance for buying! Need at least {} SOL", 
                    (config.buy_amount_sol + config.priority_fee_sol + 10_000_000) as f64 / 1e9);
                return Err(anyhow::anyhow!("Insufficient wallet balance"));
            }
        }
        Err(e) => {
            error!("Failed to check wallet balance: {}", e);
            return Err(e.into());
        }
    }

    // Create and start sniper
    let mut sniper = Sniper::new(config).await?;
    sniper.set_wallet(wallet);
    
    // Enable test mode if TEST_MODE env is set
    if env::var("TEST_MODE").is_ok() {
        sniper.enable_test_mode();
    }
    
    info!("Starting sniper bot...");
    info!("Monitoring for tokens with market cap >= $8,000");
    info!("Will buy 0.05 SOL worth of tokens when threshold is met");
    if env::var("TEST_MODE").is_ok() {
        info!("TEST MODE: Will stop after first successful buy");
    }
    info!("Press Ctrl+C to stop\n");

    sniper.start().await?;
    
    Ok(())
}

/// Load wallet keypair from env
fn load_wallet_from_env() -> Result<Keypair> {
    let private_key = env::var("WALLET_PRIVATE_KEY")
        .map_err(|_| anyhow::anyhow!("WALLET_PRIVATE_KEY environment variable not set"))?;
    
    // Try to parse as base58 private key
    let decoded = bs58::decode(&private_key)
        .into_vec()
        .map_err(|e| anyhow::anyhow!("Failed to decode private key: {}", e))?;
    
    if decoded.len() != 64 {
        return Err(anyhow::anyhow!("Invalid private key length: expected 64 bytes, got {}", decoded.len()));
    }
    
    Ok(Keypair::from_bytes(&decoded)?)
}