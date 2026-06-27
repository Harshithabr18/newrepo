//! Handler: PUT /:bucket/*key — upload an object.

use axum::{
    body::Body,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use chrono::Utc;
use sha2::{Digest, Sha256};
use tracing::info;

use crate::{
    errors::AppError,
    metadata::models::ObjectMetadata,
    server::SharedState,
    storage::backend::StorageBackend,
    utils::hash::md5_hex,
};

/// Handler: PUT /:bucket/*key
///
/// Streams the request body to disk and records metadata.
pub async fn upload_object(
    State(state): State<SharedState>,
    Path((bucket, key)): Path<(String, String)>,
    headers: HeaderMap,
    body: Body,
) -> Result<impl IntoResponse, AppError> {
    // Bucket must exist
    if !state.db.bucket_exists(&bucket)? {
        return Err(AppError::BucketNotFound(bucket));
    }

    // Read body bytes
    let bytes = axum::body::to_bytes(body, usize::MAX)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let size = bytes.len() as u64;

    // Compute MD5 etag (S3-compatible) and SHA-256 content hash
    let etag = md5_hex(&bytes);
    let mut sha256 = Sha256::new();
    sha256.update(&bytes);
    let content_sha256 = hex::encode(sha256.finalize());

    // Content-type from header or default
    let content_type = headers
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("application/octet-stream")
        .to_string();

    // Write bytes to filesystem
    state.storage.put_object(&bucket, &key, &bytes).await?;

    // Persist metadata
    let meta = ObjectMetadata {
        bucket: bucket.clone(),
        key: key.clone(),
        size,
        etag: etag.clone(),
        content_type,
        content_sha256,
        last_modified: Utc::now(),
    };
    state.db.put_object(&meta)?;

    // Replicate to cluster peers asynchronously
    crate::cluster::replicate_object_async(
        state.cluster.clone(),
        bucket.clone(),
        key.clone(),
        bytes.to_vec(),
    );

    info!("Object uploaded: {}/{} ({} bytes)", bucket, key, size);

    Ok((
        StatusCode::OK,
        [("ETag", format!("\"{}\"", etag))],
    ))
}
