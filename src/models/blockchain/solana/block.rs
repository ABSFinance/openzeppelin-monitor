use {
	serde::{Deserialize, Serialize},
	solana_sdk::{commitment_config::CommitmentConfig, transaction::Transaction},
};

use super::transaction::SolanaTransaction;

/// Represents a Solana block with its metadata and transactions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolanaBlock {
	/// The slot number of this block
	pub slot: u64,
	/// The block hash
	pub blockhash: String,
	/// The parent block hash
	pub parent_slot: u64,
	/// The Unix timestamp of when the block was processed
	pub block_time: Option<i64>,
	/// The block height
	pub block_height: Option<u64>,
	/// The transactions in this block
	pub transactions: Vec<Transaction>,
	/// The rewards for this block
	pub rewards: Option<Vec<SolanaReward>>,
	/// The block's commitment level
	pub commitment: CommitmentConfig,
}

/// Represents a reward in a Solana block
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolanaReward {
	/// The public key of the account that received the reward
	pub pubkey: String,
	/// The amount of the reward in lamports
	pub lamports: i64,
	/// The type of reward
	pub reward_type: String,
	/// The commission if applicable
	pub commission: Option<u8>,
}

impl SolanaBlock {
	/// Creates a new SolanaBlock with the given slot and transactions
	pub fn new(
		slot: u64,
		blockhash: String,
		parent_slot: u64,
		block_time: Option<i64>,
		block_height: Option<u64>,
		transactions: Vec<Transaction>,
		rewards: Option<Vec<SolanaReward>>,
		commitment: CommitmentConfig,
	) -> Self {
		Self {
			slot,
			blockhash,
			parent_slot,
			block_time,
			block_height,
			transactions,
			rewards,
			commitment,
		}
	}

	/// Returns the block's slot number
	pub fn slot(&self) -> u64 {
		self.slot
	}

	/// Returns the block's hash
	pub fn blockhash(&self) -> &str {
		&self.blockhash
	}

	/// Returns the parent slot number
	pub fn parent_slot(&self) -> u64 {
		self.parent_slot
	}

	/// Returns the block time if available
	pub fn block_time(&self) -> Option<i64> {
		self.block_time
	}

	/// Returns the block height if available
	pub fn block_height(&self) -> Option<u64> {
		self.block_height
	}

	/// Returns a reference to the transactions in this block
	pub fn transactions(&self) -> &[Transaction] {
		&self.transactions
	}

	/// Returns a reference to the rewards in this block if available
	pub fn rewards(&self) -> Option<&[SolanaReward]> {
		self.rewards.as_deref()
	}

	/// Returns the block's commitment level
	pub fn commitment(&self) -> CommitmentConfig {
		self.commitment
	}
}
