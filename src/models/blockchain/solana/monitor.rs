use {
	crate::{
		models::{MatchConditions, Monitor, SolanaInstructionMetadata, SolanaTransaction},
		services::decoders::{AccountType, InstructionType},
	},
	serde::{Deserialize, Serialize},
	solana_sdk::{
		instruction::{AccountMeta, Instruction},
		message::VersionedMessage,
		pubkey::Pubkey,
		signature::Signature,
	},
};

/// Represents a nested instruction with metadata and potential inner instructions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NestedInstruction {
	/// Metadata about the instruction
	pub metadata: SolanaInstructionMetadata,
	/// The instruction data
	pub instruction: Instruction,
	/// Any inner instructions
	pub inner_instructions: Vec<NestedInstruction>,
}

/// Represents a matched parameter in a Solana instruction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolanaMatchParamEntry {
	/// Name of the parameter
	pub name: String,
	/// Value of the parameter
	pub value: String,
	/// Type of the parameter
	pub kind: String,
	/// Whether the parameter is indexed
	pub indexed: bool,
}

/// Represents a map of matched parameters for a Solana instruction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolanaMatchParamsMap {
	/// Signature of the instruction
	pub signature: String,
	/// Arguments of the instruction
	pub args: Option<Vec<SolanaMatchParamEntry>>,
	/// Hex signature of the instruction
	pub hex_signature: Option<String>,
}

/// Represents matched arguments in a Solana transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolanaMatchArguments {
	/// Matched instructions
	pub instructions: Option<Vec<SolanaMatchParamsMap>>,

	/// Matched accounts arguments
	pub accounts: Option<Vec<AccountMeta>>,
}

/// Represents a matched condition in a Solana transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolanaMonitorMatch {
	/// The monitor that matched
	pub monitor: Monitor,
	/// Network slug that the transaction was sent from
	pub network_slug: String,
	/// Conditions that were matched
	pub matched_on: MatchConditions,
	/// Decoded arguments from the matched conditions
	pub matched_on_args: Option<SolanaMatchArguments>,
	/// Transaction that triggered the match
	pub transaction: SolanaTransaction,
}

impl SolanaMonitorMatch {
	/// Creates a new SolanaMonitorMatch
	pub fn new(
		monitor: Monitor,
		network_slug: String,
		matched_on: MatchConditions,
		matched_on_args: Option<SolanaMatchArguments>,
		transaction: SolanaTransaction,
	) -> Self {
		Self {
			monitor,
			network_slug,
			matched_on,
			matched_on_args,
			transaction,
		}
	}

	/// Returns the slot number
	pub fn slot(&self) -> u64 {
		self.transaction.slot()
	}

	/// Returns the transaction signature
	pub fn signature(&self) -> &Signature {
		self.transaction.signature()
	}

	/// Returns the program ID
	pub fn program_id(&self) -> &Pubkey {
		match self.transaction.message() {
			VersionedMessage::Legacy(msg) => &msg.account_keys[0],
			VersionedMessage::V0(msg) => &msg.account_keys[0],
		}
	}

	// /// Returns the accounts involved
	// pub fn accounts(&self) -> Vec<AccountMeta> {
	// 	match self.transaction.message() {
	// 		VersionedMessage::Legacy(msg) => {
	// 			let ix = &msg.instructions[0];
	// 			ix.accounts
	// 				.iter()
	// 				.map(|&idx| AccountMeta {
	// 					pubkey: msg.account_keys[idx as usize],
	// 					is_signer: msg.is_signer(idx as usize),
	// 					is_writable: msg.is_maybe_writable(idx as usize, None),
	// 				})
	// 				.collect()
	// 		}
	// 		VersionedMessage::V0(msg) => {
	// 			let ix = &msg.instructions[0];
	// 			ix.accounts
	// 				.iter()
	// 				.map(|&idx| AccountMeta {
	// 					pubkey: msg.account_keys[idx as usize],
	// 					is_signer: msg.is_signer(idx as usize),
	// 					is_writable: msg.is_maybe_writable(idx as usize, None),
	// 				})
	// 				.collect()
	// 		}
	// 	}
	// }

