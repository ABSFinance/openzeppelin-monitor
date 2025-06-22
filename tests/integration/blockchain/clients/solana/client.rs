use crate::integration::mocks::{
	create_solana_test_network_with_urls, create_solana_valid_server_mock_network_response,
	MockSolanaClientTrait, MockSolanaTransportClient,
};
use mockall::predicate;
use mockito::Server;
use openzeppelin_monitor::{
	models::{
		default_ui_transaction_status_meta, BlockType, Network, SecretString, SecretValue,
		SolanaBlock, SolanaTransaction,
	},
	services::blockchain::{BlockChainClient, SolanaClient, SolanaClientTrait},
};
use solana_sdk::{
	commitment_config::CommitmentConfig,
	hash::Hash,
	instruction::{AccountMeta, Instruction},
	pubkey::Pubkey,
	signature::{Keypair, Signature, Signer},
	transaction::Transaction,
};

#[tokio::test(flavor = "multi_thread")]
async fn test_get_transaction_by_signature() {
	let mut mock = MockSolanaClientTrait::<MockSolanaTransportClient>::new();

	let fee_payer = Keypair::new();
	let program_id = Pubkey::new_unique();
	let account1 = Keypair::new();
	let account2 = Pubkey::new_unique();

	let instruction = Instruction {
		program_id,
		accounts: vec![
			AccountMeta::new(account1.pubkey(), true),
			AccountMeta::new(account2, false),
		],
		data: vec![1, 2, 3, 4],
	};

	let signature = Signature::new_unique();
	let recent_blockhash = Hash::new_unique();

	// Create a properly signed transaction with all required signers
	let transaction = Transaction::new_signed_with_payer(
		&[instruction],
		Some(&fee_payer.pubkey()),
		&[&fee_payer, &account1],
		recent_blockhash,
	);

	let expected_transaction = SolanaTransaction {
		signature,
		transaction: transaction.into(),
		meta: default_ui_transaction_status_meta(),
		slot: 12345,
		block_time: Some(1678901234),
	};

	mock.expect_get_transaction_by_signature()
		.with(predicate::eq("test_signature".to_string()))
		.times(1)
		.returning(move |_| Ok(expected_transaction.clone()));

	let result = mock.get_transaction_by_signature("test_signature").await;
	assert!(result.is_ok());
	assert_eq!(result.unwrap().signature, signature);
}

#[tokio::test(flavor = "multi_thread")]
async fn test_get_latest_slot() {
	let mut mock = MockSolanaClientTrait::<MockSolanaTransportClient>::new();
	mock.expect_get_latest_slot()
		.times(1)
		.returning(|| Ok(100u64));

	let result = mock.get_latest_slot().await;
	assert!(result.is_ok());
	assert_eq!(result.unwrap(), 100u64);
}

#[tokio::test(flavor = "multi_thread")]
async fn test_get_block_time() {
	let mut mock = MockSolanaClientTrait::<MockSolanaTransportClient>::new();
	mock.expect_get_block_time()
		.with(predicate::eq(12345u64))
		.times(1)
		.returning(|_| Ok(1678901234i64));

	let result = mock.get_block_time(12345).await;
	assert!(result.is_ok());
	assert_eq!(result.unwrap(), 1678901234i64);
}

#[tokio::test(flavor = "multi_thread")]
async fn test_get_blocks() {
	let mut mock = MockSolanaClientTrait::<MockSolanaTransportClient>::new();

	let fee_payer = Keypair::new();
	let program_id = Pubkey::new_unique();
	let account1 = Keypair::new();
	let account2 = Pubkey::new_unique();

	let instruction = Instruction {
		program_id,
		accounts: vec![
			AccountMeta::new(account1.pubkey(), true),
			AccountMeta::new(account2, false),
		],
		data: vec![1, 2, 3, 4],
	};

	let signature = Signature::new_unique();
	let recent_blockhash = Hash::new_unique();

	// Create a properly signed transaction with all required signers
	let transaction = Transaction::new_signed_with_payer(
		&[instruction],
		Some(&fee_payer.pubkey()),
		&[&fee_payer, &account1],
		recent_blockhash,
	);

	let expected_transaction = SolanaTransaction {
		signature,
		transaction: transaction.into(),
		meta: default_ui_transaction_status_meta(),
		slot: 12345,
		block_time: Some(1678901234),
	};

	let block = BlockType::Solana(Box::new(SolanaBlock {
		slot: 12345,
		blockhash: signature.to_string(),
		parent_slot: 12344,
		transactions: vec![expected_transaction],
		block_time: Some(1678901234),
		block_height: Some(12345),
		rewards: None,
		commitment: CommitmentConfig::default(),
	}));

	let blocks = vec![block];

	mock.expect_get_blocks()
		.with(predicate::eq(1u64), predicate::eq(Some(2u64)))
		.times(1)
		.returning(move |_, _| Ok(blocks.clone()));

	let result = mock.get_blocks(1, Some(2)).await;
	assert!(result.is_ok());
	let blocks = result.unwrap();
	assert_eq!(blocks.len(), 1);
	match &blocks[0] {
		BlockType::Solana(block) => assert_eq!(block.slot, 12345),
		_ => panic!("Expected Solana block"),
	}
}

