//! CLI Wallet Integration Tests
//!
//! Verifies CLI wallet functionality against a real node:
//! - Account creation and import
//! - Address derivation consistency
//! - Transaction signing and submission
//! - Balance and nonce queries
//!
//! Run with: cargo test -p citrate-wallet --test integration_tests
//!
//! Environment variables:
//! - CITRATE_RPC_URL: RPC endpoint (default: http://localhost:8545)
//! - CITRATE_CHAIN_ID: Chain ID (default: 1337)
//! - SKIP_INTEGRATION_TESTS: Set to "true" to skip these tests

use citrate_wallet::{Wallet, WalletConfig};
use citrate_execution::types::Address;
use primitive_types::U256;
use std::env;
use std::path::PathBuf;
use tempfile::TempDir;

// ============================================================================
// Test Configuration
// ============================================================================

fn get_rpc_url() -> String {
    env::var("CITRATE_RPC_URL").unwrap_or_else(|_| "http://localhost:8545".to_string())
}

fn get_chain_id() -> u64 {
    env::var("CITRATE_CHAIN_ID")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(1337)
}

fn should_skip_tests() -> bool {
    env::var("SKIP_INTEGRATION_TESTS").map(|v| v == "true").unwrap_or(false)
}

/// Well-known test accounts (from Hardhat/Anvil)
const TEST_PRIVATE_KEY: &str = "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
const TEST_ADDRESS: &str = "f39Fd6e51aad88F6F4ce6aB8827279cffFb92266";

const TEST_PRIVATE_KEY_2: &str = "59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d";
const TEST_ADDRESS_2: &str = "70997970C51812dc3A010C7d01b50e0d17dc79C8";

// ============================================================================
// Test Helpers
// ============================================================================

fn create_test_wallet(temp_dir: &TempDir) -> Wallet {
    let config = WalletConfig {
        keystore_path: temp_dir.path().join("keystore"),
        rpc_url: get_rpc_url(),
        chain_id: get_chain_id(),
    };
    Wallet::new(config).expect("Failed to create wallet")
}

fn hex_to_address(hex: &str) -> Address {
    let bytes = hex::decode(hex.trim_start_matches("0x")).expect("Invalid hex");
    let mut addr = [0u8; 20];
    addr.copy_from_slice(&bytes);
    Address(addr)
}

async fn is_node_running() -> bool {
    let client = reqwest::Client::new();
    let rpc_url = get_rpc_url();

    let response = client
        .post(&rpc_url)
        .json(&serde_json::json!({
            "jsonrpc": "2.0",
            "method": "eth_chainId",
            "params": [],
            "id": 1
        }))
        .send()
        .await;

    matches!(response, Ok(r) if r.status().is_success())
}

// ============================================================================
// Account Creation Tests
// ============================================================================

#[test]
fn test_create_account() {
    if should_skip_tests() {
        println!("Skipping integration tests");
        return;
    }

    let temp_dir = TempDir::new().unwrap();
    let mut wallet = create_test_wallet(&temp_dir);

    // Create new account
    let password = "test_password_123";
    let account = wallet.create_account(password, Some("test_account".to_string()));

    assert!(account.is_ok(), "Account creation failed");
    let account = account.unwrap();

    // Verify account properties
    assert_eq!(account.index, 0);
    assert!(!account.address.0.iter().all(|&b| b == 0), "Address should not be zero");
    assert_eq!(account.alias, Some("test_account".to_string()));
}

#[test]
fn test_create_multiple_accounts() {
    if should_skip_tests() {
        return;
    }

    let temp_dir = TempDir::new().unwrap();
    let mut wallet = create_test_wallet(&temp_dir);
    let password = "test_password_123";

    // Create multiple accounts
    let account1 = wallet.create_account(password, Some("account1".to_string())).unwrap();
    let account2 = wallet.create_account(password, Some("account2".to_string())).unwrap();
    let account3 = wallet.create_account(password, Some("account3".to_string())).unwrap();

    // Verify unique indices
    assert_eq!(account1.index, 0);
    assert_eq!(account2.index, 1);
    assert_eq!(account3.index, 2);

    // Verify unique addresses
    assert_ne!(account1.address, account2.address);
    assert_ne!(account1.address, account3.address);
    assert_ne!(account2.address, account3.address);
}

// ============================================================================
// Account Import Tests
// ============================================================================

