# Add read-only alert list command

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document must be maintained in accordance with `.agents/PLANS.md`.

## Purpose / Big Picture

After this plan is implemented, the Rust-native `tv` command will cover the old JavaScript CLI's read-only `alert list` surface. A user or downstream adapter will be able to list TradingView price alerts from the current logged-in TradingView session without returning to the JavaScript bridge.

This is intentionally a small read-only slice. It does not create, delete, enable, disable, or edit alerts. The Rust CLI keeps the improved JSON envelope `{ success, command, data }`; practical old CLI alert fields remain available inside `data`.

## Progress

- [x] (2026-04-24 16:32 JST) Read continuity, current CLI/ops/tests, migration inventory, and old JavaScript `alert list` implementation from `tradesdontlie/tradingview-mcp`.
- [x] (2026-04-24 16:36 JST) Add `tv alert list` CLI and dispatch.
- [x] (2026-04-24 16:37 JST) Implement `alert_list` in a separate operation module.
- [x] (2026-04-24 16:38 JST) Add operation and CLI contract tests.
- [x] (2026-04-24 16:42 JST) Update README, migration inventory, contract note, handoff note, and agent guide.
- [x] (2026-04-24 16:40 JST) Run validation and live smoke against TradingView Desktop CDP.
- [x] (2026-04-24 16:46 JST) Record outcomes and commit implementation as `b6b05a3 feat(alert): Add alert list command`; this documentation update follows as the companion commit.

## Surprises & Discoveries

- Observation: The old JavaScript `alert list` command is read-only and uses TradingView's pricealerts endpoint from inside the page session.
  Evidence: `src/core/alerts.js` in `tradesdontlie/tradingview-mcp` calls `fetch('https://pricealerts.tradingview.com/list_alerts', { credentials: 'include' })` and maps response rows into alert fields.

- Observation: Old `alert create` and `alert delete` are not the same risk class as `alert list`.
  Evidence: `src/core/alerts.js` opens alert UI, dispatches keyboard events, edits inputs, clicks create buttons, or opens context menus for delete.

- Observation: The available live TradingView Desktop session could execute the command path, but the page fetch could not reach the alerts endpoint.
  Evidence: `cargo run -- alert list` returned `success: true`, `alert_count: 0`, `alerts: []`, and `error: "Failed to fetch"` under `data`.

## Decision Log

- Decision: Implement `alert list` by itself.
  Rationale: It preserves an old CLI read surface while avoiding alert mutation, DOM form automation, and manual confirmation workflows.
  Date/Author: 2026-04-24 / Codex

- Decision: Return API-level errors as a successful read payload with `error` when the page fetch returns an unexpected response.
  Rationale: The old CLI returned `success: true` with `alert_count`, `source`, `alerts`, and optional `error`. Keeping that practical shape under `data` preserves information compatibility.
  Date/Author: 2026-04-24 / Codex

## Outcomes & Retrospective

The alert list slice is implemented. The CLI now supports `tv alert list` under the existing Rust JSON envelope, with operation code isolated in `src/ops/alert.rs`.

Automated validation passed with `cargo fmt --check`, `cargo clippy --all-targets --all-features`, `cargo test`, `git diff --check`, and a tracked-doc local absolute path scan. Unit tests cover normal alert payloads, API error payload preservation, and malformed payload fallback. CLI contract tests now cover top-level help, alert subcommand help, and structured CDP connection errors for `tv alert list`.

Live smoke against TradingView Desktop CDP reached the page and returned a successful read payload with `error: "Failed to fetch"`. This validates the command path and the preserved API-error payload shape, but it does not prove a live successful alerts endpoint response.

## Context and Orientation

The Rust CLI is a single binary named `tv`. `src/cli.rs` defines command-line parsing with `clap`. `src/main.rs` dispatches parsed commands and wraps successful results with `src/output.rs`. `src/cdp.rs` defines `RuntimeEvaluator`, the trait used to evaluate JavaScript inside TradingView Desktop through Chrome DevTools Protocol, or CDP.

The operation layer uses `src/ops.rs` as a thin facade. Feature implementations live under `src/ops/`. Alerts are a new feature group, so their implementation should live in `src/ops/alert.rs`, not in an existing chart, data, or diagnostics module.

