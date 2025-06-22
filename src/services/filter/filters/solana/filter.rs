use crate::{
	models::{
		BlockType, ContractSpec, InstructionCondition, Monitor, MonitorMatch, Network,
		SolanaContractSpec, SolanaMatchConditions, TransactionCondition, TransactionStatus,
	},
	services::{
		decoders::{Decoder, InstructionType},
		filter::{
			error::FilterError,
			filters::{solana::helpers::SolanaFilterHelpers, BlockFilter},
		},
	},
};

use {
	crate::models::blockchain::solana::{
		NestedInstructions, SolanaInstructionsWithMetadata, SolanaMatchArguments,
		SolanaMatchParamEntry, SolanaMatchParamsMap, SolanaMonitorMatch, SolanaTransaction,
		SolanaTransactionMetadata,
	},
	std::marker::PhantomData,
};

use async_trait::async_trait;

// Add Carbon's instruction types
use carbon_core::instruction::DecodedInstruction;

use tracing::instrument;

pub struct SolanaBlockFilter<T> {
	pub _client: PhantomData<T>,
	pub helpers: SolanaFilterHelpers,
	pub decoder: Decoder,
}

impl<T> SolanaBlockFilter<T> {
	pub fn new() -> Self {
		Self {
			_client: PhantomData,
			helpers: SolanaFilterHelpers::new(),
			decoder: Decoder::new(),
		}
	}

	/// Evaluates a match expression against provided parameters (EVM-style)
	pub fn evaluate_expression(
		&self,
		expression: &str,
		args: &Option<Vec<SolanaMatchParamEntry>>,
	) -> bool {
		let Some(args) = args else {
			return false;
		};

		let or_conditions: Vec<&str> = expression.split(" OR ").collect();
		for or_condition in or_conditions {
			let and_conditions: Vec<&str> = or_condition.trim().split(" AND ").collect();
			let and_result = and_conditions.iter().all(|condition| {
				let clean_condition = condition.trim().trim_matches(|c| c == '(' || c == ')');
				let parts: Vec<&str> = clean_condition.split_whitespace().collect();
				if parts.len() != 3 {
					return false;
				}
				let [param_name, operator, value] = [parts[0], parts[1], parts[2]];
				let Some(param) = args.iter().find(|p| p.name == param_name) else {
					return false;
				};
				match param.kind.as_str() {
					"u8" | "u64" | "u128" | "u32" | "usize" | "i64" => {
						let param_value = if param.kind == "i64" {
							param.value.parse::<i64>().unwrap_or(0) as u64
						} else {
							param.value.parse::<u64>().unwrap_or(0)
						};
						let compare_value = if value.starts_with('-') {
							value.parse::<i64>().unwrap_or(0) as u64
						} else {
							value.parse::<u64>().unwrap_or(0)
						};

						match operator {
							">" => param_value > compare_value,
							">=" => param_value >= compare_value,
							"<" => param_value < compare_value,
							"<=" => param_value <= compare_value,
							"==" => param_value == compare_value,
							"!=" => param_value != compare_value,
							_ => false,
						}
					}
					"string" | "pubkey" => match operator {
						"==" => param.value == value,
						"!=" => param.value != value,
						"starts_with" => param.value.starts_with(value),
						"ends_with" => param.value.ends_with(value),
						"contains" => param.value.contains(value),
						_ => false,
					},
					_ => false,
				}
			});
			if and_result {
				return true;
			}
		}
		false
	}

	/// Finds transactions that match the monitor's conditions (EVM-style)
	pub fn find_matching_transaction(
		&self,
		transaction: &SolanaTransaction,
		monitor: &Monitor,
		matched_transactions: &mut Vec<TransactionCondition>,
	) {
		if monitor.match_conditions.transactions.is_empty() {
			matched_transactions.push(TransactionCondition {
				expression: None,
				status: TransactionStatus::Any,
			});
		} else {
			for condition in &monitor.match_conditions.transactions {
				// No status logic for Solana
				let status_matches = true;
				if status_matches {
					if let Some(expr) = &condition.expression {
						let tx_params = vec![
							SolanaMatchParamEntry {
								name: "signature".to_string(),
								value: transaction.signature.to_string(),
								kind: "string".to_string(),
								indexed: false,
							},
							SolanaMatchParamEntry {
								name: "block_time".to_string(),
								value: transaction.block_time.unwrap_or(0).to_string(),
								kind: "i64".to_string(),
								indexed: false,
							},
							SolanaMatchParamEntry {
								name: "slot".to_string(),
								value: transaction.slot.to_string(),
								kind: "u64".to_string(),
								indexed: false,
							},
						];
						if self.evaluate_expression(expr, &Some(tx_params)) {
							matched_transactions.push(TransactionCondition {
								expression: Some(expr.to_string()),
								status: TransactionStatus::Any,
							});
							break;
						}
					} else {
						matched_transactions.push(TransactionCondition {
							expression: None,
							status: TransactionStatus::Any,
						});
						break;
					}
				}
			}
		}
	}

