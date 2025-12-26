# Sprint 05: Precompile Completion

**Status**: Blocked (waiting for Sprint 04)
**Priority**: P2 Medium
**Duration**: 2-3 days
**Depends On**: Sprint 04

---

## Problem Statement

Several EVM precompiles are stub implementations that return dummy data. This causes certain contracts to fail or produce incorrect results.

### Current State (Stubs)
- `0x03` RIPEMD160 - Returns dummy output
- `0x05` MODEXP - Returns zero
- `0x06` ECADD - Returns 64 zero bytes
- `0x07` ECMUL - Returns 64 zero bytes
- `0x08` ECPAIRING - Returns 1 always
- `0x09` BLAKE2F - Returns 64 zero bytes

### Target State
- RIPEMD160 fully implemented
- MODEXP fully implemented
- EC operations fully implemented (for BLS signatures)

---

## Work Breakdown

### Task 1: Implement RIPEMD160

**File**: `core/execution/src/precompiles/mod.rs`

```rust
use ripemd::{Ripemd160, Digest};

fn ripemd160(&self, input: &[u8], gas_limit: u64) -> Result<PrecompileResult, PrecompileError> {
    // Gas cost: 600 + 120 * ceil(len/32)
    let gas_cost = 600 + 120 * ((input.len() as u64 + 31) / 32);

    if gas_cost > gas_limit {
        return Err(PrecompileError::OutOfGas);
    }

    // Compute RIPEMD-160
    let mut hasher = Ripemd160::new();
    hasher.update(input);
    let result = hasher.finalize();

    // Return 32-byte output (12 zero bytes + 20-byte hash)
    let mut output = [0u8; 32];
    output[12..].copy_from_slice(&result);

    Ok(PrecompileResult {
        output: output.to_vec(),
        gas_used: gas_cost,
    })
}
```

Add to Cargo.toml:
```toml
ripemd = "0.1"
```

**Acceptance Criteria**:
- [ ] Returns correct RIPEMD-160 hash
- [ ] Gas cost calculated correctly
- [ ] Passes standard test vectors

---

### Task 2: Implement MODEXP

**File**: `core/execution/src/precompiles/mod.rs`

```rust
use num_bigint::BigUint;

fn modexp(&self, input: &[u8], gas_limit: u64) -> Result<PrecompileResult, PrecompileError> {
    // Parse input according to EIP-198
    // [0..32]   = base length
    // [32..64]  = exp length
    // [64..96]  = mod length
    // [96..]    = base || exp || mod

    if input.len() < 96 {
        let mut padded = vec![0u8; 96];
        padded[..input.len()].copy_from_slice(input);
        return self.modexp_inner(&padded, gas_limit);
    }

    self.modexp_inner(input, gas_limit)
}

fn modexp_inner(&self, input: &[u8], gas_limit: u64) -> Result<PrecompileResult, PrecompileError> {
    let base_len = BigUint::from_bytes_be(&input[0..32]).to_u64_digits().first().copied().unwrap_or(0) as usize;
    let exp_len = BigUint::from_bytes_be(&input[32..64]).to_u64_digits().first().copied().unwrap_or(0) as usize;
    let mod_len = BigUint::from_bytes_be(&input[64..96]).to_u64_digits().first().copied().unwrap_or(0) as usize;

    // Calculate gas (EIP-2565)
    let gas_cost = self.modexp_gas_cost(base_len, exp_len, mod_len, input);
    if gas_cost > gas_limit {
        return Err(PrecompileError::OutOfGas);
    }

    // Extract base, exp, mod
    let data = &input[96..];
    let base_bytes = get_padded_slice(data, 0, base_len);
    let exp_bytes = get_padded_slice(data, base_len, exp_len);
    let mod_bytes = get_padded_slice(data, base_len + exp_len, mod_len);

    let base = BigUint::from_bytes_be(&base_bytes);
    let exp = BigUint::from_bytes_be(&exp_bytes);
    let modulus = BigUint::from_bytes_be(&mod_bytes);

    // Handle edge cases
    if modulus.is_zero() {
        return Ok(PrecompileResult {
            output: vec![0u8; mod_len],
            gas_used: gas_cost,
        });
    }

    // Compute base^exp mod modulus
    let result = base.modpow(&exp, &modulus);

    // Convert to fixed-size output
    let mut output = vec![0u8; mod_len];
    let result_bytes = result.to_bytes_be();
    let offset = mod_len.saturating_sub(result_bytes.len());
    output[offset..].copy_from_slice(&result_bytes);

    Ok(PrecompileResult {
        output,
        gas_used: gas_cost,
    })
}
```

Add to Cargo.toml:
```toml
num-bigint = "0.4"
```

**Acceptance Criteria**:
- [ ] Computes modular exponentiation correctly
- [ ] Gas cost per EIP-2565
- [ ] Handles edge cases (zero modulus, large numbers)

---

### Task 3: Implement EC Operations (BN256)

**File**: `core/execution/src/precompiles/mod.rs`

