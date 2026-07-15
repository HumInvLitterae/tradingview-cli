# Harden the TradingView launch environment and exit reporting

This ExecPlan is a living document. The sections `Progress`, `Surprises &
Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to
date as work proceeds. This document follows `.agents/PLANS.md`.

## Purpose / Big Picture

After this change, `tv launch` can directly start TradingView Desktop even when
the shell, editor, or agent process running `tv` has
`ELECTRON_RUN_AS_NODE` set. That variable changes how an Electron executable
starts: if TradingView inherits it, the executable can behave like a Node
process instead of the Chromium application and never open the Chrome DevTools
Protocol endpoint used by this CLI.

The command will also stop reporting a successful launch when the directly
spawned TradingView child has actually exited before any launch attempt made
CDP ready. A still-running process that has not finished loading retains the
existing bounded warning response. The user can observe the distinction in the
normal JSON envelope: confirmed child exit becomes a structured `connection`
error with exit code 2, while a live but not-yet-ready child remains a success
payload with `cdp_ready: false`.

This is a narrow compatibility correction. It does not redesign process
lifetime, kill an existing TradingView session by default, add restart or
monitor behavior, or implement Windows package-identity launch.

## Progress

- [x] (2026-07-15) Read `.agents/PLANS.md`, the current roadmap and work
  inventory, the current launch implementation and tests, and the earlier
  launch process-handling plans.
- [x] (2026-07-15) Inspected upstream pull request `#336` and the current
  upstream launch implementation. Confirmed that the relevant evidence is
  child-environment removal plus child-exit classification after CDP did not
  bind.
- [x] (2026-07-15) Created this self-contained ExecPlan and separated this
  work from Windows MSIX package-identity feasibility.
- [x] (2026-07-15) Synchronized the plan index, roadmap, work inventory,
  changelog, and local continuity ledger, then completed docs-only validation.
- [ ] Add deterministic direct-spawn environment construction and tests that
  prove `ELECTRON_RUN_AS_NODE` is removed from the TradingView child.
- [ ] Add bounded post-readiness child-state classification and public-safe
  connection errors for exited or unobservable direct children.
- [ ] Preserve and test existing-CDP reuse, live-but-not-ready warning success,
  macOS system-launcher behavior, no-kill default, and explicit-path behavior.
- [ ] Update help, stable docs, packaged guidance, and the small runtime-skill
  launch guidance without expanding unrelated workflows.
- [ ] Run focused launch tests, CLI contract tests, the full Rust baseline,
  public-hygiene checks, packaging checks, and guide parity.
- [ ] Obtain focused independent review, apply any required corrections, and
  archive this plan only after the implementation and review are green.

## Surprises & Discoveries

- Observation: normal no-path launch on macOS already uses the system app
  launcher rather than spawning the TradingView executable as a direct child.
  Evidence: `crates/cli/src/ops/launch.rs` selects `macos_open` before binary
  resolution when the platform is macOS and `--path` is absent.

- Observation: direct spawn currently inherits the complete parent environment
  and immediately calls `try_wait`, but discards that process-state result.
  Evidence: the direct-spawn chain builds `std::process::Command` without an
  environment override, then assigns `let _ = child.try_wait()` before the CDP
  readiness wait.

- Observation: the current successful `cdp_ready: false` response deliberately
  represents a process that may still be loading, while upstream `#336` only
  converts the response to failure when the child has exited or was killed.
  Evidence: the upstream patch retains its existing warning success for a
  child with no exit status and adds a separate failure result only when an
  exit or signal is observed.

- Observation: current Rust error kinds already assign a stable process exit
  code to connection failures.
  Evidence: `tradingview_core::AppError::exit_code` maps
  `ErrorKind::Connection` to exit code 2.

## Decision Log

- Decision: Remove `ELECTRON_RUN_AS_NODE` only from commands that directly
  spawn the TradingView executable.
  Rationale: that is the confirmed failure boundary. Discovery helpers such as
  `which`, `mdfind`, and PowerShell are not Electron applications. The normal
  macOS path launches through the system `open` utility and is not a direct
  TradingView child; changing helper-process environments would not prove or
  improve the direct-spawn contract.
  Date/Author: 2026-07-15 / Codex.

