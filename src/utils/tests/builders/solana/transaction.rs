use crate::models::{
	default_ui_transaction_status_meta, SolanaDecodedInstruction, SolanaTransaction,
};
use agave_reserved_account_keys::ReservedAccountKeys;
use solana_instruction;
use solana_sdk::{
	message::{v0::LoadedMessage, Message, VersionedMessage},
	pubkey::Pubkey,
	signature::Signature,
};
use solana_transaction_status::UiTransactionStatusMeta;

/// Builder for creating test Solana transactions
pub struct TransactionBuilder {
	slot: Option<u64>,
	signature: Option<Signature>,
	fee_payer: Option<Pubkey>,
	meta: Option<UiTransactionStatusMeta>,
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
	pub fn meta(mut self, meta: UiTransactionStatusMeta) -> Self {
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
		SolanaTransaction {
			signature: self.signature.unwrap_or_else(Signature::new_unique),
			transaction: match self.message.unwrap_or_else(|| {
				VersionedMessage::Legacy(Message::new(&[], Some(&Pubkey::new_unique())))
			}) {
				VersionedMessage::Legacy(msg) => {
					solana_sdk::transaction::VersionedTransaction::from(
						solana_sdk::transaction::Transaction::new_unsigned(msg),
					)
				}
				VersionedMessage::V0(msg) => {
					let loaded_msg = LoadedMessage::new(
						msg.clone(),
						solana_sdk::message::v0::LoadedAddresses::default(),
						&ReservedAccountKeys::empty_key_set(),
					);
					let instructions: Vec<_> = msg
						.instructions
						.iter()
						.map(|ix| solana_instruction::Instruction {
							program_id: loaded_msg.account_keys()[ix.program_id_index as usize],
							accounts: ix
								.accounts
								.iter()
								.map(|&idx| solana_instruction::AccountMeta {
									pubkey: loaded_msg.account_keys()[idx as usize],
									is_signer: loaded_msg.is_signer(idx as usize),
									is_writable: loaded_msg.is_writable(idx as usize),
								})
								.collect(),
							data: ix.data.clone(),
						})
						.collect();
					solana_sdk::transaction::VersionedTransaction::from(
						solana_sdk::transaction::Transaction::new_unsigned(Message::new(
							&instructions,
							Some(&loaded_msg.account_keys()[0]),
						)),
					)
				}
			},
			meta: default_ui_transaction_status_meta(),
			slot: self.slot.unwrap_or(0),
			block_time: self.block_time,
		}
	}
}
