use {
	crate::models::blockchain::solana::SolanaMatchArguments,
	crate::models::{SolanaContractSpec, SolanaTransaction},
	crate::{
		models::{
			blockchain::solana::{SolanaMatchParamEntry, SolanaMatchParamsMap},
			BlockType, ContractSpec, FunctionCondition, MatchConditions, Monitor, MonitorMatch,
			Network, TransactionCondition, TransactionStatus,
		},
		services::{
			decoders::InstructionType,
			filter::{error::FilterError, filters::BlockFilter},
		},
	},
	hex,
	solana_client::rpc_client::RpcClient,
	solana_sdk::{
		instruction::Instruction, message::Message, signature::Keypair, signer::Signer,
		system_instruction, transaction::Transaction,
	},
	std::marker::PhantomData,
};

use async_trait::async_trait;

pub struct SolanaBlockFilter<T> {
	pub _client: PhantomData<T>,
}

impl<T> SolanaBlockFilter<T> {
	pub fn new() -> Self {
		Self {
			_client: PhantomData,
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
					"u8" | "u64" | "u128" | "u32" | "usize" => {
						let Ok(param_value) = param.value.parse::<u64>() else {
							return false;
						};
						let Ok(compare_value) = value.parse::<u64>() else {
							return false;
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
		transaction: &Transaction,
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
								value: transaction.signatures[0].to_string(),
								kind: "string".to_string(),
								indexed: false,
							},
							SolanaMatchParamEntry {
								name: "fee_payer".to_string(),
								value: transaction.message.account_keys[0].to_string(),
								kind: "pubkey".to_string(),
								indexed: false,
							},
							SolanaMatchParamEntry {
								name: "recent_blockhash".to_string(),
								value: transaction.message.recent_blockhash.to_string(),
								kind: "string".to_string(),
								indexed: false,
							},
							SolanaMatchParamEntry {
								name: "fee".to_string(),
								value: transaction
									.message
									.header
									.num_required_signatures
									.to_string(),
								kind: "u8".to_string(),
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

	/// Finds function calls in a transaction that match the monitor's conditions.
	///
	/// Decodes the transaction instructions and matches against
	/// the monitor's function conditions.
	///
	/// # Arguments
	/// * `contract_specs` - List of contract specifications
	/// * `transaction` - The transaction containing the function call
	/// * `monitor` - Monitor containing function match conditions
	/// * `matched_functions` - Vector to store matching functions
	/// * `matched_on_args` - Arguments from matched function calls
	pub fn find_matching_functions_for_transaction(
		&self,
		contract_specs: &[(String, SolanaContractSpec)],
		transaction: &SolanaTransaction,
		monitor: &Monitor,
		matched_functions: &mut Vec<FunctionCondition>,
		matched_on_args: &mut SolanaMatchArguments,
	) {
		if !monitor.match_conditions.functions.is_empty() {
			// Process each instruction in the transaction
			for instruction in &transaction.instructions {
				// Find the matching monitored address for the instruction
				if let Some(monitored_addr) = monitor
					.addresses
					.iter()
					.find(|addr| instruction.program_id.to_string() == addr.address)
				{
					// Process the matching address's decoder
					if let Some((_, decoder)) = contract_specs
						.iter()
						.find(|(address, _)| address == &monitored_addr.address)
					{
						// Check if we have a matching function condition
						for function_condition in &monitor.match_conditions.functions {
							if let Some(expr) = &function_condition.expression {
								// Create parameter entries for the instruction
								let mut params = vec![SolanaMatchParamEntry {
									name: "program_id".to_string(),
									value: instruction.program_id.to_string(),
									kind: "pubkey".to_string(),
									indexed: false,
								}];

								// Add account parameters with proper arrangement
								if let Some(decoder) = decoder.instruction.as_ref() {
									match decoder {
										InstructionType::KaminoLendingInstruction(
											kamino_instruction,
										) => {
											// Add arranged accounts based on instruction type
											if let Some(arranged_accounts) = kamino_instruction
												.arrange_accounts(&instruction.accounts)
											{
												for (name, account) in arranged_accounts.iter() {
													params.push(SolanaMatchParamEntry {
														name: name.to_string(),
														value: account.to_string(),
														kind: "pubkey".to_string(),
														indexed: false,
													});
												}
											}
										}
										_ => {
											// Fallback to generic account handling
											for (i, account) in
												instruction.accounts.iter().enumerate()
											{
												params.push(SolanaMatchParamEntry {
													name: format!("account_{}", i),
													value: account.pubkey.to_string(),
													kind: "pubkey".to_string(),
													indexed: account.is_signer,
												});
											}
										}
									}
								}

								// Add data parameter if present
								if !instruction.data.is_empty() {
									params.push(SolanaMatchParamEntry {
										name: "data".to_string(),
										value: hex::encode(&instruction.data),
										kind: "bytes".to_string(),
										indexed: false,
									});
								}

								// Evaluate the expression
								if self.evaluate_expression(expr, &Some(params)) {
									matched_functions.push(function_condition.clone());
									matched_on_args.instructions =
										Some(vec![SolanaMatchParamsMap {
											signature: instruction.program_id.to_string(),
											args: Some(params),
											hex_signature: Some(hex::encode(&instruction.data)),
										}]);
									break;
								}
							} else {
								// If no expression, match any function call to this program
								matched_functions.push(function_condition.clone());
								break;
							}
						}
					}
				}
			}
		}
	}
}

impl<T> Default for SolanaBlockFilter<T> {
	fn default() -> Self {
		Self::new()
	}
}

#[async_trait]
impl<T: Send + Sync> BlockFilter for SolanaBlockFilter<T> {
	type Client = RpcClient;

	async fn filter_block(
		&self,
		_client: &Self::Client,
		_network: &Network,
		_block: &BlockType,
		_monitors: &[Monitor],
		_contract_specs: Option<&[(String, ContractSpec)]>,
	) -> Result<Vec<MonitorMatch>, FilterError> {
		// TODO: Implement Solana-specific block filtering logic
		Ok(Vec::new())
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		models::{AddressWithSpec, EventCondition, FunctionCondition, MatchConditions},
		utils::tests::builders::solana::monitor::MonitorBuilder,
	};
	use solana_sdk::instruction::Instruction;

	use solana_sdk::{
		message::Message, signature::Keypair, signer::Signer, system_instruction,
		transaction::Transaction,
	};

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

	fn create_test_transaction(
		fee_payer: &Keypair,
		instructions: Vec<Instruction>,
		recent_blockhash: solana_sdk::hash::Hash,
	) -> Transaction {
		let message = Message::new(&instructions, Some(&fee_payer.pubkey()));
		Transaction::new(&[fee_payer], message, recent_blockhash)
	}

	#[test]
	fn test_find_matching_transaction_empty_conditions_matches_all() {
		let filter = create_test_filter();
		let mut matched = Vec::new();
		let monitor = create_test_monitor(vec![], vec![], vec![], vec![]);

		let fee_payer = Keypair::new();
		let recent_blockhash = solana_sdk::hash::Hash::new_unique();
		let transaction = create_test_transaction(&fee_payer, vec![], recent_blockhash);

		filter.find_matching_transaction(&transaction, &monitor, &mut matched);

		assert_eq!(matched.len(), 1);
		assert_eq!(matched[0].expression, None);
		assert_eq!(matched[0].status, TransactionStatus::Any);
	}

	#[test]
	fn test_find_matching_transaction_with_signature_expression() {
		let filter = create_test_filter();
		let mut matched = Vec::new();

		let fee_payer = Keypair::new();
		let recent_blockhash = solana_sdk::hash::Hash::new_unique();
		let transaction = create_test_transaction(&fee_payer, vec![], recent_blockhash);

		let monitor = create_test_monitor(
			vec![],
			vec![],
			vec![TransactionCondition {
				expression: Some(format!("signature == {}", transaction.signatures[0])),
				status: TransactionStatus::Any,
			}],
			vec![],
		);

		filter.find_matching_transaction(&transaction, &monitor, &mut matched);

		assert_eq!(matched.len(), 1);
		assert_eq!(
			matched[0].expression,
			Some(format!("signature == {}", transaction.signatures[0]))
		);
		assert_eq!(matched[0].status, TransactionStatus::Any);
	}

	#[test]
	fn test_find_matching_transaction_with_fee_payer_expression() {
		let filter = create_test_filter();
		let mut matched = Vec::new();

		let fee_payer = Keypair::new();
		let recent_blockhash = solana_sdk::hash::Hash::new_unique();
		let transaction = create_test_transaction(&fee_payer, vec![], recent_blockhash);

		let monitor = create_test_monitor(
			vec![],
			vec![],
			vec![TransactionCondition {
				expression: Some(format!("fee_payer == {}", fee_payer.pubkey())),
				status: TransactionStatus::Any,
			}],
			vec![],
		);

		filter.find_matching_transaction(&transaction, &monitor, &mut matched);

		assert_eq!(matched.len(), 1);
		assert_eq!(
			matched[0].expression,
			Some(format!("fee_payer == {}", fee_payer.pubkey()))
		);
		assert_eq!(matched[0].status, TransactionStatus::Any);
	}

	#[test]
	fn test_find_matching_transaction_with_complex_expression() {
		let filter = create_test_filter();
		let mut matched = Vec::new();

		let fee_payer = Keypair::new();
		let recent_blockhash = solana_sdk::hash::Hash::new_unique();
		let transaction = create_test_transaction(&fee_payer, vec![], recent_blockhash);

		let monitor = create_test_monitor(
			vec![],
			vec![],
			vec![TransactionCondition {
				expression: Some(format!(
					"fee_payer == {} AND signature == {}",
					fee_payer.pubkey(),
					transaction.signatures[0]
				)),
				status: TransactionStatus::Any,
			}],
			vec![],
		);

		filter.find_matching_transaction(&transaction, &monitor, &mut matched);

		assert_eq!(matched.len(), 1);
		assert_eq!(
			matched[0].expression,
			Some(format!(
				"fee_payer == {} AND signature == {}",
				fee_payer.pubkey(),
				transaction.signatures[0]
			))
		);
		assert_eq!(matched[0].status, TransactionStatus::Any);
	}

	#[test]
	fn test_find_matching_transaction_no_match() {
		let filter = create_test_filter();
		let mut matched = Vec::new();

		let fee_payer = Keypair::new();
		let recent_blockhash = solana_sdk::hash::Hash::new_unique();
		let transaction = create_test_transaction(&fee_payer, vec![], recent_blockhash);

		let different_pubkey = Keypair::new().pubkey();
		let monitor = create_test_monitor(
			vec![],
			vec![],
			vec![TransactionCondition {
				expression: Some(format!("fee_payer == {}", different_pubkey)),
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

		let fee_payer = Keypair::new();
		let recipient = Keypair::new();
		let recent_blockhash = solana_sdk::hash::Hash::new_unique();

		let transfer_instruction =
			system_instruction::transfer(&fee_payer.pubkey(), &recipient.pubkey(), 1000);

		let transaction =
			create_test_transaction(&fee_payer, vec![transfer_instruction], recent_blockhash);

		let monitor = create_test_monitor(
			vec![],
			vec![],
			vec![TransactionCondition {
				expression: Some(format!(
					"fee_payer == {} AND recent_blockhash == {}",
					fee_payer.pubkey(),
					recent_blockhash
				)),
				status: TransactionStatus::Any,
			}],
			vec![],
		);

		filter.find_matching_transaction(&transaction, &monitor, &mut matched);

		assert_eq!(matched.len(), 1);
		assert_eq!(
			matched[0].expression,
			Some(format!(
				"fee_payer == {} AND recent_blockhash == {}",
				fee_payer.pubkey(),
				recent_blockhash
			))
		);
		assert_eq!(matched[0].status, TransactionStatus::Any);
	}
}
