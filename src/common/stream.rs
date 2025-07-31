//! gRPC streaming utilities

use crate::{
    common::{Config, SniperEvent},
    error::SniperError,
    utils::parser,
};
use anyhow::Result;
use futures::{sink::SinkExt, stream::StreamExt};
use std::collections::HashMap;
use tokio::sync::mpsc;
use tonic::transport::ClientTlsConfig;
use tracing::{error, info};
use yellowstone_grpc_client::GeyserGrpcClient;
use yellowstone_grpc_proto::prelude::*;

pub struct StreamClient {
    config: Config,
    event_sender: mpsc::UnboundedSender<SniperEvent>,
}

impl StreamClient {
    pub fn new(config: Config, event_sender: mpsc::UnboundedSender<SniperEvent>) -> Self {
        Self {
            config,
            event_sender,
        }
    }

    pub async fn start(&mut self) -> Result<(), SniperError> {
        info!("CONNECTING to gRPC endpoint: {}", self.config.grpc_endpoint);

        let mut client = GeyserGrpcClient::build_from_shared(self.config.grpc_endpoint.clone())
            .map_err(|e| SniperError::GrpcConnectionFailed(e.to_string()))?
            .tls_config(ClientTlsConfig::new())
            .map_err(|e| SniperError::GrpcConnectionFailed(e.to_string()))?
            .connect()
            .await
            .map_err(|e| SniperError::GrpcConnectionFailed(e.to_string()))?;

        info!("CONNECTED to gRPC endpoint");

        let _ = self.event_sender.send(SniperEvent::ConnectionStatusChanged {
            connected: true,
            endpoint: self.config.grpc_endpoint.clone(),
        });

        let (mut subscribe_tx, mut subscribe_rx) = client
            .subscribe()
            .await
            .map_err(|e| SniperError::GrpcConnectionFailed(e.to_string()))?;

        let request = self.create_subscription_request();
        
        subscribe_tx
            .send(request)
            .await
            .map_err(|e| SniperError::GrpcConnectionFailed(e.to_string()))?;

        info!("SUBSCRIPTION ACTIVE - monitoring Pump transactions...");

        while let Some(update) = subscribe_rx.next().await {
            match update {
                Ok(update) => {
                    if let Err(e) = self.handle_update(update).await {
                        error!("Error handling update: {}", e);
                    }
                }
                Err(e) => {
                    error!("Stream error: {}", e);
                    
                    let _ = self.event_sender.send(SniperEvent::ConnectionStatusChanged {
                        connected: false,
                        endpoint: self.config.grpc_endpoint.clone(),
                    });
                    
                    tokio::time::sleep(tokio::time::Duration::from_millis(5000)).await;
                    
                    return Err(SniperError::GrpcConnectionFailed(e.to_string()));
                }
            }
        }

        Ok(())
    }

    fn create_subscription_request(&self) -> SubscribeRequest {
        use crate::constants::PUMPFUN_PROGRAM_ID;

        SubscribeRequest {
            // bonding curve updates
            accounts: [(
                "bonding_curves".to_string(),
                SubscribeRequestFilterAccounts {
                    account: vec![],
                    owner: vec![PUMPFUN_PROGRAM_ID.to_string()],
                    filters: vec![
                        SubscribeRequestFilterAccountsFilter {
                            filter: Some(
                                subscribe_request_filter_accounts_filter::Filter::Datasize(
                                    8 + 32 + 32 + 8 + 8 + 8 + 8 + 1 // BondingCurveAccount size
                                )
                            ),
                        },
                    ],
                },
            )]
            .into(),
            slots: HashMap::new(),
            transactions: [(
                "pumpfun_transactions".to_string(),
                SubscribeRequestFilterTransactions {
                    vote: Some(false),
                    failed: Some(false),
                    signature: None,
                    account_include: vec![PUMPFUN_PROGRAM_ID.to_string()],
                    account_exclude: vec![],
                    account_required: vec![],
                },
            )]
            .into(),
            transactions_status: HashMap::new(),
            blocks: HashMap::new(),
            blocks_meta: HashMap::new(),
            entry: HashMap::new(),
            accounts_data_slice: vec![],
            ping: None,
            commitment: Some(CommitmentLevel::Processed as i32),
        }
    }

    async fn handle_update(&self, update: SubscribeUpdate) -> Result<()> {
        match update.update_oneof {
            Some(subscribe_update::UpdateOneof::Transaction(transaction)) => {
                self.handle_transaction(transaction).await
            }
            Some(subscribe_update::UpdateOneof::Account(account)) => {
                self.handle_account_update(account).await
            }
            Some(subscribe_update::UpdateOneof::Ping(_)) => {
                Ok(())
            }
            _ => Ok(()),
        }
    }

    async fn handle_transaction(&self, transaction: SubscribeUpdateTransaction) -> Result<()> {
        if let Some(transaction_info) = transaction.transaction {
            if let Some(ref meta) = transaction_info.meta {
                if meta.err.is_none() {
                    let signature = bs58::encode(&transaction_info.signature).into_string();
                    
                    if parser::is_create_transaction(&transaction_info) {
                        info!("TOKEN CREATION DETECTED: {}", signature);
                        
                        if let Some(token_info) = parser::parse_token_creation(&transaction_info, signature) {
                            if let Err(e) = self.event_sender.send(SniperEvent::TokenCreated(token_info)) {
                                error!("Failed to send token creation event: {}", e);
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    async fn handle_account_update(&self, account_update: SubscribeUpdateAccount) -> Result<()> {
        if let Some(account_info) = account_update.account {
            let account_key = bs58::encode(&account_info.pubkey).into_string();
            if let Ok(pubkey) = account_key.parse::<solana_sdk::pubkey::Pubkey>() {
                match solana_sdk::borsh1::try_from_slice_unchecked::<crate::accounts::BondingCurveAccount>(&account_info.data) {
                    Ok(bonding_curve_data) => {
                        if let Err(e) = self.event_sender.send(crate::common::SniperEvent::BondingCurveUpdated {
                            bonding_curve: pubkey,
                            data: bonding_curve_data,
                        }) {
                            error!("Failed to send bonding curve update: {}", e);
                        }
                    }
                    Err(_) => {}
                }
            }
        }
        Ok(())
    }
}