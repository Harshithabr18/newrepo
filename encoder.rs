//! Reed-Solomon encoder implementation.

use crate::errors::AppError;
use super::{GF256, Matrix};

/// Encode data into N data shards and M parity shards.
///
/// Returns a vector containing the data shards followed by the parity shards.
pub fn encode(
    data: &[u8],
    data_shards: usize,
    parity_shards: usize,
) -> Result<Vec<Vec<u8>>, AppError> {
    if data.is_empty() {
        return Err(AppError::ErasureEncode("cannot encode empty data".into()));
    }

    let gf = GF256::new();

    // 1. Calculate shard size (padded to be a multiple of data_shards)
    let shard_size = (data.len() + data_shards - 1) / data_shards;
    let padded_len = shard_size * data_shards;

    let mut padded_data = data.to_vec();
    padded_data.resize(padded_len, 0);

    // 2. Split data into N shards
    let mut shards = Vec::with_capacity(data_shards + parity_shards);
    for i in 0..data_shards {
        let start = i * shard_size;
        let end = start + shard_size;
        shards.push(padded_data[start..end].to_vec());
    }

    // Initialize parity shards to 0
    for _ in 0..parity_shards {
        shards.push(vec![0; shard_size]);
    }

    // 3. Generate systematic Cauchy matrix
    let total_shards = data_shards + parity_shards;
    let matrix = Matrix::systematic_cauchy(total_shards, data_shards, &gf);

    // 4. Compute parity shards by multiplying generator matrix rows by data shards
    for p in 0..parity_shards {
        let row_idx = data_shards + p;
        for c in 0..shard_size {
            let mut sum = 0u8;
            for d in 0..data_shards {
                let matrix_val = matrix.get(row_idx, d);
                let data_val = shards[d][c];
                sum = gf.add(sum, gf.mul(matrix_val, data_val));
            }
            shards[row_idx][c] = sum;
        }
    }

    Ok(shards)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_and_basic_properties() {
        let data = b"hello world, this is a test of reed solomon erasure coding!";
        let shards = encode(data, 4, 2).unwrap();

        assert_eq!(shards.len(), 6); // 4 data + 2 parity
        assert_eq!(shards[0].len(), shards[4].len()); // all shards same size
    }
}