	pub fn find_matching_instruction_for_transaction(
		&self,
		contract_specs: &[(String, SolanaContractSpec)],
		transaction: &SolanaTransaction,
		monitor: &Monitor,
		matched_functions: &mut Vec<InstructionCondition>,
		matched_on_args: &mut SolanaMatchArguments,
	) {
		if !monitor.match_conditions.functions.is_empty() {
			let transaction_metadata: &SolanaTransactionMetadata =
				&(*transaction).clone().try_into().unwrap();

			let instructions_with_metadata: SolanaInstructionsWithMetadata =
				SolanaFilterHelpers::extract_instructions_with_metadata(
					transaction_metadata,
					transaction,
				)
				.unwrap();

			let nested_instructions: NestedInstructions = instructions_with_metadata.into();

			for nested_instruction in nested_instructions.iter() {
				// Find matching contract spec and decoder

				if let Some((_, contract_spec)) = contract_specs.iter().find(|(address, _)| {
					address == &nested_instruction.instruction.program_id.to_string()
				}) {
					if let Some(decoded_instruction) = self
						.decoder
						.decode_instruction(&nested_instruction.instruction)
					{
						if Self::instruction_types_match(&decoded_instruction, contract_spec) {
							for condition in &monitor.match_conditions.functions {
								// Match the instruction type based on the signature
								let matches = SolanaFilterHelpers::matches_instruction_type(
									&decoded_instruction,
									&condition.signature,
								);

								if matches {
									if let Some(expr) = &condition.expression {
										// Create match parameters for the instruction
										let params = self.extract_fields(&decoded_instruction.data);

										if self.evaluate_expression(expr, &Some(params.clone())) {
											matched_functions.push(InstructionCondition {
												signature: condition.signature.clone(),
												expression: Some(expr.to_string()),
											});
											if let Some(instructions) =
												&mut matched_on_args.instructions
											{
												instructions.push(SolanaMatchParamsMap {
													signature: condition.signature.clone(),
													args: Some(params.clone()),
												});
											};
											break;
										}
									} else {
										matched_functions.push(InstructionCondition {
											signature: condition.signature.clone(),
											expression: None,
										});
										if let Some(instructions) =
											&mut matched_on_args.instructions
										{
											instructions.push(SolanaMatchParamsMap {
												signature: condition.signature.clone(),
												args: None,
											});
										}
										break;
									}
								}
							}
						}
					}
				}
			}
		}
	}

	fn extract_fields(&self, data: &InstructionType) -> Vec<SolanaMatchParamEntry> {
		let mut params = Vec::new();

		// Helper function to add a field
		fn add_field<T: std::fmt::Debug>(
			params: &mut Vec<SolanaMatchParamEntry>,
			name: &str,
			value: &T,
		) {
			params.push(SolanaMatchParamEntry {
				name: name.to_string(),
				value: format!("{:?}", value),
				kind: std::any::type_name::<T>().to_string(),
				indexed: false,
			});
		}

		// Convert to JSON to get field names and values
		let json = serde_json::to_value(data).unwrap();

		// Extract the inner struct data
		if let serde_json::Value::Object(map) = json {
			for (_, value) in map {
				if let serde_json::Value::Object(inner_map) = value {
					for (_, value) in inner_map {
						if let serde_json::Value::Object(inner_inner_map) = value {
							for (key, value) in inner_inner_map {
								match value {
									serde_json::Value::Number(n) => {
										// Try i64 first, then fall back to u64
										if let Some(i) = n.as_i64() {
											add_field(&mut params, &key, &i);
										} else if let Some(u) = n.as_u64() {
											add_field(&mut params, &key, &u);
										}
									}
									serde_json::Value::Bool(b) => {
										add_field(&mut params, &key, &b);
									}
									serde_json::Value::String(s) => {
										add_field(&mut params, &key, &s);
									}
									_ => {}
								}
							}
						}
					}
				}
			}
		}

		params
	}

