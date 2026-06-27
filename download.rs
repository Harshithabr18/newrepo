//! Handler: GET /:bucket/*key — download an object.

use axum::{
    body::Body,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use tracing::info;

use crate::{errors::AppError, server::SharedState, storage::backend::StorageBackend};

/// Handler: GET /:bucket/*key
///
/// Streams the stored bytes back to the client with correct Content-Type
/// and ETag headers.
pub async fn download_object(
    State(state): State<SharedState>,
    Path((bucket, key)): Path<(String, String)>,
) -> Result<impl IntoResponse, AppError> {
    // Fetch metadata (returns 404 if not found)
    let meta = state
        .db
        .get_object(&bucket, &key)?
        .ok_or_else(|| AppError::ObjectNotFound {
            bucket: bucket.clone(),
            key: key.clone(),
        })?;

    // Read bytes from storage
    let bytes = state.storage.get_object(&bucket, &key).await?;

    info!("Object downloaded: {}/{} ({} bytes)", bucket, key, bytes.len());

    let mut headers = HeaderMap::new();
    headers.insert(
        "Content-Type",
        meta.content_type.parse().unwrap(),
    );
    headers.insert(
        "ETag",
        format!("\"{}\"", meta.etag).parse().unwrap(),
    );
    headers.insert(
        "Content-Length",
        meta.size.to_string().parse().unwrap(),
    );
    headers.insert(
        "Last-Modified",
        meta.last_modified.to_rfc2822().parse().unwrap(),
    );

    Ok(Response::builder()
        .status(StatusCode::OK)
        .body(Body::from(bytes))
        .unwrap())
}
