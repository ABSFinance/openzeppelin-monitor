use {
	crate::models::SolanaTransactionMetadata,
	serde::{Deserialize, Serialize},
	solana_sdk::{
		instruction::{AccountMeta, Instruction},
		pubkey::Pubkey,
	},
	std::ops::{Deref, DerefMut},
};

/// Represents a nested instruction with metadata, including potential inner
/// instructions.
///
/// The `NestedInstruction` struct allows for recursive instruction handling,
/// where each instruction may have associated metadata and a list of nested
/// instructions.
///
/// # Fields
///
/// - `metadata`: The metadata associated with the instruction.
/// - `instruction`: The Solana instruction being processed.
/// - `inner_instructions`: A vector of `NestedInstruction`, representing any
///   nested instructions.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NestedInstruction {
	pub metadata: InstructionMetadata,
	pub instruction: solana_instruction::Instruction,
	pub inner_instructions: NestedInstructions,
}

#[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct NestedInstructions(pub Vec<NestedInstruction>);

impl NestedInstructions {
	pub fn iter(&self) -> std::slice::Iter<NestedInstruction> {
		self.0.iter()
	}

	pub fn len(&self) -> usize {
		self.0.len()
	}

	pub fn is_empty(&self) -> bool {
		self.len() == 0
	}

	pub fn push(&mut self, nested_instruction: NestedInstruction) {
		self.0.push(nested_instruction);
	}
}

impl Deref for NestedInstructions {
	type Target = [NestedInstruction];

	fn deref(&self) -> &[NestedInstruction] {
		&self.0[..]
	}
}

impl DerefMut for NestedInstructions {
	fn deref_mut(&mut self) -> &mut [NestedInstruction] {
		&mut self.0[..]
	}
}

impl Clone for NestedInstructions {
	fn clone(&self) -> Self {
		NestedInstructions(self.0.clone())
	}
}

/// Metadata associated with a specific instruction, including transaction-level details
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InstructionMetadata {
	pub transaction_metadata: SolanaTransactionMetadata,
	pub stack_height: u32,
	pub index: u32,
}

pub type InstructionsWithMetadata = Vec<(InstructionMetadata, solana_instruction::Instruction)>;

impl From<InstructionsWithMetadata> for NestedInstructions {
	fn from(instructions: InstructionsWithMetadata) -> Self {
		log::trace!("from(instructions: {:?})", instructions);
		let mut nested_ixs = NestedInstructions::default();

		for (metadata, instruction) in instructions {
			let nested_instruction = NestedInstruction {
				metadata: metadata.clone(),
				instruction,
				inner_instructions: NestedInstructions::default(),
			};

			// compose root level of ixs
			if metadata.stack_height == 1 || metadata.index == 0 {
				nested_ixs.push(nested_instruction);
				continue;
			}
			nested_ixs[metadata.index as usize]
				.inner_instructions
				.push(nested_instruction);
		}

		nested_ixs
	}
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
