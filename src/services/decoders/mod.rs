//! Solana-specific decoders for program instructions and accounts
//!
//! This module provides traits and implementations for decoding Solana program
//! instructions and account data.

use crate::models::SolanaDecodedInstruction;
use crate::services::decoders::kamino_lending_decoder::src::{
	accounts::KaminoLendingAccount, instructions::KaminoLendingInstruction,
};
use serde::{Deserialize, Serialize};
use solana_sdk::{
	account_info::AccountInfo,
	instruction::{AccountMeta, Instruction},
	pubkey::Pubkey,
};
use std::fmt::Debug;

/// Enum representing different types of Solana accounts that can be decoded
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AccountType {
	AssociatedTokenAccount,
	KaminoLendingAccount(KaminoLendingAccount),
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

#[derive(Debug, Clone)]
pub struct DecodedAccount<T> {
	pub lamports: u64,
	pub data: T,
	pub owner: Pubkey,
	pub executable: bool,
	pub rent_epoch: u64,
}

pub trait AccountDecoder<'a> {
	type AccountType;

	fn decode_account(
		&self,
		account: &'a solana_account::Account,
	) -> Option<DecodedAccount<Self::AccountType>>;
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DecodedInstruction<T> {
	pub program_id: Pubkey,
	pub data: T,
	pub accounts: Vec<AccountMeta>,
}

pub trait InstructionDecoder<'a> {
	type InstructionType;

	fn decode_instruction(
		&self,
		instruction: &'a solana_instruction::Instruction,
	) -> Option<DecodedInstruction<Self::InstructionType>>;
}

#[derive(Debug, thiserror::Error)]
pub enum DecoderError {
	#[error("Invalid instruction data: {0}")]
	InvalidData(String),
}
