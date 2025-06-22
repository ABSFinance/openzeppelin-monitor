//! Solana blockchain client implementation.
//!
//! This module provides functionality to interact with the Solana blockchain,
//! supporting operations like block retrieval, transaction lookup, and event filtering.

use anyhow::Context;
use async_trait::async_trait;
use solana_client::rpc_client::RpcClient;

use std::{any::Any, str::FromStr};

use crate::models::default_ui_transaction_status_meta;

use crate::{
	models::{BlockType, Network, SolanaBlock, SolanaTransaction},
	services::{
		blockchain::{
			client::{BlockChainClient, BlockFilterFactory},
			transports::SolanaTransportClient,
		},
		filter::SolanaBlockFilter,
	},
};

/// Client implementation for the Solana blockchain
///
/// Provides high-level access to Solana blockchain data and operations through HTTP transport.
#[derive(Clone)]
pub struct SolanaClient<T: Send + Sync + Clone> {
	/// The underlying Solana transport client for RPC communication
	http_client: T,
}

impl<T: Send + Sync + Clone> SolanaClient<T> {
	/// Creates a new Solana client instance with a specific transport client
	pub fn new_with_transport(http_client: T) -> Self {
		Self { http_client }
	}
}

impl<T: Send + Sync + Clone + 'static> SolanaClient<T> {
	/// Returns a reference to the Any trait for downcasting
	pub fn as_any(&self) -> &dyn Any {
		self
	}
}

impl SolanaClient<SolanaTransportClient> {
	/// Creates a new Solana client instance
	///
	/// # Arguments
	/// * `network` - Network configuration containing RPC endpoints and chain details
	///
	/// # Returns
	/// * `Result<Self, anyhow::Error>` - New client instance or connection error
	pub async fn new(network: &Network) -> Result<Self, anyhow::Error> {
		let http_client = SolanaTransportClient::new(network).await?;
		Ok(Self::new_with_transport(http_client))
	}
}

#[async_trait]
impl<T: Send + Sync + Clone> BlockFilterFactory<Self> for SolanaClient<T> {
	type Filter = SolanaBlockFilter<Self>;

	fn filter() -> Self::Filter {
		SolanaBlockFilter::new()
	}
}

/// Trait for Solana blockchain clients
///
/// This trait defines the interface for interacting with Solana blockchain nodes,
/// providing methods for common operations like block retrieval and transaction lookup.
#[async_trait]
pub trait SolanaClientTrait: BlockChainClient {
	/// Gets the underlying RPC client
	fn rpc_client(&self) -> &RpcClient;

	/// Gets a block by slot number
	async fn get_block_by_slot(&self, slot: u64) -> Result<BlockType, anyhow::Error>;

	/// Gets a transaction by signature
	async fn get_transaction_by_signature(
		&self,
		signature: &str,
	) -> Result<SolanaTransaction, anyhow::Error>;

	/// Gets the latest slot number
	async fn get_latest_slot(&self) -> Result<u64, anyhow::Error>;

	/// Gets the block time for a given slot
	async fn get_block_time(&self, slot: u64) -> Result<i64, anyhow::Error>;
}

#[async_trait]
impl BlockChainClient for SolanaClient<SolanaTransportClient> {
	async fn get_blocks(
		&self,
		start: u64,
		end_block: Option<u64>,
	) -> Result<Vec<BlockType>, anyhow::Error> {
		let mut blocks = Vec::new();
		let end_slot = if let Some(end) = end_block {
			end
		} else {
			// If no end block specified, get the latest slot
			self.http_client.client.get_slot()?
		};

		for slot in start..=end_slot {
			match self.get_block_by_slot(slot).await {
				Ok(block) => blocks.push(block),
				Err(e) => {
					// Log error but continue with other blocks
					log::warn!("Failed to get block at slot {}: {:?}", slot, e);
					continue;
				}
			}
		}

		Ok(blocks)
	}

	async fn get_latest_block_number(&self) -> Result<u64, anyhow::Error> {
		// For Solana, the latest block number is equivalent to the latest slot
		self.get_latest_slot().await
	}
}

