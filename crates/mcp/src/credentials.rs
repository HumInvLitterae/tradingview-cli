//! SDK credential adapter; its boxed-future boundary does not require async-trait.

use crate::{Failure, Result, http::Endpoints};
use rmcp::transport::auth::{AuthError, CredentialStore, StoredCredentials};
use serde::{Deserialize, Serialize};
use std::{
    future::Future,
    path::PathBuf,
    pin::Pin,
    process::Stdio,
    sync::{Arc, Mutex},
};
use tokio::{
    io::{AsyncReadExt, AsyncWrite, AsyncWriteExt},
    process::Command,
    time::{Instant, timeout_at},
};

#[cfg(target_os = "windows")]
mod windows;

const SERVICE: &str = "tradingview-cli.mcp";
const PROFILE: &str = "default";
const MAX_RECORD: usize = 64 * 1024;
const MAX_IPC: usize = MAX_RECORD * 5;

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Record {
    version: u8,
    resource: String,
    issuer: String,
    redirect_uri: String,
    credentials: StoredCredentials,
}

#[derive(Serialize, Deserialize)]
enum Operation {
    Load,
    AuthorizeAccess,
    Save(Vec<u8>),
    Clear,
}

#[derive(Serialize, Deserialize)]
struct Reply {
    result: Result<Option<Vec<u8>>>,
}

#[derive(Clone)]
enum Backend {
    Native(PathBuf),
    #[cfg(test)]
    Memory(Arc<Mutex<MemoryStore>>),
}

#[cfg(test)]
#[derive(Default)]
struct MemoryStore {
    record: Option<Vec<u8>>,
    fail_saves: bool,
    load_calls: usize,
    fail_loads_after: Option<usize>,
}

#[derive(Clone)]
pub(crate) struct Store {
    backend: Backend,
    endpoints: Endpoints,
    deadline: Instant,
    redirect: Arc<Mutex<String>>,
    fault: Arc<Mutex<Option<Failure>>>,
    // One Store belongs to one operation under Admission. None means unread;
    // Some(None) means the store was successfully read and no record exists.
    loaded_record: Arc<Mutex<Option<Option<StoredCredentials>>>>,
    pub last_saved_bytes: Arc<Mutex<Option<usize>>>,
}

type StoreFuture<'a, T> =
    Pin<Box<dyn Future<Output = std::result::Result<T, AuthError>> + Send + 'a>>;

impl Store {
    pub fn native(endpoints: Endpoints, deadline: Instant) -> Result<Self> {
        Self::native_worker(
            endpoints,
            deadline,
            std::env::current_exe().map_err(|_| Failure::StorageUnavailable)?,
        )
    }

    pub fn native_worker(
        endpoints: Endpoints,
        deadline: Instant,
        executable: PathBuf,
    ) -> Result<Self> {
        if !executable.is_absolute() || !executable.is_file() {
            return Err(Failure::StorageUnavailable);
        }
        Ok(Self::new(Backend::Native(executable), endpoints, deadline))
    }

    fn new(backend: Backend, endpoints: Endpoints, deadline: Instant) -> Self {
        Self {
            backend,
            endpoints,
            deadline,
            redirect: Arc::new(Mutex::new(String::new())),
            fault: Arc::new(Mutex::new(None)),
            loaded_record: Arc::new(Mutex::new(None)),
            last_saved_bytes: Arc::new(Mutex::new(None)),
        }
    }

    #[cfg(test)]
    pub fn memory(endpoints: Endpoints, deadline: Instant) -> Self {
        Self::new(Backend::Memory(Default::default()), endpoints, deadline)
    }

    #[cfg(test)]
    pub fn fail_saves(&self) {
        if let Backend::Memory(state) = &self.backend {
            state.lock().unwrap().fail_saves = true;
        }
    }

    #[cfg(test)]
    pub fn next_operation(&self) -> Self {
        Self::new(self.backend.clone(), self.endpoints.clone(), self.deadline)
    }

    #[cfg(test)]
    pub fn fail_loads_after(&self, successful_loads: usize) {
        if let Backend::Memory(state) = &self.backend {
            state.lock().unwrap().fail_loads_after = Some(successful_loads);
        }
    }

