use lattice_wallet::wallet::{Wallet, WalletConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Lattice Wallet Example ===");

    // Configure and create wallet
    let mut config = WalletConfig::default();
    config.rpc_url = "http://localhost:8545".to_string();
    config.chain_id = 1337;
    config.keystore_path = std::path::PathBuf::from("test_keystore.json");

    let mut wallet = Wallet::new(config)?;
    // Create a new account (for demo only)
    let account = wallet.create_account("test-password", Some("demo".into()))?;
    println!("Created account: {}", account.address);
    
    println!("=== Example Complete ===");
    Ok(())
}
