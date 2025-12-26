//citrate/cli/src/utils/keystore.rs
//
// Ed25519 keystore - aligned with wallet for account portability

use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use anyhow::{Context, Result};
use ed25519_dalek::SigningKey;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_256};
use std::fs;
use std::path::Path;

#[derive(Serialize, Deserialize)]
struct Keystore {
    version: u32,
    key_type: String, // "ed25519" for new keys
    encrypted_key: String,
    salt: String,
    nonce: String,
    public_key: Option<String>, // Store public key for reference
}

/// Save an ed25519 signing key to an encrypted keystore file
pub fn save_key(signing_key: &SigningKey, password: &str, path: &Path) -> Result<()> {
    // Generate random salt
    let mut salt = [0u8; 32];
    OsRng.fill_bytes(&mut salt);

    // Derive encryption key from password
    let encryption_key = derive_key(password, &salt)?;

    // Generate random nonce for AES-GCM
    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    // Encrypt the private key using AES-256-GCM
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&encryption_key));
    let key_bytes = signing_key.to_bytes();

    let ciphertext = cipher
        .encrypt(nonce, key_bytes.as_ref())
        .map_err(|e| anyhow::anyhow!("Encryption failed: {}", e))?;

    // Store public key for reference (not encrypted)
    let public_key = signing_key.verifying_key();

    let keystore = Keystore {
        version: 2, // Version 2 = ed25519
        key_type: "ed25519".to_string(),
        encrypted_key: hex::encode(ciphertext),
        salt: hex::encode(salt),
        nonce: hex::encode(nonce_bytes),
        public_key: Some(hex::encode(public_key.to_bytes())),
    };

    // Create parent directory if needed
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let json = serde_json::to_string_pretty(&keystore)?;
    fs::write(path, json).with_context(|| format!("Failed to write keystore to {:?}", path))?;

    Ok(())
}

/// Load an ed25519 signing key from an encrypted keystore file
pub fn load_key(path: &Path, password: &str) -> Result<SigningKey> {
    let contents = fs::read_to_string(path)
        .with_context(|| format!("Failed to read keystore from {:?}", path))?;

    let keystore: Keystore = serde_json::from_str(&contents).context("Invalid keystore format")?;

    // Check key type
    if keystore.version >= 2 && keystore.key_type != "ed25519" {
        anyhow::bail!("Unsupported key type: {}", keystore.key_type);
    }

    // Decode salt and nonce
    let salt = hex::decode(&keystore.salt).context("Invalid salt format")?;
    let nonce_bytes = hex::decode(&keystore.nonce).context("Invalid nonce format")?;
    let nonce = Nonce::from_slice(&nonce_bytes);

    // Derive key from password
    let encryption_key = derive_key(password, &salt)?;

    // Decrypt the private key
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&encryption_key));
    let ciphertext =
        hex::decode(&keystore.encrypted_key).context("Invalid encrypted key format")?;

    let plaintext = cipher
        .decrypt(nonce, ciphertext.as_ref())
        .map_err(|_| anyhow::anyhow!("Decryption failed - invalid password or corrupted keystore"))?;

    // Convert to ed25519 signing key
    let key_bytes: [u8; 32] = plaintext
        .try_into()
        .map_err(|_| anyhow::anyhow!("Invalid key length"))?;

    Ok(SigningKey::from_bytes(&key_bytes))
}

fn derive_key(password: &str, salt: &[u8]) -> Result<[u8; 32]> {
    // Use SHA3-256 with multiple iterations for key derivation
    let mut key = [0u8; 32];
    let mut hasher = Sha3_256::new();

    // Initial hash
    hasher.update(password.as_bytes());
    hasher.update(salt);
    let mut hash = hasher.finalize();

    // Apply 10000 iterations to strengthen the key derivation
    for _ in 0..10000 {
        let mut new_hasher = Sha3_256::new();
        new_hasher.update(&hash);
        new_hasher.update(password.as_bytes());
        new_hasher.update(salt);
        hash = new_hasher.finalize();
    }

    key.copy_from_slice(&hash);
    Ok(key)
}
