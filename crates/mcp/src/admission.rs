//! Per-user coordination; this file never contains tokens or market data.

use crate::{Failure, Result};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tokio::time::{Instant, sleep, timeout_at};

#[derive(Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct State {
    last_dispatch_ms: u64,
    cooldown_until_ms: u64,
}

pub(crate) struct Admission {
    _lock: File,
    #[cfg(windows)]
    _directory_guard: File,
    directory: PathBuf,
    state: State,
}

pub(crate) fn now_ms() -> Result<u64> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| Failure::Clock)
        .and_then(|d| u64::try_from(d.as_millis()).map_err(|_| Failure::Clock))
}

enum StateFileMode {
    Read,
    Lock,
    New,
}

fn private_open(path: &Path, mode: StateFileMode) -> Result<File> {
    if path
        .symlink_metadata()
        .is_ok_and(|m| !m.is_file() || m.file_type().is_symlink())
    {
        return Err(Failure::LocalState);
    }

    let mut options = OpenOptions::new();
    options.read(true);
    match mode {
        StateFileMode::Read => {}
        StateFileMode::Lock => {
            options.write(true).create(true);
        }
        StateFileMode::New => {
            options.write(true).create_new(true);
        }
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        use windows_sys::Win32::Storage::FileSystem::FILE_FLAG_OPEN_REPARSE_POINT;
        options.custom_flags(FILE_FLAG_OPEN_REPARSE_POINT);
    }
    let file = options.open(path).map_err(|_| Failure::LocalState)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if file
            .metadata()
            .map_err(|_| Failure::LocalState)?
            .permissions()
            .mode()
            & 0o077
            != 0
        {
            return Err(Failure::LocalState);
        }
    }
    #[cfg(windows)]
    crate::windows_state::validate_file(&file)?;
    Ok(file)
}

impl Admission {
    pub(crate) fn check_cooldown(&self) -> Result<()> {
        if self.state.cooldown_until_ms > now_ms()? {
            return Err(Failure::RateLimited);
        }
        Ok(())
    }

    pub(crate) async fn acquire(directory: &Path, deadline: Instant) -> Result<Self> {
        #[cfg(windows)]
        let directory_guard = crate::windows_state::prepare(directory)?;
        #[cfg(not(windows))]
        if !directory.exists() {
            let mut builder = fs::DirBuilder::new();
            builder.recursive(true);
            #[cfg(unix)]
            {
                use std::os::unix::fs::DirBuilderExt;
                builder.mode(0o700);
            }
            builder.create(directory).map_err(|_| Failure::LocalState)?;
        }
        let meta = directory
            .symlink_metadata()
            .map_err(|_| Failure::LocalState)?;
        if !meta.is_dir() || meta.file_type().is_symlink() {
            return Err(Failure::LocalState);
        }

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if meta.permissions().mode() & 0o077 != 0 {
                return Err(Failure::LocalState);
            }
        }

        let lock = private_open(&directory.join("operation.lock"), StateFileMode::Lock)?;
        timeout_at(deadline, async {
            loop {
                match lock.try_lock() {
                    Ok(()) => break,
                    Err(std::fs::TryLockError::WouldBlock) => {
                        sleep(Duration::from_millis(20)).await
                    }
                    Err(_) => return Err(Failure::LocalState),
                }
            }
            Ok(())
        })
        .await
        .map_err(|_| Failure::Timeout)??;
        let path = directory.join("admission.json");
        let state = if path.exists() {
            read_private_json(&path, 1024)?
        } else {
            State::default()
        };
        Ok(Self {
            _lock: lock,
            #[cfg(windows)]
            _directory_guard: directory_guard,
            directory: directory.to_owned(),
            state,
        })
    }

    fn save(&self) -> Result<()> {
        write_private_json(&self.directory.join("admission.json"), &self.state)
    }

    pub(crate) async fn before_tool(&mut self, deadline: Instant) -> Result<()> {
        let now = now_ms()?;
        if self.state.last_dispatch_ms > now.saturating_add(1000) {
            return Err(Failure::Clock);
        }
        if self.state.cooldown_until_ms > now {
            return Err(Failure::RateLimited);
        }
        let wait = self
            .state
            .last_dispatch_ms
            .saturating_add(1000)
            .saturating_sub(now);
        timeout_at(deadline, sleep(Duration::from_millis(wait)))
            .await
            .map_err(|_| Failure::Timeout)?;
        self.state.last_dispatch_ms = now_ms()?;
        self.save()
    }

    pub(crate) fn cooldown(&mut self, seconds: u64) -> Result<()> {
        self.state.cooldown_until_ms = now_ms()?.saturating_add(seconds.saturating_mul(1000));
        self.save()
    }
}