	fn instruction_types_match(
		decoded: &DecodedInstruction<InstructionType>,
		spec: &SolanaContractSpec,
	) -> bool {
		std::mem::discriminant(&decoded.data) == std::mem::discriminant(spec.instruction_type())
	}
}

impl<T> Default for SolanaBlockFilter<T> {
	fn default() -> Self {
		Self::new()
	}
}

#[async_trait]
impl<T: Send + Sync> BlockFilter for SolanaBlockFilter<T> {
	type Client = T;

	#[instrument(skip_all, fields(network = %network.slug))]
	async fn filter_block(
		&self,
		_client: &Self::Client,
		network: &Network,
		block: &BlockType,
		monitors: &[Monitor],
		contract_specs: Option<&[(String, ContractSpec)]>,
	) -> Result<Vec<MonitorMatch>, FilterError> {
		let solana_block = match block {
			BlockType::Solana(block) => block,
			_ => {
				return Err(FilterError::block_type_mismatch(
					"Expected Solana block",
					None,
					None,
				))
			}
		};

		tracing::debug!("Processing Solana block {}", solana_block.slot);

		let mut matching_results = Vec::new();

		// Cast contract specs to SolanaContractSpec
		let contract_specs = contract_specs
			.unwrap_or(&[])
			.iter()
			.filter_map(|(address, spec)| match spec {
				ContractSpec::Solana(spec) => Some((address.clone(), spec.clone())),
				_ => None,
			})
			.collect::<Vec<(String, SolanaContractSpec)>>();

		for monitor in monitors {
			tracing::debug!("Processing monitor: {:?}", monitor.name);
			let monitored_addresses: Vec<String> = monitor
				.addresses
				.iter()
				.map(|a| a.address.clone())
				.collect();

			// Process all transactions in the block
			for transaction in &solana_block.transactions {
				// Reset matched_on_args for each transaction
				let mut matched_on_args = SolanaMatchArguments {
					instructions: Some(Vec::new()),
					accounts: Some(Vec::new()),
				};

				let mut matched_instructions = Vec::<InstructionCondition>::new();
				let mut matched_transactions = Vec::<TransactionCondition>::new();

				// Check transaction match conditions
				self.find_matching_transaction(transaction, monitor, &mut matched_transactions);

				// Check instruction match conditions
				self.find_matching_instruction_for_transaction(
					&contract_specs,
					transaction,
					monitor,
					&mut matched_instructions,
					&mut matched_on_args,
				);

				// Check if any monitored addresses are involved in this transaction
				let mut involved_addresses = Vec::new();

				// Add transaction accounts to involved addresses
				match transaction.message() {
					solana_sdk::message::VersionedMessage::Legacy(msg) => {
						for account_key in &msg.account_keys {
							involved_addresses.push(account_key.to_string());
						}
					}
					solana_sdk::message::VersionedMessage::V0(msg) => {
						for account_key in &msg.account_keys {
							involved_addresses.push(account_key.to_string());
						}
					}
				}

				// Remove duplicates
				involved_addresses.sort_unstable();
				involved_addresses.dedup();

				let has_address_match = monitored_addresses
					.iter()
					.any(|addr| involved_addresses.contains(addr));

				// Only proceed if we have a matching address
				if has_address_match {
					let monitor_conditions = &monitor.match_conditions;
					let has_instruction_match = !monitor_conditions.functions.is_empty()
						&& !matched_instructions.is_empty();
					let has_transaction_match = !monitor_conditions.transactions.is_empty()
						&& !matched_transactions.is_empty();

					let should_match: bool = match (
						monitor_conditions.functions.is_empty(),
						monitor_conditions.transactions.is_empty(),
					) {
						// Case 1: No conditions defined, match everything
						(true, true) => true,

						// Case 2: Only transaction conditions defined
						(true, false) => has_transaction_match,

						// Case 3: Only instruction conditions defined
						(false, true) => has_instruction_match,

						// Case 4: Both conditions exist, they must be satisfied together
						(false, false) => has_instruction_match && has_transaction_match,
					};

					if should_match {
						matching_results.push(MonitorMatch::Solana(Box::new(SolanaMonitorMatch {
							monitor: Monitor {
								// Omit contract spec from monitor since we do not need it here
								addresses: monitor
									.addresses
									.iter()
									.map(|addr| crate::models::AddressWithSpec {
										contract_spec: None,
										..addr.clone()
									})
									.collect(),
								..monitor.clone()
							},
							transaction: transaction.clone(),
							network_slug: network.slug.clone(),
							matched_on: SolanaMatchConditions {
								instructions: matched_instructions
									.clone()
									.into_iter()
									.filter(|_| has_instruction_match)
									.collect(),
								accounts: vec![], // TODO: Implement account matching if needed
								transactions: matched_transactions
									.clone()
									.into_iter()
									.filter(|_| has_transaction_match)
									.collect(),
							},
							matched_on_args: Some(SolanaMatchArguments {
								instructions: if has_instruction_match {
									matched_on_args.instructions.clone()
								} else {
									None
								},
								accounts: matched_on_args.accounts.clone(),
							}),
						})));
					}
				}
			}
		}

		Ok(matching_results)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		models::{
			AddressWithSpec, EventCondition, FunctionCondition, MatchConditions,
			SolanaDecodedInstruction,
		},
		utils::tests::{
			builders::solana::monitor::MonitorBuilder, solana::transaction::TransactionBuilder,
		},
	};
	use carbon_jupiter_dca_decoder::instructions::{open_dca, JupiterDcaInstruction};
	use solana_instruction::AccountMeta;
	use solana_sdk::pubkey::Pubkey;
	use solana_sdk::{instruction::Instruction, message::VersionedMessage};

