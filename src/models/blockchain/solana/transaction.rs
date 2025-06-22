use {
	crate::{models::blockchain::solana::block::SolanaBlock, services::filter::error::FilterError},
	serde::{Deserialize, Serialize},
	solana_account_decoder::parse_token::UiTokenAmount,
	solana_sdk::{
		message::{v0::LoadedAddresses, Message, VersionedMessage},
		pubkey::Pubkey,
		signature::Signature,
		transaction::{Result as TransactionResult, VersionedTransaction},
		transaction_context::TransactionReturnData,
	},
	solana_transaction_status::{InnerInstructions, Rewards, UiTransactionStatusMeta},
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TransactionTokenBalance {
	pub account_index: u8,
	pub mint: String,
	pub ui_token_amount: UiTokenAmount,
	pub owner: String,
	pub program_id: String,
}

/// Transaction status metadata containing execution status, fees, balances, etc.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TransactionStatusMeta {
	/// Transaction execution status
	pub status: TransactionResult<()>,
	/// Fee paid for the transaction
	pub fee: u64,
	/// Pre-transaction account balances
	pub pre_balances: Vec<u64>,
	/// Post-transaction account balances
	pub post_balances: Vec<u64>,
	/// Inner instructions if any
	pub inner_instructions: Option<Vec<InnerInstructions>>,
	/// Log messages if any
	pub log_messages: Option<Vec<String>>,
	/// Pre-transaction token balances if any
	pub pre_token_balances: Option<Vec<TransactionTokenBalance>>,
	/// Post-transaction token balances if any
	pub post_token_balances: Option<Vec<TransactionTokenBalance>>,
	/// Rewards if any
	pub rewards: Option<Rewards>,
	/// Loaded addresses
	pub loaded_addresses: LoadedAddresses,
	/// Return data if any
	pub return_data: Option<TransactionReturnData>,
	/// Compute units consumed if available
	pub compute_units_consumed: Option<u64>,
}

impl Default for TransactionStatusMeta {
	fn default() -> Self {
		Self {
			status: Ok(()),
			fee: 0,
			pre_balances: Vec::new(),
			post_balances: Vec::new(),
			inner_instructions: None,
			log_messages: None,
			pre_token_balances: None,
			post_token_balances: None,
			rewards: None,
			loaded_addresses: LoadedAddresses::default(),
			return_data: None,
			compute_units_consumed: None,
		}
	}
}

/// Metadata associated with a Solana transaction
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TransactionMetadata {
	/// The slot number in which this transaction was processed
	pub slot: u64,
	/// The unique signature of this transaction
	pub signature: Signature,
	/// The public key of the fee payer account
	pub fee_payer: Pubkey,
	/// Transaction status metadata containing execution status, fees, balances, etc.
	pub meta: TransactionStatusMeta,
	/// The versioned message containing transaction instructions and account keys
	pub message: VersionedMessage,
	/// The Unix timestamp of when the transaction was processed
	pub block_time: Option<i64>,
}

impl Default for TransactionMetadata {
	fn default() -> Self {
		Self {
			slot: 0,
			signature: Signature::new_unique(),
			fee_payer: Pubkey::new_unique(),
			meta: TransactionStatusMeta::default(),
			message: VersionedMessage::Legacy(Message::default()),
			block_time: None,
		}
	}
}

/// Represents a Solana transaction with its metadata and instructions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SolanaTransaction {
	/// The unique signature of this transaction
	pub signature: Signature,
	/// The versioned transaction containing the message and signatures
	pub transaction: VersionedTransaction,
	/// Transaction status metadata containing execution status, fees, balances, etc.
	pub meta: UiTransactionStatusMeta,
	/// The slot number in which this transaction was processed
	pub slot: u64,
	/// The Unix timestamp of when the transaction was processed
	pub block_time: Option<i64>,
}

impl SolanaTransaction {
	/// Creates a new SolanaTransaction from a block and transaction index
	pub fn new(block: &SolanaBlock, tx_index: usize) -> Option<Self> {
		block.transactions.get(tx_index).map(|tx| Self {
			signature: tx.signature,
			transaction: tx.transaction.clone(),
			meta: default_ui_transaction_status_meta(),
			slot: block.slot,
			block_time: block.block_time,
		})
	}

	/// Returns the transaction signature
	pub fn signature(&self) -> &Signature {
		&self.signature
	}

	/// Returns the transaction slot
	pub fn slot(&self) -> u64 {
		self.slot
	}

	/// Returns the transaction status metadata
	pub fn meta(&self) -> &UiTransactionStatusMeta {
		&self.meta
	}

	/// Returns the transaction message
	pub fn message(&self) -> &VersionedMessage {
		&self.transaction.message
	}

	/// Returns the block time if available
	pub fn block_time(&self) -> Option<i64> {
		self.block_time
	}
}

/// Creates a default UiTransactionStatusMeta instance
pub fn default_ui_transaction_status_meta() -> UiTransactionStatusMeta {
	UiTransactionStatusMeta {
		err: None,
		status: Ok(()),
		fee: 0,
		pre_balances: Vec::new(),
		post_balances: Vec::new(),
		inner_instructions: solana_transaction_status::option_serializer::OptionSerializer::none(),
		log_messages: solana_transaction_status::option_serializer::OptionSerializer::none(),
		pre_token_balances: solana_transaction_status::option_serializer::OptionSerializer::none(),
		post_token_balances: solana_transaction_status::option_serializer::OptionSerializer::none(),
		rewards: solana_transaction_status::option_serializer::OptionSerializer::none(),
		loaded_addresses: solana_transaction_status::option_serializer::OptionSerializer::none(),
		return_data: solana_transaction_status::option_serializer::OptionSerializer::none(),
		compute_units_consumed:
			solana_transaction_status::option_serializer::OptionSerializer::none(),
	}
}