	/// Returns the instruction data
	pub fn data(&self) -> &[u8] {
		match self.transaction.message() {
			VersionedMessage::Legacy(msg) => &msg.instructions[0].data,
			VersionedMessage::V0(msg) => &msg.instructions[0].data,
		}
	}

	/// Returns the instruction index
	pub fn instruction_index(&self) -> usize {
		0 // Since we're only storing the matched instruction
	}

	/// Returns the stack height
	pub fn stack_height(&self) -> usize {
		0 // Since we're only storing the matched instruction
	}

	/// Returns the network slug
	pub fn network_slug(&self) -> &str {
		&self.network_slug
	}

	/// Returns the matched conditions
	pub fn matched_on(&self) -> &MatchConditions {
		&self.matched_on
	}

	/// Returns the matched arguments
	pub fn matched_on_args(&self) -> Option<&SolanaMatchArguments> {
		self.matched_on_args.as_ref()
	}

	/// Returns the transaction that triggered the match
	pub fn transaction(&self) -> &SolanaTransaction {
		&self.transaction
	}
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct DecoderType {
	pub account: Option<AccountType>,
	pub instruction: Option<InstructionType>,
}

impl<'a> Default for DecoderType {
	fn default() -> Self {
		Self {
			account: None,
			instruction: None,
		}
	}
}

/// Contract specification for a Solana program
///
/// This structure represents the parsed specification of a Solana program,
/// containing information about account and instruction decoders that can be used
/// to decode program data and instructions.
#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq)]
pub struct ContractSpec(InstructionType);

#[cfg(test)]
mod tests {
	use crate::{
		models::{
			MatchConditions, SolanaDecodedInstruction, SolanaInstructionDecoder,
			SolanaTransactionStatusMeta,
		},
		utils::tests::solana::{
			instruction::{InstructionBuilder, InstructionMetadataBuilder},
			monitor::MonitorBuilder,
			transaction::TransactionBuilder,
		},
	};

	use super::*;
	use solana_sdk::{
		instruction::{AccountMeta, Instruction},
		message::{Message, VersionedMessage},
		pubkey::Pubkey,
		transaction::VersionedTransaction,
	};
	use std::str::FromStr;

	// Helper function to create a test monitor
	fn create_test_monitor() -> Monitor {
		MonitorBuilder::new()
			.name("KaminoLendMonitor")
			.networks(vec!["solana_mainnet".to_string()])
			.address("11111111111111111111111111111111", None)
			.function("transfer", Some("amount > 100"))
			.build()
	}

	// Helper function to create a test Kamino Lend instruction
	fn create_kamino_lend_instruction() -> Instruction {
		let instruction = InstructionBuilder::new()
			.program_id(Pubkey::from_str("11111111111111111111111111111111").unwrap())
			.account(AccountMeta::new(Pubkey::new_unique(), false)) // user
			.account(AccountMeta::new(Pubkey::new_unique(), true))  // lending market
			.account(AccountMeta::new(Pubkey::new_unique(), true))  // reserve
			.account(AccountMeta::new(Pubkey::new_unique(), true))  // user deposit account
			.account(AccountMeta::new(Pubkey::new_unique(), true))  // reserve liquidity supply
			.account(AccountMeta::new(Pubkey::new_unique(), true))  // reserve collateral mint
			.account(AccountMeta::new(Pubkey::new_unique(), true))  // lending market authority
			.account(AccountMeta::new(Pubkey::new_unique(), false)) // token program
			.data(vec![
				0x01, // instruction discriminator for deposit
				0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // amount (u64)
			])
			.build();
		instruction
	}

