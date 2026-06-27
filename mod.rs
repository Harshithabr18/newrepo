//! Custom Reed-Solomon Erasure Coding implementation.
//!
//! Provides GF(256) arithmetic, matrix operations, and encoding/decoding
//! for data durability.

pub mod decoder;
pub mod encoder;

pub use decoder::decode;
pub use encoder::encode;

/// GF(256) math tables and operations.
pub struct GF256 {
    pub exp: [u8; 512],
    pub log: [u8; 256],
}

impl GF256 {
    /// Initialize the GF(256) tables using the generator polynomial 0x11d.
    pub fn new() -> Self {
        let mut exp = [0u8; 512];
        let mut log = [0u8; 256];
        let mut val: u16 = 1;
        for i in 0..255 {
            exp[i] = val as u8;
            log[val as usize] = i as u8;
            val <<= 1;
            if val & 0x100 != 0 {
                val ^= 0x11d;
            }
        }
        for i in 255..512 {
            exp[i] = exp[i - 255];
        }
        // log[0] is mathematically undefined, but we leave it as 0.
        Self { exp, log }
    }

    /// Add two GF(256) elements (bitwise XOR).
    #[inline(always)]
    pub fn add(&self, a: u8, b: u8) -> u8 {
        a ^ b
    }

    /// Subtract two GF(256) elements (bitwise XOR).
    #[inline(always)]
    pub fn sub(&self, a: u8, b: u8) -> u8 {
        a ^ b
    }

    /// Multiply two GF(256) elements.
    #[inline(always)]
    pub fn mul(&self, a: u8, b: u8) -> u8 {
        if a == 0 || b == 0 {
            0
        } else {
            let idx = (self.log[a as usize] as usize) + (self.log[b as usize] as usize);
            self.exp[idx]
        }
    }

    /// Divide two GF(256) elements.
    #[inline(always)]
    pub fn div(&self, a: u8, b: u8) -> u8 {
        assert!(b != 0, "division by zero in GF(256)");
        if a == 0 {
            0
        } else {
            let idx = (self.log[a as usize] as i32) - (self.log[b as usize] as i32) + 255;
            self.exp[idx as usize]
        }
    }
}

/// A simple matrix in GF(256).
#[derive(Clone)]
pub struct Matrix {
    pub rows: usize,
    pub cols: usize,
    pub data: Vec<u8>,
}

impl Matrix {
    pub fn new(rows: usize, cols: usize) -> Self {
        Self {
            rows,
            cols,
            data: vec![0; rows * cols],
        }
    }

    #[inline(always)]
    pub fn get(&self, r: usize, c: usize) -> u8 {
        self.data[r * self.cols + c]
    }

    #[inline(always)]
    pub fn set(&mut self, r: usize, c: usize, val: u8) {
        self.data[r * self.cols + c] = val;
    }

    /// Construct a systematic Cauchy matrix of size `rows x cols` (where `rows` is total_shards, `cols` is data_shards).
    /// Row 0..cols: Identity matrix
    /// Row cols..rows: Cauchy matrix C[i][j] = 1 / (x_i XOR y_j)
    /// where x_i = i, y_j = j
    pub fn systematic_cauchy(rows: usize, cols: usize, gf: &GF256) -> Self {
        let mut m = Self::new(rows, cols);
        // Identity part
        for r in 0..cols {
            m.set(r, r, 1);
        }
        // Cauchy part
        for r in cols..rows {
            for c in 0..cols {
                let xi = r as u8;
                let yj = c as u8;
                m.set(r, c, gf.div(1, gf.add(xi, yj)));
            }
        }
        m
    }

    /// Invert this matrix in-place using Gaussian elimination.
    pub fn invert(&self, gf: &GF256) -> Option<Self> {
        assert_eq!(self.rows, self.cols, "matrix must be square for inversion");
        let n = self.rows;
        let mut aug = Self::new(n, n * 2);
        
        // Setup augmented matrix [A | I]
        for r in 0..n {
            for c in 0..n {
                aug.set(r, c, self.get(r, c));
            }
            aug.set(r, n + r, 1);
        }

        // Perform Gaussian elimination
        for i in 0..n {
            // Find pivot
            let mut pivot_row = i;
            while pivot_row < n && aug.get(pivot_row, i) == 0 {
                pivot_row += 1;
            }
            if pivot_row == n {
                return None; // Singular matrix, cannot invert
            }

            if pivot_row != i {
                // Swap rows
                for col in 0..(n * 2) {
                    let t = aug.get(i, col);
                    aug.set(i, col, aug.get(pivot_row, col));
                    aug.set(pivot_row, col, t);
                }
            }

            // Scale pivot row to 1
            let pivot = aug.get(i, i);
            for col in i..(n * 2) {
                let val = aug.get(i, col);
                aug.set(i, col, gf.div(val, pivot));
            }

            // Eliminate column elements in other rows
            for r in 0..n {
                if r != i {
                    let factor = aug.get(r, i);
                    if factor != 0 {
                        for col in i..(n * 2) {
                            let val_i = aug.get(i, col);
                            let val_r = aug.get(r, col);
                            let prod = gf.mul(val_i, factor);
                            aug.set(r, col, gf.sub(val_r, prod));
                        }
                    }
                }
            }
        }

        // Extract inverted matrix
        let mut inv = Self::new(n, n);
        for r in 0..n {
            for c in 0..n {
                inv.set(r, c, aug.get(r, n + c));
            }
        }
        Some(inv)
    }
}