- Decision: Preserve success with `cdp_ready: false` when the directly spawned
  child is still running after the existing bounded readiness window.
  Rationale: absence of CDP readiness does not prove launch failure. A slow
  TradingView startup must remain distinguishable from confirmed process exit.
  Date/Author: 2026-07-15 / Codex.

- Decision: Check the direct child only after the existing CDP readiness and
  macOS fallback attempts have failed to produce a version response.
  Rationale: CDP readiness is the command's success criterion. It takes
  precedence over the lifetime of the original launcher process, and delaying
  classification avoids rejecting platforms where a short-lived launcher may
  hand off to the real application successfully.
  Date/Author: 2026-07-15 / Codex.

- Decision: Treat an exited direct child and a failed child-status observation
  as structured connection failures, using only a fixed public-safe detail
  whitelist.
  Rationale: after CDP failed to appear, either state means `tv` cannot verify
  that the direct launch remains viable. Raw operating-system error text,
  executable paths, environment values, and process output are not required to
  explain the next action.
  Date/Author: 2026-07-15 / Codex.

- Decision: Keep Windows MSIX package-identity launch out of this plan.
  Rationale: current evidence for TradingView Desktop 3.3 package activation
  requires a Windows-host feasibility matrix. It is a distinct launch method,
  not a prerequisite for removing one inherited environment variable or
  reporting a terminated direct child honestly.
  Date/Author: 2026-07-15 / Codex.

## Outcomes & Retrospective

Planning is complete. Implementation, validation, live evidence, and focused
independent review have not started. The completed outcome must preserve the
existing launch methods and success payload while making direct child
environment and confirmed-exit behavior deterministic.

## Context and Orientation

The `tv launch` command is declared in `crates/cli/src/cli.rs` and dispatched
through `crates/cli/src/app/dispatch.rs`. Its operation is implemented in
`crates/cli/src/ops/launch.rs`. The operation first probes the configured local
CDP `/json/version` endpoint. If CDP already responds, it returns a successful
payload with `used_existing: true` and starts no process.

If CDP is absent, `--kill-existing` optionally terminates existing TradingView
processes. This remains opt-in because terminating the app can discard user
state. On macOS, a normal launch without `--path` invokes
`open -a TradingView --args --remote-debugging-port=<PORT>` so LaunchServices,
not the `tv` process, owns the app lifetime. An explicit macOS `--path` and the
normal Windows and Linux paths directly spawn a TradingView executable with
`std::process::Command`. Windows AppX discovery also ends in this direct-spawn
path.

`ELECTRON_RUN_AS_NODE` is an Electron runtime variable. Rust `Command`
inherits the parent environment unless an entry is explicitly changed or
removed. `Command::env_remove("ELECTRON_RUN_AS_NODE")` records that the direct
child must not receive the variable even when the parent has it.

After a new launch, `wait_for_cdp_version` polls for at most 15 one-second
attempts under an absolute deadline. Current code returns a success payload
with a warning whenever that wait ends without a CDP version. For a direct
spawn, the code has a `Child` handle and can call `try_wait` to distinguish a
still-running child from one that has exited. The normal macOS system-launcher
path does not expose the TradingView child handle and therefore cannot make
that claim.

`tradingview_core::AppError` supplies the shared JSON error envelope.
`ErrorKind::Connection` produces process exit code 2. This plan changes no JSON
success field and introduces no new top-level command or option.

## Plan of Work

### Milestone 1: Make direct-spawn environment construction deterministic

Refactor only the direct TradingView spawn configuration in
`crates/cli/src/ops/launch.rs` into a private helper named
`configure_direct_spawn`. It accepts a mutable `std::process::Command` and the
CDP port, and owns the existing `--remote-debugging-port`, null standard
streams, and `ELECTRON_RUN_AS_NODE` removal. The production launch branch
constructs `Command::new` for the resolved executable, calls this helper, and
then calls `spawn` exactly once.

