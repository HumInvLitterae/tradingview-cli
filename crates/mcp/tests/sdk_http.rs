//! The selected http crate must produce the exact response expected by rmcp.

use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use rmcp::transport::auth::{
    AuthorizationManager, OAuthHttpClient, OAuthHttpClientFuture, OAuthHttpRequest,
};
use serde_json::json;

#[derive(Clone, Default)]
struct FixtureHttp(Arc<AtomicUsize>);
impl OAuthHttpClient for FixtureHttp {
    fn execute(&self, request: OAuthHttpRequest) -> OAuthHttpClientFuture<'_> {
        Box::pin(async move {
            assert_eq!(request.request.method(), http::Method::POST);
            assert_eq!(request.request.uri(), "https://example.invalid/register");
            self.0.fetch_add(1, Ordering::SeqCst);
            Ok(http::Response::builder()
                .status(http::StatusCode::CREATED)
                .header(http::header::CONTENT_TYPE, "application/json")
                .body(serde_json::to_vec(&json!({
                    "client_id": "synthetic-client",
                    "redirect_uris": ["http://127.0.0.1:12345/callback"]
                }))?)?)
        })
    }
}

#[tokio::test]
async fn http_response_is_compatible_with_sdk_oauth_extension() {
    let http = FixtureHttp::default();
    let mut manager = AuthorizationManager::new_with_oauth_http_client(
        "https://example.invalid/mcp",
        Arc::new(http.clone()),
    )
    .await
    .unwrap();
    manager.set_metadata(
        serde_json::from_value(json!({
            "issuer": "https://example.invalid",
            "authorization_endpoint": "https://example.invalid/authorize",
            "token_endpoint": "https://example.invalid/token",
            "registration_endpoint": "https://example.invalid/register",
            "response_types_supported": ["code"],
            "code_challenge_methods_supported": ["S256"]
        }))
        .unwrap(),
    );
    let config = manager
        .register_client("fixture", "http://127.0.0.1:12345/callback", &["mcp:read"])
        .await
        .unwrap();
    assert_eq!(config.client_id, "synthetic-client");
    assert_eq!(http.0.load(Ordering::SeqCst), 1);
}
