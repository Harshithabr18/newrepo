//! Handler: DELETE /:bucket — delete a bucket.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use tracing::info;

use crate::{errors::AppError, server::SharedState, storage::backend::StorageBackend};

/// Delete a bucket.
///
/// - Returns `204 No Content` on success.
/// - Returns `404 Not Found` if bucket does not exist.
/// - Returns `409 Conflict` if bucket is not empty.
pub async fn delete_bucket(
    State(state): State<SharedState>,
    Path(bucket): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    if !state.db.bucket_exists(&bucket)? {
        return Err(AppError::BucketNotFound(bucket));
    }

    // Refuse deletion of non-empty buckets
    if state.db.bucket_has_objects(&bucket)? {
        return Err(AppError::BucketNotEmpty(bucket));
    }

    // Remove from disk and metadata
    state.storage.delete_bucket(&bucket).await?;
    state.db.delete_bucket(&bucket)?;

    info!("Bucket deleted: {}", bucket);
    Ok(StatusCode::NO_CONTENT)
}
