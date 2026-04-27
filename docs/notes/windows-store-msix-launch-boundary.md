# Windows Store/MSIX launch boundary

This note records the current boundary for Windows Microsoft Store / MSIX
TradingView Desktop launch support in the Rust `tv` CLI.

## Summary

The Rust launcher has Windows AppX/MSIX discovery helpers, but discovery is not
the same as confirmed launch support with Chrome DevTools Protocol enabled.

As of this note, Microsoft Store / MSIX TradingView Desktop launch remains a
Windows-live-verification backlog item. Do not describe it as fully supported
until a Windows smoke proves that `tv launch` can start the app with
`--remote-debugging-port` and that `tv status` can connect to the resulting CDP
endpoint.

## Upstream evidence

- Upstream PR #110 proposes an AUMID / shortcut activation path for MSIX
  installs. The key idea is that direct WindowsApps executable launch may not
  accept custom command-line arguments, while shell activation may be able to
  pass `--remote-debugging-port`.
- Upstream PR #114 reports that the Microsoft Store build can ignore
  `--remote-debugging-port` and may never open the CDP port because of the
  packaged-app sandbox.

These reports point in different implementation directions, so this repository
should not resolve the behavior from a macOS development environment.

## Current Rust behavior

- `tv launch` first succeeds if the configured CDP endpoint is already
  responding.
- Standalone executable discovery and explicit `tv launch --path <PATH>` remain
  the preferred launch paths.
- Windows AppX/MSIX discovery exists as bounded PowerShell-based discovery, but
  the effective CDP-enabled launch behavior has not been verified on Windows.
- AUMID / shortcut activation is not implemented in the Rust CLI.

## Windows verification backlog

When a Windows environment is available, verify:

- Standalone TradingView Desktop install:
  - `tv launch`
  - `tv status`
  - `tv tab list`
- Microsoft Store / MSIX install:
  - whether `tv launch` finds the install
  - whether direct launch opens `127.0.0.1:<port>/json/version`
  - whether AUMID / shortcut activation can pass `--remote-debugging-port`
  - whether `tv status` and `tv tab list` work after launch

If Store/MSIX cannot reliably open CDP, update user-facing docs to mark it as
unsupported for `tv launch` and recommend the standalone Desktop build. If AUMID
activation works, implement it in a separate Windows-specific ExecPlan with live
smoke evidence.

## Documentation boundary

Until Windows smoke is available, describe the standalone Windows Desktop build
as the recommended install for `tv launch`. Store/MSIX compatibility should be
described as unverified, not as supported and not as impossible.
