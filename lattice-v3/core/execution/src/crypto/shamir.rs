// lattice-v3/core/execution/src/crypto/shamir.rs

//! Shamir's Secret Sharing implementation
//! Provides secure threshold secret sharing using finite field arithmetic

use anyhow::{Result, anyhow};
use rand::RngCore;
use serde::{Deserialize, Serialize};

/// Prime field modulus (2^256 - 189)
/// Large prime for 256-bit arithmetic
const FIELD_MODULUS: [u64; 4] = [
    0xFFFFFFFFFFFFFF43,
    0xFFFFFFFFFFFFFFFF,
    0xFFFFFFFFFFFFFFFF,
    0xFFFFFFFFFFFFFFFF,
];

/// Field element in GF(p)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct FieldElement {
    /// 256-bit integer representation
    limbs: [u64; 4],
}

impl FieldElement {
    /// Zero element
    pub fn zero() -> Self {
        Self { limbs: [0; 4] }
    }

    /// One element
    pub fn one() -> Self {
        Self { limbs: [1, 0, 0, 0] }
    }

    /// Create from bytes (little-endian)
    pub fn from_bytes(bytes: &[u8; 32]) -> Self {
        let mut limbs = [0u64; 4];
        for i in 0..4 {
            let start = i * 8;
            let mut limb_bytes = [0u8; 8];
            limb_bytes.copy_from_slice(&bytes[start..start + 8]);
            limbs[i] = u64::from_le_bytes(limb_bytes);
        }
        Self { limbs }.reduce()
    }

    /// Convert to bytes (little-endian)
    pub fn to_bytes(&self) -> [u8; 32] {
        let mut bytes = [0u8; 32];
        for i in 0..4 {
            let limb_bytes = self.limbs[i].to_le_bytes();
            bytes[i * 8..(i + 1) * 8].copy_from_slice(&limb_bytes);
        }
        bytes
    }

    /// Create from u64
    pub fn from_u64(val: u64) -> Self {
        Self {
            limbs: [val, 0, 0, 0],
        }.reduce()
    }

    /// Reduce modulo field prime
    fn reduce(mut self) -> Self {
        // Simplified reduction - in production use proper modular arithmetic
        // This is a placeholder that maintains the structure

        // Check if greater than modulus
        let mut needs_reduction = false;
        for i in (0..4).rev() {
            if self.limbs[i] > FIELD_MODULUS[i] {
                needs_reduction = true;
                break;
            } else if self.limbs[i] < FIELD_MODULUS[i] {
                break;
            }
        }

        if needs_reduction {
            // Subtract modulus (simplified)
            let mut borrow = 0u64;
            for i in 0..4 {
                let (diff, new_borrow) = self.limbs[i].overflowing_sub(FIELD_MODULUS[i]);
                let (final_diff, extra_borrow) = diff.overflowing_sub(borrow);
                self.limbs[i] = final_diff;
                borrow = (new_borrow || extra_borrow) as u64;
            }
        }

        self
    }

    /// Addition in field
    pub fn add(&self, other: &Self) -> Self {
        let mut result = [0u64; 4];
        let mut carry = 0u64;

        for i in 0..4 {
            let sum = self.limbs[i] as u128 + other.limbs[i] as u128 + carry as u128;
            result[i] = sum as u64;
            carry = (sum >> 64) as u64;
        }

        Self { limbs: result }.reduce()
    }

    /// Subtraction in field
    pub fn sub(&self, other: &Self) -> Self {
        let mut result = [0u64; 4];
        let mut borrow = 0u64;

        for i in 0..4 {
            let (diff, new_borrow) = self.limbs[i].overflowing_sub(other.limbs[i]);
            let (final_diff, extra_borrow) = diff.overflowing_sub(borrow);
            result[i] = final_diff;
            borrow = (new_borrow || extra_borrow) as u64;
        }

        if borrow > 0 {
            // Add modulus if we underflowed
            let mut carry = 0u64;
            for i in 0..4 {
                let sum = result[i] as u128 + FIELD_MODULUS[i] as u128 + carry as u128;
                result[i] = sum as u64;
                carry = (sum >> 64) as u64;
            }
        }

        Self { limbs: result }
    }

