//! Solana-specific decoders for program instructions and accounts
//!
//! This module provides traits and implementations for decoding Solana program
//! instructions and account data.
use crate::models::SolanaDecodedInstruction;
use serde::{Deserialize, Serialize};
use solana_sdk::{
	account_info::AccountInfo,
	instruction::{AccountMeta, Instruction},
	pubkey::Pubkey,
};
use std::fmt::Debug;

use carbon_kamino_lending_decoder::{
	accounts::KaminoLendingAccount, instructions::KaminoLendingInstruction, KaminoLendingDecoder,
	PROGRAM_ID as KAMINO_LENDING_PROGRAM_ID,
};

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

pub fn decode_kamino_lending_account(account: &solana_account::Account) -> Option<AccountType> {
	if account.owner != KAMINO_LENDING_PROGRAM_ID {
		return None;
	}

	KaminoLendingDecoder::decode_account(account.data).map(AccountType::KaminoLendingAccount)
}

pub fn decode_kamino_lending_instruction(
	instruction: &solana_instruction::Instruction,
) -> Option<InstructionType> {
	if instruction.program_id != KAMINO_LENDING_PROGRAM_ID {
		return None;
	}

	KaminoLendingDecoder::decode_instruction(instruction.data)
		.map(InstructionType::KaminoLendingInstruction)
}