#[tokio::test(flavor = "multi_thread")]
async fn test_get_latest_block_number() {
	let mut mock = MockSolanaClientTrait::<MockSolanaTransportClient>::new();
	mock.expect_get_latest_block_number()
		.times(1)
		.returning(|| Ok(100u64));

	let result = mock.get_latest_block_number().await;
	assert!(result.is_ok());
	assert_eq!(result.unwrap(), 100u64);
}

#[tokio::test(flavor = "multi_thread")]
async fn test_new_client() {
	let mut server = Server::new_async().await;
	let mock = create_solana_valid_server_mock_network_response(&mut server);
	let network = create_solana_test_network_with_urls(vec![&server.url()]);
	let result = SolanaClient::new(&network).await;
	assert!(result.is_ok(), "Client creation should succeed");
	let client = result.unwrap();
	let _slot_result = client.get_latest_slot().await;
	// The important thing is that the request was made to the mock server
	// The RPC call may fail due to mock server not behaving like a real Solana node
	mock.assert();
}

#[tokio::test(flavor = "multi_thread")]
async fn test_get_block_by_slot() {
	let mut mock = MockSolanaClientTrait::<MockSolanaTransportClient>::new();

	let fee_payer = Keypair::new();
	let program_id = Pubkey::new_unique();
	let account1 = Keypair::new();
	let account2 = Pubkey::new_unique();

	let instruction = Instruction {
		program_id,
		accounts: vec![
			AccountMeta::new(account1.pubkey(), true),
			AccountMeta::new(account2, false),
		],
		data: vec![1, 2, 3, 4],
	};

	let signature = Signature::new_unique();
	let recent_blockhash = Hash::new_unique();

	// Create a properly signed transaction with all required signers
	let transaction = Transaction::new_signed_with_payer(
		&[instruction],
		Some(&fee_payer.pubkey()),
		&[&fee_payer, &account1],
		recent_blockhash,
	);

	let expected_transaction = SolanaTransaction {
		signature,
		transaction: transaction.into(),
		meta: default_ui_transaction_status_meta(),
		slot: 12345,
		block_time: Some(1678901234),
	};

	let expected_block = BlockType::Solana(Box::new(SolanaBlock {
		slot: 12345,
		blockhash: signature.to_string(),
		parent_slot: 12344,
		transactions: vec![expected_transaction],
		block_time: Some(1678901234),
		block_height: Some(12345),
		rewards: None,
		commitment: CommitmentConfig::default(),
	}));

	mock.expect_get_block_by_slot()
		.with(predicate::eq(12345u64))
		.times(1)
		.returning(move |_| Ok(expected_block.clone()));

	let result = mock.get_block_by_slot(12345).await;
	assert!(result.is_ok());
	match result.unwrap() {
		BlockType::Solana(block) => assert_eq!(block.slot, 12345),
		_ => panic!("Expected Solana block"),
	}
}