Do not clear the full environment. TradingView may need ordinary user and
platform environment entries. Remove exactly `ELECTRON_RUN_AS_NODE`; do not
add a generic environment denylist. Do not apply this helper to `open`,
PowerShell, `which`, `mdfind`, `taskkill`, or `pkill`.

Add a unit test that constructs the command without spawning it, seeds one
unrelated explicit environment entry, calls the helper, and inspects
`Command::get_envs`. The test must find an explicit removal entry for
`ELECTRON_RUN_AS_NODE`, retain the unrelated entry and remote-debugging
argument, and therefore detect an accidental `env_clear`. It must not modify
the test process environment, because process-global environment mutation is
unsafe under parallel tests.

At the end of this milestone, command construction is deterministic and
testable on every host without TradingView Desktop.

### Milestone 2: Distinguish confirmed child exit from slow startup

Remove the discarded immediate `try_wait` result. After the existing readiness
window and any existing macOS fallback have both failed to obtain a CDP version,
observe the original direct child once with `try_wait`.

Represent the observation with a private, I/O-free enum such as
`DirectChildState` with `Running`, `Exited { code: Option<i32> }`, and
`Unavailable` variants. A small adapter converts the real `try_wait` result to
that enum without carrying raw operating-system error text. A separate pure
classifier receives the readiness result plus launch metadata and applies
these rules:

- CDP ready: return the existing success payload regardless of the original
  child state.
- CDP not ready and direct child still running: return the existing success
  payload with `cdp_ready: false` and the existing warning.
- CDP not ready and direct child exited: return `AppError` with
  `ErrorKind::Connection`, fixed message
  `TradingView exited before CDP became ready`, and exit code 2.
- CDP not ready and child state could not be observed: return `AppError` with
  `ErrorKind::Connection`, fixed message
  `TradingView process state could not be verified after CDP startup failed`,
  and exit code 2.

The new error details must be built from a fixed whitelist. For confirmed exit
they contain `reason: "direct_spawn_exited_before_cdp_ready"`, `cdp_port`,
`launch_method` for the original direct attempt, `final_launch_method`,
`fallback_used`, `kill_existing`, `process_started: true`,
`process_running: false`, nullable integer `exit_code`, and a fixed
`next_action_hint`. For unavailable observation, use
`reason: "direct_spawn_status_unavailable"`, omit the unverified exit code,
and use the same non-identifying context. Do not include executable paths,
environment values, raw child output, operating-system error text, target IDs,
or account-local metadata.

Keep the existing macOS no-path branch unchanged: it has no direct TradingView
child to inspect, so CDP timeout remains a successful `cdp_ready: false`
response with warning. Keep all existing successful payload fields unchanged.

Add deterministic tests for all classifier branches. The tests must prove
that an exited child yields `connection` and exit 2, a running child preserves
the warning success, CDP readiness wins over stale child status, unavailable
status fails without raw details, and each error detail object has exactly the
documented public-safe key set. Do not launch a real app or rely on shell
commands in unit tests.

At the end of this milestone, downstream callers can distinguish confirmed
direct-process termination from a bounded slow-start warning.

### Milestone 3: Synchronize user and agent guidance

Update the `tv launch` long help in `crates/cli/src/cli.rs` to state that direct
launch removes the incompatible Electron mode and that confirmed direct-child
exit is a connection failure. Do not expose implementation internals or imply
that all CDP timeouts are failures.

Update the launch sections in `README.md`, `docs/getting-started.md`,
`docs/ja/getting-started.md`, `docs/development.md`,
`packaging/agent/AGENTS.md`, and the short launch guidance in
`.agents/skills/chart-analysis/SKILL.md`. The guidance should tell agents to
run `tv readiness` after a warning response, but to treat a structured launch
connection error as evidence that the directly spawned process is no longer a
verified running app. Recommend manual app launch or correcting an explicit
path before retrying. Keep `--kill-existing` behind explicit user approval.

