use super::types::{SerializableProof, VerificationResult, ProofType, ZKPError, VerifyingKey};
use ark_groth16::{Groth16, PreparedVerifyingKey, prepare_verifying_key};
use ark_bls12_381::{Bls12_381, Fr};
use ark_snark::SNARK;
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;

/// Proof verifier for ZKP operations
pub struct Verifier {
    verifying_keys: Arc<RwLock<HashMap<ProofType, VerifyingKey>>>,
    prepared_vks: Arc<RwLock<HashMap<ProofType, PreparedVerifyingKey<Bls12_381>>>>,
}

impl Verifier {
    pub fn new() -> Self {
        Self {
            verifying_keys: Arc::new(RwLock::new(HashMap::new())),
            prepared_vks: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Add a verifying key for a proof type
    pub fn add_verifying_key(&self, proof_type: ProofType, vk: VerifyingKey) {
        let prepared_vk = prepare_verifying_key(&vk);
        self.verifying_keys.write().insert(proof_type, vk);
        self.prepared_vks.write().insert(proof_type, prepared_vk);
    }

    /// Verify a proof
    pub fn verify(
        &self,
        proof_type: ProofType,
        proof: &SerializableProof,
    ) -> Result<VerificationResult, ZKPError> {
        let prepared_vk = self.prepared_vks.read()
            .get(&proof_type)
            .ok_or_else(|| ZKPError::KeyNotFound(format!("{:?}", proof_type)))?
            .clone();

        let proof_obj = proof.to_proof()?;
        
        // Convert public inputs from strings to field elements
        let public_inputs = self.parse_public_inputs(&proof.public_inputs)?;

        let is_valid = Groth16::<Bls12_381>::verify_with_processed_vk(
            &prepared_vk,
            &public_inputs,
            &proof_obj,
        ).map_err(|e| ZKPError::VerificationError(e.to_string()))?;

        Ok(VerificationResult {
            is_valid,
            public_inputs: proof.public_inputs.clone(),
            error_message: if !is_valid {
                Some("Proof verification failed".to_string())
            } else {
                None
            },
        })
    }

    /// Batch verify multiple proofs of the same type
    pub fn batch_verify(
        &self,
        proof_type: ProofType,
        proofs: &[SerializableProof],
    ) -> Result<Vec<VerificationResult>, ZKPError> {
        let prepared_vk = self.prepared_vks.read()
            .get(&proof_type)
            .ok_or_else(|| ZKPError::KeyNotFound(format!("{:?}", proof_type)))?
            .clone();

        let mut results = Vec::new();

        for proof in proofs {
            let proof_obj = proof.to_proof()?;
            let public_inputs = self.parse_public_inputs(&proof.public_inputs)?;

            let is_valid = Groth16::<Bls12_381>::verify_with_processed_vk(
                &prepared_vk,
                &public_inputs,
                &proof_obj,
            ).unwrap_or(false);

            results.push(VerificationResult {
                is_valid,
                public_inputs: proof.public_inputs.clone(),
                error_message: if !is_valid {
                    Some("Proof verification failed".to_string())
                } else {
                    None
                },
            });
        }

        Ok(results)
    }

    /// Verify model execution proof
    pub fn verify_model_execution(
        &self,
        proof: &SerializableProof,
        expected_model_hash: &[u8],
        expected_input_hash: &[u8],
        expected_output_hash: &[u8],
    ) -> Result<bool, ZKPError> {
        // Verify the proof
        let result = self.verify(ProofType::ModelExecution, proof)?;
        
        if !result.is_valid {
            return Ok(false);
        }

        // Verify public inputs match expected values
        if proof.public_inputs.len() < 3 {
            return Err(ZKPError::InvalidPublicInputs);
        }

        let model_hash_str = &proof.public_inputs[0];
        let input_hash_str = &proof.public_inputs[1];
        let output_hash_str = &proof.public_inputs[2];

        // Verify hashes match
        if hex::encode(expected_model_hash) != *model_hash_str ||
           hex::encode(expected_input_hash) != *input_hash_str ||
           hex::encode(expected_output_hash) != *output_hash_str {
            return Ok(false);
        }

        Ok(true)
    }

    /// Verify gradient submission proof
    pub fn verify_gradient_submission(
        &self,
        proof: &SerializableProof,
        expected_model_hash: &[u8],
        expected_dataset_hash: &[u8],
        min_loss: Option<f64>,
        min_samples: Option<u64>,
    ) -> Result<bool, ZKPError> {
        // Verify the proof
        let result = self.verify(ProofType::GradientSubmission, proof)?;
        
        if !result.is_valid {
            return Ok(false);
        }

        // Verify public inputs
        if proof.public_inputs.len() < 5 {
            return Err(ZKPError::InvalidPublicInputs);
        }

        let model_hash_str = &proof.public_inputs[0];
        let dataset_hash_str = &proof.public_inputs[1];
        let loss_str = &proof.public_inputs[3];
        let samples_str = &proof.public_inputs[4];

        // Verify hashes match
        if hex::encode(expected_model_hash) != *model_hash_str ||
           hex::encode(expected_dataset_hash) != *dataset_hash_str {
            return Ok(false);
        }

        // Verify loss threshold if provided
        if let Some(min) = min_loss {
            let loss: f64 = loss_str.parse()
                .map_err(|_| ZKPError::InvalidPublicInputs)?;
            if loss > min {
                return Ok(false);
            }
        }

        // Verify sample count if provided
        if let Some(min) = min_samples {
            let samples: u64 = samples_str.parse()
                .map_err(|_| ZKPError::InvalidPublicInputs)?;
            if samples < min {
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// Parse public inputs from strings to field elements
    fn parse_public_inputs(&self, inputs: &[String]) -> Result<Vec<Fr>, ZKPError> {
        let mut field_elements = Vec::new();
        
        for input in inputs {
            // Try to parse as hex first
            if input.starts_with("0x") || input.len() == 64 {
                let bytes = hex::decode(input.trim_start_matches("0x"))
                    .map_err(|_| ZKPError::InvalidPublicInputs)?;
                
                // Convert bytes to field element
                // This is simplified - real implementation would be more careful
                let mut value = Fr::from(0u64);
                for byte in bytes.iter().take(8) {
                    value = value * Fr::from(256u64) + Fr::from(*byte as u64);
                }
                field_elements.push(value);
            } else {
                // Try to parse as number
                let num: u64 = input.parse()
                    .unwrap_or(0);
                field_elements.push(Fr::from(num));
            }
        }
        
        Ok(field_elements)
    }

    /// Verify aggregated proofs (for scalability)
    pub fn verify_aggregated(
        &self,
        proof_type: ProofType,
        aggregated_proof: &SerializableProof,
        _individual_public_inputs: Vec<Vec<String>>,
    ) -> Result<bool, ZKPError> {
        // In a real implementation, this would verify an aggregated proof
        // that proves multiple statements at once
        
        let result = self.verify(proof_type, aggregated_proof)?;
        
        if !result.is_valid {
            return Ok(false);
        }

        // Additional verification logic for aggregated proofs would go here
        
        Ok(true)
    }
}

impl Default for Verifier {
    fn default() -> Self { Self::new() }
}
