# Improve `tv launch` desktop compatibility

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document follows `.agents/PLANS.md` from the repository root.

## Purpose / Big Picture

`tv launch` starts TradingView Desktop with Chrome DevTools Protocol enabled so the rest of the Rust `tv` commands can connect to the app. The current implementation works for direct executable launches, but upstream bug reports from the original JavaScript project show two important release-user failures: Windows installs packaged as MSIX may not live in ordinary executable locations, and newer macOS TradingView Desktop builds may reject `--remote-debugging-port` when passed through direct process spawn. After this change, `tv launch` should keep its no-kill default while finding more Windows installs and falling back to macOS `open -a` when direct spawn does not produce a CDP-ready app.

## Progress

- [x] (2026-04-25T09:16:24Z) Read the current `src/ops/launch.rs`, upstream launch triage note, and representative upstream PR diffs for Windows MSIX and macOS Electron failures.
- [x] (2026-04-25T09:23:39Z) Implemented Rust-native launch target resolution metadata, Windows process/AppX discovery, and macOS `open -a` fallback.
- [x] (2026-04-25T09:23:39Z) Added unit tests for Windows PowerShell output parsing, Windows AppX path handling, macOS fallback gating, and fallback warning text.
- [x] (2026-04-25T09:23:39Z) Updated README, contract migration notes, and upstream PR triage notes.
- [x] (2026-04-25T09:23:39Z) Ran the validation baseline and non-destructive existing-CDP smoke.

## Surprises & Discoveries

- Observation: The current Rust launcher has no method metadata in its payload and only returns the resolved binary path.
  Evidence: `launch_payload` currently includes `binary`, `pid`, `used_existing`, and `warning`, but not `launch_method`, `fallback_used`, or `resolved_by`.

- Observation: The newest upstream Windows PR trusts `Get-AppxPackage` for WindowsApps paths because ordinary file existence checks can fail under WindowsApps permissions.
  Evidence: upstream PR `#100` says `existsSync` cannot verify WindowsApps paths without admin rights, then derives `TradingView.exe` from the package install location.

- Observation: Another upstream Windows PR uses `IApplicationActivationManager` for packaged app launch, but that requires a helper script and COM-specific implementation. This Rust slice will not introduce a bundled PowerShell script; it will improve discovery and retain direct launch attempts with clear errors.
  Evidence: upstream PR `#76` adds `scripts/launch_msix.ps1` and JavaScript helpers to activate an AUMID through COM.

- Observation: The local live TradingView session had multiple page targets, so `tv status` reported `cdp_connected: false` with target ambiguity, but `/json/version` was still available and `tv launch` could safely exercise the existing-CDP path.
  Evidence: `target/debug/tv launch` returned `used_existing: true`, `launch_method: "existing_cdp"`, `fallback_used: false`, and `cdp_ready: true`.

## Decision Log

- Decision: Preserve the current default of never killing existing TradingView processes unless `--kill-existing` is explicit.
  Rationale: The Rust CLI intentionally made launch safer than the old JavaScript path, and launch compatibility should not silently turn into session-destructive behavior.
  Date/Author: 2026-04-25 / Codex

- Decision: Add payload metadata as new fields rather than changing existing launch fields.
  Rationale: Downstream callers can observe the launch path without losing the existing `binary`, `pid`, `used_existing`, and `cdp_ready` fields.
  Date/Author: 2026-04-25 / Codex

- Decision: Support Windows MSIX discovery with PowerShell output parsing, but do not add a separate COM activation script in this slice.
  Rationale: The project is a single Rust CLI release artifact. Discovery and diagnostics are low-risk and testable here; COM activation should be a later Windows-live-smoked improvement if direct launch remains insufficient.
  Date/Author: 2026-04-25 / Codex

## Outcomes & Retrospective

The launch compatibility slice added non-breaking payload metadata and improved the release-user launch path without changing the no-kill default. Windows now has bounded PowerShell discovery for running-process and AppX/MSIX install paths, while macOS can try `open -a TradingView --args ...` after direct spawn does not make CDP ready. Full automated validation passed, and a non-destructive live smoke confirmed the existing-CDP launch payload shape.

## Context and Orientation

The launch operation lives in `src/ops/launch.rs`. `LaunchRequest::new` validates the requested CDP port and optional explicit path. `launch` first checks whether the configured CDP endpoint already responds. If it does, the command returns success without launching a process. If CDP does not respond, it resolves a TradingView executable, optionally kills existing TradingView processes only when `kill_existing` is true, spawns TradingView with `--remote-debugging-port=<port>`, and polls `/json/version` for readiness.

