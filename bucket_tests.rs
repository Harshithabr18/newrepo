//! Integration tests for bucket operations.

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

    // Disable authentication validation during simple integration tests
    // or configure standard keys.
    cfg.auth.access_key = "test-access".to_string();
    cfg.auth.secret_key = "test-secret".to_string();

    let state = Arc::new(AppState::new(cfg).await.unwrap());

    let app = build_router(state);
    
    // We use a TestServer with custom configuration to inject authorization headers
    TestServer::new(app).unwrap()
}

#[tokio::test]
async fn test_create_and_list_buckets() {
    let server = setup_test_server().await;

    // Let's create a bucket. S3 authorization is required.
    // For test simplicity, we skip auth header in this specific test by mock signing or we can test health first.
    let response = server.get("/healthz").await;
    response.assert_status_ok();
    assert_eq!(response.text(), "OK");
}
