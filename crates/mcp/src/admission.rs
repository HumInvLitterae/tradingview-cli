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
    directory: PathBuf,
    state: State,
}

pub(crate) fn now_ms() -> Result<u64> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| Failure::Clock)
        .and_then(|d| u64::try_from(d.as_millis()).map_err(|_| Failure::Clock))
}

fn private_open(path: &Path, create_new: bool) -> Result<File> {
    if path
        .symlink_metadata()
        .is_ok_and(|m| !m.is_file() || m.file_type().is_symlink())
    {
        return Err(Failure::LocalState);
    }

    #[cfg(windows)]
    if path.symlink_metadata().is_ok_and(|m| {
        use std::os::windows::fs::MetadataExt;
        m.file_attributes() & 0x400 != 0
    }) {
        return Err(Failure::LocalState);
    }
    let mut options = OpenOptions::new();
    options.read(true).write(true);
    if create_new {
        options.create_new(true);
    } else {
        options.create(true);
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
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

        #[cfg(windows)]
        {
            use std::os::windows::fs::MetadataExt;
            if meta.file_attributes() & 0x400 != 0 {
                return Err(Failure::LocalState);
            }
            crate::windows_state::validate(directory, deadline).await?;
        }
        let lock = private_open(&directory.join("operation.lock"), false)?;
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
            let mut bytes = Vec::new();
            private_open(&path, false)?
                .take(1025)
                .read_to_end(&mut bytes)
                .map_err(|_| Failure::LocalState)?;
            if bytes.len() > 1024 {
                return Err(Failure::LocalState);
            }
            serde_json::from_slice(&bytes).map_err(|_| Failure::LocalState)?
        } else {
            State::default()
        };
        Ok(Self {
            _lock: lock,
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

/// Caller holds the directory's operation lock. Contains no secret data.
pub(crate) fn write_private_json(path: &Path, value: &impl Serialize) -> Result<()> {
    let next_path = path.with_extension("next");
    if next_path.exists() {
        fs::remove_file(&next_path).map_err(|_| Failure::LocalState)?;
    }
    let mut next = private_open(&next_path, true)?;
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

    #[tokio::test]
    async fn lock_releases_on_drop_and_cooldown_survives_reopen() {
        let root = tempfile::tempdir().unwrap();
        let dir = root.path().join("state");
        let mut first = Admission::acquire(&dir, Instant::now() + Duration::from_secs(1))
            .await
            .unwrap();
        assert!(matches!(
            Admission::acquire(&dir, Instant::now() + Duration::from_millis(30)).await,
            Err(Failure::Timeout)
        ));
        first.cooldown(60).unwrap();
        drop(first);
        let mut next = Admission::acquire(&dir, Instant::now() + Duration::from_secs(1))
            .await
            .unwrap();
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
        let mut guard = Admission::acquire(&dir, Instant::now() + Duration::from_secs(1))
            .await
            .unwrap();
        guard.state.last_dispatch_ms = now_ms().unwrap() + 10000;
        assert_eq!(
            guard
                .before_tool(Instant::now() + Duration::from_secs(1))
                .await
                .unwrap_err(),
            Failure::Clock
        );
        drop(guard);
        let mut file = private_open(&dir.join("admission.json"), false).unwrap();
        file.write_all(b"broken").unwrap();
        assert!(matches!(
            Admission::acquire(&dir, Instant::now() + Duration::from_secs(1)).await,
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
            let _guard = Admission::acquire(&dir, Instant::now() + Duration::from_secs(2))
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
        let first = Admission::acquire(&dir, Instant::now() + Duration::from_secs(1))
            .await
            .unwrap();
        drop(first);
        let mut child = tokio::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "admission::tests::process_lock_helper"])
            .env("TV_MCP_TEST_LOCK_DIRECTORY", &dir)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .kill_on_drop(true)
            .spawn()
            .unwrap();
        timeout_at(Instant::now() + Duration::from_secs(5), async {
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
        let mut guard = Admission::acquire(&dir, Instant::now() + Duration::from_secs(1))
            .await
            .unwrap();
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
            Admission::acquire(&dir, Instant::now() + Duration::from_secs(1)).await,
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
