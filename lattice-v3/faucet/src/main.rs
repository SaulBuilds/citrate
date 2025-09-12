use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use ed25519_dalek::SigningKey;
use lattice_consensus::types::{Hash, PublicKey, Signature, Transaction};
use lattice_execution::types::Address;
use primitive_types::U256;
use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_256};
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_http::cors::CorsLayer;
use tracing::{info, error};
use tracing_subscriber;

#[derive(Clone)]
struct FaucetState {
    signing_key: Arc<SigningKey>,
    rpc_url: String,
    nonce: Arc<Mutex<u64>>,
    chain_id: u64,
    faucet_address: Address,
}

#[derive(Debug, Deserialize)]
struct FaucetRequest {
    address: String,
}

#[derive(Debug, Serialize)]
struct FaucetResponse {
    success: bool,
    tx_hash: Option<String>,
    message: String,
    amount: String,
}

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    info!("Starting Lattice Faucet Service");
    
    // Faucet private key (for testing only - uses a portion of treasury funds)
    // In production, this would be a separate funded account
    let faucet_key_bytes = hex::decode(
        "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
    ).expect("Invalid faucet key");
    
    let signing_key = SigningKey::from_bytes(
        &faucet_key_bytes.try_into().unwrap()
    );
    
    // Calculate faucet address from public key
    let public_key = signing_key.verifying_key();
    let mut hasher = Sha3_256::new();
    hasher.update(public_key.as_bytes());
    let hash = hasher.finalize();
    let mut addr_bytes = [0u8; 20];
    addr_bytes.copy_from_slice(&hash[12..32]);
    let faucet_address = Address(addr_bytes);
    
    info!("Faucet address: 0x{}", hex::encode(&faucet_address.0));
    
    let state = FaucetState {
        signing_key: Arc::new(signing_key),
        rpc_url: "http://localhost:8545".to_string(),
        nonce: Arc::new(Mutex::new(0)),
        chain_id: 1337,
        faucet_address,
    };
    
    // Build router
    let app = Router::new()
        .route("/", get(root))
        .route("/faucet", post(request_tokens))
        .route("/status", get(status))
        .layer(CorsLayer::permissive())
        .with_state(state);
    
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001")
        .await
        .unwrap();
    
    info!("Faucet listening on http://0.0.0.0:3001");
    info!("Request test tokens: POST /faucet with {{\"address\": \"0x...\"}}");
    
    axum::serve(listener, app).await.unwrap();
}

async fn root() -> &'static str {
    "Lattice Testnet Faucet - POST /faucet with {\"address\": \"0x...\"}"
}

async fn status() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "online",
        "network": "lattice-testnet",
        "amount_per_request": "10 LATT"
    }))
}

