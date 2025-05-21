use crate::models::{
	SolanaDecodedInstruction, SolanaTransaction, SolanaTransactionMetadata,
	SolanaTransactionStatusMeta,
};
use solana_sdk::{
	message::{Message, VersionedMessage},
	pubkey::Pubkey,
	signature::Signature,
};

/// Builder for creating test Solana transactions
pub struct TransactionBuilder {
	slot: Option<u64>,
	signature: Option<Signature>,
	fee_payer: Option<Pubkey>,
	meta: Option<SolanaTransactionStatusMeta>,
	message: Option<VersionedMessage>,
	block_time: Option<i64>,
	instructions: Vec<SolanaDecodedInstruction<Vec<u8>>>,
}

impl TransactionBuilder {
	/// Creates a new TransactionBuilder with default values
	pub fn new() -> Self {
		Self {
			slot: None,
			signature: None,
			fee_payer: None,
			meta: None,
			message: None,
			block_time: None,
			instructions: Vec::new(),
		}
	}

	/// Sets the slot number
	pub fn slot(mut self, slot: u64) -> Self {
		self.slot = Some(slot);
		self
	}

	/// Sets the transaction signature
	pub fn signature(mut self, signature: Signature) -> Self {
		self.signature = Some(signature);
		self
	}

	/// Sets the fee payer's public key
	pub fn fee_payer(mut self, fee_payer: Pubkey) -> Self {
		self.fee_payer = Some(fee_payer);
		self
	}

	/// Sets the transaction status metadata
	pub fn meta(mut self, meta: SolanaTransactionStatusMeta) -> Self {
		self.meta = Some(meta);
		self
	}

	/// Sets the transaction message
	pub fn message(mut self, message: VersionedMessage) -> Self {
		self.message = Some(message);
		self
	}

	/// Sets the block time
	pub fn block_time(mut self, block_time: i64) -> Self {
		self.block_time = Some(block_time);
		self
	}

	/// Adds an instruction to the transaction
	pub fn instruction(mut self, instruction: SolanaDecodedInstruction<Vec<u8>>) -> Self {
		self.instructions.push(instruction);
		self
	}

	/// Builds the SolanaTransaction
	pub fn build(self) -> SolanaTransaction {
		let metadata = SolanaTransactionMetadata {
			slot: self.slot.unwrap_or(0),
			signature: self.signature.unwrap_or_else(Signature::new_unique),
			fee_payer: self.fee_payer.unwrap_or_else(Pubkey::new_unique),
			meta: self.meta.unwrap_or_default(),
			message: self.message.unwrap_or_else(|| {
				VersionedMessage::Legacy(Message::new(&[], Some(&Pubkey::new_unique())))
			}),
			block_time: self.block_time,
		};

		SolanaTransaction {
			metadata,
			instructions: self.instructions,
		}
	}
}