impl TryFrom<SolanaTransaction> for UiTransactionStatusMeta {
	type Error = FilterError;

	fn try_from(value: SolanaTransaction) -> Result<Self, Self::Error> {
		log::trace!("try_from(transaction_update: {:?})", value);

		Ok(UiTransactionStatusMeta {
			err: value.meta.err,
			status: value.meta.status,
			fee: value.meta.fee,
			pre_balances: value.meta.pre_balances,
			post_balances: value.meta.post_balances,
			inner_instructions: value.meta.inner_instructions,
			log_messages: value.meta.log_messages,
			pre_token_balances: value.meta.pre_token_balances,
			post_token_balances: value.meta.post_token_balances,
			rewards: value.meta.rewards,
			loaded_addresses: value.meta.loaded_addresses,
			return_data: value.meta.return_data,
			compute_units_consumed: value.meta.compute_units_consumed,
		})
	}
}

impl TryFrom<SolanaTransaction> for TransactionMetadata {
	type Error = FilterError;

	fn try_from(value: SolanaTransaction) -> Result<Self, Self::Error> {
		// Convert UiTransactionStatusMeta to TransactionStatusMeta
		let meta = TransactionStatusMeta {
			status: value.meta.status,
			fee: value.meta.fee,
			pre_balances: value.meta.pre_balances,
			post_balances: value.meta.post_balances,
			inner_instructions: None, // Skip complex conversion for now
			log_messages: Some(value.meta.log_messages.unwrap_or_else(Vec::new)),
			pre_token_balances: None,  // Skip complex conversion for now
			post_token_balances: None, // Skip complex conversion for now
			rewards: Some(value.meta.rewards.unwrap_or_else(Vec::new)),
			loaded_addresses: LoadedAddresses::default(), // Use default for now
			return_data: None,                            // Skip complex conversion for now
			compute_units_consumed: value.meta.compute_units_consumed.map(|c| c),
		};

		// Extract fee payer from the transaction message
		let fee_payer = match &value.transaction.message {
			VersionedMessage::Legacy(msg) => msg.account_keys[0],
			VersionedMessage::V0(msg) => msg.account_keys[0],
		};

		Ok(TransactionMetadata {
			slot: value.slot,
			signature: value.signature,
			fee_payer,
			meta,
			message: value.transaction.message,
			block_time: value.block_time,
		})
	}
}

#[cfg(test)]
mod tests {
	use crate::utils::tests::solana::transaction::TransactionBuilder;

	use super::*;
	use solana_sdk::{
		commitment_config::CommitmentConfig,
		instruction::{AccountMeta, Instruction},
		message::Message,
		pubkey::Pubkey,
		signature::{Keypair, Signature, Signer},
	};

	// Helper function to create a test transaction
	fn create_test_transaction() -> SolanaTransaction {
		let fee_payer = Keypair::new();
		let program_id = Pubkey::new_unique();
		let account1 = Pubkey::new_unique();
		let account2 = Pubkey::new_unique();

		let instruction = Instruction {
			program_id,
			accounts: vec![
				AccountMeta::new(account1, true),
				AccountMeta::new(account2, false),
			],
			data: vec![1, 2, 3, 4],
		};

		let message = Message::new(&[instruction], Some(&fee_payer.pubkey()));
		let signature = Signature::new_unique();

		TransactionBuilder::new()
			.slot(12345)
			.signature(signature)
			.message(VersionedMessage::Legacy(message))
			.block_time(1678901234)
			.build()
	}

	#[test]
	fn test_signature() {
		let tx = create_test_transaction();
		let signature = tx.signature();
		assert_eq!(signature, &tx.signature);
	}

	#[test]
	fn test_slot() {
		let tx = create_test_transaction();
		let slot = tx.slot();
		assert_eq!(slot, tx.slot);
	}

	#[test]
	fn test_meta() {
		let tx = create_test_transaction();
		let meta = tx.meta();
		assert_eq!(meta, &tx.meta);
	}

	#[test]
	fn test_message() {
		let tx = create_test_transaction();
		let message = tx.message();
		assert_eq!(message, &tx.transaction.message);
	}

	#[test]
	fn test_block_time() {
		let tx = create_test_transaction();
		let block_time = tx.block_time();
		assert_eq!(block_time, tx.block_time);
	}

	#[test]
	fn test_transaction_creation_from_block() {
		let block = SolanaBlock {
			slot: 12345,
			blockhash: Signature::new_unique().to_string(),
			parent_slot: 12344,
			transactions: vec![create_test_transaction()],
			block_time: Some(1678901234),
			block_height: Some(12345),
			rewards: None,
			commitment: CommitmentConfig::default(),
		};

		let tx = SolanaTransaction::new(&block, 0).unwrap();
		assert_eq!(tx.slot, block.slot);
		assert_eq!(tx.block_time, block.block_time);
	}
}