    #[cfg(test)]
    pub fn storage_loads(&self) -> usize {
        match &self.backend {
            Backend::Memory(state) => state.lock().unwrap().load_calls,
            Backend::Native(_) => panic!("fixture counter requires synthetic storage"),
        }
    }

    pub fn set_redirect(&self, redirect: String) -> Result<()> {
        let url = reqwest::Url::parse(&redirect).map_err(|_| Failure::BindingMismatch)?;
        if url.scheme() != "http"
            || url.host_str() != Some("127.0.0.1")
            || url.port().is_none()
            || url.path() != "/callback"
            || url.query().is_some()
            || url.fragment().is_some()
            || !url.username().is_empty()
            || url.password().is_some()
        {
            return Err(Failure::BindingMismatch);
        }
        *self
            .redirect
            .lock()
            .map_err(|_| Failure::StorageUnavailable)? = redirect;
        Ok(())
    }

    pub fn redirect_uri(&self) -> Result<String> {
        Ok(self
            .redirect
            .lock()
            .map_err(|_| Failure::StorageUnavailable)?
            .clone())
    }

    pub fn failure(&self) -> Option<Failure> {
        self.fault.lock().ok().and_then(|v| *v)
    }

    fn store_error(&self, error: Failure) -> AuthError {
        if let Ok(mut fault) = self.fault.lock() {
            *fault = Some(error);
        }
        AuthError::CredentialStoreError(error.to_string())
    }

    async fn operation(&self, operation: Operation) -> Result<Option<Vec<u8>>> {
        if Instant::now() >= self.deadline {
            return Err(Failure::Timeout);
        }
        let native_failure = match &operation {
            Operation::Load => Failure::CredentialRead,
            Operation::Save(_) => Failure::CredentialWrite,
            _ => Failure::StorageUnavailable,
        };
        let result = match &self.backend {
            Backend::Native(executable) => {
                worker_request(executable, operation, self.deadline).await
            }
            #[cfg(test)]
            Backend::Memory(state) => {
                let mut state = state.lock().unwrap();
                match operation {
                    Operation::Load => {
                        state.load_calls += 1;
                        if state
                            .fail_loads_after
                            .is_some_and(|limit| state.load_calls > limit)
                        {
                            Err(Failure::StorageUnavailable)
                        } else {
                            Ok(state.record.clone())
                        }
                    }
                    Operation::AuthorizeAccess => Ok(None),
                    Operation::Save(bytes) => {
                        if state.fail_saves {
                            Err(Failure::StorageUnavailable)
                        } else {
                            state.record = Some(bytes);
                            Ok(None)
                        }
                    }
                    Operation::Clear => {
                        state.record = None;
                        Ok(None)
                    }
                }
            }
        };
        result.map_err(|error| {
            if error == Failure::StorageUnavailable {
                native_failure
            } else {
                error
            }
        })
    }

    fn cache_record(&self, record: Option<Option<StoredCredentials>>) -> Result<()> {
        *self
            .loaded_record
            .lock()
            .map_err(|_| Failure::StorageUnavailable)? = record;
        Ok(())
    }

    pub async fn load_record(&self) -> Result<Option<StoredCredentials>> {
        if Instant::now() >= self.deadline {
            return Err(Failure::Timeout);
        }
        if let Some(record) = self
            .loaded_record
            .lock()
            .map_err(|_| Failure::StorageUnavailable)?
            .clone()
        {
            return Ok(record);
        }
        let Some(bytes) = self.operation(Operation::Load).await? else {
            self.cache_record(Some(None))?;
            return Ok(None);
        };
        if bytes.len() > MAX_RECORD {
            return Err(Failure::StorageTooLarge);
        }
        let record: Record =
            serde_json::from_slice(&bytes).map_err(|_| Failure::CredentialRecordDecode)?;
        if record.version != 1
            || record.resource != self.endpoints.resource
            || record.issuer != self.endpoints.issuer
            || record.credentials.issuer.as_ref() != Some(&record.issuer)
        {
            return Err(Failure::BindingMismatch);
        }
        self.set_redirect(record.redirect_uri)?;
        validate_grant(&record.credentials)?;
        self.cache_record(Some(Some(record.credentials.clone())))?;
        Ok(Some(record.credentials))
    }

