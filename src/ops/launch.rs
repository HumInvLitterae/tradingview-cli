use std::{
    env,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::Duration,
};

use serde_json::{Value, json};

use crate::{
    error::{AppError, ErrorKind},
    transport::TransportConfig,
};

const LAUNCH_READY_ATTEMPTS: usize = 15;
const LAUNCH_READY_DELAY: Duration = Duration::from_secs(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LaunchMethod {
    ExistingCdp,
    DirectSpawn,
    MacosOpen,
    WindowsAppxDirect,
}

impl LaunchMethod {
    fn as_str(self) -> &'static str {
        match self {
            Self::ExistingCdp => "existing_cdp",
            Self::DirectSpawn => "direct_spawn",
            Self::MacosOpen => "macos_open",
            Self::WindowsAppxDirect => "windows_appx_direct",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ResolvedBy {
    ExplicitPath,
    CandidatePath,
    PathEnv,
    #[allow(dead_code)]
    Mdfind,
    #[allow(dead_code)]
    WindowsProcess,
    WindowsAppx,
}

impl ResolvedBy {
    fn as_str(self) -> &'static str {
        match self {
            Self::ExplicitPath => "explicit_path",
            Self::CandidatePath => "candidate_path",
            Self::PathEnv => "path_env",
            Self::Mdfind => "mdfind",
            Self::WindowsProcess => "windows_process",
            Self::WindowsAppx => "windows_appx",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LaunchTarget {
    path: PathBuf,
    resolved_by: ResolvedBy,
}

struct LaunchPayloadInput {
    binary: Option<PathBuf>,
    pid: Option<u32>,
    used_existing: bool,
    cdp_ready: bool,
    launch_method: LaunchMethod,
    resolved_by: Option<ResolvedBy>,
    fallback_used: bool,
    version: CdpVersion,
    warning: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LaunchRequest {
    pub host: String,
    pub port: u16,
    pub binary_path: Option<PathBuf>,
    pub kill_existing: bool,
}

impl LaunchRequest {
    pub fn new(
        config: &TransportConfig,
        port: Option<u16>,
        binary_path: Option<PathBuf>,
        kill_existing: bool,
    ) -> Result<Self, AppError> {
        let port = port.unwrap_or(config.port);
        if port == 0 {
            return Err(AppError::new(
                ErrorKind::Validation,
                "CDP port must be between 1 and 65535",
            ));
        }
        if let Some(path) = binary_path.as_deref() {
            validate_binary_path(path)?;
        }
        Ok(Self {
            host: config.host.clone(),
            port,
            binary_path,
            kill_existing,
        })
    }

    fn transport_config(&self) -> TransportConfig {
        TransportConfig {
            host: self.host.clone(),
            port: self.port,
            target_id: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CdpVersion {
    browser: Option<String>,
    user_agent: Option<String>,
}

pub async fn launch(request: LaunchRequest) -> Result<Value, AppError> {
    if let Some(version) = cdp_version(&request.transport_config()).await? {
        return Ok(launch_payload(
            &request,
            LaunchPayloadInput {
                binary: None,
                pid: None,
                used_existing: true,
                cdp_ready: true,
                launch_method: LaunchMethod::ExistingCdp,
                resolved_by: None,
                fallback_used: false,
                version,
                warning: None,
            },
        ));
    }

    let target = resolve_launch_target(request.binary_path.as_deref())?;
    if request.kill_existing {
        kill_existing_tradingview();
        tokio::time::sleep(Duration::from_millis(1500)).await;
    }

    let launch_method = if target.resolved_by == ResolvedBy::WindowsAppx {
        LaunchMethod::WindowsAppxDirect
    } else {
        LaunchMethod::DirectSpawn
    };
    let mut child = Command::new(&target.path)
        .arg(format!("--remote-debugging-port={}", request.port))
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|err| {
            AppError::new(
                ErrorKind::Connection,
                format!("Failed to launch TradingView: {err}"),
            )
            .with_details(json!({ "binary": path_display(&target.path) }))
        })?;
    let pid = child.id();
    // The child should continue running after this CLI exits. Dropping the handle does not kill it.
    let _ = child.try_wait();

    let mut last_version = wait_for_cdp_version(&request).await?;
    let mut final_method = launch_method;
    let mut fallback_used = false;
    if last_version.is_none()
        && should_attempt_macos_open(env::consts::OS)
        && launch_with_macos_open(&request).is_ok()
    {
        fallback_used = true;
        final_method = LaunchMethod::MacosOpen;
        last_version = wait_for_cdp_version(&request).await?;
    }
    let ready = last_version.is_some();
    let warning = launch_warning(ready, fallback_used, request.kill_existing);

    Ok(launch_payload(
        &request,
        LaunchPayloadInput {
            binary: Some(target.path),
            pid: Some(pid),
            used_existing: false,
            cdp_ready: ready,
            launch_method: final_method,
            resolved_by: Some(target.resolved_by),
            fallback_used,
            version: last_version.unwrap_or(CdpVersion {
                browser: None,
                user_agent: None,
            }),
            warning,
        },
    ))
}

#[cfg(test)]
fn resolve_binary_path(explicit_path: Option<&Path>) -> Result<PathBuf, AppError> {
    Ok(resolve_launch_target(explicit_path)?.path)
}

fn resolve_launch_target(explicit_path: Option<&Path>) -> Result<LaunchTarget, AppError> {
    if let Some(path) = explicit_path {
        validate_binary_path(path)?;
        return Ok(LaunchTarget {
            path: path.to_path_buf(),
            resolved_by: ResolvedBy::ExplicitPath,
        });
    }

    let candidates = platform_candidate_paths();
    for candidate in &candidates {
        if candidate.is_file() {
            return Ok(LaunchTarget {
                path: candidate.clone(),
                resolved_by: ResolvedBy::CandidatePath,
            });
        }
    }

    if let Some(path) = find_on_path() {
        return Ok(LaunchTarget {
            path,
            resolved_by: ResolvedBy::PathEnv,
        });
    }

    #[cfg(target_os = "windows")]
    if let Some(path) = find_windows_process_path() {
        return Ok(LaunchTarget {
            path,
            resolved_by: ResolvedBy::WindowsProcess,
        });
    }

    #[cfg(target_os = "windows")]
    if let Some(path) = find_windows_appx_executable() {
        return Ok(LaunchTarget {
            path,
            resolved_by: ResolvedBy::WindowsAppx,
        });
    }

    #[cfg(target_os = "macos")]
    if let Some(path) = find_with_mdfind() {
        return Ok(LaunchTarget {
            path,
            resolved_by: ResolvedBy::Mdfind,
        });
    }

    Err(AppError::new(
        ErrorKind::Validation,
        "TradingView executable not found. Provide --path or launch TradingView manually with --remote-debugging-port.",
    )
    .with_details(json!({
        "searched": path_list(&candidates),
    })))
}

pub fn platform_candidate_paths() -> Vec<PathBuf> {
    platform_candidate_paths_for(env::consts::OS)
}

fn platform_candidate_paths_for(os: &str) -> Vec<PathBuf> {
    match os {
        "macos" => {
            let mut paths = vec![PathBuf::from(
                "/Applications/TradingView.app/Contents/MacOS/TradingView",
            )];
            if let Some(home) = env::var_os("HOME") {
                paths.push(
                    PathBuf::from(home)
                        .join("Applications")
                        .join("TradingView.app")
                        .join("Contents")
                        .join("MacOS")
                        .join("TradingView"),
                );
            }
            paths
        }
        "windows" => {
            let mut paths = Vec::new();
            for var in ["LOCALAPPDATA", "PROGRAMFILES", "PROGRAMFILES(X86)"] {
                if let Some(root) = env::var_os(var) {
                    paths.push(
                        PathBuf::from(root)
                            .join("TradingView")
                            .join("TradingView.exe"),
                    );
                }
            }
            if paths.is_empty() {
                paths.push(PathBuf::from(
                    r"C:\Program Files\TradingView\TradingView.exe",
                ));
            }
            paths
        }
        _ => {
            let mut paths = vec![
                PathBuf::from("/opt/TradingView/tradingview"),
                PathBuf::from("/opt/TradingView/TradingView"),
                PathBuf::from("/usr/bin/tradingview"),
                PathBuf::from("/snap/tradingview/current/tradingview"),
            ];
            if let Some(home) = env::var_os("HOME") {
                paths.push(
                    PathBuf::from(home)
                        .join(".local")
                        .join("share")
                        .join("TradingView")
                        .join("TradingView"),
                );
            }
            paths
        }
    }
}

fn validate_binary_path(path: &Path) -> Result<(), AppError> {
    if path.as_os_str().is_empty() {
        return Err(AppError::new(
            ErrorKind::Validation,
            "TradingView binary path must not be empty",
        ));
    }
    if !path.is_file() {
        return Err(AppError::new(
            ErrorKind::Validation,
            "TradingView binary path does not exist or is not a file",
        )
        .with_details(json!({ "path": path_display(path) })));
    }
    Ok(())
}

async fn cdp_version(config: &TransportConfig) -> Result<Option<CdpVersion>, AppError> {
    let url = format!("http://{}:{}/json/version", config.host, config.port);
    let response = match reqwest::get(url).await {
        Ok(response) => response,
        Err(_) => return Ok(None),
    };
    if !response.status().is_success() {
        return Ok(None);
    }
    let value = response
        .json::<Value>()
        .await
        .map_err(|err| AppError::new(ErrorKind::Connection, err.to_string()))?;
    Ok(Some(cdp_version_from_value(&value)))
}

async fn wait_for_cdp_version(request: &LaunchRequest) -> Result<Option<CdpVersion>, AppError> {
    for _ in 0..LAUNCH_READY_ATTEMPTS {
        tokio::time::sleep(LAUNCH_READY_DELAY).await;
        if let Some(version) = cdp_version(&request.transport_config()).await? {
            return Ok(Some(version));
        }
    }
    Ok(None)
}

pub fn cdp_version_from_value(value: &Value) -> CdpVersion {
    CdpVersion {
        browser: value
            .get("Browser")
            .and_then(Value::as_str)
            .map(ToString::to_string),
        user_agent: value
            .get("User-Agent")
            .and_then(Value::as_str)
            .map(ToString::to_string),
    }
}

fn launch_payload(request: &LaunchRequest, input: LaunchPayloadInput) -> Value {
    json!({
        "launched": !input.used_existing,
        "platform": env::consts::OS,
        "binary": input.binary.as_deref().map(path_display),
        "pid": input.pid,
        "cdp_port": request.port,
        "cdp_url": format!("http://{}:{}", request.host, request.port),
        "cdp_ready": input.cdp_ready,
        "browser": input.version.browser,
        "user_agent": input.version.user_agent,
        "used_existing": input.used_existing,
        "kill_existing": request.kill_existing,
        "launch_method": input.launch_method.as_str(),
        "resolved_by": input.resolved_by.map(ResolvedBy::as_str),
        "fallback_used": input.fallback_used,
        "warning": input.warning,
    })
}

fn path_display(path: &Path) -> String {
    path.display().to_string()
}

fn path_list(paths: &[PathBuf]) -> Vec<String> {
    paths.iter().map(|path| path_display(path)).collect()
}

fn find_on_path() -> Option<PathBuf> {
    let command = if cfg!(target_os = "windows") {
        "where"
    } else {
        "which"
    };
    let arg = if cfg!(target_os = "windows") {
        "TradingView.exe"
    } else {
        "tradingview"
    };
    let output = Command::new(command).arg(arg).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let first = stdout
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())?;
    let path = PathBuf::from(first);
    path.is_file().then_some(path)
}

#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
fn first_non_empty_line(stdout: &str) -> Option<&str> {
    stdout.lines().map(str::trim).find(|line| !line.is_empty())
}

#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
fn appx_executable_from_install_location(stdout: &str) -> Option<PathBuf> {
    let install_location = first_non_empty_line(stdout)?;
    Some(PathBuf::from(install_location).join("TradingView.exe"))
}

#[cfg(target_os = "windows")]
fn find_windows_process_path() -> Option<PathBuf> {
    let output = Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            "Get-Process TradingView -ErrorAction SilentlyContinue | Select-Object -First 1 -ExpandProperty Path",
        ])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    first_non_empty_line(&stdout).map(PathBuf::from)
}

#[cfg(target_os = "windows")]
fn find_windows_appx_executable() -> Option<PathBuf> {
    let output = Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            "(Get-AppxPackage -Name TradingView.Desktop -ErrorAction SilentlyContinue).InstallLocation",
        ])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    appx_executable_from_install_location(&stdout)
}

#[cfg(target_os = "macos")]
fn find_with_mdfind() -> Option<PathBuf> {
    let output = Command::new("mdfind")
        .arg("kMDItemFSName == TradingView.app")
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let app = stdout
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())?;
    let path = PathBuf::from(app)
        .join("Contents")
        .join("MacOS")
        .join("TradingView");
    path.is_file().then_some(path)
}

fn should_attempt_macos_open(os: &str) -> bool {
    os == "macos"
}

fn launch_with_macos_open(request: &LaunchRequest) -> Result<(), AppError> {
    if !should_attempt_macos_open(env::consts::OS) {
        return Err(AppError::new(
            ErrorKind::Validation,
            "macOS open fallback is only available on macOS",
        ));
    }

    let status = Command::new("open")
        .args([
            "-a",
            "TradingView",
            "--args",
            &format!("--remote-debugging-port={}", request.port),
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(|err| {
            AppError::new(
                ErrorKind::Connection,
                format!("Failed to launch TradingView with macOS open fallback: {err}"),
            )
        })?;
    if !status.success() {
        return Err(AppError::new(
            ErrorKind::Connection,
            "macOS open fallback exited unsuccessfully",
        ));
    }
    Ok(())
}

fn launch_warning(ready: bool, fallback_used: bool, kill_existing: bool) -> Option<String> {
    if ready {
        return None;
    }
    if fallback_used && !kill_existing {
        return Some(
            "TradingView launched but CDP is not responding yet. If TradingView was already running without CDP, retry with --kill-existing to restart it with the debug port.".to_string(),
        );
    }
    Some("TradingView launched but CDP is not responding yet. It may still be loading.".to_string())
}

fn kill_existing_tradingview() {
    let result = if cfg!(target_os = "windows") {
        Command::new("taskkill")
            .args(["/F", "/IM", "TradingView.exe"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
    } else {
        Command::new("pkill")
            .args(["-f", "TradingView"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
    };
    let _ = result;
}

#[cfg(test)]
mod tests {
    use tempfile::NamedTempFile;

    use super::*;

    #[test]
    fn launch_request_uses_env_port_unless_overridden() {
        let config = TransportConfig {
            host: "localhost".to_string(),
            port: 9333,
            target_id: None,
        };

        let request = LaunchRequest::new(&config, None, None, false).unwrap();
        assert_eq!(request.port, 9333);

        let request = LaunchRequest::new(&config, Some(9444), None, true).unwrap();
        assert_eq!(request.port, 9444);
        assert!(request.kill_existing);
    }

    #[test]
    fn launch_request_rejects_port_zero() {
        let config = TransportConfig::default();

        let error = LaunchRequest::new(&config, Some(0), None, false).unwrap_err();

        assert_eq!(error.kind, ErrorKind::Validation);
    }

    #[test]
    fn launch_request_rejects_missing_explicit_path() {
        let config = TransportConfig::default();

        let error = LaunchRequest::new(
            &config,
            None,
            Some(PathBuf::from("target/does-not-exist-tradingview")),
            false,
        )
        .unwrap_err();

        assert_eq!(error.kind, ErrorKind::Validation);
        assert_eq!(
            error.message,
            "TradingView binary path does not exist or is not a file"
        );
    }

    #[test]
    fn resolve_binary_path_accepts_existing_explicit_file() {
        let file = NamedTempFile::new().unwrap();

        let resolved = resolve_binary_path(Some(file.path())).unwrap();

        assert_eq!(resolved, file.path());
    }

    #[test]
    fn first_non_empty_line_ignores_blank_powershell_lines() {
        assert_eq!(
            first_non_empty_line("\r\n  \n  C:\\Program Files\\WindowsApps\\TradingView.exe\r\n"),
            Some("C:\\Program Files\\WindowsApps\\TradingView.exe")
        );
    }

    #[test]
    fn appx_executable_from_install_location_appends_tradingview_exe() {
        let path = appx_executable_from_install_location(
            "C:\\Program Files\\WindowsApps\\TradingView.Desktop_2.14.0_x64__n534cwy3pjxzj\r\n",
        )
        .unwrap();

        assert_eq!(
            path,
            PathBuf::from(
                "C:\\Program Files\\WindowsApps\\TradingView.Desktop_2.14.0_x64__n534cwy3pjxzj"
            )
            .join("TradingView.exe")
        );
    }

    #[test]
    fn launch_warning_mentions_kill_existing_after_macos_fallback_without_restart() {
        let warning = launch_warning(false, true, false).unwrap();

        assert!(warning.contains("--kill-existing"));
    }

    #[test]
    fn launch_warning_is_absent_when_cdp_ready() {
        assert!(launch_warning(true, true, false).is_none());
    }

    #[test]
    fn macos_open_fallback_is_platform_gated() {
        assert!(should_attempt_macos_open("macos"));
        assert!(!should_attempt_macos_open("windows"));
        assert!(!should_attempt_macos_open("linux"));
    }

    #[test]
    fn platform_candidate_paths_are_non_empty_for_known_platforms() {
        assert!(!platform_candidate_paths_for("macos").is_empty());
        assert!(!platform_candidate_paths_for("windows").is_empty());
        assert!(!platform_candidate_paths_for("linux").is_empty());
    }

    #[test]
    fn cdp_version_from_value_normalizes_browser_fields() {
        let version = cdp_version_from_value(&json!({
            "Browser": "TradingView/1.0",
            "User-Agent": "Test Agent",
        }));

        assert_eq!(version.browser.as_deref(), Some("TradingView/1.0"));
        assert_eq!(version.user_agent.as_deref(), Some("Test Agent"));
    }
}
