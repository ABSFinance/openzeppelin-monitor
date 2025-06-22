//! Solana transport client implementation.
//!
//! This module provides a transport client for interacting with Solana nodes
//! via JSON-RPC, supporting:
//! - Multiple RPC endpoints with automatic failover
//! - Configurable retry policies
//! - Connection health checks
//! - Endpoint rotation for high availability

use async_trait::async_trait;
use reqwest::Client;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};
use serde::Serialize;
use serde_json::{json, Value};
use solana_client::rpc_client::RpcClient;
use std::sync::Arc;

use crate::{
	models::Network,
	services::blockchain::transports::{
		BlockchainTransport, EndpointManager, RotatingTransport, TransientErrorRetryStrategy,
	},
};

/// Solana transport client for blockchain interactions
///
/// This client provides a foundation for making JSON-RPC requests to Solana nodes
/// with built-in support for:
/// - Connection pooling and reuse
/// - Automatic endpoint rotation on failure
/// - Configurable retry policies
///
/// The client is thread-safe and can be shared across multiple tasks.
#[derive(Clone)]
pub struct SolanaTransportClient {
	/// RPC client for making requests
	pub client: Arc<RpcClient>,
	/// Manages RPC endpoint rotation and request handling for high availability
	endpoint_manager: EndpointManager,
}

impl SolanaTransportClient {
	/// Creates a new Solana transport client with automatic endpoint management
	///
	/// This constructor attempts to connect to available endpoints in order of their
	/// weight until a successful connection is established.
	///
	/// # Arguments
	/// * `network` - Network configuration containing RPC URLs, weights, and other details
	///
	/// # Returns
	/// * `Result<Self, anyhow::Error>` - New client instance or connection error
	pub async fn new(network: &Network) -> Result<Self, anyhow::Error> {
		let mut rpc_urls: Vec<_> = network
			.rpc_urls
			.iter()
			.filter(|rpc_url| rpc_url.type_ == "rpc" && rpc_url.weight > 0)
			.collect();

		rpc_urls.sort_by(|a, b| b.weight.cmp(&a.weight));

		if rpc_urls.is_empty() {
			return Err(anyhow::anyhow!("No valid RPC URLs found"));
		}

		let client = Arc::new(RpcClient::new(rpc_urls[0].url.as_ref().to_string()));

		let middleware_client = ClientBuilder::new(Client::new())
			.with(RetryTransientMiddleware::new_with_policy(
				ExponentialBackoff::builder().build_with_max_retries(3),
			))
			.build();

		let endpoint_manager = EndpointManager::new(
			middleware_client,
			rpc_urls[0].url.as_ref(),
			rpc_urls[1..]
				.iter()
				.map(|url| url.url.as_ref().to_string())
				.collect(),
		);

		Ok(Self {
			client,
			endpoint_manager,
		})
	}
}

#[async_trait]
impl BlockchainTransport for SolanaTransportClient {
	async fn get_current_url(&self) -> String {
		self.endpoint_manager.active_url.read().await.clone()
	}

	async fn send_raw_request<P>(
		&self,
		method: &str,
		params: Option<P>,
	) -> Result<Value, anyhow::Error>
	where
		P: Into<Value> + Send + Clone + Serialize,
	{
		self.endpoint_manager
			.send_raw_request(self, method, params)
			.await
	}

	async fn customize_request<P: Into<Value> + Send + Clone + Serialize>(
		&self,
		method: &str,
		params: Option<P>,
	) -> Value {
		json!({
			"jsonrpc": "2.0",
			"id": 1,
			"method": method,
			"params": params.map(|p| p.into()).unwrap_or(Value::Null)
		})
	}

	fn set_retry_policy(
		&mut self,
		retry_policy: ExponentialBackoff,
		retry_strategy: Option<TransientErrorRetryStrategy>,
	) -> Result<(), anyhow::Error> {
		let strategy = retry_strategy.unwrap_or(TransientErrorRetryStrategy);
		self.endpoint_manager
			.set_retry_policy(retry_policy, strategy);
		Ok(())
	}

	fn update_endpoint_manager_client(
		&mut self,
		client: ClientWithMiddleware,
	) -> Result<(), anyhow::Error> {
		self.endpoint_manager.update_client(client);
		Ok(())
	}
}

#[async_trait]
impl RotatingTransport for SolanaTransportClient {
	async fn try_connect(&self, url: &str) -> Result<(), anyhow::Error> {
		// Create a temporary RPC client to test the connection
		let test_client = RpcClient::new(url.to_string());
		test_client.get_slot()?;
		Ok(())
	}

	async fn update_client(&self, _url: &str) -> Result<(), anyhow::Error> {
		Ok(())
	}
}