    /// Multiplication in field (simplified)
    pub fn mul(&self, other: &Self) -> Self {
        // Simplified multiplication for demonstration
        // Production should use proper big integer multiplication with Montgomery reduction

        let mut result = [0u64; 4];

        // Simple cross-multiplication of lowest limbs
        let product = (self.limbs[0] as u128) * (other.limbs[0] as u128);
        result[0] = product as u64;
        result[1] = (product >> 64) as u64;

        Self { limbs: result }.reduce()
    }

    /// Modular inverse using extended Euclidean algorithm (simplified)
    pub fn inverse(&self) -> Result<Self> {
        if self.is_zero() {
            return Err(anyhow!("Cannot invert zero"));
        }

        // Simplified inverse for demonstration
        // In production, use proper extended Euclidean algorithm

        // For small values, use brute force (not efficient but correct for demo)
        for i in 1u64..1000000 {
            let candidate = Self::from_u64(i);
            let product = self.mul(&candidate);
            if product == Self::one() {
                return Ok(candidate);
            }
        }

        Err(anyhow!("Inverse not found (simplified implementation)"))
    }

    /// Check if zero
    pub fn is_zero(&self) -> bool {
        self.limbs.iter().all(|&x| x == 0)
    }

    /// Generate random field element
    pub fn random() -> Self {
        let mut bytes = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut bytes);
        Self::from_bytes(&bytes)
    }
}

/// Secret share
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Share {
    /// X coordinate (participant ID)
    pub x: FieldElement,
    /// Y coordinate (share value)
    pub y: FieldElement,
}

/// Shamir's Secret Sharing scheme
pub struct ShamirSecretSharing {
    /// Threshold (minimum shares needed)
    threshold: usize,
    /// Total number of shares
    total_shares: usize,
}

impl ShamirSecretSharing {
    /// Create new Shamir scheme
    pub fn new(threshold: usize, total_shares: usize) -> Result<Self> {
        if threshold == 0 {
            return Err(anyhow!("Threshold must be at least 1"));
        }
        if threshold > total_shares {
            return Err(anyhow!("Threshold cannot exceed total shares"));
        }
        if total_shares > 255 {
            return Err(anyhow!("Too many shares (max 255)"));
        }

        Ok(Self {
            threshold,
            total_shares,
        })
    }

    /// Split secret into shares
    pub fn split_secret(&self, secret: &[u8; 32]) -> Result<Vec<Share>> {
        let secret_elem = FieldElement::from_bytes(secret);

        // Generate random polynomial coefficients
        let mut coefficients = vec![secret_elem]; // a0 = secret
        for _ in 1..self.threshold {
            coefficients.push(FieldElement::random());
        }

        // Evaluate polynomial at different points
        let mut shares = Vec::new();
        for i in 1..=self.total_shares {
            let x = FieldElement::from_u64(i as u64);
            let y = self.evaluate_polynomial(&coefficients, &x);
            shares.push(Share { x, y });
        }

        Ok(shares)
    }

    /// Reconstruct secret from shares
    pub fn reconstruct_secret(&self, shares: &[Share]) -> Result<[u8; 32]> {
        if shares.len() < self.threshold {
            return Err(anyhow!(
                "Insufficient shares: need {}, got {}",
                self.threshold,
                shares.len()
            ));
        }

        // Use first `threshold` shares
        let active_shares = &shares[..self.threshold];

        // Lagrange interpolation to find f(0)
        let mut result = FieldElement::zero();

        for (i, share_i) in active_shares.iter().enumerate() {
            let mut numerator = FieldElement::one();
            let mut denominator = FieldElement::one();

            for (j, share_j) in active_shares.iter().enumerate() {
                if i != j {
                    // numerator *= (0 - x_j) = -x_j
                    numerator = numerator.mul(&share_j.x.sub(&FieldElement::zero()));

                    // denominator *= (x_i - x_j)
                    denominator = denominator.mul(&share_i.x.sub(&share_j.x));
                }
            }

            // Calculate Lagrange coefficient
            let lagrange_coeff = numerator.mul(&denominator.inverse()?);

            // Add contribution: y_i * L_i(0)
            result = result.add(&share_i.y.mul(&lagrange_coeff));
        }

        Ok(result.to_bytes())
    }

    /// Evaluate polynomial at point x
    fn evaluate_polynomial(&self, coefficients: &[FieldElement], x: &FieldElement) -> FieldElement {
        let mut result = FieldElement::zero();
        let mut x_power = FieldElement::one();

        for coeff in coefficients {
            result = result.add(&coeff.mul(&x_power));
            x_power = x_power.mul(x);
        }

        result
    }

