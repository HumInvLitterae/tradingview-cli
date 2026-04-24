# Add bounded TradingView launch command

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document follows `.agents/PLANS.md`.

## Purpose / Big Picture

After this change, a user can run `tv launch` to start TradingView Desktop with Chrome DevTools Protocol enabled for this Rust CLI. Chrome DevTools Protocol, or CDP, is the local debugging protocol used by this CLI to talk to TradingView Desktop. This closes the old CLI's launch surface while keeping the Rust version safer than the old JavaScript command.

The old JavaScript CLI killed existing TradingView processes by default before launching. The Rust command must not do that. Rust defaults to no-kill behavior: if CDP already responds, `tv launch` reports the existing endpoint and does not spawn a new process; if CDP is not ready, it finds or uses a TradingView binary path and launches with `--remote-debugging-port`.

## Progress

- [x] (2026-04-24 18:09Z) Read `.agents/PLANS.md`, current CLI dispatch and transport code, status operation, migration notes, and old JavaScript launch implementation.
- [x] (2026-04-24 18:09Z) Created this ExecPlan.
- [x] (2026-04-24 18:09Z) Add `tv launch` command-line surface and dispatch.
- [x] (2026-04-24 18:09Z) Implement bounded local TradingView launch operation.
- [x] (2026-04-24 18:09Z) Add unit and CLI contract tests.
- [x] (2026-04-24 18:09Z) Update README, AGENTS, migration inventory, contract notes, handoff note, and affected skills.
- [x] (2026-04-24 18:09Z) Run automated validation, skill validation, and live smoke.
- [x] (2026-04-24 18:09Z) Commit the completed slice.

## Surprises & Discoveries

- Windows candidate path generation originally returned no candidates on non-Windows hosts when Windows-specific environment variables were absent. The helper now includes a generic Windows fallback so pure candidate generation tests are host-independent.

## Decision Log

- Decision: Rust `tv launch` defaults to no-kill behavior.
  Rationale: Launch is a local process-control command. Killing an existing TradingView Desktop session can discard unsaved user state, so it must require explicit `--kill-existing`.
  Date/Author: 2026-04-24 / Codex.

- Decision: Treat an already responding CDP endpoint as success and avoid spawning another process.
  Rationale: This makes `tv launch` idempotent and safe to use in readiness ladders such as `tv status` then `tv launch`.
  Date/Author: 2026-04-24 / Codex.

## Outcomes & Retrospective

Implemented `tv launch [--port <PORT>] [--path <PATH>] [--kill-existing]` as a bounded launcher. The command first probes the configured CDP endpoint; when it is already available, it returns success with `used_existing: true`, `launched: false`, and does not spawn a new process. When CDP is unavailable, it resolves a TradingView binary from `--path` or platform candidates, optionally kills existing TradingView processes only when `--kill-existing` is explicit, launches with the requested remote-debugging port, and polls readiness.

The Rust payload lives under `data` and includes `launched`, `used_existing`, `platform`, `binary`, `pid`, `cdp_port`, `cdp_url`, `cdp_ready`, `browser`, `user_agent`, `kill_existing`, and optional `warning`.

## Context and Orientation

The Rust CLI is a single binary named `tv`. Command-line shape lives in `src/cli.rs`. Normal command dispatch lives in `src/main.rs`, which wraps operation payloads in the Rust JSON envelope. CDP target discovery lives in `src/transport.rs`; status checks live in `src/ops/status.rs`.

This launch slice should add a new operation module at `src/ops/launch.rs`. It should reuse `transport::TransportConfig` for host and port. The operation should first check whether the configured CDP endpoint already answers. If it does, the command returns success with `used_existing: true`. If it does not, the operation resolves a TradingView binary path, optionally kills existing TradingView processes only when `--kill-existing` is provided, spawns TradingView with the requested remote-debugging port, and polls CDP readiness for a short period.

## Plan of Work

