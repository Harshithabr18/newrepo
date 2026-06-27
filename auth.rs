//! Authentication middleware implementing AWS Signature Version 4 verification.

use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use tracing::{debug, warn};

use crate::{
    auth::signature_v4::{parse_auth_header, verify_signature},
    errors::AppError,
    server::SharedState,
};

/// Middleware to enforce AWS Signature V4 authentication.
pub async fn auth_layer(
    State(state): State<SharedState>,
    req: Request,
    next: Next,
) -> Result<Response, AppError> {
    // 1. Check for Authorization header
    let auth_val = match req.headers().get("authorization") {
        Some(val) => match val.to_str() {
            Ok(s) => s,
            Err(_) => return Err(AppError::MissingAuth),
        },
        None => return Err(AppError::MissingAuth),
    };

    // 2. Parse authorization header
    let auth_header = match parse_auth_header(auth_val) {
        Some(parsed) => parsed,
        None => {
            warn!("Failed to parse Authorization header: {}", auth_val);
            return Err(AppError::MissingAuth);
        }
    };

    // 3. Verify access key
    if auth_header.access_key != state.access_key {
        warn!(
            "Access key mismatch: expected {}, got {}",
            state.access_key, auth_header.access_key
        );
        return Err(AppError::AccessDenied);
    }

    // 4. Extract parts needed for signature verification
    let method = req.method().as_str();
    let uri = req.uri().path();
    let query_string = req.uri().query().unwrap_or("");

    // Convert headers to Vec<(String, String)> for verification
    let mut header_list = Vec::new();
    for (k, v) in req.headers() {
        if let Ok(val_str) = v.to_str() {
            header_list.push((k.to_string(), val_str.to_string()));
        }
    }

    // Determine payload hash (usually passed in x-amz-content-sha256 header)
    let body_hash = req
        .headers()
        .get("x-amz-content-sha256")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("UNSIGNED-PAYLOAD")
        .to_string();

    // 5. Verify Signature
    if !verify_signature(
        method,
        uri,
        query_string,
        &header_list,
        &body_hash,
        &auth_header,
        &state.secret_key,
    ) {
        warn!("AWS Signature V4 verification failed for uri: {}", uri);
        return Err(AppError::InvalidSignature);
    }

    debug!("AWS Signature V4 verified successfully for user: {}", auth_header.access_key);

    // Continue request processing
    Ok(next.run(req).await)
}
