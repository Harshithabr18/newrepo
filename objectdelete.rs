//! Handler: DELETE /:bucket/*key — delete an object.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use tracing::info;

use crate::{errors::AppError, server::SharedState, storage::backend::StorageBackend};

/// Handler: DELETE /:bucket/*key
pub async fn delete_object(
    State(state): State<SharedState>,
    Path((bucket, key)): Path<(String, String)>,
) -> Result<impl IntoResponse, AppError> {
    if !state.db.bucket_exists(&bucket)? {
        return Err(AppError::BucketNotFound(bucket));
    }

    if state.db.get_object(&bucket, &key)?.is_none() {
        return Err(AppError::ObjectNotFound { bucket, key });
    }

    state.storage.delete_object(&bucket, &key).await?;
    state.db.delete_object(&bucket, &key)?;

    info!("Object deleted: {}/{}", bucket, key);
    Ok(StatusCode::NO_CONTENT)
}