	use solana_sdk::{message::Message, signature::Keypair, signer::Signer};
	use solana_signature::Signature;

	fn create_test_filter() -> SolanaBlockFilter<()> {
		SolanaBlockFilter::new()
	}

	fn create_test_monitor(
		event_conditions: Vec<EventCondition>,
		function_conditions: Vec<FunctionCondition>,
		transaction_conditions: Vec<TransactionCondition>,
		addresses: Vec<AddressWithSpec>,
	) -> Monitor {
		MonitorBuilder::new()
			.name("test")
			.networks(vec!["solana_mainnet".to_string()])
			.match_conditions(MatchConditions {
				events: event_conditions,
				functions: function_conditions,
				transactions: transaction_conditions,
			})
			.addresses_with_spec(
				addresses
					.into_iter()
					.map(|a| (a.address, a.contract_spec))
					.collect(),
			)
			.build()
	}

	fn create_test_transaction() -> SolanaTransaction {
		let fee_payer = Keypair::new();
		let program_id = Pubkey::new_unique();
		let account1 = Pubkey::new_unique();
		let account2 = Pubkey::new_unique();

		let instruction = Instruction {
			program_id,
			accounts: vec![
				AccountMeta::new(account1, true),
				AccountMeta::new(account2, false),
			],
			data: vec![1, 2, 3, 4],
		};

		let message = Message::new(&[instruction], Some(&fee_payer.pubkey()));
		let signature = Signature::new_unique();

		TransactionBuilder::new()
			.slot(12345)
			.signature(signature)
			.message(VersionedMessage::Legacy(message))
			.block_time(1678901234)
			.build()
	}

	#[test]
	fn test_find_matching_transaction_empty_conditions_matches_all() {
		let filter = create_test_filter();
		let mut matched = Vec::new();
		let monitor = create_test_monitor(vec![], vec![], vec![], vec![]);

		let transaction = create_test_transaction();

		filter.find_matching_transaction(&transaction, &monitor, &mut matched);

		assert_eq!(matched.len(), 1);
		assert_eq!(matched[0].expression, None);
		assert_eq!(matched[0].status, TransactionStatus::Any);
	}

	#[test]
	fn test_find_matching_transaction_with_signature_expression() {
		let filter = create_test_filter();
		let mut matched = Vec::new();

		let transaction = create_test_transaction();

		let monitor = create_test_monitor(
			vec![],
			vec![],
			vec![TransactionCondition {
				expression: Some("block_time > 0".to_string()),
				status: TransactionStatus::Any,
			}],
			vec![],
		);

		filter.find_matching_transaction(&transaction, &monitor, &mut matched);

		assert_eq!(matched.len(), 1);
		assert_eq!(matched[0].expression, Some("block_time > 0".to_string()));
		assert_eq!(matched[0].status, TransactionStatus::Any);
	}

	#[test]
	fn test_find_matching_transaction_with_fee_payer_expression() {
		let filter = create_test_filter();
		let mut matched = Vec::new();

		let transaction = create_test_transaction();

		let monitor = create_test_monitor(
			vec![],
			vec![],
			vec![TransactionCondition {
				expression: Some("block_time > 0".to_string()),
				status: TransactionStatus::Any,
			}],
			vec![],
		);

		filter.find_matching_transaction(&transaction, &monitor, &mut matched);

		assert_eq!(matched.len(), 1);
		assert_eq!(matched[0].expression, Some("block_time > 0".to_string()));
		assert_eq!(matched[0].status, TransactionStatus::Any);
	}