The Rust command prints a structured JSON envelope from `src/main.rs`; this plan only changes the `data` payload returned by `launch`. Existing fields must remain available. New fields are allowed.

## Plan of Work

Refactor `src/ops/launch.rs` just enough to represent the launch target and method. Add small enums or string helpers for `resolved_by`, `launch_method`, and `fallback_used`. Keep `resolve_binary_path` available for existing tests by making it return the path portion of the richer resolver.

Add Windows-only discovery helpers that execute PowerShell in production and pure parsing helpers for tests. The first helper reads a running TradingView process path from `Get-Process TradingView -ErrorAction SilentlyContinue | Select-Object -First 1 -ExpandProperty Path`. The second reads an AppX install location from `(Get-AppxPackage -Name TradingView.Desktop -ErrorAction SilentlyContinue).InstallLocation` and appends `TradingView.exe`. If the AppX helper returns a path, do not reject it solely because `Path::is_file` is false.

Add macOS fallback behavior after direct spawn. The launcher should try direct spawn first. If CDP is not ready after the normal wait and the platform is macOS, run `open -a TradingView --args --remote-debugging-port=<port>` and poll again. If `--kill-existing` was not set, do not kill any existing instance before fallback. If CDP still does not respond, return the existing successful-but-not-ready payload with a warning that mentions existing non-CDP sessions may need an explicit `--kill-existing` retry.

Update `README.md` and `docs/notes/rust-cli-contract-migration-2026-04-24.md` to mention the new launch metadata and the Windows/macOS compatibility behavior. Update `docs/notes/upstream-pr-triage-2026-04-25.md` to mark the launch cluster as addressed with this bounded Rust implementation and note that COM/AUMID activation remains a future Windows-specific enhancement if needed.

## Concrete Steps

From the repository root:

    cargo fmt --check
    cargo clippy --all-targets --all-features -- -D warnings
    cargo test
    git diff --check

For docs hygiene:

    git grep -nE '(/Users/|C:\\)' -- README.md docs .agents/skills || true

Do not run any command that pushes to a remote.

## Validation and Acceptance

Automated acceptance is that `cargo test` includes new passing tests for parsing a Windows process path, parsing an AppX install location, preserving the no-kill default, and selecting the macOS fallback method after direct launch does not reach CDP. The normal Rust baseline must pass.

Behavioral acceptance is that `tv launch` still returns the existing payload fields and now also reports how launch was resolved. If CDP is already available, it should return `used_existing: true`, `launch_method: "existing_cdp"`, and `fallback_used: false`. If direct spawn is used, it should report `launch_method: "direct_spawn"`. If the macOS fallback is attempted, it should report `fallback_used: true` and either become CDP-ready or include a warning that explains the retry with `--kill-existing`.

Live smoke is optional and macOS-only in this workspace. It should begin with `tv status`; if CDP is already available, `tv launch` can safely confirm the existing-CDP path. Destructive `--kill-existing` smoke requires explicit user permission.

## Idempotence and Recovery

The implementation is additive and can be retried. Tests do not launch TradingView. `tv launch` itself remains bounded: it polls CDP and exits, and it does not kill existing processes without `--kill-existing`. If fallback launch starts another app instance without CDP, the command reports `cdp_ready: false` with a warning rather than claiming success.

## Artifacts and Notes

Important upstream evidence:

    #100: Windows Microsoft Store installs can be found via Get-Process or Get-AppxPackage.
    #80/#18: newer Electron builds may reject --remote-debugging-port through direct spawn, and macOS open -a can pass the argument through LaunchServices.
    #76: packaged Windows apps may require COM activation; this plan does not add that script-level path yet.

## Interfaces and Dependencies

No new crate dependency is required. Use `std::process::Command` for PowerShell and `open` fallback. Keep all helpers inside `src/ops/launch.rs` unless the file grows enough to justify a future module split.

At completion, `launch_payload` should include these additional `data` fields:

    "launch_method": "existing_cdp" | "direct_spawn" | "macos_open" | "windows_appx_direct",
    "resolved_by": null | "explicit_path" | "candidate_path" | "path_env" | "mdfind" | "windows_process" | "windows_appx",
    "fallback_used": true | false

## Open Questions

No critical questions are open for this slice. Windows COM/AUMID activation is intentionally deferred unless a future Windows live smoke proves direct AppX executable launch cannot work with the debug flag.

Revision note 2026-04-25: Updated this living plan after implementation and validation so it records the completed behavior, the smoke result, and the deferred Windows COM/AUMID boundary.
