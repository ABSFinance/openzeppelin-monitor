use {
	crate::models::blockchain::solana::block::SolanaBlock,
	serde::{Deserialize, Serialize},
	solana_account_decoder::parse_token::UiTokenAmount,
	solana_sdk::{
		instruction::AccountMeta,
		message::{v0::LoadedAddresses, Message, VersionedMessage},
		pubkey::Pubkey,
		signature::Signature,
		transaction::Result as TransactionResult,
		transaction_context::TransactionReturnData,
	},
	solana_transaction_status::{InnerInstructions, Rewards},
};

use super::instruction::DecodedInstruction;

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
	/// Metadata about the transaction
	pub metadata: TransactionMetadata,
	/// The decoded instructions in the transaction
	pub instructions: Vec<DecodedInstruction<Vec<u8>>>,
}

impl SolanaTransaction {
	/// Creates a new SolanaTransaction from a block and transaction index
	pub fn new(block: &SolanaBlock, tx_index: usize) -> Option<Self> {
		block.transactions.get(tx_index).map(|tx| {
			let metadata = TransactionMetadata {
				slot: block.slot,
				signature: tx.signatures[0],
				fee_payer: tx.message.account_keys[0],
				meta: TransactionStatusMeta::default(),
				message: VersionedMessage::Legacy(tx.message.clone()),
				block_time: block.block_time,
			};

			let instructions = tx
				.message
				.instructions
				.iter()
				.map(|ix| DecodedInstruction {
					program_id: *ix.program_id(&tx.message.account_keys),
					data: ix.data.clone(),
					accounts: ix
						.accounts
						.iter()
						.map(|&idx| AccountMeta {
							pubkey: tx.message.account_keys[idx as usize],
							is_signer: tx.message.is_signer(idx as usize),
							is_writable: tx.message.is_maybe_writable(idx as usize, None),
						})
						.collect(),
				})
				.collect();

			Self {
				metadata,
				instructions,
			}
		})
	}

	/// Returns the transaction signature
	pub fn signature(&self) -> &Signature {
		&self.metadata.signature
	}

	/// Returns the transaction slot
	pub fn slot(&self) -> u64 {
		self.metadata.slot
	}

	/// Returns the fee payer's public key
	pub fn fee_payer(&self) -> &Pubkey {
		&self.metadata.fee_payer
	}

	/// Returns the transaction status metadata
	pub fn meta(&self) -> &TransactionStatusMeta {
		&self.metadata.meta
	}

	/// Returns the transaction message
	pub fn message(&self) -> &VersionedMessage {
		&self.metadata.message
	}

	/// Returns the block time if available
	pub fn block_time(&self) -> Option<i64> {
		self.metadata.block_time
	}

	/// Returns a reference to the decoded instructions
	pub fn instructions(&self) -> &[DecodedInstruction<Vec<u8>>] {
		&self.instructions
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
			.fee_payer(fee_payer.pubkey())
			.message(VersionedMessage::Legacy(message))
			.block_time(1678901234)
			.instruction(DecodedInstruction {
				program_id,
				data: vec![1, 2, 3, 4],
				accounts: vec![
					AccountMeta::new(account1, true),
					AccountMeta::new(account2, false),
				],
			})
			.build()
	}

	#[test]
	fn test_signature() {
		let tx = create_test_transaction();
		let signature = tx.signature();
		assert_eq!(signature, &tx.metadata.signature);
	}

	#[test]
	fn test_slot() {
		let tx = create_test_transaction();
		let slot = tx.slot();
		assert_eq!(slot, tx.metadata.slot);
	}

	#[test]
	fn test_fee_payer() {
		let tx = create_test_transaction();
		let fee_payer = tx.fee_payer();
		assert_eq!(fee_payer, &tx.metadata.fee_payer);
	}

	#[test]
	fn test_meta() {
		let tx = create_test_transaction();
		let meta = tx.meta();
		assert_eq!(meta, &tx.metadata.meta);
	}

	#[test]
	fn test_message() {
		let tx = create_test_transaction();
		let message = tx.message();
		assert_eq!(message, &tx.metadata.message);
	}

	#[test]
	fn test_block_time() {
		let tx = create_test_transaction();
		let block_time = tx.block_time();
		assert_eq!(block_time, tx.metadata.block_time);
	}

	#[test]
	fn test_instructions() {
		let tx = create_test_transaction();
		let instructions = tx.instructions();
		assert_eq!(instructions, &tx.instructions);
	}

	#[test]
	fn test_transaction_creation_from_block() {
		let block = SolanaBlock {
			slot: 12345,
			blockhash: Signature::new_unique().to_string(),
			parent_slot: 12344,
			transactions: vec![solana_sdk::transaction::Transaction {
				signatures: vec![Signature::new_unique()],
				message: Message::new(
					&[Instruction {
						program_id: Pubkey::new_unique(),
						accounts: vec![AccountMeta::new(Pubkey::new_unique(), true)],
						data: vec![1, 2, 3, 4],
					}],
					Some(&Pubkey::new_unique()),
				),
			}],
			block_time: Some(1678901234),
			block_height: Some(12345),
			rewards: None,
			commitment: CommitmentConfig::confirmed(),
		};

		let tx = SolanaTransaction::new(&block, 0).unwrap();
		assert_eq!(tx.slot(), block.slot);
		assert_eq!(tx.block_time(), block.block_time);
		assert_eq!(tx.instructions().len(), 1);
		assert_eq!(tx.instructions()[0].data, vec![1, 2, 3, 4]);
	}
}
