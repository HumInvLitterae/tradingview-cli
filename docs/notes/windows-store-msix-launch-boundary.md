# Windows Store/MSIX launch boundary

This note records the current boundary for Windows Microsoft Store / MSIX
TradingView Desktop launch support in the Rust `tv` CLI.

## Summary

The Rust launcher has Windows AppX/MSIX discovery helpers and direct AppX launch
has now been smoke-tested on Windows with Chrome DevTools Protocol enabled.

On 2026-04-29, Microsoft Store / MSIX TradingView Desktop
`TradingView.Desktop 3.1.0.7818` was verified with the existing Rust
`windows_appx_direct` path. `tv launch` started the app with
`--remote-debugging-port`, `/json/version` returned a `TradingView/3.1.0`
desktop user agent, and `tv status` plus `tv tab list` connected to the
resulting CDP endpoint.

This is verified compatibility for that Store/MSIX version, not a guarantee for
all future Microsoft Store, TradingView Desktop, Electron, or Windows packaged
app behavior. The standalone Windows Desktop build remains the recommended
install path for `tv launch`, with Store/MSIX treated as smoke-tested but still
more distribution-sensitive.

## Upstream evidence

- Upstream PR #110 proposes an AUMID / shortcut activation path for MSIX
  installs. The key idea is that direct WindowsApps executable launch may not
  accept custom command-line arguments, while shell activation may be able to
  pass `--remote-debugging-port`.
- Upstream PR #114 reports that the Microsoft Store build can ignore
  `--remote-debugging-port` and may never open the CDP port because of the
  packaged-app sandbox.

These reports pointed in different implementation directions. The Windows v3
smoke resolved the current Rust boundary: keep the existing direct executable
path and do not add AUMID / shortcut activation unless a future Store/MSIX build
proves direct AppX launch no longer opens CDP.

## Current Rust behavior

- `tv launch` first succeeds if the configured CDP endpoint is already
  responding.
- Standalone executable discovery and explicit `tv launch --path <PATH>` remain
  the preferred launch paths.
- Windows AppX/MSIX discovery uses bounded PowerShell-based discovery of the
  TradingView package install location, then directly starts the packaged
  `TradingView.exe` with `--remote-debugging-port=<PORT>`.
- AUMID / shortcut activation is not implemented in the Rust CLI because the
  direct AppX executable path works for the verified Store/MSIX v3 build.

## Windows verification evidence

Verified on 2026-04-29 against Microsoft Store / MSIX
`TradingView.Desktop 3.1.0.7818`:

- Manifest application id: `TradingView.Desktop`
- EntryPoint: `Windows.FullTrustApplication`
- `tv launch --port <free-port>` from a stopped TradingView state returned
  `launch_method: "windows_appx_direct"`, `resolved_by: "windows_appx"`, and
  `cdp_ready: true`.
- `127.0.0.1:<free-port>/json/version` returned a desktop user agent containing
  `TradingView/3.1.0`, `Chrome/140.0.7339.133`, and `Electron/38.2.2`.
- `TV_CDP_PORT=<free-port> tv status` connected to the chart target.
- `TV_CDP_PORT=<free-port> tv tab list` returned both a chart target and the
  TradingView app-window target.

Do not record the local WindowsApps absolute path, target ids, chart ids, or
account-local metadata in tracked docs.

## Documentation boundary

Describe the standalone Windows Desktop build as the recommended install for
`tv launch`. Store/MSIX compatibility should be described as verified for
`TradingView.Desktop 3.1.0.7818` through direct AppX launch, not as universally
guaranteed. If a future Store/MSIX version cannot reliably open CDP, update
user-facing docs to mark that version as unsupported for `tv launch` and
recommend the standalone Desktop build or explicit `tv launch --path <PATH>`.
If AUMID activation works in that future failure mode, implement it in a
separate Windows-specific ExecPlan with live smoke evidence.