    pub async fn save_record(&self, credentials: StoredCredentials) -> Result<()> {
        validate_grant(&credentials)?;
        if credentials.issuer.as_ref() != Some(&self.endpoints.issuer) {
            return Err(Failure::BindingMismatch);
        }
        let record = Record {
            version: 1,
            resource: self.endpoints.resource.clone(),
            issuer: self.endpoints.issuer.clone(),
            redirect_uri: self
                .redirect
                .lock()
                .map_err(|_| Failure::StorageUnavailable)?
                .clone(),
            credentials: credentials.clone(),
        };
        self.set_redirect(record.redirect_uri.clone())?;
        let bytes = serde_json::to_vec(&record).map_err(|_| Failure::StorageUnavailable)?;
        if bytes.len() > MAX_RECORD {
            return Err(Failure::StorageTooLarge);
        }
        let size = bytes.len();
        #[cfg(target_os = "windows")]
        if !windows_record_fits(size) {
            return Err(Failure::StorageTooLarge);
        }
        // Invalidate first: a failed write must never publish the new token in
        // memory or conceal uncertainty about the persistent store.
        self.cache_record(None)?;
        self.operation(Operation::Save(bytes)).await?;
        self.cache_record(Some(Some(credentials)))?;
        *self
            .last_saved_bytes
            .lock()
            .map_err(|_| Failure::StorageUnavailable)? = Some(size);
        Ok(())
    }

    pub async fn clear_record(&self) -> Result<()> {
        self.cache_record(None)?;
        self.operation(Operation::Clear).await?;
        self.cache_record(Some(None))
    }

    pub async fn authorize_access(&self) -> Result<()> {
        self.cache_record(None)?;
        self.operation(Operation::AuthorizeAccess).await.map(|_| ())
    }
}

fn validate_grant(credentials: &StoredCredentials) -> Result<()> {
    if credentials.granted_scopes.is_empty()
        || credentials.client_id.is_empty()
        || credentials.granted_scopes.iter().any(|s| s != "mcp:read")
    {
        return Err(Failure::AccessDenied);
    }
    let value = serde_json::to_value(credentials).map_err(|_| Failure::StorageUnavailable)?;
    if !value
        .pointer("/token_response/access_token")
        .and_then(serde_json::Value::as_str)
        .is_some_and(|s| !s.is_empty())
    {
        return Err(Failure::AuthRequired);
    }
    if !value
        .pointer("/token_response/token_type")
        .and_then(serde_json::Value::as_str)
        .is_some_and(|s| s.eq_ignore_ascii_case("bearer"))
    {
        return Err(Failure::InvalidResponse);
    }
    Ok(())
}