	#[test]
	fn test_find_matching_transaction_with_complex_expression() {
		let filter = create_test_filter();
		let mut matched = Vec::new();

		let transaction = create_test_transaction();

		let monitor = create_test_monitor(
			vec![],
			vec![],
			vec![TransactionCondition {
				expression: Some("block_time > 0 AND slot > 0".to_string()),
				status: TransactionStatus::Any,
			}],
			vec![],
		);

		filter.find_matching_transaction(&transaction, &monitor, &mut matched);

		assert_eq!(matched.len(), 1);
		assert_eq!(
			matched[0].expression,
			Some("block_time > 0 AND slot > 0".to_string())
		);
		assert_eq!(matched[0].status, TransactionStatus::Any);
	}

	#[test]
	fn test_find_matching_transaction_no_match() {
		let filter = create_test_filter();
		let mut matched = Vec::new();

		let transaction = create_test_transaction();

		let monitor = create_test_monitor(
			vec![],
			vec![],
			vec![TransactionCondition {
				expression: Some("block_time < 0".to_string()),
				status: TransactionStatus::Any,
			}],
			vec![],
		);

		filter.find_matching_transaction(&transaction, &monitor, &mut matched);

		assert_eq!(matched.len(), 0);
	}

	#[test]
	fn test_find_matching_transaction_with_system_transfer() {
		let filter = create_test_filter();
		let mut matched = Vec::new();

		let transaction = create_test_transaction();

		let monitor = create_test_monitor(
			vec![],
			vec![],
			vec![TransactionCondition {
				expression: Some("block_time > 0".to_string()),
				status: TransactionStatus::Any,
			}],
			vec![],
		);

		filter.find_matching_transaction(&transaction, &monitor, &mut matched);

		assert_eq!(matched.len(), 1);
		assert_eq!(matched[0].expression, Some("block_time > 0".to_string()));
		assert_eq!(matched[0].status, TransactionStatus::Any);
	}

	#[test]
	fn test_find_matching_functions_for_transaction() {
		let filter = create_test_filter();
		let mut matched_functions = Vec::new();
		let mut matched_on_args = SolanaMatchArguments {
			instructions: Some(Vec::new()),
			accounts: Some(Vec::new()),
		};

		// Read instruction from fixture
		let instruction = carbon_test_utils::read_instruction("tests/fixtures/open_dca_ix.json")
			.expect("read fixture");

		// Create transaction with the instruction
		let transaction = TransactionBuilder::new()
			.slot(12345)
			.signature(Signature::new_unique())
			.instruction(SolanaDecodedInstruction {
				program_id: instruction.program_id,
				accounts: instruction.accounts,
				data: instruction.data,
			})
			.block_time(1678901234)
			.build();

		// Create contract spec
		let contract_spec = SolanaContractSpec(InstructionType::JupiterDCA(
			JupiterDcaInstruction::OpenDca(open_dca::OpenDca {
				application_idx: 1739688565,
				in_amount: 5000000,
				in_amount_per_cycle: 100000,
				cycle_frequency: 60,
				min_out_amount: Some(0),
				max_out_amount: Some(0),
				start_at: Some(0),
				close_wsol_in_ata: Some(false),
			}),
		));

		let program_id = "DCA265Vj8a9CEuX1eb1LWRnDT7uK6q1xMipnNyatn23M".to_string();
		let contract_specs = vec![(program_id.clone(), contract_spec.clone())];

		// Create monitor with function condition
		let monitor = MonitorBuilder::new()
			.name("test")
			.networks(vec!["solana_mainnet".to_string()])
			.match_conditions(MatchConditions {
				events: vec![],
				functions: vec![FunctionCondition {
					signature: "OpenDca".to_string(),
					expression: Some("in_amount > 0".to_string()),
				}],
				transactions: vec![],
			})
			.addresses_with_spec(vec![(
				program_id,
				Some(ContractSpec::Solana(contract_spec)),
			)])
			.build();

		// Test function matching
		filter.find_matching_instruction_for_transaction(
			&contract_specs,
			&transaction,
			&monitor,
			&mut matched_functions,
			&mut matched_on_args,
		);

		assert_eq!(matched_functions.len(), 1);
		assert_eq!(matched_functions[0].signature, "OpenDca");
		assert_eq!(
			matched_functions[0].expression,
			Some("in_amount > 0".to_string())
		);
	}
}