#[async_trait]
impl SolanaClientTrait for SolanaClient<SolanaTransportClient> {
	fn rpc_client(&self) -> &RpcClient {
		&self.http_client.client
	}

	async fn get_block_by_slot(&self, slot: u64) -> Result<BlockType, anyhow::Error> {
		// Get block with configuration similar to Carbon's RpcBlockCrawler
		let block_config = solana_client::rpc_config::RpcBlockConfig {
			max_supported_transaction_version: Some(0),
			..Default::default()
		};

		let block = self
			.http_client
			.client
			.get_block_with_config(slot, block_config)?;

		// Convert UiConfirmedBlock to our SolanaBlock format
		let transactions: Vec<SolanaTransaction> = block
			.transactions
			.unwrap_or_default()
			.into_iter()
			.filter_map(|encoded_tx| {
				// Skip failed transactions
				if let Some(meta) = &encoded_tx.meta {
					if meta.status.is_err() {
						return None;
					}
				}

				// Decode the transaction
				let decoded_tx = encoded_tx.transaction.decode()?;

				Some(SolanaTransaction {
					signature: decoded_tx.signatures[0],
					transaction: decoded_tx,
					meta: encoded_tx
						.meta
						.unwrap_or_else(default_ui_transaction_status_meta),
					slot,
					block_time: block.block_time,
				})
			})
			.collect();

		// Convert rewards from Solana SDK format to our format
		let rewards = block.rewards.map(|rewards| {
			rewards
				.into_iter()
				.map(|reward| crate::models::SolanaReward {
					pubkey: reward.pubkey,
					lamports: reward.lamports,
					reward_type: reward
						.reward_type
						.map(|rt| format!("{:?}", rt))
						.unwrap_or_default(),
					commission: reward.commission,
				})
				.collect()
		});

		let solana_block = SolanaBlock {
			slot,
			blockhash: block.blockhash,
			parent_slot: block.parent_slot,
			transactions,
			block_time: block.block_time,
			block_height: block.block_height,
			rewards,
			commitment: solana_sdk::commitment_config::CommitmentConfig::confirmed(),
		};

		Ok(BlockType::Solana(Box::new(solana_block)))
	}

	async fn get_transaction_by_signature(
		&self,
		signature: &str,
	) -> Result<SolanaTransaction, anyhow::Error> {
		// Parse signature
		let signature =
			solana_signature::Signature::from_str(signature).context("Invalid signature format")?;

		// Get transaction with configuration similar to Carbon's RpcTransactionCrawler
		let tx_config = solana_client::rpc_config::RpcTransactionConfig {
			encoding: Some(solana_transaction_status::UiTransactionEncoding::Base64),
			commitment: Some(solana_sdk::commitment_config::CommitmentConfig::confirmed()),
			max_supported_transaction_version: Some(0),
		};

		let encoded_tx = self
			.http_client
			.client
			.get_transaction_with_config(&signature, tx_config)?;

		// Skip failed transactions
		if let Some(meta) = &encoded_tx.transaction.meta {
			if meta.status.is_err() {
				return Err(anyhow::anyhow!("Transaction failed"));
			}
		}

		// Decode the transaction
		let decoded_tx = encoded_tx
			.transaction
			.transaction
			.decode()
			.ok_or_else(|| anyhow::anyhow!("Failed to decode transaction"))?;

		// Convert to our SolanaTransaction format
		let transaction = SolanaTransaction {
			signature,
			transaction: decoded_tx,
			meta: encoded_tx
				.transaction
				.meta
				.unwrap_or_else(default_ui_transaction_status_meta),
			slot: encoded_tx.slot,
			block_time: encoded_tx.block_time,
		};

		Ok(transaction)
	}

	async fn get_latest_slot(&self) -> Result<u64, anyhow::Error> {
		// Get the current slot from the RPC client
		let slot = self.http_client.client.get_slot()?;
		Ok(slot)
	}

	async fn get_block_time(&self, slot: u64) -> Result<i64, anyhow::Error> {
		// Get block time for a specific slot
		let block_time = self.http_client.client.get_block_time(slot)?;
		Ok(block_time)
	}
}