First update `src/cli.rs`. Add a top-level `Launch` command with `--port <PORT>`, `--path <PATH>`, and `--kill-existing`. The port type should be `Option<u16>` so clap rejects invalid port values before execution. The path should be `Option<PathBuf>`.

Then update `src/main.rs`. Add dispatch for `Command::Launch` before read and mutation commands. It should build a launch request from CLI arguments and `TransportConfig::from_env()`, then call `ops::launch`.

Add `src/ops/launch.rs`. Define a `LaunchRequest` with `host`, `port`, optional `binary_path`, and `kill_existing`. Implement pure helpers for binary candidates and validation so tests do not spawn real processes. The operation should use `reqwest` to call the configured CDP `/json/version`; if that succeeds, return `launched: false`, `used_existing: true`, `cdp_ready: true`, and version details. If not ready, resolve a binary path from `--path` or platform candidates, optionally run a platform-specific process kill when `kill_existing` is true, then spawn the process with `--remote-debugging-port=<PORT>`. Poll `/json/version` up to about 15 seconds. If ready, return launched success with process id and browser details. If not ready, still return success with `cdp_ready: false` and a warning because the process may still be loading.

Finally update tests and durable docs. Docs should move `launch` from deferred backlog to implemented, record that Rust launch is no-kill by default, and keep remaining high-risk surfaces deferred.

## Concrete Steps

Run all commands from the repository root.

Targeted validation while implementing:

    cargo test ops::launch -- --nocapture
    cargo test --test cli_contract launch -- --nocapture

Full validation before commit:

    cargo fmt --check
    cargo clippy --all-targets --all-features -- -D warnings
    cargo test
    git diff --check
    rg -n "(/[U]sers/|[C]:\\\\)" README.md AGENTS.md docs .agents/skills || true

Because `.agents/skills/chart-analysis` changes, run the skill validator against that skill before committing.

## Validation and Acceptance

Automated acceptance is that tests prove CLI help, invalid path handling, launch request defaults, candidate generation, and existing-CDP payload normalization. `tv launch --path target/does-not-exist` should fail with a structured validation error before any spawn attempt.

Live smoke should run only against the current local environment. If TradingView Desktop is already running with CDP, run `cargo run --quiet -- launch` and expect a success envelope with `data.used_existing: true`, `data.cdp_ready: true`, and `data.cdp_port`. Do not smoke `--kill-existing`; that mode is intentionally manual.

## Idempotence and Recovery

The default command is idempotent because it returns the existing CDP endpoint when available and does not kill existing sessions. If a new process is launched but CDP does not become ready within the poll window, the command reports `cdp_ready: false`; the user can retry `tv status` or `tv launch` after TradingView finishes loading. No TradingView account, chart, Pine, drawing, alert, replay, tab, or watchlist state should be mutated by this command.

## Artifacts and Notes

Validation passed:

    cargo test ops::launch -- --nocapture
    cargo test --test cli_contract launch -- --nocapture
    python /path/to/skill-creator/scripts/quick_validate.py .agents/skills/chart-analysis
    cargo fmt --check
    cargo clippy --all-targets --all-features -- -D warnings
    cargo test
    git diff --check
    rg -n "(/[U]sers/|[C]:\\\\)" README.md AGENTS.md docs .agents/skills || true

Live smoke passed against the current local TradingView Desktop CDP session:

    cargo run --quiet -- status
    cargo run --quiet -- launch

`tv launch` returned `success: true`, `used_existing: true`, `launched: false`, `cdp_ready: true`, `cdp_port: 9222`, and `kill_existing: false`. The destructive `--kill-existing` mode was not live-smoked.

## Interfaces and Dependencies

At completion, the CLI exposes:

    tv launch [--port <PORT>] [--path <PATH>] [--kill-existing]

At completion, `src/ops/launch.rs` exposes `LaunchRequest` and `launch`. No new crates are required.

## Open Questions

No unresolved critical questions remain for this slice.