The old JavaScript CLI returned command payload fields at the top level. This Rust CLI returns command payloads under `data`. For example, old `tv alert list` returned `{ "success": true, "alert_count": 2 }`, while Rust should return `{ "success": true, "command": "alert", "data": { "alert_count": 2 } }`.

## Plan of Work

First, extend the CLI surface. Add a top-level `Alert` command to `src/cli.rs` with an `AlertCommand::List` subcommand. Add `"alert"` to `Command::name()`. Update `src/main.rs` imports and dispatch so `alert list` connects to CDP and calls `ops::alert_list`.

Next, add `src/ops/alert.rs`. The operation should evaluate asynchronous JavaScript in the TradingView page that fetches `https://pricealerts.tradingview.com/list_alerts` with `credentials: 'include'`, maps `data.r` rows into alert objects, and returns `{ alert_count, source: "internal_api", alerts, error }`. Preserve fields from the old CLI: `alert_id`, `symbol`, `type`, `message`, `active`, `condition`, `resolution`, `created`, `last_fired`, and `expiration`.

Then, update `src/ops.rs` to declare `mod alert;` and re-export `alert_list`.

Finally, add tests and docs. Operation tests should use `FakeRuntime`; CLI tests should prove help output and connection error behavior. Documentation should move `alert list` from deferred backlog to implemented surface while keeping `alert create` and other alert mutation work deferred.

## Concrete Steps

Run commands from the repository root.

After editing, run:

    cargo fmt --check
    cargo clippy --all-targets --all-features
    cargo test
    git diff --check

Also scan tracked docs for local absolute filesystem paths:

    git grep with the repository's standard local-absolute-path pattern

If TradingView Desktop is already running with CDP enabled, run:

    cargo run -- alert list

An empty alert list is a valid success when the page session is logged in and the endpoint returns no alerts. A login/session/API problem should be recorded in this plan as live smoke evidence and should not invalidate automated tests.

## Validation and Acceptance

The implementation is accepted when `tv --help` lists `alert`, `tv alert --help` lists `list`, `tv alert list` attempts a CDP connection like other read commands, and `ops::alert_list` returns practical old CLI fields under the Rust `data` envelope.

On success, the payload must include `alert_count`, `source`, `alerts`, and optional `error`. Alert rows should include `alert_id`, `symbol`, `type`, `message`, `active`, `condition`, `resolution`, `created`, `last_fired`, and `expiration` when the upstream response provides them.

Automated acceptance requires `cargo fmt --check`, `cargo clippy --all-targets --all-features`, `cargo test`, and `git diff --check` to pass.

## Idempotence and Recovery

The implementation is additive. Running tests repeatedly should not change tracked files. The command is read-only and must not create, delete, enable, disable, or edit alerts.

If live smoke fails because TradingView Desktop is not running, no chart target is available, the user is not logged in, or the internal alerts endpoint rejects the session, keep the automated validation result and record the smoke blocker in this plan.

## Artifacts and Notes

Important source evidence:

    old JavaScript core: https://github.com/tradesdontlie/tradingview-mcp/blob/main/src/core/alerts.js
    old JavaScript CLI command group: https://github.com/tradesdontlie/tradingview-mcp/blob/main/src/cli/commands/alerts.js
    Rust CLI modules: src/cli.rs, src/main.rs, src/ops.rs, src/cdp.rs
    migration policy: docs/notes/rust-cli-contract-migration-2026-04-24.md
    development guideline: docs/development.md

## Interfaces and Dependencies

No new Rust crates are required.

`src/cli.rs` must expose:

    tv alert list

`src/ops.rs` must expose:

    pub async fn alert_list(runtime: &mut impl RuntimeEvaluator) -> Result<Value, AppError>

`RuntimeEvaluator` remains the test seam for JavaScript evaluation. Tests must continue using fake evaluators rather than requiring a running TradingView Desktop.

## Open Questions

No critical open questions block this implementation. Later work can decide whether alert creation or deletion belongs in this CLI after a separate safety and recovery plan.

Revision note: initial plan for the read-only `alert list` migration slice after selecting it over pane mutation and watchlist mutation.