impl CredentialStore for Store {
    fn load<'a, 'f>(&'a self) -> StoreFuture<'f, Option<StoredCredentials>>
    where
        'a: 'f,
        Self: 'f,
    {
        Box::pin(async move { self.load_record().await.map_err(|e| self.store_error(e)) })
    }

    fn save<'a, 'f>(&'a self, credentials: StoredCredentials) -> StoreFuture<'f, ()>
    where
        'a: 'f,
        Self: 'f,
    {
        Box::pin(async move {
            self.save_record(credentials)
                .await
                .map_err(|e| self.store_error(e))
        })
    }

    fn clear<'a, 'f>(&'a self) -> StoreFuture<'f, ()>
    where
        'a: 'f,
        Self: 'f,
    {
        Box::pin(async move { self.clear_record().await.map_err(|e| self.store_error(e)) })
    }
    // The outer Admission guard spans the entire operation, including refresh
    // and save. Reacquiring it in this adapter would deadlock.
}

pub(crate) fn windows_record_fits(bytes: usize) -> bool {
    bytes <= 2560
}

async fn worker_request(
    executable: &PathBuf,
    operation: Operation,
    deadline: Instant,
) -> Result<Option<Vec<u8>>> {
    let input = serde_json::to_vec(&operation).map_err(|_| Failure::StorageUnavailable)?;
    if input.len() > MAX_IPC {
        return Err(Failure::StorageTooLarge);
    }
    let mut child = Command::new(executable)
        .arg("--credential-worker")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .kill_on_drop(true)
        .spawn()
        .map_err(|_| Failure::CredentialWorkerSpawn)?;
    let mut stdin = child.stdin.take().ok_or(Failure::CredentialWorkerWrite)?;
    let mut stdout = child.stdout.take().ok_or(Failure::CredentialWorkerRead)?;
    let result = timeout_at(deadline, async {
        stdin
            .write_all(&input)
            .await
            .map_err(|_| Failure::CredentialWorkerWrite)?;
        drop(stdin);
        let mut bytes = Vec::new();
        (&mut stdout)
            .take((MAX_IPC + 1) as u64)
            .read_to_end(&mut bytes)
            .await
            .map_err(|_| Failure::CredentialWorkerRead)?;
        if bytes.len() > MAX_IPC {
            return Err(Failure::StorageTooLarge);
        }
        if !child
            .wait()
            .await
            .map_err(|_| Failure::CredentialWorkerExit)?
            .success()
        {
            return Err(Failure::CredentialWorkerExit);
        }
        serde_json::from_slice::<Reply>(&bytes)
            .map_err(|_| Failure::CredentialWorkerReply)?
            .result
    })
    .await;
    match result {
        Ok(Ok(value)) => Ok(value),
        outcome => {
            let _ = child.kill().await;
            let _ = child.wait().await;
            outcome.unwrap_or(Err(Failure::Timeout))
        }
    }
}

/// Internal worker IPC only. The harness installs no tracing/log subscriber,
/// including when RUST_LOG is set. No token or record is printed in normal mode.
#[doc(hidden)]
pub async fn credential_worker() -> Result<()> {
    use std::io::IsTerminal;
    if std::io::stdin().is_terminal() || std::io::stdout().is_terminal() {
        return Err(Failure::StorageUnavailable);
    }
    let mut bytes = Vec::new();
    tokio::io::stdin()
        .take((MAX_IPC + 1) as u64)
        .read_to_end(&mut bytes)
        .await
        .map_err(|_| Failure::StorageUnavailable)?;
    if bytes.len() > MAX_IPC {
        return Err(Failure::StorageTooLarge);
    }
    let operation = serde_json::from_slice(&bytes).map_err(|_| Failure::StorageUnavailable)?;
    let result = native_operation(operation).await;
    write_reply(&mut tokio::io::stdout(), Reply { result }).await
}

async fn write_reply(output: &mut (impl AsyncWrite + Unpin), reply: Reply) -> Result<()> {
    let bytes = serde_json::to_vec(&reply).map_err(|_| Failure::StorageUnavailable)?;
    output
        .write_all(&bytes)
        .await
        .map_err(|_| Failure::StorageUnavailable)?;
    // Tokio's stdout is buffered. Await completion on this same handle before
    // the worker returns; write_all alone does not establish reply delivery.
    output
        .flush()
        .await
        .map_err(|_| Failure::StorageUnavailable)
}

#[cfg(target_os = "macos")]
async fn native_operation(operation: Operation) -> Result<Option<Vec<u8>>> {
    use security_framework::{os::macos::keychain::SecKeychain, passwords};
    let store_error = |e: security_framework::base::Error| match e.code() {
        -25308 | -25293 => Failure::StorageInteractionRequired,
        _ => Failure::StorageUnavailable,
    };
    if matches!(operation, Operation::AuthorizeAccess) {
        // Explicit local action only. The OS owns the consent UI; normal reads
        // still refuse to prompt. Never alter the item's ACL programmatically.
        return passwords::get_generic_password(SERVICE, PROFILE)
            .map(|_| None)
            .map_err(store_error);
    }
    // This one-operation child has no worker that can reenable Keychain UI.
    let _guard =
        SecKeychain::disable_user_interaction().map_err(|_| Failure::StorageUnavailable)?;
    match operation {
        Operation::Load => match passwords::get_generic_password(SERVICE, PROFILE) {
            Ok(bytes) => Ok(Some(bytes)),
            Err(e) if e.code() == -25300 => Ok(None),
            Err(e) => Err(store_error(e)),
        },
        Operation::Save(bytes) => passwords::set_generic_password(SERVICE, PROFILE, &bytes)
            .map(|()| None)
            .map_err(store_error),
        Operation::Clear => match passwords::delete_generic_password(SERVICE, PROFILE) {
            Ok(()) => Ok(None),
            Err(e) if e.code() == -25300 => Ok(None),
            Err(e) => Err(store_error(e)),
        },
        Operation::AuthorizeAccess => unreachable!(),
    }
}

#[cfg(target_os = "windows")]
async fn native_operation(operation: Operation) -> Result<Option<Vec<u8>>> {
    windows::operation(operation, "tradingview-cli.mcp/default")
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
async fn native_operation(_operation: Operation) -> Result<Option<Vec<u8>>> {
    // Platform gates remain explicit; do not fall back to plaintext or invoke
    // Linux unlock/delete prompts before its native store is implemented.
    let _ = (SERVICE, PROFILE);
    Err(Failure::StorageUnavailable)
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::time::Duration;

    fn record() -> StoredCredentials {
        serde_json::from_value(serde_json::json!({
            "client_id": "synthetic-client",
            "issuer": "https://www.tradingview.com",
            "token_response": {
                "access_token": "synthetic-access",
                "token_type": "Bearer",
                "refresh_token": "synthetic-refresh",
                "expires_in": 3600
            },
            "granted_scopes": ["mcp:read"],
            "token_received_at": 1
        }))
        .unwrap()
    }

    #[tokio::test]
    async fn native_independent_store_binding_scopes_and_failed_save() {
        let store = Store::memory(
            Endpoints::tradingview(),
            Instant::now() + Duration::from_secs(5),
        );
        store
            .set_redirect("http://127.0.0.1:12345/callback".into())
            .unwrap();
        store.save_record(record()).await.unwrap();
        let mut wrong = record();
        wrong.issuer = Some("https://example.invalid".into());
        assert!(matches!(
            store.save_record(wrong).await,
            Err(Failure::BindingMismatch)
        ));
        let mut broad = record();
        broad.granted_scopes.push("mcp:tools".into());
        assert!(matches!(
            store.save_record(broad).await,
            Err(Failure::AccessDenied)
        ));
        store.fail_saves();
        assert!(matches!(
            store.save_record(record()).await,
            Err(Failure::CredentialWrite)
        ));
        assert!(store.load_record().await.unwrap().is_some());
        store.clear_record().await.unwrap();
        assert!(store.load_record().await.unwrap().is_none());
    }

    #[tokio::test]
    async fn record_snapshot_is_local_to_one_operation_and_tracks_durable_changes() {
        let store = Store::memory(
            Endpoints::tradingview(),
            Instant::now() + Duration::from_secs(5),
        );
        store
            .set_redirect("http://127.0.0.1:12345/callback".into())
            .unwrap();
        store.save_record(record()).await.unwrap();

        let reader = store.next_operation();
        let original = reader.load_record().await.unwrap().unwrap();
        reader.clone().load_record().await.unwrap();
        assert_eq!(reader.storage_loads(), 1);
        let mut expired = reader.clone();
        expired.deadline = Instant::now();
        assert_eq!(expired.load_record().await.unwrap_err(), Failure::Timeout);
        assert_eq!(reader.storage_loads(), 1);

        let mut rotated = original.clone();
        rotated.token_received_at = Some(2);
        reader.save_record(rotated).await.unwrap();
        assert_eq!(
            reader
                .load_record()
                .await
                .unwrap()
                .unwrap()
                .token_received_at,
            Some(2)
        );
        assert_eq!(reader.storage_loads(), 1);

        reader.fail_saves();
        assert_eq!(
            reader.save_record(original).await.unwrap_err(),
            Failure::CredentialWrite
        );
        assert_eq!(
            reader
                .load_record()
                .await
                .unwrap()
                .unwrap()
                .token_received_at,
            Some(2)
        );
        assert_eq!(reader.storage_loads(), 2);

        reader.clear_record().await.unwrap();
        assert!(reader.load_record().await.unwrap().is_none());
        assert_eq!(reader.storage_loads(), 2);
        assert!(
            reader
                .next_operation()
                .load_record()
                .await
                .unwrap()
                .is_none()
        );
        assert_eq!(reader.storage_loads(), 3);
    }

    #[tokio::test]
    async fn failed_or_invalid_reads_are_not_cached_as_valid_credentials() {
        let store = Store::memory(
            Endpoints::tradingview(),
            Instant::now() + Duration::from_secs(5),
        );
        store.fail_loads_after(0);
        assert_eq!(
            store.load_record().await.unwrap_err(),
            Failure::CredentialRead
        );
        assert_eq!(store.storage_loads(), 1);

        let Backend::Memory(state) = &store.backend else {
            unreachable!()
        };
        {
            let mut state = state.lock().unwrap();
            state.fail_loads_after = None;
            state.record = Some(b"synthetic-private-invalid-record".to_vec());
        }
        assert_eq!(
            store.load_record().await.unwrap_err(),
            Failure::CredentialRecordDecode
        );
        assert_eq!(store.storage_loads(), 2);

        state.lock().unwrap().record = None;
        assert!(store.load_record().await.unwrap().is_none());
        assert_eq!(store.storage_loads(), 3);
    }

    #[tokio::test]
    async fn worker_reply_is_flushed_before_success_is_reported() {
        let mut output = tokio::io::BufWriter::new(Vec::new());
        write_reply(
            &mut output,
            Reply {
                result: Ok(Some(b"synthetic-record".to_vec())),
            },
        )
        .await
        .unwrap();
        let reply: Reply = serde_json::from_slice(output.get_ref()).unwrap();
        assert_eq!(reply.result.unwrap(), Some(b"synthetic-record".to_vec()));
        assert!(output.buffer().is_empty());
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn worker_exit_invalid_reply_and_native_failure_remain_distinguishable() {
        use std::os::unix::fs::PermissionsExt;

        let root = tempfile::tempdir().unwrap();
        let script = root.path().join("synthetic-worker");
        assert_eq!(
            worker_request(
                &script,
                Operation::Load,
                Instant::now() + Duration::from_secs(5),
            )
            .await
            .unwrap_err(),
            Failure::CredentialWorkerSpawn
        );

        for (body, expected) in [
            ("exit 7", Failure::CredentialWorkerExit),
            (
                "printf '%s' 'synthetic-private-invalid-reply'",
                Failure::CredentialWorkerReply,
            ),
            (
                "printf '%s' '{\"result\":{\"Err\":\"storage_unavailable\"}}'",
                Failure::StorageUnavailable,
            ),
        ] {
            std::fs::write(&script, format!("#!/bin/sh\ncat >/dev/null\n{body}\n")).unwrap();
            std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o700)).unwrap();
            assert_eq!(
                worker_request(
                    &script,
                    Operation::Load,
                    Instant::now() + Duration::from_secs(5),
                )
                .await
                .unwrap_err(),
                expected
            );
        }
    }

    #[test]
    fn windows_blob_cap_is_checked_without_truncation() {
        assert!(windows_record_fits(2560));
        assert!(!windows_record_fits(2561));
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn windows_native_record_round_trip_and_oversize_preserves_old_value() {
        // Synthetic per-test entry; never touches the account's default profile.
        let target = format!(
            "tradingview-cli.mcp-test/{}/{}",
            std::process::id(),
            crate::admission::now_ms().unwrap()
        );
        struct Cleanup(String);
        impl Drop for Cleanup {
            fn drop(&mut self) {
                let _ = windows::operation(Operation::Clear, &self.0);
            }
        }
        let _cleanup = Cleanup(target.clone());
        assert!(
            windows::operation(Operation::Load, &target)
                .unwrap()
                .is_none()
        );
        windows::operation(Operation::Save(b"synthetic-original".to_vec()), &target).unwrap();
        assert_eq!(
            windows::operation(Operation::Load, &target).unwrap(),
            Some(b"synthetic-original".to_vec())
        );
        windows::operation(Operation::Save(b"synthetic-rotated".to_vec()), &target).unwrap();
        assert_eq!(
            windows::operation(Operation::Save(vec![0; 2561]), &target),
            Err(Failure::StorageTooLarge)
        );
        assert_eq!(
            windows::operation(Operation::Load, &target).unwrap(),
            Some(b"synthetic-rotated".to_vec())
        );
        windows::operation(Operation::Clear, &target).unwrap();
        assert!(
            windows::operation(Operation::Load, &target)
                .unwrap()
                .is_none()
        );
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn stuck_worker_is_killed_and_reaped_at_deadline() {
        use std::os::unix::fs::PermissionsExt;
        let root = tempfile::tempdir().unwrap();
        let script = root.path().join("stalled-worker");
        std::fs::write(&script, "#!/bin/sh\nexec sleep 30\n").unwrap();
        std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o700)).unwrap();
        let result = worker_request(
            &script,
            Operation::Load,
            Instant::now() + Duration::from_millis(100),
        )
        .await;
        assert!(matches!(result, Err(Failure::Timeout)));
    }
}