Update `CHANGELOG.md`, this plan, `docs/v0.28-roadmap.md`,
`docs/v0.28-work-items.md`, `docs/plans/README.md`, and local
`CONTINUITY.md` as implementation and review progress. Do not create a new
runtime skill or expand skill Core Workflow sections beyond the launch decision
needed at readiness.

At the end of this milestone, CLI help, public docs, packaged guidance, and the
runtime skill all describe the same process-state distinction.

### Milestone 4: Validate and obtain focused review

Run focused launch tests first, then CLI contracts and the full baseline. The
implementation is complete only when the command-construction and outcome
classifier tests would fail against the old code and pass against the new
code, existing launch contract tests remain green, public hygiene finds no
private values, and the full workspace baseline passes.

After local validation, request focused independent review of direct-spawn
environment removal, process-state precedence, public-safe error details,
no-kill behavior, macOS system-launcher preservation, docs alignment, and the
Windows MSIX exclusion. Apply review corrections and rerun the affected checks
before archiving the plan.

## Concrete Steps

Run all commands from the repository root.

Inspect the implementation boundary before editing:

    rg -n "Command::new|spawn|try_wait|wait_for_cdp_version|launch_warning|macos_open" crates/cli/src/ops/launch.rs
    rg -n "tv launch|kill-existing|cdp_ready" README.md docs packaging/agent/AGENTS.md .agents/skills/chart-analysis

Implement and format the focused change, then run:

    cargo fmt --check
    cargo test -p tradingview-cli ops::launch -- --nocapture
    cargo test -p tradingview-cli --test cli_contract_desktop launch -- --nocapture
    cargo test -p tradingview-cli --test cli_contract launch -- --nocapture

Run the complete baseline and repository checks:

    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1
    python3 scripts/check-public-hygiene.py --self-test
    python3 scripts/check-public-hygiene.py
    bash -n scripts/stage-release-package-files.sh
    cmp -s AGENTS.md CLAUDE.md
    git diff --check

Validate the changed chart-analysis skill with the repository's configured
skill validator. Use a portable validator command from `docs/development.md`;
do not record a machine-specific validator path in this plan.

An optional public-safe live smoke may run only when starting or reusing
TradingView is acceptable. Begin with:

    target/debug/tv readiness
    target/debug/tv launch

Existing-CDP reuse is non-mutating and should return `used_existing: true`.
Testing direct spawn with a parent `ELECTRON_RUN_AS_NODE` value requires a
stopped app or an explicitly approved disposable launch context. Never add
`--kill-existing` merely to make the smoke convenient. Record only launch
method, CDP readiness, error kind/reason, and whether the process remained
observable; do not record local paths, process output, target IDs, or raw JSON.

## Validation and Acceptance

The implementation is accepted when all of the following behavior is proven:

- the direct TradingView `Command` explicitly removes
  `ELECTRON_RUN_AS_NODE` without clearing unrelated environment entries;
- existing CDP reuse still starts no process and returns the current success
  fields;
- normal macOS no-path launch still uses `macos_open` and does not acquire a
  direct child-status claim;
- direct launch that reaches CDP still returns the current success payload;
- direct launch with no CDP and a running child still returns success with
  `cdp_ready: false` and a warning;
- direct launch with no CDP and an exited child returns a normal stderr JSON
  error envelope with `kind: "connection"`, process exit code 2, and only the
  fixed public-safe details;
- inability to observe the child after CDP failure also returns a sanitized
  connection error rather than guessing that the launch succeeded;
- no default or fallback path kills TradingView unless `--kill-existing` was
  explicit;
- no alternate Windows package activation, restart loop, daemon, dependency,
  source fallback, or public command/option is introduced;
- focused tests, full baseline, public hygiene, packaging checks, guide parity,
  and skill validation are green.

The existing-CDP smoke is supplemental. Deterministic tests are the required
evidence for environment removal and exit classification. A direct-spawn live
smoke may strengthen confidence but is not required when it would require
terminating a user's running application.

## Idempotence and Recovery