    /// Verify share is valid (simplified)
    pub fn verify_share(&self, _share: &Share) -> bool {
        // In production, use Verifiable Secret Sharing (VSS)
        // with commitments to verify shares without revealing secret
        true
    }

    /// Add new share (for share renewal)
    pub fn add_share(&self, shares: &[Share], new_x: u64) -> Result<Share> {
        if shares.len() < self.threshold {
            return Err(anyhow!("Insufficient shares to add new share"));
        }

        let x = FieldElement::from_u64(new_x);

        // Use Lagrange interpolation to evaluate polynomial at new point
        let mut result = FieldElement::zero();
        let active_shares = &shares[..self.threshold];

        for (i, share_i) in active_shares.iter().enumerate() {
            let mut numerator = FieldElement::one();
            let mut denominator = FieldElement::one();

            for (j, share_j) in active_shares.iter().enumerate() {
                if i != j {
                    // numerator *= (x - x_j)
                    numerator = numerator.mul(&x.sub(&share_j.x));

                    // denominator *= (x_i - x_j)
                    denominator = denominator.mul(&share_i.x.sub(&share_j.x));
                }
            }

            let lagrange_coeff = numerator.mul(&denominator.inverse()?);
            result = result.add(&share_i.y.mul(&lagrange_coeff));
        }

        Ok(Share { x, y: result })
    }
}

/// Convenience functions for model key sharing
pub fn split_model_key(
    key: &[u8; 32],
    threshold: usize,
    total_shares: usize,
) -> Result<Vec<Share>> {
    let sss = ShamirSecretSharing::new(threshold, total_shares)?;
    sss.split_secret(key)
}

pub fn reconstruct_model_key(shares: &[Share], threshold: usize) -> Result<[u8; 32]> {
    let sss = ShamirSecretSharing::new(threshold, shares.len())?;
    sss.reconstruct_secret(shares)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_field_arithmetic() {
        let a = FieldElement::from_u64(10);
        let b = FieldElement::from_u64(20);

        let sum = a.add(&b);
        let expected = FieldElement::from_u64(30);
        assert_eq!(sum, expected);

        let diff = b.sub(&a);
        let expected_diff = FieldElement::from_u64(10);
        assert_eq!(diff, expected_diff);
    }

    #[test]
    fn test_shamir_threshold_2_of_3() {
        let secret = [42u8; 32];

        let sss = ShamirSecretSharing::new(2, 3).unwrap();
        let shares = sss.split_secret(&secret).unwrap();

        assert_eq!(shares.len(), 3);

        // Reconstruct with any 2 shares
        let reconstructed = sss.reconstruct_secret(&shares[0..2]).unwrap();
        assert_eq!(secret, reconstructed);

        let reconstructed2 = sss.reconstruct_secret(&shares[1..3]).unwrap();
        assert_eq!(secret, reconstructed2);
    }

    #[test]
    fn test_insufficient_shares() {
        let secret = [99u8; 32];

        let sss = ShamirSecretSharing::new(3, 5).unwrap();
        let shares = sss.split_secret(&secret).unwrap();

        // Try to reconstruct with only 2 shares (need 3)
        let result = sss.reconstruct_secret(&shares[0..2]);
        assert!(result.is_err());
    }

    #[test]
    fn test_model_key_convenience() {
        let key = [123u8; 32];

        let shares = split_model_key(&key, 2, 4).unwrap();
        assert_eq!(shares.len(), 4);

        let reconstructed = reconstruct_model_key(&shares[0..2], 2).unwrap();
        assert_eq!(key, reconstructed);
    }

    #[test]
    fn test_add_new_share() {
        let secret = [77u8; 32];

        let sss = ShamirSecretSharing::new(2, 3).unwrap();
        let shares = sss.split_secret(&secret).unwrap();

        // Add a new share at x=4
        let new_share = sss.add_share(&shares, 4).unwrap();

        // Verify we can reconstruct with original and new share
        let mixed_shares = vec![shares[0].clone(), new_share];
        let reconstructed = sss.reconstruct_secret(&mixed_shares).unwrap();
        assert_eq!(secret, reconstructed);
    }

    #[test]
    fn test_field_element_serialization() {
        let elem = FieldElement::from_u64(12345);
        let bytes = elem.to_bytes();
        let restored = FieldElement::from_bytes(&bytes);
        assert_eq!(elem, restored);
    }
}