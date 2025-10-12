use anyhow::{Context, Result};
use secp256k1::SecretKey;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Serialize, Deserialize)]
struct Keystore {
    version: u32,
    encrypted_key: String,
    salt: String,
    iv: String,
}

pub fn save_key(secret_key: &SecretKey, password: &str, path: &Path) -> Result<()> {
    // Simple encryption (in production, use proper key derivation and encryption)
    use sha3::{Digest, Sha3_256};

    let mut hasher = Sha3_256::new();
    hasher.update(password.as_bytes());
    let password_hash = hasher.finalize();

    // Generate salt and IV
    let salt = hex::encode(&password_hash[..16]);
    let iv = hex::encode(&password_hash[16..32]);

    // XOR encrypt the key (simplified - use AES in production)
    let key_bytes = secret_key.secret_bytes();
    let mut encrypted = Vec::new();
    for (i, byte) in key_bytes.iter().enumerate() {
        encrypted.push(byte ^ password_hash[i % 32]);
    }

    let keystore = Keystore {
        version: 1,
        encrypted_key: hex::encode(encrypted),
        salt,
        iv,
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

    // Derive key from password
    use sha3::{Digest, Sha3_256};

    let mut hasher = Sha3_256::new();
    hasher.update(password.as_bytes());
    let password_hash = hasher.finalize();

    // Verify salt matches
    let expected_salt = hex::encode(&password_hash[..16]);
    if keystore.salt != expected_salt {
        anyhow::bail!("Invalid password");
    }

    // Decrypt the key
    let encrypted_bytes = hex::decode(&keystore.encrypted_key)?;
    let mut decrypted = Vec::new();
    for (i, byte) in encrypted_bytes.iter().enumerate() {
        decrypted.push(byte ^ password_hash[i % 32]);
    }

    SecretKey::from_slice(&decrypted).context("Failed to recover private key")
}
