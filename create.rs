//! Handler: PUT /:bucket — create a new bucket.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use chrono::Utc;
use tracing::info;

use crate::{
    errors::AppError,
    metadata::models::BucketMetadata,
    server::SharedState,
    storage::backend::StorageBackend,
};

/// Create a bucket.
///
/// - Returns `200 OK` on success.
/// - Returns `409 Conflict` if the bucket already exists.
pub async fn create_bucket(
    State(state): State<SharedState>,
    Path(bucket): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    // Validate bucket name (simplified S3 rules)
    validate_bucket_name(&bucket)?;

    // Check for duplicate
    if state.db.bucket_exists(&bucket)? {
        return Err(AppError::BucketAlreadyExists(bucket));
    }

    // Create on-disk directory
    state.storage.create_bucket(&bucket).await?;

    // Persist metadata
    let meta = BucketMetadata {
        name: bucket.clone(),
        created_at: Utc::now(),
        region: state.region.clone(),
    };
    state.db.put_bucket(&meta)?;

    info!("Bucket created: {}", bucket);
    Ok(StatusCode::OK)
}

/// Validate S3 bucket naming rules.
fn validate_bucket_name(name: &str) -> Result<(), AppError> {
    if name.len() < 3 || name.len() > 63 {
        return Err(AppError::BadRequest(format!(
            "bucket name '{}' must be 3–63 characters",
            name
        )));
    }
    if !name
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    {
        return Err(AppError::BadRequest(format!(
            "bucket name '{}' contains invalid characters",
            name
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_bucket_name_passes() {
        assert!(validate_bucket_name("my-bucket").is_ok());
        assert!(validate_bucket_name("my-bucket-123").is_ok());
    }

    #[test]
    fn short_bucket_name_fails() {
        assert!(validate_bucket_name("ab").is_err());
    }

    #[test]
    fn uppercase_bucket_name_fails() {
        assert!(validate_bucket_name("MyBucket").is_err());
    }
}
