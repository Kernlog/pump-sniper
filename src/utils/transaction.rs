//! Transaction execution

use crate::{
    accounts::{BondingCurveAccount, GlobalAccount, TokenInfo},
    common::Config,
    error::SniperError,
    instructions::BuyInstruction,
    utils::pda::derive_global_pda,
};
use anyhow::Result;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    compute_budget::ComputeBudgetInstruction,
    signature::{Keypair, Signature},
    signer::Signer,
    transaction::Transaction,
};
use tracing::info;

pub struct TransactionExecutor {
    rpc_client: RpcClient,
    config: Config,
}

impl TransactionExecutor {
    pub fn new(config: Config) -> Self {
        let rpc_client = RpcClient::new_with_commitment(
            config.rpc_endpoint.clone(),
            CommitmentConfig::confirmed(),
        );

        Self { rpc_client, config }
    }

    pub async fn fetch_global_account(&self) -> Result<GlobalAccount, SniperError> {
        let global_pda = derive_global_pda()?;

        match self.rpc_client.get_account(&global_pda) {
            Ok(account) => {
                match solana_sdk::borsh1::try_from_slice_unchecked::<GlobalAccount>(&account.data) {
                    Ok(global_data) => Ok(global_data),
                    Err(e) => Err(SniperError::SerializationError(format!(
                        "Failed to deserialize global account: {}",
                        e
                    ))),
                }
            }
            Err(e) => Err(SniperError::RpcError(format!(
                "Failed to fetch global account: {}",
                e
            ))),
        }
    }

    pub async fn fetch_bonding_curve_data(
        &self,
        bonding_curve: &solana_sdk::pubkey::Pubkey,
    ) -> Result<BondingCurveAccount, SniperError> {
        // progressive retry: 0ms, 100ms, 200ms
        let delays = [0, 100, 200];

        for (attempt, &delay_ms) in delays.iter().enumerate() {
            if delay_ms > 0 {
                tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;
            }

            match self.rpc_client.get_account(bonding_curve) {
                Ok(account) => {
                    match solana_sdk::borsh1::try_from_slice_unchecked::<BondingCurveAccount>(
                        &account.data,
                    ) {
                        Ok(bonding_curve_data) => return Ok(bonding_curve_data),
                        Err(e) => {
                            return Err(SniperError::SerializationError(format!(
                                "Failed to deserialize bonding curve: {}",
                                e
                            )))
                        }
                    }
                }
                Err(e) if attempt == delays.len() - 1 => {
                    return Err(SniperError::RpcError(format!(
                        "Account not found after {} attempts: {}",
                        delays.len(),
                        e
                    )));
                }
                Err(_) => {
                    continue;
                }
            }
        }

        Err(SniperError::RpcError("Unexpected error".to_string()))
    }

    pub fn build_buy_transaction(
        &self,
        payer: &Keypair,
        token_info: &TokenInfo,
        bonding_curve_data: &BondingCurveAccount,
        sol_amount: u64,
        fee_recipient: &solana_sdk::pubkey::Pubkey,
    ) -> Result<Transaction, SniperError> {
        let expected_tokens = bonding_curve_data.get_buy_price(sol_amount)?;

        // slippage protection
        let max_sol_cost = sol_amount + (sol_amount * self.config.max_slippage_bps / 10000);

        let buy_instruction_data = BuyInstruction {
            amount: expected_tokens,
            max_sol_cost,
        };

        let buy_instruction = buy_instruction_data.create_instruction(
            payer,
            &token_info.mint,
            fee_recipient,
            &token_info.creator,
        )?;

        let mut instructions = Vec::with_capacity(4);

        // priority fee
        let priority_fee_microlamports =
            (self.config.priority_fee_sol * 1_000_000) / self.config.compute_unit_limit as u64;
        instructions.push(ComputeBudgetInstruction::set_compute_unit_price(
            priority_fee_microlamports,
        ));

        // compute limit
        instructions.push(ComputeBudgetInstruction::set_compute_unit_limit(
            self.config.compute_unit_limit,
        ));

        instructions.push(
            spl_associated_token_account::instruction::create_associated_token_account(
                &payer.pubkey(),
                &payer.pubkey(),
                &token_info.mint,
                &spl_token::id(),
            ),
        );

        instructions.push(buy_instruction);

        let recent_blockhash = self
            .rpc_client
            .get_latest_blockhash()
            .map_err(|e| SniperError::RpcError(e.to_string()))?;

        let transaction = Transaction::new_signed_with_payer(
            &instructions,
            Some(&payer.pubkey()),
            &[payer],
            recent_blockhash,
        );

        Ok(transaction)
    }

    pub async fn execute_buy(
        &self,
        payer: &Keypair,
        token_info: &TokenInfo,
        sol_amount: u64,
    ) -> Result<Signature, SniperError> {
        info!(
            "FAST BUY: {} - {} SOL",
            token_info.symbol,
            sol_amount as f64 / 1e9
        );

        // fetch parallel
        let (global_result, bonding_result) = tokio::join!(
            self.fetch_global_account(),
            self.fetch_bonding_curve_data(&token_info.bonding_curve)
        );

        let global_account = global_result?;
        let bonding_curve_data = bonding_result?;

        let transaction = self.build_buy_transaction(
            payer,
            token_info,
            &bonding_curve_data,
            sol_amount,
            &global_account.fee_recipient,
        )?;

        use solana_client::rpc_config::RpcSendTransactionConfig;

        let send_config = RpcSendTransactionConfig {
            skip_preflight: true,
            preflight_commitment: Some(solana_sdk::commitment_config::CommitmentLevel::Processed),
            encoding: None,
            max_retries: Some(0),
            min_context_slot: None,
        };

        let signature = self
            .rpc_client
            .send_transaction_with_config(&transaction, send_config)
            .map_err(|e| SniperError::TransactionFailed(e.to_string()))?;

        info!(
            "Buy transaction sent for {} - TX: {}",
            token_info.display_name(),
            signature
        );

        Ok(signature)
    }

    pub async fn simulate_buy(
        &self,
        payer: &Keypair,
        token_info: &TokenInfo,
        sol_amount: u64,
    ) -> Result<(u64, u64), SniperError> {
        let global_account = self.fetch_global_account().await?;
        let fee_recipient = global_account.fee_recipient;

        let bonding_curve_data = self
            .fetch_bonding_curve_data(&token_info.bonding_curve)
            .await?;

        let expected_tokens = bonding_curve_data.get_buy_price(sol_amount)?;

        let transaction = self.build_buy_transaction(
            payer,
            token_info,
            &bonding_curve_data,
            sol_amount,
            &fee_recipient,
        )?;

        let simulation_result = self
            .rpc_client
            .simulate_transaction(&transaction)
            .map_err(|e| SniperError::TransactionFailed(e.to_string()))?;

        if let Some(err) = simulation_result.value.err {
            return Err(SniperError::TransactionFailed(format!(
                "Simulation failed: {:?}",
                err
            )));
        }

        let compute_units = simulation_result.value.units_consumed.unwrap_or(200_000);

        Ok((expected_tokens, compute_units))
    }
}
