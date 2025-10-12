// lattice-v3/core/execution/src/zkp/backend.rs

// ZKP backend for managing proof generation and verification
use super::prover::Prover;
use super::types::{ProofRequest, ProofResponse, ProofType, SerializableProof, ZKPError};
use super::verifier::Verifier;
use std::sync::Arc;
use std::time::Instant;

/// ZKP backend for managing proof generation and verification
pub struct ZKPBackend {
    prover: Arc<Prover>,
    verifier: Arc<Verifier>,
}

impl Default for ZKPBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl ZKPBackend {
    pub fn new() -> Self {
        let prover = Arc::new(Prover::new());
        let verifier = Arc::new(Verifier::new());

        Self { prover, verifier }
    }

    /// Initialize the backend with setup for all proof types
    pub fn initialize(&self) -> Result<(), ZKPError> {
        // Setup all proof types
        self.prover.setup(ProofType::ModelExecution)?;
        self.prover.setup(ProofType::GradientSubmission)?;
        self.prover.setup(ProofType::StateTransition)?;
        self.prover.setup(ProofType::DataIntegrity)?;

        Ok(())
    }

    /// Generate proof based on request
    pub fn generate_proof(&self, request: ProofRequest) -> Result<ProofResponse, ZKPError> {
        let start = Instant::now();

        let proof = match request.proof_type {
            ProofType::ModelExecution => {
                // Parse circuit data for model execution
                let circuit_data: super::types::ModelExecutionCircuit =
                    bincode::deserialize(&request.circuit_data)
                        .map_err(|_| ZKPError::InvalidCircuit)?;

                self.prover.prove_model_execution(
                    circuit_data.model_hash,
                    circuit_data.input_hash,
                    circuit_data.output_hash,
                    circuit_data.computation_trace,
                )?
            }
            ProofType::GradientSubmission => {
                // Parse circuit data for gradient submission
                let circuit_data: super::types::GradientProofCircuit =
                    bincode::deserialize(&request.circuit_data)
                        .map_err(|_| ZKPError::InvalidCircuit)?;

                self.prover.prove_gradient_submission(
                    circuit_data.model_hash,
                    circuit_data.dataset_hash,
                    circuit_data.gradient_hash,
                    circuit_data.loss_value,
                    circuit_data.num_samples,
                )?
            }
            ProofType::StateTransition => {
                // Parse circuit data for state transition
                let circuit_data: super::circuits::StateTransitionCircuit =
                    bincode::deserialize(&request.circuit_data)
                        .map_err(|_| ZKPError::InvalidCircuit)?;

                self.prover.prove_state_transition(
                    circuit_data.old_state_root,
                    circuit_data.new_state_root,
                    circuit_data.transaction_hash,
                )?
            }
            ProofType::DataIntegrity => {
                // Parse circuit data for data integrity
                let circuit_data: super::circuits::DataIntegrityCircuit =
                    bincode::deserialize(&request.circuit_data)
                        .map_err(|_| ZKPError::InvalidCircuit)?;

                self.prover.prove_data_integrity(
                    circuit_data.data_hash,
                    circuit_data.merkle_path,
                    circuit_data.merkle_root,
                    circuit_data.leaf_index,
                )?
            }
        };

        let generation_time_ms = start.elapsed().as_millis() as u64;

        Ok(ProofResponse {
            proof,
            proof_type: request.proof_type,
            generation_time_ms,
        })
    }

    /// Verify a proof
    pub fn verify_proof(
        &self,
        proof_type: ProofType,
        proof: &SerializableProof,
    ) -> Result<bool, ZKPError> {
        let result = self.verifier.verify(proof_type, proof)?;
        Ok(result.is_valid)
    }

    /// Generate proof for tensor computation
    pub fn prove_tensor_computation(
        &self,
        operation: &str,
        input_tensors: Vec<Vec<u8>>,
        output_tensor: Vec<u8>,
    ) -> Result<SerializableProof, ZKPError> {
        use sha3::{Digest, Sha3_256};

        // Hash inputs and output
        let mut hasher = Sha3_256::new();
        for input in &input_tensors {
            hasher.update(input);
        }
        let input_hash = hasher.finalize_reset().to_vec();

        hasher.update(&output_tensor);
        let output_hash = hasher.finalize_reset().to_vec();

        // Create computation trace
        let computation_trace = vec![super::types::ComputationStep {
            operation: operation.to_string(),
            input_values: vec![input_tensors.len() as f64],
            output_value: 1.0,
        }];

        // Model hash (for tensor operations, we use operation type as identifier)
        hasher.update(operation.as_bytes());
        let model_hash = hasher.finalize().to_vec();

        self.prover
            .prove_model_execution(model_hash, input_hash, output_hash, computation_trace)
    }

    /// Verify tensor computation proof
    pub fn verify_tensor_computation(
        &self,
        proof: &SerializableProof,
        operation: &str,
        input_tensors: Vec<Vec<u8>>,
        output_tensor: Vec<u8>,
    ) -> Result<bool, ZKPError> {
        use sha3::{Digest, Sha3_256};

        // Compute expected hashes
        let mut hasher = Sha3_256::new();

        hasher.update(operation.as_bytes());
        let model_hash = hasher.finalize_reset().to_vec();

        for input in &input_tensors {
            hasher.update(input);
        }
        let input_hash = hasher.finalize_reset().to_vec();

        hasher.update(&output_tensor);
        let output_hash = hasher.finalize().to_vec();

        self.verifier
            .verify_model_execution(proof, &model_hash, &input_hash, &output_hash)
    }

    /// Generate proof for model training
    pub fn prove_training_round(
        &self,
        model_id: &[u8],
        dataset_id: &[u8],
        gradients: Vec<u8>,
        loss: f64,
        batch_size: u64,
    ) -> Result<SerializableProof, ZKPError> {
        use sha3::{Digest, Sha3_256};

        let mut hasher = Sha3_256::new();
        hasher.update(&gradients);
        let gradient_hash = hasher.finalize().to_vec();

        self.prover.prove_gradient_submission(
            model_id.to_vec(),
            dataset_id.to_vec(),
            gradient_hash,
            loss,
            batch_size,
        )
    }

    /// Batch generate proofs
    pub fn batch_generate_proofs(
        &self,
        requests: Vec<ProofRequest>,
    ) -> Result<Vec<ProofResponse>, ZKPError> {
        let mut responses = Vec::new();

        for request in requests {
            let response = self.generate_proof(request)?;
            responses.push(response);
        }

        Ok(responses)
    }

    /// Get proving time estimate
    pub fn estimate_proving_time(&self, proof_type: ProofType) -> u64 {
        // Estimates in milliseconds
        match proof_type {
            ProofType::ModelExecution => 500,
            ProofType::GradientSubmission => 750,
            ProofType::StateTransition => 300,
            ProofType::DataIntegrity => 400,
        }
    }
}