```rust
use ark_bn254::{G1Projective, G2Projective, Bn254, Fr};
use ark_ec::pairing::Pairing;
use ark_ff::Field;

fn bn256_add(&self, input: &[u8], gas_limit: u64) -> Result<PrecompileResult, PrecompileError> {
    const GAS_COST: u64 = 150; // EIP-1108
    if gas_limit < GAS_COST {
        return Err(PrecompileError::OutOfGas);
    }

    // Parse two G1 points (64 bytes each)
    let p1 = parse_g1_point(&input[0..64])?;
    let p2 = parse_g1_point(&input[64..128])?;

    // Add points
    let result = p1 + p2;

    // Serialize result
    let output = serialize_g1_point(&result);

    Ok(PrecompileResult {
        output,
        gas_used: GAS_COST,
    })
}

fn bn256_mul(&self, input: &[u8], gas_limit: u64) -> Result<PrecompileResult, PrecompileError> {
    const GAS_COST: u64 = 6000; // EIP-1108
    if gas_limit < GAS_COST {
        return Err(PrecompileError::OutOfGas);
    }

    // Parse G1 point (64 bytes) and scalar (32 bytes)
    let p = parse_g1_point(&input[0..64])?;
    let s = parse_scalar(&input[64..96])?;

    // Scalar multiplication
    let result = p * s;

    let output = serialize_g1_point(&result);

    Ok(PrecompileResult {
        output,
        gas_used: GAS_COST,
    })
}

fn bn256_pairing(&self, input: &[u8], gas_limit: u64) -> Result<PrecompileResult, PrecompileError> {
    // Gas: 34000 * k + 45000
    let k = input.len() / 192;
    let gas_cost = 34000 * k as u64 + 45000;

    if gas_limit < gas_cost {
        return Err(PrecompileError::OutOfGas);
    }

    if input.len() % 192 != 0 {
        return Err(PrecompileError::InvalidInput);
    }

    // Parse pairs of (G1, G2) points
    let mut pairs = Vec::new();
    for i in 0..k {
        let offset = i * 192;
        let g1 = parse_g1_point(&input[offset..offset+64])?;
        let g2 = parse_g2_point(&input[offset+64..offset+192])?;
        pairs.push((g1, g2));
    }

    // Compute pairing check: e(a1, b1) * e(a2, b2) * ... == 1
    let result = Bn254::multi_pairing(
        pairs.iter().map(|(g1, _)| g1),
        pairs.iter().map(|(_, g2)| g2),
    );

    // Return 1 if pairing check passes, 0 otherwise
    let output = if result.0.is_one() {
        vec![0u8; 31, 1]
    } else {
        vec![0u8; 32]
    };

    Ok(PrecompileResult {
        output,
        gas_used: gas_cost,
    })
}
```

Add to Cargo.toml:
```toml
ark-bn254 = "0.4"
ark-ec = "0.4"
ark-ff = "0.4"
```

**Acceptance Criteria**:
- [ ] ECADD works for BN256 curve
- [ ] ECMUL works for BN256 curve
- [ ] ECPAIRING verifies pairings correctly

---

### Task 4: Implement BLAKE2F

**File**: `core/execution/src/precompiles/mod.rs`

```rust
use blake2::Blake2bVar;

fn blake2f(&self, input: &[u8], gas_limit: u64) -> Result<PrecompileResult, PrecompileError> {
    // Input: rounds (4) || h (64) || m (128) || t (16) || f (1) = 213 bytes
    if input.len() != 213 {
        return Err(PrecompileError::InvalidInput);
    }

    let rounds = u32::from_be_bytes([input[0], input[1], input[2], input[3]]);
    let gas_cost = rounds as u64;

    if gas_cost > gas_limit {
        return Err(PrecompileError::OutOfGas);
    }

    // Parse state vector h (8 x 8 bytes)
    let mut h = [0u64; 8];
    for i in 0..8 {
        h[i] = u64::from_le_bytes(input[4 + i*8..4 + (i+1)*8].try_into().unwrap());
    }

    // Parse message block m (16 x 8 bytes)
    let mut m = [0u64; 16];
    for i in 0..16 {
        m[i] = u64::from_le_bytes(input[68 + i*8..68 + (i+1)*8].try_into().unwrap());
    }

    // Parse offset counters t (2 x 8 bytes)
    let t = [
        u64::from_le_bytes(input[196..204].try_into().unwrap()),
        u64::from_le_bytes(input[204..212].try_into().unwrap()),
    ];

    // Final block flag
    let f = input[212] != 0;

    // Run BLAKE2b compression
    blake2b_compress(&mut h, &m, &t, f, rounds);

    // Output: new state h (64 bytes)
    let mut output = vec![0u8; 64];
    for i in 0..8 {
        output[i*8..(i+1)*8].copy_from_slice(&h[i].to_le_bytes());
    }

    Ok(PrecompileResult {
        output,
        gas_used: gas_cost,
    })
}
```

**Acceptance Criteria**:
- [ ] BLAKE2F compression works correctly
- [ ] Gas cost = rounds

---

### Task 5: Add Test Coverage

**File**: `core/execution/src/precompiles/mod.rs` (tests section)

Add tests for each precompile using standard test vectors from Ethereum.

**Acceptance Criteria**:
- [ ] All precompiles have test coverage
- [ ] Test vectors from Ethereum pass

---

## Testing Checklist

### Standard Test Vectors
```bash
cargo test -p citrate-execution precompiles
```

| Precompile | Test Vectors | Status |
|------------|--------------|--------|
| RIPEMD160 | EIP-152 | |
| MODEXP | EIP-198/EIP-2565 | |
| ECADD | EIP-196 | |
| ECMUL | EIP-196 | |
| ECPAIRING | EIP-197 | |
| BLAKE2F | EIP-152 | |

---

## Files Modified

| File | Changes |
|------|---------|
| `core/execution/Cargo.toml` | Add crypto dependencies |
| `core/execution/src/precompiles/mod.rs` | Implement all precompiles |

---

## Definition of Done

- [ ] All precompiles fully implemented
- [ ] All test vectors pass
- [ ] No performance regressions
- [ ] Git commit: "Sprint 05: Precompile Completion"
