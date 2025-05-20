use {
	crate::models::blockchain::solana::block::SolanaBlock,
	async_trait::async_trait,
	serde::{Deserialize, Serialize},
	solana_account_decoder::parse_token::UiTokenAmount,
	solana_sdk::{
		instruction::{AccountMeta, Instruction},
		message::{v0::LoadedAddresses, Message, VersionedMessage},
		pubkey::Pubkey,
		signature::Signature,
		transaction::{Result as TransactionResult, Transaction},
		transaction_context::TransactionReturnData,
	},
	solana_transaction_status::{InnerInstructions, Rewards},
	std::sync::Arc,
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
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// Serializable version of transaction status metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableTransactionMeta {
	/// Error if transaction failed
	pub err: Option<String>,
	/// Fee paid for the transaction
	pub fee: u64,
	/// Pre-transaction account balances
	pub pre_balances: Vec<u64>,
	/// Post-transaction account balances
	pub post_balances: Vec<u64>,
	/// Inner instructions if any
	pub inner_instructions: Vec<SerializableInnerInstruction>,
	/// Log messages if any
	pub log_messages: Option<Vec<String>>,
	/// Pre-transaction token balances if any
	pub pre_token_balances: Option<Vec<SerializableTokenBalance>>,
	/// Post-transaction token balances if any
	pub post_token_balances: Option<Vec<SerializableTokenBalance>>,
}

impl Default for SerializableTransactionMeta {
	fn default() -> Self {
		Self {
			err: None,
			fee: 0,
			pre_balances: Vec::new(),
			post_balances: Vec::new(),
			inner_instructions: Vec::new(),
			log_messages: None,
			pre_token_balances: None,
			post_token_balances: None,
		}
	}
}

/// Serializable version of inner instruction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableInnerInstruction {
	/// Index of the instruction
	pub index: u8,
	/// The instruction data
	pub instruction: SerializableInstruction,
}

/// Serializable version of instruction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableInstruction {
	/// Program ID that owns the instruction
	pub program_id: Pubkey,
	/// Accounts involved in the instruction
	pub accounts: Vec<u8>,
	/// Instruction data
	pub data: Vec<u8>,
}

/// Serializable version of token balance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableTokenBalance {
	/// Account index
	pub account_index: u8,
	/// Token mint address
	pub mint: String,
	/// Token owner address
	pub owner: String,
	/// Token amount
	pub ui_token_amount: SerializableTokenAmount,
}

/// Serializable version of token amount
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableTokenAmount {
	/// Raw amount as string
	pub amount: String,
	/// Number of decimals
	pub decimals: u8,
	/// UI amount if available
	pub ui_amount: Option<f64>,
}

/// Metadata associated with a Solana transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// A decoded instruction containing program ID, data, and associated accounts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecodedInstruction<T> {
	/// The program ID that owns the instruction
	pub program_id: Pubkey,
	/// The decoded data payload for the instruction
	pub data: T,
	/// The accounts involved in the instruction
	pub accounts: Vec<AccountMeta>,
}

/// Represents a Solana transaction with its metadata and instructions
#[derive(Debug, Clone, Serialize, Deserialize)]
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
							is_writable: tx.message.is_writable(idx as usize),
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

/// Input type for transaction processing
#[derive(Debug, Clone)]
pub struct TransactionInput {
	/// Transaction metadata
	pub metadata: TransactionMetadata,
	/// Transaction instructions
	pub instructions: Vec<Instruction>,
}

/// A trait for processing transactions
#[async_trait]
pub trait TransactionProcessor<Input = TransactionInput>: Send + Sync {
	type Output;

	/// Process a transaction with the given input
	async fn process(
		&mut self,
		input: Input,
		metrics: Arc<MetricsCollection>,
	) -> Result<Self::Output, TransactionError>;
}

/// A pipe for processing transactions
pub struct TransactionPipe<P: TransactionProcessor<TransactionInput>> {
	processor: P,
}

impl<P: TransactionProcessor<TransactionInput>> TransactionPipe<P> {
	/// Creates a new transaction pipe with the given processor
	pub fn new(processor: P) -> Self {
		Self { processor }
	}

	/// Process a transaction
	pub async fn process(
		&mut self,
		metadata: TransactionMetadata,
		instructions: Vec<Instruction>,
		metrics: Arc<MetricsCollection>,
	) -> Result<P::Output, TransactionError> {
		let input = TransactionInput {
			metadata,
			instructions,
		};
		self.processor.process(input, metrics).await
	}
}

/// A collection of metrics for transaction processing
#[derive(Debug, Clone, Default)]
pub struct MetricsCollection {
	/// Number of instructions processed
	pub instructions_processed: usize,
	/// Number of transactions processed
	pub transactions_processed: usize,
	/// Processing time in milliseconds
	pub processing_time_ms: u64,
}

/// Errors that can occur during transaction processing
#[derive(Debug, thiserror::Error)]
pub enum TransactionError {
	#[error("Failed to process transaction: {0}")]
	ProcessingError(String),
	#[error("Invalid instruction data: {0}")]
	InvalidInstructionData(String),
	#[error("Missing required account: {0}")]
	MissingAccount(Pubkey),
	#[error("Invalid program ID: {0}")]
	InvalidProgramId(Pubkey),
}