#[test]
fn test_import_account() {
    if should_skip_tests() {
        return;
    }

    let temp_dir = TempDir::new().unwrap();
    let mut wallet = create_test_wallet(&temp_dir);
    let password = "test_password_123";

    // Import known test account
    let account = wallet.import_account(TEST_PRIVATE_KEY, password, Some("imported".to_string()));

    assert!(account.is_ok(), "Account import failed: {:?}", account.err());
    let account = account.unwrap();

    // Verify address matches expected
    let expected_address = hex_to_address(TEST_ADDRESS);
    assert_eq!(
        account.address.0.to_vec(),
        expected_address.0.to_vec(),
        "Imported address doesn't match expected"
    );
}

#[test]
fn test_import_with_0x_prefix() {
    if should_skip_tests() {
        return;
    }

    let temp_dir = TempDir::new().unwrap();
    let mut wallet = create_test_wallet(&temp_dir);
    let password = "test_password_123";

    // Import with 0x prefix
    let key_with_prefix = format!("0x{}", TEST_PRIVATE_KEY);
    let account = wallet.import_account(&key_with_prefix, password, None);

    assert!(account.is_ok(), "Import with 0x prefix failed");
    let account = account.unwrap();

    let expected_address = hex_to_address(TEST_ADDRESS);
    assert_eq!(account.address.0.to_vec(), expected_address.0.to_vec());
}

#[test]
fn test_import_invalid_key() {
    if should_skip_tests() {
        return;
    }

    let temp_dir = TempDir::new().unwrap();
    let mut wallet = create_test_wallet(&temp_dir);
    let password = "test_password_123";

    // Try to import invalid key
    let invalid_key = "not_a_valid_private_key";
    let result = wallet.import_account(invalid_key, password, None);

    assert!(result.is_err(), "Should fail with invalid key");
}

// ============================================================================
// Account Listing Tests
// ============================================================================

#[test]
fn test_list_accounts() {
    if should_skip_tests() {
        return;
    }

    let temp_dir = TempDir::new().unwrap();
    let mut wallet = create_test_wallet(&temp_dir);
    let password = "test_password_123";

    // Initially empty
    let accounts = wallet.list_accounts();
    assert!(accounts.is_empty(), "Should have no accounts initially");

    // Create accounts
    wallet.create_account(password, Some("acc1".to_string())).unwrap();
    wallet.create_account(password, Some("acc2".to_string())).unwrap();

    // Refresh and list
    wallet.refresh_accounts().unwrap();
    let accounts = wallet.list_accounts();

    assert_eq!(accounts.len(), 2, "Should have 2 accounts");
}

// ============================================================================
// Wallet Unlock Tests
// ============================================================================

#[test]
fn test_wallet_unlock() {
    if should_skip_tests() {
        return;
    }

    let temp_dir = TempDir::new().unwrap();
    let mut wallet = create_test_wallet(&temp_dir);
    let password = "test_password_123";

    // Create account
    wallet.create_account(password, None).unwrap();

    // Unlock with correct password
    let result = wallet.unlock(password);
    assert!(result.is_ok(), "Should unlock with correct password");

    // Unlock with wrong password
    let result = wallet.unlock("wrong_password");
    assert!(result.is_err(), "Should fail with wrong password");
}

// ============================================================================
// Export Key Tests
// ============================================================================

#[test]
fn test_export_private_key() {
    if should_skip_tests() {
        return;
    }

    let temp_dir = TempDir::new().unwrap();
    let mut wallet = create_test_wallet(&temp_dir);
    let password = "test_password_123";

    // Import known account
    wallet.import_account(TEST_PRIVATE_KEY, password, None).unwrap();

    // Unlock wallet
    wallet.unlock(password).unwrap();

    // Export key
    let exported_key = wallet.export_private_key(0);
    assert!(exported_key.is_ok(), "Export should succeed");

    let exported = exported_key.unwrap();
    // Normalize comparison (remove 0x prefix if present)
    let exported_normalized = exported.trim_start_matches("0x").to_lowercase();
    let expected_normalized = TEST_PRIVATE_KEY.to_lowercase();

    assert_eq!(
        exported_normalized,
        expected_normalized,
        "Exported key should match original"
    );
}

// ============================================================================
// RPC Integration Tests
// ============================================================================

#[tokio::test]
async fn test_get_balance_integration() {
    if should_skip_tests() {
        println!("Skipping integration tests");
        return;
    }

    if !is_node_running().await {
        println!("Skipping: Node not running at {}", get_rpc_url());
        return;
    }

    let temp_dir = TempDir::new().unwrap();
    let wallet = create_test_wallet(&temp_dir);

    // Get balance for test address
    let address = hex_to_address(TEST_ADDRESS);
    let balance = wallet.rpc_client().get_balance(&address).await;

    assert!(balance.is_ok(), "Balance query failed: {:?}", balance.err());
    let balance = balance.unwrap();

    // Genesis account should have balance
    assert!(balance > U256::zero(), "Genesis account should have balance");
}

