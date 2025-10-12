use lattice_wallet::wallet::{Wallet, WalletConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Lattice Wallet Example ===");

    // Configure and create wallet
    let config = WalletConfig {
        rpc_url: "http://localhost:8545".to_string(),
        chain_id: 1337,
        keystore_path: std::path::PathBuf::from("test_keystore.json"),
        ..Default::default()
    };

    let mut wallet = Wallet::new(config)?;
    // Create a new account (for demo only)
    let account = wallet.create_account("test-password", Some("demo".into()))?;
    println!("Created account: {}", account.address);

    println!("=== Example Complete ===");
    Ok(())
}
