use lattice_wallet::{keystore::Keystore, wallet::Wallet, transaction::TransactionBuilder};
use lattice_execution::types::Address;
use primitive_types::U256;
use ed25519_dalek::SigningKey;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Lattice Transaction Test ===");
    
    // Create wallet with RPC connection
    let keystore = Keystore::new("test_keystore.json");
    let mut wallet = Wallet::new(keystore, "http://localhost:8545", 1337).await?;
    
    // Treasury private key (test only)
    let treasury_key_bytes = hex::decode("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef")?;
    let signing_key = SigningKey::from_bytes(&treasury_key_bytes.try_into().unwrap());
    
    // Add treasury account
    wallet.import_key(signing_key.clone())?;
    
    // Get treasury address
    let treasury_addr = wallet.get_account(0)?.address();
    println!("Treasury address: 0x{}", hex::encode(&treasury_addr.0));
    
    // Check balance
    let balance = wallet.get_balance(treasury_addr).await?;
    println!("Treasury balance: {} wei", balance);
    
    // Create recipient address
    let recipient = Address([0x22; 20]); // 0x2222...2222
    println!("Recipient address: 0x{}", hex::encode(&recipient.0));
    
    // Build transaction
    let tx = TransactionBuilder::new()
        .from(treasury_addr)
        .to(recipient)
        .value(U256::from(1_000_000_000_000_000_000u64)) // 1 LATT
        .nonce(0)
        .gas_price(1_000_000_000) // 1 gwei
        .gas_limit(21000)
        .chain_id(1337)
        .build_and_sign(&signing_key)?;
    
    println!("Transaction hash: 0x{}", hex::encode(&tx.transaction.hash.0));
    
    // Send transaction
    println!("Sending transaction...");
    let tx_hash = wallet.send_raw_transaction(&tx.raw).await?;
    println!("Submitted with hash: 0x{}", hex::encode(&tx_hash.0));
    
    // Wait a bit for block
    println!("Waiting for block inclusion...");
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
    
    // Check recipient balance
    let recipient_balance = wallet.get_balance(recipient).await?;
    println!("Recipient balance after: {} wei", recipient_balance);
    
    println!("=== Test Complete ===");
    Ok(())
}