	#[test]
	fn test_solana_monitor_match_creation() {
		let monitor = create_test_monitor();
		let instruction = create_kamino_lend_instruction();
		let metadata = InstructionMetadataBuilder::new().build();
		let transaction = TransactionBuilder::new()
			.slot(metadata.transaction_metadata.slot)
			.signature(metadata.transaction_metadata.signature)
			.fee_payer(metadata.transaction_metadata.fee_payer)
			.block_time(metadata.transaction_metadata.block_time.unwrap_or(0))
			.instruction(SolanaDecodedInstruction {
				program_id: instruction.program_id,
				data: instruction.data.clone(),
				accounts: instruction.accounts.clone(),
			})
			.build();

		let monitor_match = SolanaMonitorMatch::new(
			monitor.clone(),
			"solana_mainnet".to_string(),
			MatchConditions {
				functions: vec![],
				events: vec![],
				transactions: vec![],
			},
			None,
			transaction.clone(),
		);

		assert_eq!(monitor_match.monitor.name, "KaminoLendMonitor");
		assert_eq!(monitor_match.slot(), metadata.transaction_metadata.slot);
		assert_eq!(
			monitor_match.signature(),
			&metadata.transaction_metadata.signature
		);
		assert_eq!(monitor_match.program_id(), &instruction.program_id);
		assert_eq!(monitor_match.data(), &instruction.data);
		assert_eq!(monitor_match.instruction_index(), 0);
		assert_eq!(monitor_match.stack_height(), 0);
		assert_eq!(monitor_match.network_slug, "solana_mainnet");
		assert_eq!(
			monitor_match.matched_on,
			MatchConditions {
				functions: vec![],
				events: vec![],
				transactions: vec![],
			}
		);
		assert_eq!(monitor_match.transaction, transaction);
	}

	#[test]
	fn test_nested_instruction_handling() {
		let monitor = create_test_monitor();
		let metadata = InstructionMetadataBuilder::new().build();

		// Create a nested instruction
		let nested_instruction = NestedInstruction {
			metadata: InstructionMetadataBuilder::new().stack_height(1).build(),
			instruction: Instruction {
				program_id: Pubkey::new_unique(),
				accounts: vec![
					AccountMeta::new(Pubkey::new_unique(), false),
					AccountMeta::new(Pubkey::new_unique(), true),
				],
				data: vec![0x02, 0x00, 0x00, 0x00],
			},
			inner_instructions: vec![],
		};

		let monitor_match = SolanaMonitorMatch::new(
			monitor,
			"solana_mainnet".to_string(),
			MatchConditions {
				functions: vec![],
				events: vec![],
				transactions: vec![],
			},
			None,
			SolanaTransaction {
				signature: metadata.transaction_metadata.signature,
				transaction: VersionedTransaction::from(
					solana_sdk::transaction::Transaction::new_unsigned(Message::new(
						&[nested_instruction.instruction],
						Some(&metadata.transaction_metadata.fee_payer),
					)),
				),
				meta: SolanaTransactionStatusMeta::default(),
				slot: metadata.transaction_metadata.slot,
				block_time: metadata.transaction_metadata.block_time,
			},
		);

		assert_eq!(
			match monitor_match.transaction.message() {
				VersionedMessage::Legacy(msg) => msg.instructions.len(),
				VersionedMessage::V0(msg) => msg.instructions.len(),
			},
			1
		);
		let instruction = match monitor_match.transaction.message() {
			VersionedMessage::Legacy(msg) => &msg.instructions[0],
			VersionedMessage::V0(msg) => &msg.instructions[0],
		};
		assert_eq!(instruction.accounts.len(), 2);
	}

	#[test]
	fn test_instruction_decoder_trait() {
		struct TestDecoder;

		impl<'a> SolanaInstructionDecoder<'a> for TestDecoder {
			type InstructionType = String;

			fn decode_instruction(
				&self,
				instruction: &'a Instruction,
			) -> Option<SolanaDecodedInstruction<Self::InstructionType>> {
				if instruction.program_id
					== Pubkey::from_str("11111111111111111111111111111111").unwrap()
				{
					Some(SolanaDecodedInstruction {
						program_id: instruction.program_id,
						data: "Kamino Lend Deposit".to_string(),
						accounts: instruction.accounts.clone(),
					})
				} else {
					None
				}
			}
		}

		let decoder = TestDecoder;
		let instruction = create_kamino_lend_instruction();

		let decoded = decoder.decode_instruction(&instruction).unwrap();
		assert_eq!(decoded.data, "Kamino Lend Deposit");
		assert_eq!(decoded.accounts.len(), 8);
	}
}
