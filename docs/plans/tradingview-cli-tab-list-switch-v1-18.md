# Add tab list and switch commands

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document follows `.agents/PLANS.md` from the repository root.

## Purpose / Big Picture

The old JavaScript `tv` CLI exposed `tv tab list/new/close/switch`. After this change, the Rust CLI can list existing TradingView chart tabs and switch to an existing tab by index. It does not open or close tabs.

This is the next old CLI migration slice because `tab list` helps operators and downstream tooling understand multiple TradingView chart targets, and `tab switch` activates an existing chart without creating or destroying session state. `tab new` and `tab close` remain deferred because they mutate the desktop session more broadly, and `tab close` can be destructive.

## Progress

- [x] (2026-04-24 00:00Z) Confirmed the worktree was clean before starting.
- [x] (2026-04-24 00:00Z) Inspected old JavaScript `tab` CLI and core implementation.
- [x] (2026-04-24 00:00Z) Created this ExecPlan and recorded that `tab new` and `tab close` are deferred.
- [x] (2026-04-24 00:00Z) Add Rust CLI surface and dispatch for `tv tab list` and `tv tab switch <INDEX>`.
- [x] (2026-04-24 00:00Z) Implement tab operations in a new capability module under `src/ops/`.
- [x] (2026-04-24 00:00Z) Add unit and CLI contract tests.
- [x] (2026-04-24 00:00Z) Update README, migration inventory, lifecycle audit, handoff docs, and agent guide.
- [x] (2026-04-24 00:00Z) Run automated validation baseline.
- [x] (2026-04-24 00:00Z) Run live TradingView Desktop smoke if a CDP session is available and record the result.
- [ ] Commit the completed slice.

## Surprises & Discoveries

No surprises have been discovered yet.

## Decision Log

- Decision: Implement only `tab list` and `tab switch` in this slice.
  Rationale: These commands operate on existing chart tabs and do not create or close resources. `tab new` and `tab close` need a separate safety plan because they mutate the desktop session.
  Date/Author: 2026-04-24 / Codex

- Decision: Use CDP target-list HTTP endpoints instead of connecting to a chart runtime.
  Rationale: Tab listing and activation are target-level operations. They should work even when multiple chart targets would make the usual runtime target selection ambiguous.
  Date/Author: 2026-04-24 / Codex

## Outcomes & Retrospective

Implemented `tv tab list` and `tv tab switch <INDEX>` as target-level commands that use the CDP HTTP target list and activation endpoint. `tab new` and `tab close` remain deferred.

Automated validation passed:

    cargo fmt --check
    cargo clippy --all-targets --all-features -- -D warnings
    cargo test
    git diff --check
    git grep -nE '(/Users/|C:\\)' -- README.md AGENTS.md docs .agents/skills || true

Live TradingView Desktop smoke passed with one chart target:

    cargo run --quiet -- tab list
    cargo run --quiet -- tab switch 0
    cargo run --quiet -- tab list

The live target reported `tab_count: 1`, chart id `OykuI24Y`, and `tab switch 0` returned `action: "switched"`.

## Context and Orientation

This repository implements a Rust-native command-line binary named `tv`. The entrypoint in `src/main.rs` parses commands, calls operation functions re-exported from `src/ops.rs`, and prints JSON envelopes.

Chrome DevTools Protocol, abbreviated CDP, exposes a local HTTP target list at `/json/list`. Each target has an id, title, type, URL, and optional websocket debugger URL. Existing runtime commands use `transport::discover_target` to pick one chart target, but `tab list` must not do that because listing tabs is useful precisely when there are multiple chart targets.

## Plan of Work

First, extend the CLI surface in `src/cli.rs` with a new top-level `Tab` command and a `TabCommand` subcommand enum. The subcommands are `list` and `switch`. Do not add `new` or `close`.

Next, update `src/main.rs` dispatch. `tab list` and `tab switch` should use `TransportConfig::from_env()` and call tab operations directly without `connect_runtime()`.

Then, create `src/ops/tab.rs`. Implement `tab_list(config)` by fetching CDP targets, filtering to page targets whose URL contains `tradingview.com/chart`, assigning deterministic zero-based indexes, and returning `tab_count` and `tabs`. Implement `tab_switch(config, index)` by calling `tab_list`, validating the index, and using `/json/activate/<target_id>` to bring that existing tab forward.

Finally, update tests and durable docs. Unit tests should cover target filtering, chart id extraction, title cleanup, and range validation. CLI contract tests should cover help output, the absence of `new` and `close`, and structured connection failures.

## Concrete Steps

From the repository root, run the implementation loop:

    cargo fmt --check
    cargo clippy --all-targets --all-features -- -D warnings
    cargo test
    git diff --check
    run the repository's standard tracked-doc local absolute path scan

If TradingView Desktop is available through CDP, run:

    cargo run --quiet -- tab list
    cargo run --quiet -- tab switch 0
    cargo run --quiet -- tab list

Do not run any tab creation or tab closing commands.

## Validation and Acceptance

The change is accepted when `tv tab --help` lists `list` and `switch`, does not list `new` or `close`, `tv tab list` returns existing TradingView chart tabs, and `tv tab switch <INDEX>` activates an existing tab or returns a validation error for an out-of-range index.

The success JSON must use the Rust envelope. For example, `tv tab list` should print a success envelope whose `data` includes `tab_count` and `tabs`. `tv tab switch 0` should print a success envelope whose `data` includes `action: "switched"`, `index`, `tab_id`, and `chart_id`.

## Idempotence and Recovery

Automated tests are safe to rerun and must not require TradingView Desktop.

Live smoke is low-risk because it only activates an existing tab. If switching fails, do not open or close tabs as recovery.

## Artifacts and Notes

- `src/ops/tab.rs` contains target filtering, chart id extraction, and activation logic.
- `tests/cli_contract.rs` covers `tab` help, required switch index validation, and connection failure envelope behavior.
- `README.md`, `AGENTS.md`, and migration notes now list `tab list/switch` as implemented while keeping `tab new/close` deferred.

## Interfaces and Dependencies

At the end of the implementation, these commands must exist:

    tv tab list
    tv tab switch <INDEX>

The operation facade must expose:

    pub async fn tab_list(config: &TransportConfig) -> Result<Value, AppError>
    pub async fn tab_switch(config: &TransportConfig, index: usize) -> Result<Value, AppError>

No new third-party Rust dependencies are required.

## Open Questions

There are no unresolved critical questions at the plan stage. If live TradingView behavior differs from the old JavaScript assumptions, record the discovery here and keep `tab new` and `tab close` out of this slice.
