//! Solana-specific decoders for program instructions and accounts
//!
//! This module provides traits and implementations for decoding Solana program
//! instructions and account data.

use {
	carbon_core::instruction::{DecodedInstruction, InstructionDecoder},
	carbon_kamino_lending_decoder::{
		accounts::KaminoLendingAccount, instructions::KaminoLendingInstruction,
		KaminoLendingDecoder,
	},
	serde::{Deserialize, Deserializer, Serialize, Serializer},
	solana_sdk::pubkey::Pubkey,
	std::fmt,
};

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
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
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

impl From<KaminoLendingInstruction> for InstructionType {
	fn from(instruction: KaminoLendingInstruction) -> Self {
		InstructionType::KaminoLendingInstruction(instruction)
	}
}

/// A decoder that can handle different instruction types
pub struct Decoder {
	kamino_decoder: KaminoLendingDecoder,
	// Add other decoders here as needed
}

impl Decoder {
	pub fn new() -> Self {
		Self {
			kamino_decoder: KaminoLendingDecoder,
			// Initialize other decoders
		}
	}
}

impl<'a> InstructionDecoder<'a> for Decoder {
	type InstructionType = InstructionType;

	fn decode_instruction(
		&self,
		instruction: &'a solana_instruction::Instruction,
	) -> Option<DecodedInstruction<Self::InstructionType>> {
		// Try Kamino decoder
		if let Some(decoded) = self.kamino_decoder.decode_instruction(instruction) {
			return Some(DecodedInstruction {
				program_id: decoded.program_id,
				data: InstructionType::KaminoLendingInstruction(decoded.data),
				accounts: decoded.accounts,
			});
		}

		None
	}
}

/// Helper function to create match parameters for an instruction
pub fn create_match_params(
	program_id: &Pubkey,
	instruction: &InstructionType,
) -> Vec<crate::models::blockchain::solana::SolanaMatchParamEntry> {
	let mut params = vec![crate::models::blockchain::solana::SolanaMatchParamEntry {
		name: "program_id".to_string(),
		value: program_id.to_string(),
		kind: "pubkey".to_string(),
		indexed: false,
	}];

	// Add instruction-specific parameters
	match instruction {
		InstructionType::KaminoLendingInstruction(ix) => {
			match ix {
				KaminoLendingInstruction::InitLendingMarket(data) => {
					// Add InitLendingMarket specific parameters
				}
				KaminoLendingInstruction::UpdateLendingMarket(data) => {
					// Add UpdateLendingMarket specific parameters
				}
				_ => {}
			}
		}
		// Add other instruction type parameters
		_ => {}
	}

	params
}

#[cfg(test)]
mod tests {
	use super::*;
	use solana_sdk::instruction::Instruction;

	#[test]
	fn test_matches_instruction_type() {
		// Add tests for instruction type matching
	}

	#[test]
	fn test_create_match_params() {
		// Add tests for parameter creation
	}
}