Code and deterministic tests are safe to rerun. They must not mutate the
process-global test environment or start TradingView Desktop. Documentation
edits are ordinary tracked changes and can be reapplied after resolving local
conflicts.

`tv launch` remains idempotent when CDP already responds. A normal launch may
start the app, but it does not alter chart, account, Pine, drawing, Replay, or
Screener state. If an optional smoke starts TradingView and CDP remains
unavailable, leave the app running unless the owner explicitly authorizes
termination. Do not use `--kill-existing` as cleanup.

If a direct child exits, rerunning the command is safe after correcting the
launch environment or executable path. If the child is still running but CDP
is slow, use `tv readiness` before starting another app instance. The command
must not auto-retry, auto-restart, or auto-kill in either case.

## Artifacts and Notes

The relevant upstream evidence is pull request `#336` in the original
JavaScript project. It reports that TradingView Desktop 3.3.0 under an
Electron-hosted parent failed to bind CDP when it inherited
`ELECTRON_RUN_AS_NODE=1`. Its patch copies the parent environment, deletes that
one key before spawn, and reports failure only when the child has an exit or
signal after CDP did not bind. The Rust implementation should adopt the narrow
behavioral evidence, not the JavaScript envelope or process abstraction.

The current Rust implementation already solved the earlier process-lifetime
problem on macOS by using the system launcher for normal no-path launches. It
also already performs Windows process and AppX executable discovery. Those are
existing boundaries to preserve, not reasons to broaden this slice.

Expected deterministic evidence after implementation should resemble:

    configure_direct_spawn_removes_electron_run_as_node ... ok
    exited_direct_child_without_cdp_is_connection_error ... ok
    running_direct_child_without_cdp_keeps_warning_success ... ok
    cdp_ready_precedes_direct_child_exit_status ... ok

Do not paste full JSON envelopes, local executable paths, environment dumps, or
child output into tracked documentation.

## Interfaces and Dependencies

No public Rust API, CLI command, CLI option, JSON success field, Cargo feature,
or dependency changes.

In `crates/cli/src/ops/launch.rs`, add private production helpers equivalent to:

    const ELECTRON_RUN_AS_NODE: &str = "ELECTRON_RUN_AS_NODE";

    fn configure_direct_spawn(command: &mut Command, port: u16);

    enum DirectChildState {
        Running,
        Exited { code: Option<i32> },
        Unavailable,
    }

    fn observe_direct_child(child: &mut std::process::Child) -> DirectChildState;

    fn direct_launch_result(
        request: &LaunchRequest,
        input: LaunchPayloadInput,
        direct_method: LaunchMethod,
        state: DirectChildState,
    ) -> Result<Value, AppError>;

Names may change only if the resulting names more accurately describe the same
single responsibilities. `direct_launch_result` returns the existing success
payload when `input.cdp_ready` is true or when it is false and the child is
`Running`; it returns a sanitized `Connection` error for `Exited` or
`Unavailable` only when CDP is not ready. Production only needs to observe the
child after the final CDP readiness result is absent. Keep helper ownership in
`ops/launch.rs`; this slice does not justify a new module.

Use only `std::process::Command::env_remove`, the existing Tokio timing,
`tradingview_cdp::CdpHttpSession`, and `tradingview_core::AppError`. Do not add a
signal handler, process supervisor, background task, retry library, or platform
activation dependency.

## Open Questions

- UNCONFIRMED: whether TradingView Desktop 3.2 or 3.3 installed through Windows
  MSIX requires package-identity-preserving activation instead of the current
  direct AppX executable launch. This remains a separate Windows-host
  feasibility item and does not block this plan.

- UNCONFIRMED: whether the macOS `open` path can ever expose a reliable
  TradingView child exit status. This plan deliberately makes no such claim;
  it preserves the existing bounded readiness warning for that path.

## Change Note

2026-07-15: Created the plan after current-build indicator insertion completed.
The plan incorporates the narrow upstream environment/exit evidence, preserves
the existing slow-start and macOS process-lifetime behavior, fixes public-safe
error precedence, and explicitly excludes Windows package-identity work.
