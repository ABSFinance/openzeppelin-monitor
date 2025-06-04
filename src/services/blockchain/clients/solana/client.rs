//! Solana blockchain client implementation.
//!
//! This module provides functionality to interact with the Solana blockchain,
//! supporting operations like block retrieval, transaction lookup, and event filtering.

use anyhow::Context;
use async_trait::async_trait;
use solana_client::rpc_client::RpcClient;
use std::marker::PhantomData;

use crate::{
	models::{BlockType, ContractSpec, Network, SolanaBlock, SolanaTransaction},
	services::{
		blockchain::{
			client::{BlockChainClient, BlockFilterFactory},
			// transports::SolanaTransportClient,s
			BlockchainTransport,
		},
		// filter::SolanaBlockFilter,
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
impl<T: Send + Sync + Clone> BlockChainClient for SolanaClient<T> {
	async fn get_blocks(&self, start: u64, end: u64) -> Result<Vec<BlockType>, anyhow::Error> {
		unimplemented!("Solana get_blocks not implemented")
	}

	async fn get_latest_block_number(&self) -> Result<u64, anyhow::Error> {
		unimplemented!("Solana get_latest_block_number not implemented")
	}
}

// #[async_trait]
// impl<T: Send + Sync + Clone> BlockFilterFactory<SolanaClient<T>> for SolanaClient<T> {
// 	type Filter = SolanaBlockFilter;

// 	fn create_filter(&self) -> Self::Filter {
// 		SolanaBlockFilter::new()
// 	}
// }
