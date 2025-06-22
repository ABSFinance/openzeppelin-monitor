//! Solana-specific decoders for program instructions and accounts
//!
//! This module provides traits and implementations for decoding Solana program
//! instructions and account data.

use {
	carbon_core::{
		instruction::{DecodedInstruction, InstructionDecoder},
		try_decode_instructions,
	},
	carbon_jupiter_dca_decoder::{instructions::JupiterDcaInstruction, JupiterDcaDecoder},
	carbon_kamino_farms_decoder::{instructions::KaminoFarmsInstruction, KaminoFarmsDecoder},
	carbon_kamino_lending_decoder::{
		accounts::KaminoLendingAccount, instructions::KaminoLendingInstruction,
		KaminoLendingDecoder,
	},
	serde::{Deserialize, Deserializer, Serialize, Serializer},
	solana_sdk::pubkey::Pubkey,
	std::fmt,
};

macro_rules! try_decode_instructions {
	($instruction:expr, $($variant:path => $decoder:expr),* $(,)?) => {{
		$(
			if let Some(decoded) = $decoder.decode_instruction($instruction) {
				return Some(DecodedInstruction {
					program_id: decoded.program_id,
					data: $variant(decoded.data),
					accounts: decoded.accounts,
				});
			}
		)*
		None
	}};
}

/// Wrapper for KaminoLendingAccount to handle serialization
#[derive(Clone)]
pub struct KaminoLendingAccountWrapper<'a>(&'a KaminoLendingAccount);

impl<'a> fmt::Debug for KaminoLendingAccountWrapper<'a> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_tuple("KaminoLendingAccountWrapper")
			.field(&"<opaque>")
			.finish()
	}
}

impl<'a> From<&'a KaminoLendingAccount> for KaminoLendingAccountWrapper<'a> {
	fn from(account: &'a KaminoLendingAccount) -> Self {
		Self(account)
	}
}

impl Serialize for KaminoLendingAccountWrapper<'_> {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		serializer.serialize_str("KaminoLendingAccount")
	}
}

impl<'de> Deserialize<'de> for KaminoLendingAccountWrapper<'_> {
	fn deserialize<D>(_deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		Err(serde::de::Error::custom(
			"Cannot deserialize KaminoLendingAccount",
		))
	}
}

impl<'a> PartialEq for KaminoLendingAccountWrapper<'a> {
	fn eq(&self, _other: &Self) -> bool {
		std::ptr::eq(self.0, _other.0)
	}
}

impl<'a> Eq for KaminoLendingAccountWrapper<'a> {}

/// Supported account types that can be decoded
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AccountType {
	AssociatedTokenAccount,
	KaminoLendingAccount(KaminoLendingAccountWrapper<'static>),
	KaminoFarms,
	KaminoLimitOrder,
	JupiterSwap,
	JupiterLimitOrder,
	JupiterLimitOrder2,
	JupiterDCA,
	JupiterPerpetuals,
	DriftV2,
	MarginfiV2,
	MeteoraDLMM,
	MeteoraPools,
	MarinadeFinance,
	MemoProgram,
	MPLTokenMetadata,
	MPLCore,
	NameService,
	OKXDex,
	OpenbookV2,
	OrcaWhirlpool,
	PhoenixV1,
	PumpSwap,
	PumpFun,
	RaydiumAMMV4,
	RaydiumCLMM,
	RaydiumCPMM,
	RaydiumLaunchpad,
	RaydiumLiquidityLocking,
	Sharky,
	SolayerRestaking,
	StabbleStableSwap,
	StabbleWeightedSwap,
	StakeProgram,
	Token2022,
	TokenProgram,
	SystemProgram,
	Virtuals,
	Zeta,
}

/// Supported instruction types that can be decoded
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum InstructionType {
	#[default]
	Unknown,
	AssociatedTokenAccount,
	KaminoLendingInstruction(KaminoLendingInstruction),
	KaminoFarmsInstruction(KaminoFarmsInstruction),
	KaminoLimitOrder,
	JupiterSwap,
	JupiterLimitOrder,
	JupiterLimitOrder2,
	JupiterDCA(JupiterDcaInstruction),
	JupiterPerpetuals,
	DriftV2,
	MarginfiV2,
	MeteoraDLMM,
	MeteoraPools,
	MarinadeFinance,
	MemoProgram,
	MPLTokenMetadata,
	MPLCore,
	NameService,
	OKXDex,
	OpenbookV2,
	OrcaWhirlpool,
	PhoenixV1,
	PumpSwap,
	PumpFun,
	RaydiumAMMV4,
	RaydiumCLMM,
	RaydiumCPMM,
	RaydiumLaunchpad,
	RaydiumLiquidityLocking,
	Sharky,
	SolayerRestaking,
	StabbleStableSwap,
	StabbleWeightedSwap,
	StakeProgram,
	Token2022,
	TokenProgram,
	SystemProgram,
	Virtuals,
	Zeta,
}

impl From<KaminoLendingInstruction> for InstructionType {
	fn from(instruction: KaminoLendingInstruction) -> Self {
		InstructionType::KaminoLendingInstruction(instruction)
	}
}

/// Trait for decoders that can handle different instruction types
pub trait DecoderTrait {
	type InstructionType;
	fn decode_instruction(
		&self,
		instruction: &solana_instruction::Instruction,
	) -> Option<DecodedInstruction<Self::InstructionType>>;
}

/// A decoder that can handle different instruction types
pub struct Decoder {
	kamino_lending_decoder: KaminoLendingDecoder,
	kamino_farms_decoder: KaminoFarmsDecoder,
	jupiter_dca_decoder: JupiterDcaDecoder,
}

impl Decoder {
	pub fn new() -> Self {
		Self {
			kamino_lending_decoder: KaminoLendingDecoder,
			kamino_farms_decoder: KaminoFarmsDecoder,
			jupiter_dca_decoder: JupiterDcaDecoder,
		}
	}

	pub fn decode_instruction(
		&self,
		instruction: &solana_instruction::Instruction,
	) -> Option<DecodedInstruction<InstructionType>> {
		try_decode_instructions!(
			instruction,
			InstructionType::KaminoLendingInstruction => &self.kamino_lending_decoder,
			InstructionType::KaminoFarmsInstruction => &self.kamino_farms_decoder,
			InstructionType::JupiterDCA => &self.jupiter_dca_decoder,
		)
	}
}

impl Default for Decoder {
	fn default() -> Self {
		Self::new()
	}
}

/// Helper function to create match parameters for an instruction
pub fn create_match_params(
	program_id: &Pubkey,
	instruction: &InstructionType,
) -> Vec<crate::models::blockchain::solana::SolanaMatchParamEntry> {
	let params = vec![crate::models::blockchain::solana::SolanaMatchParamEntry {
		name: "program_id".to_string(),
		value: program_id.to_string(),
		kind: "pubkey".to_string(),
		indexed: false,
	}];

	// Add instruction-specific parameters
	if let InstructionType::KaminoLendingInstruction(ix) = instruction {
		match ix {
			KaminoLendingInstruction::InitLendingMarket(_data) => {
				// Add InitLendingMarket specific parameters
			}
			KaminoLendingInstruction::UpdateLendingMarket(_data) => {
				// Add UpdateLendingMarket specific parameters
			}
			_ => {}
		}
	}
	// Add other instruction type parameters

	params
}

#[cfg(test)]
mod tests {

	#[test]
	fn test_matches_instruction_type() {
		// Add tests for instruction type matching
	}

	#[test]
	fn test_create_match_params() {
		// Add tests for parameter creation
	}
}