/// Read through the same bounded, no-reparse state-file boundary used for writes.
pub(crate) fn read_private_json<T: serde::de::DeserializeOwned>(
    path: &Path,
    max_bytes: u64,
) -> Result<T> {
    let mut bytes = Vec::new();
    private_open(path, StateFileMode::Read)?
        .take(max_bytes.saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(|_| Failure::LocalState)?;
    if bytes.len() as u64 > max_bytes {
        return Err(Failure::LocalState);
    }
    serde_json::from_slice(&bytes).map_err(|_| Failure::LocalState)
}

/// Caller holds the directory's operation lock. Contains no secret data.
pub(crate) fn write_private_json(path: &Path, value: &impl Serialize) -> Result<()> {
    let next_path = path.with_extension("next");
    if next_path.exists() {
        fs::remove_file(&next_path).map_err(|_| Failure::LocalState)?;
    }
    let mut next = private_open(&next_path, StateFileMode::New)?;
    next.write_all(&serde_json::to_vec(value).map_err(|_| Failure::LocalState)?)
        .map_err(|_| Failure::LocalState)?;
    next.sync_all().map_err(|_| Failure::LocalState)?;
    drop(next);
    fs::rename(&next_path, path).map_err(|_| Failure::LocalState)?;
    #[cfg(unix)]
    File::open(path.parent().ok_or(Failure::LocalState)?)
        .and_then(|dir| dir.sync_all())
        .map_err(|_| Failure::LocalState)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn prepared(directory: &Path) -> Admission {
        Admission::acquire(directory, Instant::now() + Duration::from_secs(10))
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn state_reads_are_bounded_and_do_not_create_missing_files() {
        let root = tempfile::tempdir().unwrap();
        let directory = root.path().join("state");
        let _guard = prepared(&directory).await;
        let path = directory.join("sample.json");
        assert!(read_private_json::<serde_json::Value>(&path, 16).is_err());
        assert!(!path.exists());
        write_private_json(&path, &serde_json::json!({"n": 1})).unwrap();
        assert_eq!(
            read_private_json::<serde_json::Value>(&path, 16).unwrap(),
            serde_json::json!({"n": 1})
        );
        assert!(read_private_json::<serde_json::Value>(&path, 2).is_err());
    }

    #[tokio::test]
    async fn lock_releases_on_drop_and_cooldown_survives_reopen() {
        let root = tempfile::tempdir().unwrap();
        let dir = root.path().join("state");
        let mut first = prepared(&dir).await;
        assert!(matches!(
            Admission::acquire(&dir, Instant::now() + Duration::from_millis(30)).await,
            Err(Failure::Timeout)
        ));
        first.cooldown(60).unwrap();
        drop(first);
        let mut next = prepared(&dir).await;
        assert_eq!(
            next.before_tool(Instant::now() + Duration::from_secs(1))
                .await
                .unwrap_err(),
            Failure::RateLimited
        );
    }

    #[tokio::test]
    async fn corrupt_state_and_clock_rollback_fail_closed() {
        let root = tempfile::tempdir().unwrap();
        let dir = root.path().join("state");
        let mut guard = prepared(&dir).await;
        guard.state.last_dispatch_ms = now_ms().unwrap() + 10000;
        assert_eq!(
            guard
                .before_tool(Instant::now() + Duration::from_secs(1))
                .await
                .unwrap_err(),
            Failure::Clock
        );
        drop(guard);
        let mut file = private_open(&dir.join("admission.json"), StateFileMode::Lock).unwrap();
        file.write_all(b"broken").unwrap();
        assert!(matches!(
            Admission::acquire(&dir, Instant::now() + Duration::from_secs(10)).await,
            Err(Failure::LocalState)
        ));
    }

    #[test]
    fn process_lock_helper() {
        let Some(directory) = std::env::var_os("TV_MCP_TEST_LOCK_DIRECTORY") else {
            return;
        };
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        runtime.block_on(async {
            let dir = PathBuf::from(directory);
            let _guard = Admission::acquire(&dir, Instant::now() + Duration::from_secs(10))
                .await
                .unwrap();
            fs::write(dir.join("child-ready"), b"ready").unwrap();
            sleep(Duration::from_secs(30)).await;
        });
    }

    #[tokio::test]
    async fn process_exit_releases_lock_without_stale_pid_recovery() {
        let root = tempfile::tempdir().unwrap();
        let dir = root.path().join("state");
        let first = prepared(&dir).await;
        drop(first);
        let mut child = tokio::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "admission::tests::process_lock_helper"])
            .env("TV_MCP_TEST_LOCK_DIRECTORY", &dir)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .kill_on_drop(true)
            .spawn()
            .unwrap();
        timeout_at(Instant::now() + Duration::from_secs(10), async {
            while !dir.join("child-ready").exists() {
                sleep(Duration::from_millis(10)).await;
            }
        })
        .await
        .unwrap();
        assert!(matches!(
            Admission::acquire(&dir, Instant::now() + Duration::from_millis(30)).await,
            Err(Failure::Timeout)
        ));
        child.kill().await.unwrap();
        child.wait().await.unwrap();
        assert!(
            Admission::acquire(&dir, Instant::now() + Duration::from_secs(1))
                .await
                .is_ok()
        );
    }

    #[tokio::test]
    async fn pacing_obeys_the_original_deadline_and_preserves_dispatch_time() {
        let root = tempfile::tempdir().unwrap();
        let dir = root.path().join("state");
        let mut guard = prepared(&dir).await;
        guard
            .before_tool(Instant::now() + Duration::from_secs(1))
            .await
            .unwrap();
        let dispatched = guard.state.last_dispatch_ms;
        assert_eq!(
            guard
                .before_tool(Instant::now() + Duration::from_millis(20))
                .await
                .unwrap_err(),
            Failure::Timeout
        );
        assert_eq!(guard.state.last_dispatch_ms, dispatched);
        guard.cooldown(172800).unwrap();
        assert!(guard.state.cooldown_until_ms >= dispatched + 172800000);
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn unsafe_directory_permissions_and_symlinks_are_rejected() {
        use std::os::unix::fs::{PermissionsExt, symlink};
        let root = tempfile::tempdir().unwrap();
        let dir = root.path().join("state");
        fs::create_dir(&dir).unwrap();
        fs::set_permissions(&dir, fs::Permissions::from_mode(0o755)).unwrap();
        assert!(matches!(
            Admission::acquire(&dir, Instant::now() + Duration::from_secs(10)).await,
            Err(Failure::LocalState)
        ));
        fs::set_permissions(&dir, fs::Permissions::from_mode(0o700)).unwrap();
        let link = root.path().join("linked-state");
        symlink(&dir, &link).unwrap();
        assert!(matches!(
            Admission::acquire(&link, Instant::now() + Duration::from_secs(1)).await,
            Err(Failure::LocalState)
        ));
    }
}
