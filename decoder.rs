//! Reed-Solomon decoder and repair implementation.

use crate::errors::AppError;
use super::{GF256, Matrix};

/// Decode and reconstruct original data from surviving shards.
///
/// - `shards`: A slice of length `data_shards + parity_shards`. `None` indicates a missing shard.
/// - `data_shards`: Count of original data shards.
/// - `parity_shards`: Count of parity shards.
/// - `original_len`: Original size of the file in bytes (used to remove padding).
pub fn decode(
    shards: &[Option<Vec<u8>>],
    data_shards: usize,
    parity_shards: usize,
    original_len: usize,
) -> Result<Vec<u8>, AppError> {
    let total_shards = data_shards + parity_shards;
    if shards.len() != total_shards {
        return Err(AppError::ErasureDecode(format!(
            "invalid shard list length: expected {}, got {}",
            total_shards,
            shards.len()
        )));
    }

    // 1. Identify surviving shards and check if we have enough
    let mut surviving_indices = Vec::new();
    let mut surviving_shards = Vec::new();
    let mut shard_size = 0;

    for (i, shard) in shards.iter().enumerate() {
        if let Some(data) = shard {
            if shard_size == 0 {
                shard_size = data.len();
            } else if data.len() != shard_size {
                return Err(AppError::ErasureDecode("mismatched shard sizes".into()));
            }
            surviving_indices.push(i);
            surviving_shards.push(data);
        }
    }

    if surviving_indices.len() < data_shards {
        return Err(AppError::ErasureDecode(format!(
            "not enough surviving shards to reconstruct: need {}, got {}",
            data_shards,
            surviving_indices.len()
        )));
    }

    // We only need the first data_shards surviving shards to reconstruct
    let surviving_indices = &surviving_indices[..data_shards];
    let surviving_shards = &surviving_shards[..data_shards];

    let gf = GF256::new();

    // 2. Generate original systematic Cauchy matrix
    let gen_matrix = Matrix::systematic_cauchy(total_shards, data_shards, &gf);

    // 3. Build submatrix of rows corresponding to surviving shards
    let mut submatrix = Matrix::new(data_shards, data_shards);
    for r in 0..data_shards {
        let original_row = surviving_indices[r];
        for c in 0..data_shards {
            submatrix.set(r, c, gen_matrix.get(original_row, c));
        }
    }

    // 4. Invert submatrix
    let inv_matrix = submatrix
        .invert(&gf)
        .ok_or_else(|| AppError::ErasureDecode("reconstruction matrix is singular".into()))?;

    // 5. Multiply inverse matrix by surviving shards to recover original data shards
    let mut recovered_data = Vec::with_capacity(data_shards * shard_size);
    for d in 0..data_shards {
        let mut recovered_shard = vec![0u8; shard_size];
        for c in 0..shard_size {
            let mut sum = 0u8;
            for s in 0..data_shards {
                let inv_val = inv_matrix.get(d, s);
                let shard_val = surviving_shards[s][c];
                sum = gf.add(sum, gf.mul(inv_val, shard_val));
            }
            recovered_shard[c] = sum;
        }
        recovered_data.extend_from_slice(&recovered_shard);
    }

    // Truncate to original length to strip padding
    if original_len > recovered_data.len() {
        return Err(AppError::ErasureDecode(format!(
            "original_len {} exceeds reconstructed size {}",
            original_len,
            recovered_data.len()
        )));
    }
    recovered_data.truncate(original_len);

    Ok(recovered_data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::erasure::encoder::encode;

    #[test]
    fn test_encode_decode_roundtrip() {
        let original = b"this is some super secret storage system data!";
        let data_shards = 4;
        let parity_shards = 2;

        let encoded = encode(original, data_shards, parity_shards).unwrap();

        // Simulate losing some shards (e.g. shard 0 and shard 4)
        let sub_shards = vec![
            None, // lost
            Some(encoded[1].clone()),
            Some(encoded[2].clone()),
            Some(encoded[3].clone()),
            None, // lost
            Some(encoded[5].clone()),
        ];

        let decoded = decode(&sub_shards, data_shards, parity_shards, original.len()).unwrap();
        assert_eq!(decoded, original);
    }
}
