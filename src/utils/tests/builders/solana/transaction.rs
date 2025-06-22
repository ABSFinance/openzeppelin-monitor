use crate::models::{
	default_ui_transaction_status_meta, SolanaDecodedInstruction, SolanaTransaction,
};
use solana_instruction;
use solana_sdk::{
	message::{Message, VersionedMessage},
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
		let fee_payer = self.fee_payer.unwrap_or_else(Pubkey::new_unique);

		// Create instructions from SolanaDecodedInstruction
		let instructions: Vec<solana_instruction::Instruction> = self
			.instructions
			.iter()
			.map(|decoded_ix| solana_instruction::Instruction {
				program_id: decoded_ix.program_id,
				accounts: decoded_ix.accounts.clone(),
				data: decoded_ix.data.clone(),
			})
			.collect();

		// --- FIX: Ensure account_keys order: fee_payer, program_id, then other accounts ---
		let mut account_keys = vec![fee_payer];
		if !instructions.is_empty() {
			let program_id = instructions[0].program_id;
			if !account_keys.contains(&program_id) {
				account_keys.push(program_id);
			}
			for account in &instructions[0].accounts {
				if !account_keys.contains(&account.pubkey) {
					account_keys.push(account.pubkey);
				}
			}
			// Add any additional program_ids/accounts from other instructions
			for instruction in &instructions[1..] {
				if !account_keys.contains(&instruction.program_id) {
					account_keys.push(instruction.program_id);
				}
				for account in &instruction.accounts {
					if !account_keys.contains(&account.pubkey) {
						account_keys.push(account.pubkey);
					}
				}
			}
		} else {
			// No instructions, just fee payer
		}

		let message = Message::new(&instructions, Some(&fee_payer));

		SolanaTransaction {
			signature: self.signature.unwrap_or_else(Signature::new_unique),
			transaction: solana_sdk::transaction::VersionedTransaction::from(
				solana_sdk::transaction::Transaction::new_unsigned(message),
			),
			meta: self.meta.unwrap_or_else(default_ui_transaction_status_meta),
			slot: self.slot.unwrap_or(0),
			block_time: self.block_time,
		}
	}
}

impl Default for TransactionBuilder {
	fn default() -> Self {
		Self::new()
	}
}
