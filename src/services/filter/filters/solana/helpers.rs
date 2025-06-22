use {
	crate::{
		models::{SolanaInstructionMetadata, SolanaTransaction, SolanaTransactionMetadata},
		services::decoders::InstructionType,
	},
	agave_reserved_account_keys::ReservedAccountKeys,
	carbon_core::{error::CarbonResult, instruction::DecodedInstruction},
	solana_instruction::AccountMeta,
	solana_pubkey::Pubkey,
	solana_sdk::{
		bs58,
		message::{
			v0::{LoadedAddresses, LoadedMessage},
			VersionedMessage,
		},
		transaction::Transaction,
	},
	solana_transaction_status::UiInstruction,
};

/// Helper functions for Solana block filtering
pub struct SolanaFilterHelpers;

impl SolanaFilterHelpers {
	pub fn new() -> Self {
		Self
	}

	/// Helper function to check if an instruction matches a specific type
	fn discriminator<T: std::fmt::Debug>(instruction: T) -> String {
		let debug_str = format!("{:?}", instruction);

		debug_str
			.split('(')
			.nth(1)
			.unwrap_or("")
			.split('(')
			.next()
			.unwrap_or("")
			.to_string()
	}

	pub fn matches_instruction_type<T: Into<InstructionType> + std::fmt::Debug + Clone>(
		decoded_instruction: &DecodedInstruction<T>,
		signature: &str,
	) -> bool {
		let instruction_type: InstructionType = decoded_instruction.data.clone().into();
		let discriminator = Self::discriminator(instruction_type);
		discriminator == signature
	}

	/// Check if a transaction matches the given program ID
	pub fn matches_program_id(&self, tx: &Transaction, program_id: &str) -> bool {
		tx.message
			.account_keys
			.iter()
			.any(|key| key.to_string() == program_id)
	}

	/// Check if a transaction matches the given account
	pub fn matches_account(&self, tx: &Transaction, account: &str) -> bool {
		tx.message
			.account_keys
			.iter()
			.any(|key| key.to_string() == account)
	}

	/// Check if a transaction matches the given instruction data
	pub fn matches_instruction_data(&self, tx: &Transaction, data: &[u8]) -> bool {
		tx.message.instructions.iter().any(|ix| ix.data == data)
	}