async fn request_tokens(
    State(state): State<FaucetState>,
    Json(payload): Json<FaucetRequest>,
) -> Result<Json<FaucetResponse>, StatusCode> {
    // Parse recipient address
    let recipient_hex = payload.address.trim_start_matches("0x");
    let recipient_bytes = match hex::decode(recipient_hex) {
        Ok(b) if b.len() == 20 => b,
        _ => {
            return Ok(Json(FaucetResponse {
                success: false,
                tx_hash: None,
                message: "Invalid address format".to_string(),
                amount: "0".to_string(),
            }));
        }
    };
    
    let mut recipient_addr = [0u8; 20];
    recipient_addr.copy_from_slice(&recipient_bytes);
    let recipient = Address(recipient_addr);
    
    info!("Faucet request for address: 0x{}", hex::encode(&recipient.0));
    
    // Create transaction
    let mut nonce_guard = state.nonce.lock().await;
    let nonce = *nonce_guard;
    
    // Convert recipient address to PublicKey format for transaction
    let mut to_pk_bytes = [0u8; 32];
    to_pk_bytes[..20].copy_from_slice(&recipient.0);
    let to_pubkey = PublicKey::new(to_pk_bytes);
    
    // Convert faucet address to PublicKey for transaction
    let mut from_pk_bytes = [0u8; 32];
    from_pk_bytes[..20].copy_from_slice(&state.faucet_address.0);
    let from_pubkey = PublicKey::new(from_pk_bytes);
    
    // Build transaction
    let mut tx = Transaction {
        hash: Hash::default(),
        from: from_pubkey,
        to: Some(to_pubkey),
        value: 10_000_000_000_000_000_000u128, // 10 LATT
        data: vec![],
        nonce,
        gas_price: 1_000_000_000, // 1 gwei
        gas_limit: 21000,
        signature: Signature::new([0; 64]),
        tx_type: None,
    };
    
    // Calculate transaction hash
    tx.hash = calculate_tx_hash(&tx, state.chain_id);
    
    // Sign transaction with the inner signing key
    use ed25519_dalek::Signer;
    let signature = state.signing_key.as_ref().sign(tx.hash.as_bytes());
    tx.signature = Signature::new(signature.to_bytes());
    
    // Serialize transaction
    let tx_bytes = match bincode::serialize(&tx) {
        Ok(b) => b,
        Err(e) => {
            error!("Failed to serialize transaction: {}", e);
            return Ok(Json(FaucetResponse {
                success: false,
                tx_hash: None,
                message: "Failed to create transaction".to_string(),
                amount: "0".to_string(),
            }));
        }
    };
    
    let tx_hex = format!("0x{}", hex::encode(&tx_bytes));
    
    // Send transaction via RPC
    let client = reqwest::Client::new();
    let response = client
        .post(&state.rpc_url)
        .json(&serde_json::json!({
            "jsonrpc": "2.0",
            "method": "eth_sendRawTransaction",
            "params": [tx_hex],
            "id": 1
        }))
        .send()
        .await;
    
    match response {
        Ok(res) => {
            let json: serde_json::Value = res.json().await.unwrap_or_default();
            
            if let Some(result) = json.get("result").and_then(|r| r.as_str()) {
                // Success - increment nonce
                *nonce_guard += 1;
                
                info!("Faucet sent 10 LATT to {} - tx: {}", 
                    payload.address, result);
                
                Ok(Json(FaucetResponse {
                    success: true,
                    tx_hash: Some(result.to_string()),
                    message: "Successfully sent 10 LATT".to_string(),
                    amount: "10000000000000000000".to_string(),
                }))
            } else if let Some(error) = json.get("error") {
                error!("RPC error: {:?}", error);
                Ok(Json(FaucetResponse {
                    success: false,
                    tx_hash: None,
                    message: format!("Transaction failed: {:?}", error),
                    amount: "0".to_string(),
                }))
            } else {
                Ok(Json(FaucetResponse {
                    success: false,
                    tx_hash: None,
                    message: "Unknown RPC response".to_string(),
                    amount: "0".to_string(),
                }))
            }
        }
        Err(e) => {
            error!("Failed to send transaction: {}", e);
            Ok(Json(FaucetResponse {
                success: false,
                tx_hash: None,
                message: "Failed to connect to node".to_string(),
                amount: "0".to_string(),
            }))
        }
    }
}

fn calculate_tx_hash(tx: &Transaction, chain_id: u64) -> Hash {
    let mut hasher = Sha3_256::new();
    
    // Hash transaction fields (EIP-155 style)
    hasher.update(&tx.nonce.to_le_bytes());
    hasher.update(&tx.gas_price.to_le_bytes());
    hasher.update(&tx.gas_limit.to_le_bytes());
    
    if let Some(to) = &tx.to {
        hasher.update(&to.0);
    }
    
    hasher.update(&tx.value.to_le_bytes());
    hasher.update(&tx.data);
    hasher.update(&chain_id.to_le_bytes());
    hasher.update(&[0u8; 8]); // r placeholder
    hasher.update(&[0u8; 8]); // s placeholder
    
    let result = hasher.finalize();
    let mut hash_bytes = [0u8; 32];
    hash_bytes.copy_from_slice(&result);
    Hash::new(hash_bytes)
}