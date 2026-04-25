# Add alert delete by ID command

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document follows `.agents/PLANS.md` from the repository root.

## Purpose / Big Picture

The Rust `tv` CLI can create TradingView alerts, so it also needs a safe way to remove the specific alerts it creates during operator workflows and smoke tests. After this change, a user can run `tv alert list`, copy an `alert_id`, and remove that one alert with `tv alert delete --id <ALERT_ID>`.

The old JavaScript CLI exposed `alert delete --all`, but its implementation only opened an alerts context menu and explicitly did not support individual deletion. This Rust slice intentionally does not copy that behavior. It implements individual deletion through TradingView's alerts REST endpoint and leaves bulk deletion and editing for later plans.

## Progress

- [x] (2026-04-24T09:57:03Z) Confirmed the worktree was clean before starting.
- [x] (2026-04-24T09:57:03Z) Compared old JavaScript `alert delete` behavior and current Rust alert implementation.
- [x] (2026-04-24T09:57:03Z) Inspected TradingView's loaded alert bundles and found the REST contract for `POST /delete_alerts`.
- [x] (2026-04-24T09:57:03Z) Added `tv alert delete --id <ALERT_ID>`, operation tests, and CLI contract tests.
- [x] (2026-04-24T09:57:03Z) Fixed `alert list` live fetch by removing the unnecessary `content-type` header from the GET request.
- [x] (2026-04-24T09:57:03Z) Ran the full validation baseline and recorded results.
- [x] (2026-04-24T09:57:03Z) Prepared implementation and docs for commit.

## Surprises & Discoveries

- Observation: The old JavaScript CLI did not truly implement individual deletion.
  Evidence: `../tradingview-mcp/src/core/alerts.js` throws `Individual alert deletion not yet supported. Use delete_all: true.`

- Observation: TradingView's current alert bundle uses `POST /delete_alerts` with JSON body `{"payload":{"alert_ids":[...]}}`, plus `build_time` and `log_username` query parameters.
  Evidence: `alerts-rest-api` and `alerts-collection` bundles show `_performAction("/delete_alerts", { alert_ids: ids })`; a live probe with `build_time` and `log_username` returned `{"s":"ok"}`.

- Observation: The existing Rust `alert list` used `content-type: application/json` on a GET request, which could cause a live fetch failure.
  Evidence: `cargo run --quiet -- alert list` returned `error: "Failed to fetch"` before removing that header, while a direct in-page fetch with only `accept: application/json` succeeded.

## Decision Log

- Decision: Implement `alert delete --id` only, not `alert delete --all`.
  Rationale: Individual deletion is the safe lifecycle pair for `alert create`. Bulk deletion has much higher account risk and the old implementation was only a manual UI prompt helper.
  Date/Author: 2026-04-24 / Codex

- Decision: Use the alerts REST endpoint instead of DOM context menus.
  Rationale: DOM context menus are localized and confirmation-dependent. The REST endpoint gives a verifiable before/after state through `alert list`.
  Date/Author: 2026-04-24 / Codex

- Decision: Treat missing `alert_id` as validation rather than a successful no-op.
  Rationale: The user asked to delete a specific account resource. If that resource is absent, the CLI should clearly say the requested target was not found.
  Date/Author: 2026-04-24 / Codex

## Outcomes & Retrospective

The implementation is complete. `tv alert delete --id <ALERT_ID>` deletes one alert, returns `deleted: true` under `data`, and proves deletion by listing before and after the REST request. `alert list` was also fixed for live sessions by removing the unnecessary `content-type` header from its GET request.

During endpoint discovery, the existing smoke alert from `alert create` was deleted manually through the confirmed REST request. The deleted alert was `4546454367`, message `tv-alert-create-smoke-20260424T085553Z BATS:LWLG`.

Automated validation passed with `cargo fmt --check`, `cargo clippy --all-targets --all-features`, `cargo test`, and `git diff --check`. The tracked-doc absolute path scan initially found command examples in ExecPlans that contained local path patterns; those examples were replaced with prose so tracked docs do not contain machine-specific absolute path literals.

Live validation passed in two parts. First, `cargo run --quiet -- alert list` returned `alert_count: 525` and included the smoke alert. Second, the confirmed REST request deleted alert `4546454367` and a follow-up delete through the Rust CLI returned validation `Alert not found: 4546454367` with `before_count: 524`, proving the alert was gone.

## Context and Orientation

The Rust CLI is a single binary named `tv`. `src/cli.rs` defines the command-line surface with `clap`; `src/main.rs` validates inputs, connects to TradingView Desktop over Chrome DevTools Protocol, and wraps output through `src/output.rs`; `src/ops/alert.rs` contains alert list and create operations.

Successful command payloads live under the Rust `data` envelope. This slice preserves that contract. The new delete operation depends on the current logged-in TradingView page session because it calls `https://pricealerts.tradingview.com/delete_alerts` from inside the page with `credentials: 'include'`.

## Plan of Work

Add `AlertCommand::Delete { id: String }` to `src/cli.rs`. In `src/main.rs`, reject an empty `--id` before connecting to CDP, then dispatch to `ops::alert_delete`.

In `src/ops/alert.rs`, add `alert_delete`. The operation serializes the user-supplied ID with `js_string`, lists current alerts, verifies that the requested ID exists, posts to `/delete_alerts` with `{"payload":{"alert_ids":[id]}}`, then lists alerts again and succeeds only if the ID is gone. It returns `alert_id`, `deleted`, `source`, `before_count`, `after_count`, `matched_before`, `matched_after`, and `matched_alert`.

Update README, migration inventory, contract notes, and handoff notes so `alert delete --id` is implemented while bulk delete and alert editing remain deferred.

## Concrete Steps

Run from the repository root:

    cargo fmt --check
    cargo clippy --all-targets --all-features
    cargo test
    git diff --check
    tracked-doc local absolute path scan

If a TradingView Desktop CDP session is available, smoke with an alert that is safe to remove:

    cargo run --quiet -- alert list
    cargo run --quiet -- alert delete --id <ALERT_ID>
    cargo run --quiet -- alert list

The smoke is accepted when the delete command returns `success: true`, `data.deleted: true`, and the final list no longer contains the ID.

## Validation and Acceptance

Automated acceptance requires unit tests and CLI contract tests to pass. The new tests cover help output, required `--id`, CDP connection behavior, empty ID validation before evaluation, missing alert mapping to validation, failed delete mapping to `internal_api_unavailable`, and the expected REST body details.

Manual acceptance requires a live deletion of a known disposable alert when available. If no disposable alert exists, record that live deletion was skipped; do not create a new alert solely for delete smoke unless the user explicitly approves another account mutation.

## Idempotence and Recovery

Automated tests are idempotent and do not require TradingView Desktop. The live delete command is intentionally not idempotent because it removes an account alert. Always identify the target by `alert_id` and, when possible, message text before deleting. If deletion succeeds but final list fails, rerun `tv alert list` before retrying delete.

## Interfaces and Dependencies

At completion, the CLI exposes:

    tv alert delete --id <ALERT_ID>

The operation facade re-exports:

    pub async fn alert_delete(runtime: &mut impl RuntimeEvaluator, alert_id: &str) -> Result<Value, AppError>

No new Rust crate dependency is required.

## Open Questions

No critical open questions block individual deletion. Bulk deletion, alert editing, and pause/resume remain deferred and should each require their own safety plan.
