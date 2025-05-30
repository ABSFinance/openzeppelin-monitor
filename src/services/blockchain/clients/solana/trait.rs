//! Solana client trait definition.
//!
//! This module defines the trait for Solana blockchain clients,
//! providing a common interface for all Solana client implementations.

use async_trait::async_trait;
use solana_client::rpc_client::RpcClient;

use crate::{
	models::{BlockType, Network, SolanaTransaction},
	services::blockchain::client::BlockChainClient,
};

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
