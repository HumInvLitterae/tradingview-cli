//! Hand the authorization URL to the OS without evaluating URL text as code.

use crate::{Failure, Result};
use std::process::Stdio;
use tokio::{
    io::AsyncWriteExt,
    process::Command,
    time::{Instant, timeout_at},
};

pub(crate) async fn open(url: &str, deadline: Instant) -> Result<()> {
    #[cfg(target_os = "macos")]
    let mut command = Command::new("open");
    #[cfg(target_os = "macos")]
    command.arg(url);

    #[cfg(target_os = "windows")]
    let mut command = {
        let mut command = Command::new("powershell.exe");
        // The URL enters via stdin, never PowerShell source, arguments or logs.
        command.args([
            "-NoLogo",
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            "$ErrorActionPreference='Stop'; Start-Process -FilePath ([Console]::In.ReadToEnd())",
        ]);
        command
    };
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    let mut command = Command::new("xdg-open");
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    command.arg(url);

    command
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .kill_on_drop(true);
    let mut child = command.spawn().map_err(|_| Failure::Connection)?;
    let result = timeout_at(deadline, async {
        let mut input = child.stdin.take().ok_or(Failure::Connection)?;
        #[cfg(target_os = "windows")]
        input
            .write_all(url.as_bytes())
            .await
            .map_err(|_| Failure::Connection)?;
        // Close even on systems where the launcher does not use stdin.
        input.shutdown().await.map_err(|_| Failure::Connection)?;
        drop(input);
        if child
            .wait()
            .await
            .map_err(|_| Failure::Connection)?
            .success()
        {
            Ok(())
        } else {
            Err(Failure::Connection)
        }
    })
    .await
    .unwrap_or(Err(Failure::Timeout));
    if result.is_err() {
        let _ = child.kill().await;
        let _ = child.wait().await;
    }
    result
}
