//! Validate the Windows coordination directory without weakening inherited ACLs.

use crate::{Failure, Result};
use std::{path::Path, process::Stdio};
use tokio::{
    io::AsyncWriteExt,
    process::Command,
    time::{Instant, timeout_at},
};

pub(crate) async fn validate(directory: &Path, deadline: Instant) -> Result<()> {
    let script = r#"
$ErrorActionPreference = 'Stop'
[Console]::InputEncoding = [Text.Encoding]::UTF8
$p = [Console]::In.ReadToEnd()
$sid = [Security.Principal.WindowsIdentity]::GetCurrent().User.Value
$acl = Get-Acl -LiteralPath $p
if (@($sid, 'S-1-5-18', 'S-1-5-32-544') -notcontains $acl.GetOwner([Security.Principal.SecurityIdentifier]).Value) {
    exit 2
}
$allowed = @($sid, 'S-1-5-18', 'S-1-5-32-544', 'S-1-3-0', 'S-1-3-4')
foreach ($r in $acl.GetAccessRules($true, $true, [Security.Principal.SecurityIdentifier])) {
    if ($r.AccessControlType -eq 'Allow' -and $allowed -notcontains $r.IdentityReference.Value) {
        exit 3
    }
}
"#;
    let root = std::env::var_os("SystemRoot").ok_or(Failure::LocalState)?;
    let executable =
        std::path::PathBuf::from(root).join("System32/WindowsPowerShell/v1.0/powershell.exe");
    let mut child = Command::new(executable)
        .args([
            "-NoLogo",
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            script,
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .kill_on_drop(true)
        .spawn()
        .map_err(|_| Failure::LocalState)?;
    let result = timeout_at(deadline, async {
        let mut input = child.stdin.take().ok_or(Failure::LocalState)?;
        input
            .write_all(directory.to_str().ok_or(Failure::LocalState)?.as_bytes())
            .await
            .map_err(|_| Failure::LocalState)?;
        drop(input);
        if child
            .wait()
            .await
            .map_err(|_| Failure::LocalState)?
            .success()
        {
            Ok(())
        } else {
            Err(Failure::LocalState)
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
