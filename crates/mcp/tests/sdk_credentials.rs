//! Compile and dispatch against rmcp's actual object-safe credential interface.

//! No macro, native store, network, or real credential is involved.

use std::{future::Future, pin::Pin, sync::Mutex};

use rmcp::transport::auth::{AuthError, AuthorizationManager, CredentialStore, StoredCredentials};

type StoreFuture<'a, T> = Pin<Box<dyn Future<Output = Result<T, AuthError>> + Send + 'a>>;

#[derive(Default)]
struct FixtureStore(Mutex<Option<StoredCredentials>>);

// Keep SDK type erasure at this boundary; the implementation body can use
// ordinary async code. The two lifetimes match the SDK's expanded trait.
impl CredentialStore for FixtureStore {
    fn load<'a, 'f>(&'a self) -> StoreFuture<'f, Option<StoredCredentials>>
    where
        'a: 'f,
        Self: 'f,
    {
        Box::pin(async move { Ok(self.0.lock().unwrap().clone()) })
    }

    fn save<'a, 'f>(&'a self, credentials: StoredCredentials) -> StoreFuture<'f, ()>
    where
        'a: 'f,
        Self: 'f,
    {
        Box::pin(async move {
            *self.0.lock().unwrap() = Some(credentials);
            Ok(())
        })
    }

    fn clear<'a, 'f>(&'a self) -> StoreFuture<'f, ()>
    where
        'a: 'f,
        Self: 'f,
    {
        Box::pin(async move {
            *self.0.lock().unwrap() = None;
            Ok(())
        })
    }
}

#[tokio::test]
async fn standard_future_implementation_supports_sdk_dynamic_dispatch() {
    let store: Box<dyn CredentialStore> = Box::new(FixtureStore::default());
    assert!(store.load().await.unwrap().is_none());
    store
        .save(StoredCredentials::new(
            "synthetic-client".into(),
            None,
            vec![],
            None,
        ))
        .await
        .unwrap();
    assert_eq!(
        store.load().await.unwrap().unwrap().client_id,
        "synthetic-client"
    );
    assert!(store.acquire_refresh_guard().await.unwrap().is_none());
    store.clear().await.unwrap();
    assert!(store.load().await.unwrap().is_none());

    // Construction/setter are local; do not initialize/discover authorization.
    let mut manager = AuthorizationManager::new("https://example.invalid/mcp")
        .await
        .unwrap();
    manager.set_credential_store(FixtureStore::default());
}
