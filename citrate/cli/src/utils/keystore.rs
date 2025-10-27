//citrate/cli/src/utils/keystore.rs

use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce, Key
};
use anyhow::{Context, Result};
use rand::RngCore;
use secp256k1::SecretKey;
use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_256};
use std::fs;
use std::path::Path;

#[derive(Serialize, Deserialize)]
struct Keystore {
    version: u32,
    encrypted_key: String,
    salt: String,
    nonce: String,
}

pub fn save_key(secret_key: &SecretKey, password: &str, path: &Path) -> Result<()> {
    // Generate random salt
    let mut salt = [0u8; 32];
    OsRng.fill_bytes(&mut salt);

    // Derive key from password using PBKDF2-like approach
    let encryption_key = derive_key(password, &salt)?;

    // Generate random nonce for AES-GCM
    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    // Encrypt the private key using AES-256-GCM
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&encryption_key));
    let key_bytes = secret_key.secret_bytes();

    let ciphertext = cipher
        .encrypt(nonce, key_bytes.as_ref())
        .map_err(|e| anyhow::anyhow!("Encryption failed: {}", e))?;

    let keystore = Keystore {
        version: 1,
        encrypted_key: hex::encode(ciphertext),
        salt: hex::encode(salt),
        nonce: hex::encode(nonce_bytes),
    };

    // Create parent directory if needed
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let json = serde_json::to_string_pretty(&keystore)?;
    fs::write(path, json).with_context(|| format!("Failed to write keystore to {:?}", path))?;

    Ok(())
}

pub fn load_key(path: &Path, password: &str) -> Result<SecretKey> {
    let contents = fs::read_to_string(path)
        .with_context(|| format!("Failed to read keystore from {:?}", path))?;

    let keystore: Keystore = serde_json::from_str(&contents).context("Invalid keystore format")?;

    // Decode salt and nonce
    let salt = hex::decode(&keystore.salt).context("Invalid salt format")?;
    let nonce_bytes = hex::decode(&keystore.nonce).context("Invalid nonce format")?;
    let nonce = Nonce::from_slice(&nonce_bytes);

    // Derive key from password
    let encryption_key = derive_key(password, &salt)?;

    // Decrypt the private key
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&encryption_key));
    let ciphertext = hex::decode(&keystore.encrypted_key).context("Invalid encrypted key format")?;

    let plaintext = cipher
        .decrypt(nonce, ciphertext.as_ref())
        .map_err(|_| anyhow::anyhow!("Decryption failed - invalid password or corrupted keystore"))?;

    SecretKey::from_slice(&plaintext).context("Failed to recover private key")
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