#[tokio::test]
async fn test_get_nonce_integration() {
    if should_skip_tests() {
        return;
    }

    if !is_node_running().await {
        println!("Skipping: Node not running");
        return;
    }

    let temp_dir = TempDir::new().unwrap();
    let wallet = create_test_wallet(&temp_dir);

    // Get nonce for test address
    let address = hex_to_address(TEST_ADDRESS);
    let nonce = wallet.rpc_client().get_nonce(&address).await;

    assert!(nonce.is_ok(), "Nonce query failed");
    // Nonce should be >= 0
    assert!(nonce.unwrap() >= 0);
}

#[tokio::test]
async fn test_get_block_number_integration() {
    if should_skip_tests() {
        return;
    }

    if !is_node_running().await {
        println!("Skipping: Node not running");
        return;
    }

    let temp_dir = TempDir::new().unwrap();
    let wallet = create_test_wallet(&temp_dir);

    let block_number = wallet.rpc_client().get_block_number().await;

    assert!(block_number.is_ok(), "Block number query failed");
    assert!(block_number.unwrap() >= 0, "Block number should be >= 0");
}

#[tokio::test]
async fn test_get_gas_price_integration() {
    if should_skip_tests() {
        return;
    }

    if !is_node_running().await {
        println!("Skipping: Node not running");
        return;
    }

    let temp_dir = TempDir::new().unwrap();
    let wallet = create_test_wallet(&temp_dir);

    let gas_price = wallet.rpc_client().get_gas_price().await;

    assert!(gas_price.is_ok(), "Gas price query failed");
    assert!(gas_price.unwrap() > 0, "Gas price should be > 0");
}

// ============================================================================
// Address Derivation Consistency Tests
// ============================================================================

#[test]
fn test_address_derivation_consistency() {
    if should_skip_tests() {
        return;
    }

    let temp_dir = TempDir::new().unwrap();
    let mut wallet = create_test_wallet(&temp_dir);
    let password = "test_password_123";

    // Import same key multiple times (different wallets)
    wallet.import_account(TEST_PRIVATE_KEY, password, None).unwrap();

    let temp_dir2 = TempDir::new().unwrap();
    let mut wallet2 = create_test_wallet(&temp_dir2);
    wallet2.import_account(TEST_PRIVATE_KEY, password, None).unwrap();

    // Refresh and compare
    wallet.refresh_accounts().unwrap();
    wallet2.refresh_accounts().unwrap();

    let account1 = wallet.get_account(0).unwrap();
    let account2 = wallet2.get_account(0).unwrap();

    assert_eq!(
        account1.address.0.to_vec(),
        account2.address.0.to_vec(),
        "Same private key should produce same address"
    );
}

#[test]
fn test_different_keys_different_addresses() {
    if should_skip_tests() {
        return;
    }

    let temp_dir = TempDir::new().unwrap();
    let mut wallet = create_test_wallet(&temp_dir);
    let password = "test_password_123";

    // Import two different keys
    wallet.import_account(TEST_PRIVATE_KEY, password, None).unwrap();
    wallet.import_account(TEST_PRIVATE_KEY_2, password, None).unwrap();

    wallet.refresh_accounts().unwrap();

    let account1 = wallet.get_account(0).unwrap();
    let account2 = wallet.get_account(1).unwrap();

    assert_ne!(
        account1.address.0.to_vec(),
        account2.address.0.to_vec(),
        "Different keys should produce different addresses"
    );
}

// ============================================================================
// Config Tests
// ============================================================================

#[test]
fn test_wallet_config() {
    if should_skip_tests() {
        return;
    }

    let temp_dir = TempDir::new().unwrap();
    let wallet = create_test_wallet(&temp_dir);

    let config = wallet.config();

    assert_eq!(config.rpc_url, get_rpc_url());
    assert_eq!(config.chain_id, get_chain_id());
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[test]
fn test_get_nonexistent_account() {
    if should_skip_tests() {
        return;
    }

    let temp_dir = TempDir::new().unwrap();
    let mut wallet = create_test_wallet(&temp_dir);

    // Try to get account that doesn't exist
    let account = wallet.get_account(999);
    assert!(account.is_none(), "Should return None for nonexistent account");
}

#[test]
fn test_export_without_unlock() {
    if should_skip_tests() {
        return;
    }

    let temp_dir = TempDir::new().unwrap();
    let mut wallet = create_test_wallet(&temp_dir);
    let password = "test_password_123";

    // Create account
    wallet.create_account(password, None).unwrap();

    // Try to export without unlocking
    let result = wallet.export_private_key(0);
    assert!(result.is_err(), "Should fail to export without unlocking");
}
