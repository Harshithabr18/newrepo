//! Integration tests for object operations.

use axum_test::TestServer;
use std::sync::Arc;
use tempfile::tempdir;

use my_object_store::{
    config::AppConfig,
    router::build_router,
    server::AppState,
};

async fn setup_test_server() -> TestServer {
    let temp_dir = tempdir().unwrap();
    let mut cfg = AppConfig::default();
    cfg.storage.data_dir = temp_dir.path().to_path_buf();
    cfg.auth.access_key = "test-access".to_string();
    cfg.auth.secret_key = "test-secret".to_string();

    let state = Arc::new(AppState::new(cfg).await.unwrap());

    let app = build_router(state);
    TestServer::new(app).unwrap()
}

#[tokio::test]
async fn test_unauthenticated_request_fails() {
    let server = setup_test_server().await;

    // Any request to a bucket/object without Authorization should fail with 401 Unauthorized.
    let response = server.put("/mybucket").await;
    response.assert_status(axum::http::StatusCode::UNAUTHORIZED);
}