	pub fn extract_instructions_with_metadata(
		transaction_metadata: &SolanaTransactionMetadata,
		transaction: &SolanaTransaction,
	) -> CarbonResult<Vec<(SolanaInstructionMetadata, solana_instruction::Instruction)>> {
		log::trace!(
			"extract_instructions_with_metadata(transaction_metadata: {:?}, transaction_update: {:?})",
			transaction_metadata,
			transaction
		);
		let message = transaction.transaction.message.clone();
		let meta = transaction.meta.clone();
		let inner_instructions = meta.inner_instructions.clone();
		let loaded_addresses = meta.loaded_addresses.clone();

		let mut instructions_with_metadata =
			Vec::<(SolanaInstructionMetadata, solana_instruction::Instruction)>::new();

		match message {
			VersionedMessage::Legacy(legacy) => {
				for (i, compiled_instruction) in legacy.instructions.iter().enumerate() {
					let program_id = *legacy
						.account_keys
						.get(compiled_instruction.program_id_index as usize)
						.unwrap_or(&Pubkey::default());

					let accounts = compiled_instruction
						.accounts
						.iter()
						.filter_map(|account_index| {
							let account_pubkey =
								legacy.account_keys.get(*account_index as usize)?;
							Some(AccountMeta {
								pubkey: *account_pubkey,
								is_writable: legacy
									.is_maybe_writable(*account_index as usize, None),
								is_signer: legacy.is_signer(*account_index as usize),
							})
						})
						.collect::<Vec<_>>();

					instructions_with_metadata.push((
						SolanaInstructionMetadata {
							transaction_metadata: transaction_metadata.clone(),
							stack_height: 1,
							index: i as u32,
						},
						solana_instruction::Instruction {
							program_id,
							accounts,
							data: compiled_instruction.data.clone(),
						},
					));

					for inner_instructions_per_tx in inner_instructions
						.clone()
						.unwrap_or_else(std::vec::Vec::new)
					{
						if inner_instructions_per_tx.index == i as u8 {
							for inner_instruction in inner_instructions_per_tx.instructions.iter() {
								match inner_instruction {
									UiInstruction::Compiled(compiled_instruction) => {
										let program_id = *legacy
											.account_keys
											.get(compiled_instruction.program_id_index as usize)
											.unwrap_or(&Pubkey::default());

										let accounts: Vec<_> = compiled_instruction
											.accounts
											.iter()
											.filter_map(|account_index| {
												let account_pubkey = legacy
													.account_keys
													.get(*account_index as usize)?;

												Some(AccountMeta {
													pubkey: *account_pubkey,
													is_writable: legacy.is_maybe_writable(
														*account_index as usize,
														None,
													),
													is_signer: legacy
														.is_signer(*account_index as usize),
												})
											})
											.collect();

										instructions_with_metadata.push((
											SolanaInstructionMetadata {
												transaction_metadata: transaction_metadata.clone(),
												stack_height: compiled_instruction
													.stack_height
													.unwrap_or(1),
												index: inner_instructions_per_tx.index as u32,
											},
											solana_instruction::Instruction {
												program_id,
												accounts,
												data: bs58::decode(&compiled_instruction.data)
													.into_vec()
													.unwrap_or_else(|_| vec![]),
											},
										));
									}
									_ => {
										log::warn!(
											"Unsupported inner instruction type encountered"
										);
									}
								}
							}
						}
					}
				}
			}
			VersionedMessage::V0(v0) => {
				let loaded_addresses = LoadedAddresses {
					writable: loaded_addresses
						.clone()
						.unwrap()
						.writable
						.iter()
						.map(|s| s.parse::<Pubkey>().unwrap())
						.collect(),
					readonly: loaded_addresses
						.clone()
						.unwrap()
						.readonly
						.iter()
						.map(|s| s.parse::<Pubkey>().unwrap())
						.collect(),
				};

				let loaded_message = LoadedMessage::new(
					v0.clone(),
					loaded_addresses,
					&ReservedAccountKeys::empty_key_set(),
				);

				for (i, compiled_instruction) in v0.instructions.iter().enumerate() {
					let program_id = *loaded_message
						.account_keys()
						.get(compiled_instruction.program_id_index as usize)
						.unwrap_or(&Pubkey::default());

					let accounts = compiled_instruction
						.accounts
						.iter()
						.map(|account_index| {
							let account_pubkey =
								loaded_message.account_keys().get(*account_index as usize);

							AccountMeta {
								pubkey: account_pubkey.copied().unwrap_or_default(),
								is_writable: loaded_message.is_writable(*account_index as usize),
								is_signer: loaded_message.is_signer(*account_index as usize),
							}
						})
						.collect::<Vec<_>>();

					instructions_with_metadata.push((
						SolanaInstructionMetadata {
							transaction_metadata: transaction_metadata.clone(),
							stack_height: 1,
							index: i as u32,
						},
						solana_instruction::Instruction {
							program_id,
							accounts,
							data: compiled_instruction.data.clone(),
						},
					));

					for inner_instructions_per_tx in inner_instructions
						.clone()
						.unwrap_or_else(std::vec::Vec::new)
					{
						if inner_instructions_per_tx.index == i as u8 {
							for inner_instruction in inner_instructions_per_tx.instructions.iter() {
								match inner_instruction {
									UiInstruction::Compiled(compiled_instruction) => {
										let program_id = *loaded_message
											.account_keys()
											.get(compiled_instruction.program_id_index as usize)
											.unwrap_or(&Pubkey::default());

										let accounts = compiled_instruction
											.accounts
											.iter()
											.map(|account_index| {
												let account_pubkey = loaded_message
													.account_keys()
													.get(*account_index as usize)
													.copied()
													.unwrap_or_default();

												AccountMeta {
													pubkey: account_pubkey,
													is_writable: loaded_message
														.is_writable(*account_index as usize),
													is_signer: loaded_message
														.is_signer(*account_index as usize),
												}
											})
											.collect::<Vec<_>>();

										instructions_with_metadata.push((
											SolanaInstructionMetadata {
												transaction_metadata: transaction_metadata.clone(),
												stack_height: compiled_instruction
													.stack_height
													.unwrap_or(1),
												index: inner_instructions_per_tx.index as u32,
											},
											solana_instruction::Instruction {
												program_id,
												accounts,
												data: bs58::decode(&compiled_instruction.data)
													.into_vec()
													.unwrap_or_else(|_| vec![]),
											},
										));
									}
									_ => {
										log::warn!(
											"Unsupported inner instruction type encountered"
										);
									}
								}
							}
						}
					}
				}
			}
		}

		Ok(instructions_with_metadata)
	}
}

impl Default for SolanaFilterHelpers {
	fn default() -> Self {
		Self::new()
	}
}
