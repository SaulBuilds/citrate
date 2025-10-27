// citrate/core/execution/src/zkp/types.rs

// ZKP types
use ark_bls12_381::Bls12_381;
use ark_groth16;
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use hex;
use serde::{Deserialize, Serialize};
use std::hash::Hash;

/// Proof type alias for Groth16 over BLS12-381
pub type Proof = ark_groth16::Proof<Bls12_381>;

/// Proving key type alias
pub type ProvingKey = ark_groth16::ProvingKey<Bls12_381>;

/// Verifying key type alias
pub type VerifyingKey = ark_groth16::VerifyingKey<Bls12_381>;

/// Circuit definition for model execution proof
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelExecutionCircuit {
    pub model_hash: Vec<u8>,
    pub input_hash: Vec<u8>,
    pub output_hash: Vec<u8>,
    pub computation_trace: Vec<ComputationStep>,
}

/// Single step in computation trace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputationStep {
    pub operation: String,
    pub input_values: Vec<f64>,
    pub output_value: f64,
}

/// Training gradient proof circuit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GradientProofCircuit {
    pub model_hash: Vec<u8>,
    pub dataset_hash: Vec<u8>,
    pub gradient_hash: Vec<u8>,
    pub loss_value: f64,
    pub num_samples: u64,
}

/// Trait for circuits that can expose public inputs for proof generation
pub trait PublicInputsProducer {
    fn public_inputs(&self) -> Vec<String>;
}

impl PublicInputsProducer for ModelExecutionCircuit {
    fn public_inputs(&self) -> Vec<String> {
        vec![
            format!("0x{}", hex::encode(&self.model_hash)),
            format!("0x{}", hex::encode(&self.input_hash)),
            format!("0x{}", hex::encode(&self.output_hash)),
        ]
    }
}

impl PublicInputsProducer for GradientProofCircuit {
    fn public_inputs(&self) -> Vec<String> {
        vec![
            format!("0x{}", hex::encode(&self.model_hash)),
            format!("0x{}", hex::encode(&self.dataset_hash)),
            format!("0x{}", hex::encode(&self.gradient_hash)),
            format!("{}", self.loss_value),
            self.num_samples.to_string(),
        ]
    }
}


/// Serializable proof wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableProof {
    pub proof_bytes: Vec<u8>,
    pub public_inputs: Vec<String>,
}

impl SerializableProof {
    pub fn from_proof(proof: &Proof, public_inputs: Vec<String>) -> Result<Self, ZKPError> {
        let mut proof_bytes = Vec::new();
        proof
            .serialize_compressed(&mut proof_bytes)
            .map_err(|e| ZKPError::SerializationError(e.to_string()))?;

        Ok(Self {
            proof_bytes,
            public_inputs,
        })
    }

    pub fn to_proof(&self) -> Result<Proof, ZKPError> {
        Proof::deserialize_compressed(&self.proof_bytes[..])
            .map_err(|e| ZKPError::DeserializationError(e.to_string()))
    }
}

/// Verification result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    pub is_valid: bool,
    pub public_inputs: Vec<String>,
    pub error_message: Option<String>,
}

/// ZKP-specific errors
#[derive(Debug, thiserror::Error)]
pub enum ZKPError {
    #[error("Circuit synthesis failed: {0}")]
    SynthesisError(String),

    #[error("Proof generation failed: {0}")]
    ProvingError(String),

    #[error("Verification failed: {0}")]
    VerificationError(String),

    #[error("Invalid public inputs")]
    InvalidPublicInputs,

    #[error("Setup failed: {0}")]
    SetupError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Deserialization error: {0}")]
    DeserializationError(String),

    #[error("Key not found: {0}")]
    KeyNotFound(String),

    #[error("Invalid circuit")]
    InvalidCircuit,
}

/// Proof type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ProofType {
    ModelExecution,
    GradientSubmission,
    DataIntegrity,
    StateTransition,
}

/// Proof request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofRequest {
    pub proof_type: ProofType,
    pub circuit_data: Vec<u8>,
    pub public_inputs: Vec<String>,
}

/// Proof response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofResponse {
    pub proof: SerializableProof,
    pub proof_type: ProofType,
    pub generation_time_ms: u64,
}