#[tokio::test(flavor = "multi_thread")]
async fn test_rpc_client_access() {
	// Test that an actual RPC call fails with an invalid URL
	let network = Network {
		name: "Test Solana".to_string(),
		slug: "test_solana_rpc".to_string(),
		network_type: openzeppelin_monitor::models::BlockChainType::Solana,
		chain_id: None,
		store_blocks: Some(false),
		rpc_urls: vec![openzeppelin_monitor::models::RpcUrl {
			type_: "rpc".to_string(),
			url: SecretValue::Plain(SecretString::new(
				"http://invalid-url-that-does-not-exist.com".to_string(),
			)),
			weight: 100,
		}],
		block_time_ms: 400,
		confirmation_blocks: 1,
		cron_schedule: "0 */1 * * * *".to_string(),
		max_past_blocks: Some(200),
		network_passphrase: None,
	};
	let result = SolanaClient::new(&network).await;
	assert!(
		result.is_ok(),
		"Client creation should succeed even with invalid URL"
	);
	let client = result.unwrap();
	let slot_result = client.get_latest_slot().await;
	assert!(
		slot_result.is_err(),
		"Expected connection failure with invalid URL"
	);
}

#[tokio::test(flavor = "multi_thread")]
async fn test_client_with_multiple_endpoints() {
	let mut server1 = Server::new_async().await;
	let server2 = Server::new_async().await;
	let mock1 = create_solana_valid_server_mock_network_response(&mut server1);
	// Don't create mock2 since the client only uses the first endpoint
	let network = create_solana_test_network_with_urls(vec![&server1.url(), &server2.url()]);
	let result = SolanaClient::new(&network).await;
	assert!(
		result.is_ok(),
		"Client creation should succeed with multiple endpoints"
	);
	let client = result.unwrap();
	let _slot_result = client.get_latest_slot().await;
	// The important thing is that the request was made to the first mock server
	// The Solana client only uses the first endpoint for get_slot() calls
	mock1.assert();
}

#[tokio::test]
async fn test_client_with_empty_rpc_urls() {
	// Create a network with no RPC URLs
	let network = Network {
		name: "Test Solana".to_string(),
		slug: "test_solana_empty".to_string(),
		network_type: openzeppelin_monitor::models::BlockChainType::Solana,
		chain_id: None,
		store_blocks: Some(false),
		rpc_urls: vec![],
		block_time_ms: 400,
		confirmation_blocks: 1,
		cron_schedule: "0 */1 * * * *".to_string(),
		max_past_blocks: Some(200),
		network_passphrase: None,
	};

	// Test that client creation fails with no RPC URLs
	let result = SolanaClient::new(&network).await;
	assert!(
		result.is_err(),
		"Client creation should fail with no RPC URLs"
	);
}

#[tokio::test]
async fn test_client_with_zero_weight_urls() {
	let server = Server::new_async().await;

	// Create a network with zero weight RPC URLs
	let network = Network {
		name: "Test Solana".to_string(),
		slug: "test_solana_zero_weight".to_string(),
		network_type: openzeppelin_monitor::models::BlockChainType::Solana,
		chain_id: None,
		store_blocks: Some(false),
		rpc_urls: vec![openzeppelin_monitor::models::RpcUrl {
			type_: "rpc".to_string(),
			url: SecretValue::Plain(SecretString::new(server.url())),
			weight: 0, // Zero weight
		}],
		block_time_ms: 400,
		confirmation_blocks: 1,
		cron_schedule: "0 */1 * * * *".to_string(),
		max_past_blocks: Some(200),
		network_passphrase: None,
	};

	// Test that client creation fails with zero weight URLs
	let result = SolanaClient::new(&network).await;
	assert!(
		result.is_err(),
		"Client creation should fail with zero weight URLs"
	);
}

#[tokio::test]
async fn test_client_with_non_rpc_urls() {
	let server = Server::new_async().await;

	// Create a network with non-RPC URLs
	let network = Network {
		name: "Test Solana".to_string(),
		slug: "test_solana_non_rpc".to_string(),
		network_type: openzeppelin_monitor::models::BlockChainType::Solana,
		chain_id: None,
		store_blocks: Some(false),
		rpc_urls: vec![openzeppelin_monitor::models::RpcUrl {
			type_: "ws".to_string(), // Non-RPC type
			url: SecretValue::Plain(SecretString::new(server.url())),
			weight: 100,
		}],
		block_time_ms: 400,
		confirmation_blocks: 1,
		cron_schedule: "0 */1 * * * *".to_string(),
		max_past_blocks: Some(200),
		network_passphrase: None,
	};

	// Test that client creation fails with non-RPC URLs
	let result = SolanaClient::new(&network).await;
	assert!(
		result.is_err(),
		"Client creation should fail with non-RPC URLs"
	);
}
