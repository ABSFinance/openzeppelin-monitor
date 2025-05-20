use solana_client::rpc_client::RpcClient;
use solana_sdk::transaction::Transaction;

use crate::{
	models::{BlockType, Monitor, MonitorMatch},
	services::filter::error::FilterError,
};

/// Helper functions for Solana block filtering
pub struct SolanaFilterHelpers;

impl SolanaFilterHelpers {
	pub fn new() -> Self {
		Self
	}

	/// Check if a transaction matches the given program ID
	pub fn matches_program_id(&self, tx: &Transaction, program_id: &str) -> bool {
		tx.message
			.account_keys
			.iter()
			.any(|key| key.to_string() == program_id)
	}

	/// Check if a transaction matches the given account
	pub fn matches_account(&self, tx: &Transaction, account: &str) -> bool {
		tx.message
			.account_keys
			.iter()
			.any(|key| key.to_string() == account)
	}

	/// Check if a transaction matches the given instruction data
	pub fn matches_instruction_data(&self, tx: &Transaction, data: &[u8]) -> bool {
		tx.message.instructions.iter().any(|ix| ix.data == data)
	}
}
