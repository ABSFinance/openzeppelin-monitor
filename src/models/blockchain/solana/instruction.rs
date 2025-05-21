use {
	serde::{Deserialize, Serialize},
	solana_sdk::{
		instruction::{AccountMeta, Instruction},
		pubkey::Pubkey,
		signature::Signature,
	},
};

/// Metadata associated with a specific instruction, including transaction-level details
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DecodedInstruction<T> {
	/// The program ID that owns the instruction
	pub program_id: Pubkey,
	/// The decoded data payload for the instruction
	pub data: T,
	/// The accounts involved in the instruction
	pub accounts: Vec<AccountMeta>,
}

/// A trait for decoding Solana instructions into structured data
///
/// This trait provides a generic interface for decoding raw Solana instructions
/// into domain-specific types. Implementers can define their own instruction
/// types and decoding logic.
#[allow(dead_code)]
pub trait InstructionDecoder<'a> {
	/// The type that the instruction will be decoded into
	type InstructionType;

	/// Decode a raw instruction into a structured type
	///
	/// # Arguments
	/// * `instruction` - The raw instruction to decode
	///
	/// # Returns
	/// `Some(DecodedInstruction)` if decoding was successful, `None` otherwise
	fn decode_instruction(
		&self,
		instruction: &'a Instruction,
	) -> Option<DecodedInstruction<Self::InstructionType>>;
}
