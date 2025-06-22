use {
	crate::models::SolanaTransaction,
	serde::{Deserialize, Serialize},
	solana_sdk::{
		commitment_config::CommitmentConfig,
		message::{Message, VersionedMessage},
		transaction::Transaction,
	},
};

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
	pub transactions: Vec<SolanaTransaction>,
	/// The rewards for this block
	pub rewards: Option<Vec<SolanaReward>>,
	/// The block's commitment level
	pub commitment: CommitmentConfig,
}

/// Represents a reward in a Solana block
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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
	#[allow(clippy::too_many_arguments)]
	pub fn new(
		slot: u64,
		blockhash: String,
		parent_slot: u64,
		block_time: Option<i64>,
		block_height: Option<u64>,
		transactions: Vec<SolanaTransaction>,
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
	pub fn transactions(&self) -> &[SolanaTransaction] {
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

impl From<SolanaTransaction> for Transaction {
	fn from(solana_tx: SolanaTransaction) -> Self {
		Transaction {
			message: match &solana_tx.transaction.message {
				VersionedMessage::Legacy(msg) => msg.clone(),
				_ => Message::default(),
			},
			signatures: vec![solana_tx.signature],
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::utils::tests::builders::solana::transaction::TransactionBuilder;
	use solana_sdk::{
		commitment_config::CommitmentConfig,
		instruction::{AccountMeta, Instruction},
		message::{Message, VersionedMessage},
		pubkey::Pubkey,
		signature::{Keypair, Signer},
	};

	fn create_test_transaction() -> SolanaTransaction {
		let fee_payer = Keypair::new();
		let program_id = Pubkey::new_unique();
		let account1 = Pubkey::new_unique();
		let account2 = Pubkey::new_unique();

		let instruction = Instruction {
			program_id,
			accounts: vec![
				AccountMeta::new(account1, false),
				AccountMeta::new_readonly(account2, false),
			],
			data: vec![1, 2, 3],
		};

		let message = Message::new(&[instruction], Some(&fee_payer.pubkey()));
		let signature = fee_payer.sign_message(&message.serialize());

		TransactionBuilder::new()
			.fee_payer(fee_payer.pubkey())
			.message(VersionedMessage::Legacy(message))
			.signature(signature)
			.build()
	}

	fn create_test_reward() -> SolanaReward {
		SolanaReward {
			pubkey: "TestPubkey".to_string(),
			lamports: 1000,
			reward_type: "TestReward".to_string(),
			commission: Some(5),
		}
	}

	#[test]
	fn test_solana_block_creation() {
		let slot = 12345;
		let blockhash = "test_blockhash".to_string();
		let parent_slot = 12344;
		let block_time = Some(1678901234);
		let block_height = Some(12345);
		let transactions = vec![create_test_transaction()];
		let rewards = Some(vec![create_test_reward()]);
		let commitment = CommitmentConfig::confirmed();

		let block = SolanaBlock::new(
			slot,
			blockhash.clone(),
			parent_slot,
			block_time,
			block_height,
			transactions.clone(),
			rewards.clone(),
			commitment,
		);

		assert_eq!(block.slot(), slot);
		assert_eq!(block.blockhash(), blockhash);
		assert_eq!(block.parent_slot(), parent_slot);
		assert_eq!(block.block_time(), block_time);
		assert_eq!(block.block_height(), block_height);
		assert_eq!(block.transactions(), transactions.as_slice());
		assert_eq!(block.rewards(), rewards.as_deref());
		assert_eq!(block.commitment(), commitment);
	}

	#[test]
	fn test_solana_block_default_values() {
		let block = SolanaBlock::new(
			0,
			"".to_string(),
			0,
			None,
			None,
			vec![],
			None,
			CommitmentConfig::default(),
		);

		assert_eq!(block.slot(), 0);
		assert_eq!(block.blockhash(), "");
		assert_eq!(block.parent_slot(), 0);
		assert_eq!(block.block_time(), None);
		assert_eq!(block.block_height(), None);
		assert!(block.transactions().is_empty());
		assert!(block.rewards().is_none());
	}

	#[test]
	fn test_solana_reward_creation() {
		let reward = create_test_reward();

		assert_eq!(reward.pubkey, "TestPubkey");
		assert_eq!(reward.lamports, 1000);
		assert_eq!(reward.reward_type, "TestReward");
		assert_eq!(reward.commission, Some(5));
	}

	#[test]
	fn test_solana_block_with_multiple_transactions() {
		let transactions = vec![
			create_test_transaction(),
			create_test_transaction(),
			create_test_transaction(),
		];

		let block = SolanaBlock::new(
			12345,
			"test_blockhash".to_string(),
			12344,
			Some(1678901234),
			Some(12345),
			transactions.clone(),
			None,
			CommitmentConfig::confirmed(),
		);

		assert_eq!(block.transactions().len(), 3);
		assert_eq!(block.transactions(), transactions.as_slice());
	}

	#[test]
	fn test_solana_block_with_multiple_rewards() {
		let rewards = Some(vec![
			create_test_reward(),
			create_test_reward(),
			create_test_reward(),
		]);

		let block = SolanaBlock::new(
			12345,
			"test_blockhash".to_string(),
			12344,
			Some(1678901234),
			Some(12345),
			vec![],
			rewards.clone(),
			CommitmentConfig::confirmed(),
		);

		assert_eq!(block.rewards().unwrap().len(), 3);
		assert_eq!(block.rewards(), rewards.as_deref());
	}
}
