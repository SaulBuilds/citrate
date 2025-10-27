// citrate/core/execution/src/crypto/mod.rs

// Cryptography module for secure model storage and privacy-preserving inference

pub mod encryption;
pub mod key_manager;
pub mod secure_enclave;

pub use encryption::{
    EncryptedModel,
    ModelEncryption,
    EncryptionConfig,
    EncryptionMetadata,
    EncryptedKey,
    decrypt_model,
    encrypt_model,
};

pub use key_manager::{
    KeyManager,
    DerivedKey,
    AccessPolicy,
    KeyPurpose,
};

#[cfg(target_os = "macos")]
pub use secure_enclave::{
    AppleSecureEnclave,
    SecureEnclaveInterface,
    Attestation,
};