//! Solana-specific decoders for program instructions and accounts
//!
//! This module provides traits and implementations for decoding Solana program
//! instructions and account data.
// use crate::models::SolanaDecodedInstruction;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

use carbon_kamino_lending_decoder::{
	accounts::KaminoLendingAccount, instructions::KaminoLendingInstruction,
};

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
		// Serialize as a string representation or some other format
		serializer.serialize_str("KaminoLendingAccount")
	}
}

impl<'de> Deserialize<'de> for KaminoLendingAccountWrapper<'_> {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		// Since we can't deserialize the actual data, we'll return an error
		Err(serde::de::Error::custom(
			"Cannot deserialize KaminoLendingAccount",
		))
	}
}

impl<'a> PartialEq for KaminoLendingAccountWrapper<'a> {
	fn eq(&self, _other: &Self) -> bool {
		// Since we can't compare the inner types, we'll consider them equal if they point to the same memory
		std::ptr::eq(self.0, _other.0)
	}
}

impl<'a> Eq for KaminoLendingAccountWrapper<'a> {}

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

/// Enum representing different types of Solana instructions that can be decoded
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InstructionType {
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
