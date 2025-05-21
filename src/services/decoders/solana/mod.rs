//! Solana-specific decoders for program instructions and accounts
//!
//! This module provides traits and implementations for decoding Solana program
//! instructions and account data.

use crate::models::SolanaDecodedInstruction;
use solana_sdk::{
	account_info::AccountInfo,
	instruction::{AccountMeta, Instruction},
	pubkey::Pubkey,
};
use std::fmt::Debug;

/// Trait for decoding Solana account data
pub trait AccountDecoder: Send + Sync {
	/// The type of data that this decoder produces
	type DecodedData: Debug + Clone;

	/// Returns the program ID this decoder handles
	fn program_id(&self) -> &'static str;

	/// Decodes raw account data into a structured format
	///
	/// # Arguments
	/// * `data` - The raw account data
	/// * `owner` - The program ID that owns the account
	///
	/// # Returns
	/// `Result` containing either the decoded data or an error
	fn decode_account(
		&self,
		data: &[u8],
		owner: &Pubkey,
	) -> Result<Self::DecodedData, DecoderError>;
}

/// Trait for decoding Solana program instructions
pub trait InstructionDecoder: Send + Sync {
	/// The type of data that this decoder produces
	type DecodedData: Debug + Clone;

	/// Returns the program ID this decoder handles
	fn program_id(&self) -> &'static str;

	/// Decodes a raw instruction into a structured format
	///
	/// # Arguments
	/// * `data` - The raw instruction data
	/// * `accounts` - The accounts involved in the instruction
	///
	/// # Returns
	/// `Result` containing either the decoded instruction or an error
	fn decode_instruction(
		&self,
		data: &[u8],
		accounts: &[AccountMeta],
	) -> Result<SolanaDecodedInstruction<Self::DecodedData>, DecoderError>;
}

#[derive(Debug, thiserror::Error)]
pub enum DecoderError {
	#[error("Invalid instruction data: {0}")]
	InvalidData(String),
}
