use {
	crate::models::Monitor,
	async_trait::async_trait,
	serde::{Deserialize, Serialize},
	solana_sdk::{
		instruction::{AccountMeta, Instruction},
		pubkey::Pubkey,
		signature::Signature,
	},
	std::sync::Arc,
};

use super::transaction::{MetricsCollection, TransactionError};

/// Metadata associated with a specific instruction, including transaction-level details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstructionMetadata {
	/// The slot number where the instruction was processed
	pub slot: u64,
	/// The transaction signature
	pub signature: Signature,
	/// The fee payer's public key
	pub fee_payer: Pubkey,
	/// The block time when the transaction was processed
	pub block_time: Option<i64>,
	/// The block height
	pub block_height: Option<u64>,
	/// The block hash
	pub blockhash: Option<Signature>,
	/// The parent slot number
	pub parent_slot: Option<u64>,
	/// The height of the instruction in the call stack (0 for top-level)
	pub stack_height: usize,
	/// The index of the instruction in the transaction
	pub instruction_index: usize,
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

/// Represents a nested instruction with metadata and potential inner instructions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NestedInstruction {
	/// Metadata about the instruction
	pub metadata: InstructionMetadata,
	/// The instruction data
	pub instruction: Instruction,
	/// Any inner instructions
	pub inner_instructions: Vec<NestedInstruction>,
}

/// Represents a matched condition in a Solana transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolanaMonitorMatch {
	/// The monitor that matched
	pub monitor: Monitor,
	/// The slot number where the match occurred
	pub slot: u64,
	/// The transaction signature
	pub signature: Signature,
	/// The program ID that was matched
	pub program_id: Pubkey,
	/// The accounts involved in the match
	pub accounts: Vec<AccountMeta>,
	/// The instruction data that matched
	pub data: Vec<u8>,
	/// The index of the instruction in the transaction
	pub instruction_index: usize,
	/// The height of the instruction in the call stack (0 for top-level)
	pub stack_height: usize,
	/// Additional metadata about the match
	pub metadata: InstructionMetadata,
	/// Any nested instructions that were matched
	pub nested_instructions: Vec<NestedInstruction>,
}

impl SolanaMonitorMatch {
	/// Creates a new SolanaMonitorMatch
	pub fn new(
		monitor: Monitor,
		slot: u64,
		signature: Signature,
		program_id: Pubkey,
		accounts: Vec<AccountMeta>,
		data: Vec<u8>,
		instruction_index: usize,
		stack_height: usize,
		metadata: InstructionMetadata,
		nested_instructions: Vec<NestedInstruction>,
	) -> Self {
		Self {
			monitor,
			slot,
			signature,
			program_id,
			accounts,
			data,
			instruction_index,
			stack_height,
			metadata,
			nested_instructions,
		}
	}

	/// Returns the slot number
	pub fn slot(&self) -> u64 {
		self.slot
	}

	/// Returns the transaction signature
	pub fn signature(&self) -> &Signature {
		&self.signature
	}

	/// Returns the program ID
	pub fn program_id(&self) -> &Pubkey {
		&self.program_id
	}

	/// Returns the accounts involved
	pub fn accounts(&self) -> &[AccountMeta] {
		&self.accounts
	}

	/// Returns the instruction data
	pub fn data(&self) -> &[u8] {
		&self.data
	}

	/// Returns the instruction index
	pub fn instruction_index(&self) -> usize {
		self.instruction_index
	}

	/// Returns the stack height
	pub fn stack_height(&self) -> usize {
		self.stack_height
	}

	/// Returns the additional metadata
	pub fn metadata(&self) -> &InstructionMetadata {
		&self.metadata
	}

	/// Returns the nested instructions
	pub fn nested_instructions(&self) -> &[NestedInstruction] {
		&self.nested_instructions
	}
}

/// A trait for decoding instructions
pub trait InstructionDecoder<'a> {
	type InstructionType;

	/// Decode a raw instruction into a structured type
	fn decode_instruction(
		&self,
		instruction: &'a Instruction,
	) -> Option<DecodedInstruction<Self::InstructionType>>;
}

/// Input type for instruction processing
#[derive(Debug, Clone)]
pub struct InstructionInput<T> {
	/// Instruction metadata
	pub metadata: InstructionMetadata,
	/// Decoded instruction
	pub instruction: DecodedInstruction<T>,
}

/// A trait for processing instructions
#[async_trait]
pub trait InstructionProcessor: Send + Sync {
	type Output;

	/// Process an instruction with the given input
	async fn process(
		&mut self,
		input: InstructionInput<Vec<u8>>,
		metrics: Arc<MetricsCollection>,
	) -> Result<Self::Output, InstructionError>;
}

/// A pipe for processing instructions
pub struct InstructionPipe<
	D: for<'a> InstructionDecoder<'a, InstructionType = Vec<u8>>,
	P: InstructionProcessor,
> {
	decoder: D,
	processor: P,
}

impl<D: for<'a> InstructionDecoder<'a, InstructionType = Vec<u8>>, P: InstructionProcessor>
	InstructionPipe<D, P>
{
	/// Creates a new instruction pipe with the given decoder and processor
	pub fn new(decoder: D, processor: P) -> Self {
		Self { decoder, processor }
	}

	/// Process an instruction
	pub async fn process(
		&mut self,
		instruction: Instruction,
		metadata: InstructionMetadata,
		metrics: Arc<MetricsCollection>,
	) -> Result<P::Output, InstructionError> {
		if let Some(decoded) = self.decoder.decode_instruction(&instruction) {
			let input = InstructionInput {
				metadata,
				instruction: decoded,
			};
			self.processor.process(input, metrics).await
		} else {
			Err(InstructionError::DecodingError)
		}
	}
}

/// Errors that can occur during instruction processing
#[derive(Debug, thiserror::Error)]
pub enum InstructionError {
	#[error("Failed to decode instruction")]
	DecodingError,
	#[error("Failed to process instruction: {0}")]
	ProcessingError(String),
	#[error("Invalid instruction data: {0}")]
	InvalidData(String),
	#[error("Missing required account: {0}")]
	MissingAccount(Pubkey),
}
