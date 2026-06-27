//! my-object-store — Library root.
//!
//! Re-exports all public modules so the binary crate and tests can access
//! everything through a single, clean path.

#[path = "../auth/mod.rs"]
pub mod auth;
#[path = "../bucket/mod.rs"]
pub mod bucket;
#[path = "../cluster/mod.rs"]
pub mod cluster;
#[path = "../config/config.rs"]
pub mod config;
#[path = "../erasure/mod.rs"]
pub mod erasure;
pub mod errors;
#[path = "../metadata/mod.rs"]
pub mod metadata;
pub mod middleware;
#[path = "../multipart/mod.rs"]
pub mod multipart;
#[path = "../object/mod.rs"]
pub mod object;
pub mod router;
pub mod server;
#[path = "../storage/mod.rs"]
pub mod storage;
pub mod utils;

// Re-export the config module (lives outside src/ but is part of the crate).
pub use crate::errors::AppError;

/// Application-wide result type.
pub type Result<T> = std::result::Result<T, AppError>;
