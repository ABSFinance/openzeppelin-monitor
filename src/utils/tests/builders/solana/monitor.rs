use crate::models::{
	AddressWithSpec, ContractSpec, EventCondition, FunctionCondition, MatchConditions, Monitor,
	SolanaContractSpec, TransactionCondition, TriggerConditions,
};

/// Builder for creating test monitors
pub struct MonitorBuilder {
	name: String,
	networks: Vec<String>,
	addresses: Vec<AddressWithSpec>,
	match_conditions: MatchConditions,
	trigger_conditions: Vec<TriggerConditions>,
	triggers: Vec<String>,
	paused: bool,
}

impl MonitorBuilder {
	/// Creates a new MonitorBuilder with default values
	pub fn new() -> Self {
		Self {
			name: "TestMonitor".to_string(),
			networks: vec!["solana_mainnet".to_string()],
			addresses: vec![],
			match_conditions: MatchConditions {
				functions: vec![],
				events: vec![],
				transactions: vec![],
			},
			trigger_conditions: vec![],
			triggers: vec![],
			paused: false,
		}
	}

	/// Sets the monitor name
	pub fn name(mut self, name: &str) -> Self {
		self.name = name.to_string();
		self
	}

	/// Sets the networks
	pub fn networks(mut self, networks: Vec<String>) -> Self {
		self.networks = networks;
		self
	}

	pub fn match_conditions(mut self, match_conditions: MatchConditions) -> Self {
		self.match_conditions = match_conditions;
		self
	}

	pub fn addresses_with_spec(mut self, addresses: Vec<(String, Option<ContractSpec>)>) -> Self {
		self.addresses = addresses
			.into_iter()
			.map(|(addr, spec)| AddressWithSpec {
				address: addr.to_string(),
				contract_spec: spec,
			})
			.collect();
		self
	}

	/// Adds a function condition
	pub fn function(mut self, signature: &str, expression: Option<&str>) -> Self {
		self.match_conditions.functions.push(FunctionCondition {
			signature: signature.to_string(),
			expression: expression.map(|s| s.to_string()),
		});
		self
	}

	/// Adds an event condition
	pub fn event(mut self, signature: &str, expression: Option<&str>) -> Self {
		self.match_conditions.events.push(EventCondition {
			signature: signature.to_string(),
			expression: expression.map(|s| s.to_string()),
		});
		self
	}

	/// Adds a transaction condition
	pub fn transaction(mut self, expression: Option<&str>) -> Self {
		self.match_conditions
			.transactions
			.push(TransactionCondition {
				expression: expression.map(|s| s.to_string()),
				status: crate::models::TransactionStatus::Any,
			});
		self
	}

	/// Adds an address with contract spec
	pub fn address(
		mut self,
		address: &str,
		contract_spec: Option<crate::models::ContractSpec>,
	) -> Self {
		self.addresses.push(AddressWithSpec {
			address: address.to_string(),
			contract_spec,
		});
		self
	}

	/// Sets the trigger conditions
	pub fn trigger_conditions(mut self, conditions: Vec<TriggerConditions>) -> Self {
		self.trigger_conditions = conditions;
		self
	}

	/// Sets the triggers
	pub fn triggers(mut self, triggers: Vec<String>) -> Self {
		self.triggers = triggers;
		self
	}

	/// Sets the paused state
	pub fn paused(mut self, paused: bool) -> Self {
		self.paused = paused;
		self
	}

	/// Builds the monitor
	pub fn build(self) -> Monitor {
		Monitor {
			name: self.name,
			networks: self.networks,
			addresses: self.addresses,
			match_conditions: self.match_conditions,
			trigger_conditions: self.trigger_conditions,
			triggers: self.triggers,
			paused: self.paused,
		}
	}
}
