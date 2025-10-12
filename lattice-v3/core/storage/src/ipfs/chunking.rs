// lattice-v3/core/storage/src/ipfs/chunking.rs

//! Model chunking for efficient IPFS storage
//! Optimized for Apple Silicon and Metal GPU compatibility

use anyhow::Result;
use blake3;
use serde::{Deserialize, Serialize};

use super::Cid;
use super::ModelMetadata;

/// Chunk size optimized for M-series unified memory architecture
const DEFAULT_CHUNK_SIZE: usize = 128 * 1024 * 1024; // 128MB chunks for efficient Metal memory mapping

/// A chunk of model data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chunk {
    pub index: usize,
    pub data: Vec<u8>,
    pub hash: [u8; 32],
    pub size: usize,
}

/// Manifest for chunked models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkManifest {
    pub chunks: Vec<Cid>,
    pub total_size: u64,
    pub chunk_size: usize,
    pub metadata: ModelMetadata,
    pub metal_optimized: bool,
    pub unified_memory_compatible: bool,
}

/// Chunk a model for storage
pub fn chunk_model(model_data: &[u8], chunk_size: usize) -> Result<Vec<Chunk>> {
    let chunks: Vec<Chunk> = model_data
        .chunks(chunk_size)
        .enumerate()
        .map(|(index, data)| {
            let hash = blake3::hash(data);
            Chunk {
                index,
                data: data.to_vec(),
                hash: *hash.as_bytes(),
                size: data.len(),
            }
        })
        .collect();

    Ok(chunks)
}

/// Reconstruct model from chunks
pub fn reconstruct_model(mut chunks: Vec<Chunk>) -> Vec<u8> {
    // Sort by index to ensure correct order
    chunks.sort_by_key(|c| c.index);

    // Calculate total size
    let total_size: usize = chunks.iter().map(|c| c.size).sum();
    let mut result = Vec::with_capacity(total_size);

    // Concatenate chunks
    for chunk in chunks {
        result.extend(chunk.data);
    }

    result
}

/// Verify chunk integrity
pub fn verify_chunk(chunk: &Chunk) -> bool {
    let computed_hash = blake3::hash(&chunk.data);
    computed_hash.as_bytes() == &chunk.hash
}

/// Metal-optimized chunking for Apple Silicon
pub fn chunk_for_metal(model_data: &[u8]) -> Result<Vec<Chunk>> {
    // Use chunk size that aligns with Metal's buffer requirements
    // M1/M2/M3 GPUs prefer 16KB alignment for optimal performance
    let metal_aligned_chunk_size = align_to_metal_buffer_size(DEFAULT_CHUNK_SIZE);
    chunk_model(model_data, metal_aligned_chunk_size)
}

/// Align chunk size to Metal GPU buffer requirements
fn align_to_metal_buffer_size(size: usize) -> usize {
    const METAL_BUFFER_ALIGNMENT: usize = 16384; // 16KB alignment for Metal
    ((size + METAL_BUFFER_ALIGNMENT - 1) / METAL_BUFFER_ALIGNMENT) * METAL_BUFFER_ALIGNMENT
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_and_reconstruct() {
        let original_data = vec![1u8; 1000];
        let chunks = chunk_model(&original_data, 256).unwrap();

        assert_eq!(chunks.len(), 4); // 1000 / 256 = 3.9, so 4 chunks

        let reconstructed = reconstruct_model(chunks);
        assert_eq!(original_data, reconstructed);
    }

    #[test]
    fn test_chunk_verification() {
        let data = vec![42u8; 100];
        let chunks = chunk_model(&data, 50).unwrap();

        for chunk in &chunks {
            assert!(verify_chunk(chunk));
        }
    }

    #[test]
    fn test_metal_alignment() {
        let size = align_to_metal_buffer_size(100000);
        assert_eq!(size % 16384, 0); // Should be aligned to 16KB
    }
}
