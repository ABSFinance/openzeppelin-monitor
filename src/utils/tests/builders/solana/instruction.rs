use solana_sdk::{
	instruction::{AccountMeta, Instruction},
	pubkey::Pubkey,
	signature::Signature,
};

use crate::models::SolanaInstructionMetadata;

/// Builder for creating test instructions
pub struct InstructionBuilder {
	program_id: Pubkey,
	accounts: Vec<AccountMeta>,
	data: Vec<u8>,
}

impl InstructionBuilder {
	/// Creates a new InstructionBuilder with default values
	pub fn new() -> Self {
		Self {
			program_id: Pubkey::new_unique(),
			accounts: Vec::new(),
			data: Vec::new(),
		}
	}

	/// Sets the program ID
	pub fn program_id(mut self, program_id: Pubkey) -> Self {
		self.program_id = program_id;
		self
	}

	/// Adds an account to the instruction
	pub fn account(mut self, account: AccountMeta) -> Self {
		self.accounts.push(account);
		self
	}

	/// Sets the instruction data
	pub fn data(mut self, data: Vec<u8>) -> Self {
		self.data = data;
		self
	}

	/// Builds the instruction
	pub fn build(self) -> Instruction {
		Instruction {
			program_id: self.program_id,
			accounts: self.accounts,
			data: self.data,
		}
	}
}

/// Builder for creating test instruction metadata
pub struct InstructionMetadataBuilder {
	slot: u64,
	signature: Signature,
	fee_payer: Pubkey,
	block_time: Option<i64>,
	block_height: Option<u64>,
	blockhash: Option<Signature>,
	parent_slot: Option<u64>,
	stack_height: usize,
	instruction_index: usize,
}

impl InstructionMetadataBuilder {
	/// Creates a new InstructionMetadataBuilder with default values
	pub fn new() -> Self {
		Self {
			slot: 12345,
			signature: Signature::new_unique(),
			fee_payer: Pubkey::new_unique(),
			block_time: Some(1678901234),
			block_height: Some(12345),
			blockhash: Some(Signature::new_unique()),
			parent_slot: Some(12344),
			stack_height: 0,
			instruction_index: 0,
		}
	}

	/// Sets the slot number
	pub fn slot(mut self, slot: u64) -> Self {
		self.slot = slot;
		self
	}

	/// Sets the signature
	pub fn signature(mut self, signature: Signature) -> Self {
		self.signature = signature;
		self
	}

	/// Sets the fee payer
	pub fn fee_payer(mut self, fee_payer: Pubkey) -> Self {
		self.fee_payer = fee_payer;
		self
	}

	/// Sets the block time
	pub fn block_time(mut self, block_time: Option<i64>) -> Self {
		self.block_time = block_time;
		self
	}

	/// Sets the block height
	pub fn block_height(mut self, block_height: Option<u64>) -> Self {
		self.block_height = block_height;
		self
	}

	/// Sets the blockhash
	pub fn blockhash(mut self, blockhash: Option<Signature>) -> Self {
		self.blockhash = blockhash;
		self
	}

	/// Sets the parent slot
	pub fn parent_slot(mut self, parent_slot: Option<u64>) -> Self {
		self.parent_slot = parent_slot;
		self
	}

	/// Sets the stack height
	pub fn stack_height(mut self, stack_height: usize) -> Self {
		self.stack_height = stack_height;
		self
	}

	/// Sets the instruction index
	pub fn instruction_index(mut self, instruction_index: usize) -> Self {
		self.instruction_index = instruction_index;
		self
	}

	/// Builds the instruction metadata
	pub fn build(self) -> SolanaInstructionMetadata {
		SolanaInstructionMetadata {
			slot: self.slot,
			signature: self.signature,
			fee_payer: self.fee_payer,
			block_time: self.block_time,
			block_height: self.block_height,
			blockhash: self.blockhash,
			parent_slot: self.parent_slot,
			stack_height: self.stack_height,
			instruction_index: self.instruction_index,
		}
	}
}
