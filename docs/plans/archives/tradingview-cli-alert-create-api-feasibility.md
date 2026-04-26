# Alert create API-backed mutation feasibility

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document follows `.agents/PLANS.md`. It is self-contained so a future contributor can understand why `alert create` changed and how to validate it without reading chat history.

## Purpose / Big Picture

The user-facing goal is to make `tv alert create` less dependent on TradingView's visible alert dialog. Before this change, alert creation opened the UI dialog, looked for localized labels and inputs, typed a price, optionally typed a message, clicked a visible Create button, and trusted that the dialog action created the alert. After this change, `alert create` should prefer TradingView's logged-in alert endpoint, create the alert with the active chart symbol and resolution, then verify the new alert by reading the alert list endpoint. If the endpoint cannot be used before mutation, the previous DOM dialog path may still be used as a fallback.

## Progress

- [x] (2026-04-27) Reviewed `src/ops/alert.rs` and confirmed `alert list` and `alert delete` already use alert endpoints while `alert create` still uses DOM dialog automation.
- [x] (2026-04-27) Reviewed upstream PR #89 evidence. It records `pricealerts.tradingview.com/create_alert`, with JSON sent as a plain string body and no `Content-Type` header because a custom content type triggers a rejected CORS preflight.
- [x] (2026-04-27) Ran read-only live checks with an explicit chart target. `alert list` returned through the endpoint family, and active chart metadata exposed symbol, resolution, and currency summary fields.
- [x] (2026-04-27) Implemented API-backed `alert create` as the preferred path with DOM fallback only for API unavailability before the create request is sent.
- [x] (2026-04-27) Added operation tests for API success, API pre-mutation fallback, API post-check failure, existing validation, practical payload preservation, and delete endpoint shape.
- [x] (2026-04-27) Live-smoked one clearly marked test alert and cleaned it up in the same run.
- [x] (2026-04-27) Updated public-safe internal API docs, upstream PR notes, README, CHANGELOG, handoff notes, and `CONTINUITY.md`.
- [x] (2026-04-27) Full validation baseline passed; commit is the only remaining repository action.

## Surprises & Discoveries

- Observation: The current alert list endpoint is available from the logged-in page session, but running `tv alert list` without a target fails when multiple chart targets are open.
  Evidence: `tv alert list` returned `target_ambiguous`; rerunning with an explicit `TV_CDP_TARGET_ID` returned a successful alert count.

- Observation: Active chart metadata is sufficient to form the high-level create payload without opening the alert dialog.
  Evidence: a read-only page eval summarized active chart `symbol`, `resolution`, and `currency` without exposing cookies, tokens, or raw account payloads.

- Observation: The live smoke exposed that the previous Rust delete endpoint shape could return `invalid_request` for a newly created test alert.
  Evidence: a cleanup attempt through `tv alert delete --id` failed until the delete call was aligned to the bare alert delete endpoint used by TradingView's page session. The same test alert was then deleted, and no smoke alert residue remained.

## Decision Log

- Decision: Use the alert endpoint as the preferred `alert create` path and keep DOM fallback only before mutation.
  Rationale: Endpoint list/delete already exist in Rust, upstream PR #89 provides create endpoint evidence, and post-mutation ambiguity should not trigger a second creation attempt through DOM.
  Date/Author: 2026-04-27 / Codex.

- Decision: Do not add new CLI flags in this slice.
  Rationale: This is a stability replacement for existing `alert create`, not a new alert workflow surface.
  Date/Author: 2026-04-27 / Codex.

- Decision: Preserve the existing user-facing condition vocabulary while adding an internal condition type when API-backed.
  Rationale: Existing callers use `crossing`, `greater_than`, and `less_than`. TradingView's endpoint uses internal condition types such as `cross`, `cross_up`, and `cross_down`; the CLI should not force downstream consumers to change existing input names.
  Date/Author: 2026-04-27 / Codex.

## Outcomes & Retrospective

Implemented. `tv alert create` now prefers the logged-in alert endpoint, sends the create request without a custom `Content-Type` header, and reports success only after the alert list endpoint shows a new matching alert. The payload keeps the existing practical fields and adds API-specific fields such as `alert_id`, `symbol`, `resolution`, `condition_type`, and before/after counts when available.

The live smoke created one clearly marked disposable alert and deleted it in the same run. That smoke also found and fixed an existing `alert delete --id` request-shape bug: delete now uses the bare alert delete endpoint and still verifies absence afterward. No raw alert payloads or live alert ids are recorded in tracked docs.

## Context and Orientation

The Rust CLI prints JSON envelopes such as `{ "success": true, "command": "alert", "data": ... }`. Alert operations live in `src/ops/alert.rs`. `alert_list` calls `pricealerts.tradingview.com/list_alerts` from inside the authenticated TradingView page. `alert_delete` and `alert_delete_all` call `pricealerts.tradingview.com/delete_alerts` and verify absence by listing alerts again. `alert_create` currently uses DOM dialog automation and normalizes output through `normalize_alert_create_payload`.

The non-public alert endpoint is not an official TradingView API. It must be called from the user's logged-in TradingView page session and must not be documented with raw request payloads, alert ids, cookies, tokens, or account-linked values. Public docs may describe endpoint category, read/write status, safety boundaries, and failure behavior.

## Plan of Work

First, add `alert_create_via_api` in `src/ops/alert.rs`. It should validate the existing input first, read the active chart symbol, resolution, and currency from the page session, list alerts before creation, post to the alert create endpoint using a plain string JSON body with no custom `Content-Type`, then list alerts again and verify a new matching alert exists. It should return an error with `api_fallback_allowed: true` only when it fails before mutation, such as missing chart metadata or an unreachable list endpoint. It should return an error without fallback when the create request was sent but post-check could not verify the new alert.

Second, update `alert_create` so it tries `alert_create_via_api` first. If the API helper succeeds, return that payload. If it returns a fallback-allowed internal API error, run the existing DOM fallback unchanged. If it returns a validation error or post-mutation ambiguity, return the error directly.

Third, update tests in the same module. Existing DOM tests should add one synthetic API-unavailable response at the start so they continue covering fallback behavior. New tests should prove API-backed success, no success without post-check, and no DOM fallback after API post-check failure.

Fourth, update docs to record the new boundary. README and CHANGELOG should mention that alert creation now prefers the alert endpoint and verifies readback. `docs/internal-tradingview-apis.md` and the upstream PR #89 audit should mark `alert create` as API-backed when endpoint evidence is available. The handoff note should remove `alert create` as the highest-value replacement candidate.

## Concrete Steps

From the repository root, confirm current state:

    git status --short

Take read-only evidence:

    tv tab list
    TV_CDP_TARGET_ID=<chart-target> tv alert list
    TV_ALLOW_UNSAFE_UI_EVAL=1 TV_CDP_TARGET_ID=<chart-target> tv ui eval '<read-only chart metadata summary>'

Implement the alert API helper in `src/ops/alert.rs`. Then run focused tests:

    cargo test alert -- --nocapture
    cargo test --test cli_contract alert -- --nocapture

If focused tests pass, run one live smoke with a clear disposable message:

    TV_CDP_TARGET_ID=<chart-target> tv alert create --price <TEST_PRICE> --condition crossing --message "tv-cli smoke alert <short suffix>"
    TV_CDP_TARGET_ID=<chart-target> tv alert delete --id <RETURNED_ALERT_ID>

Only record the message pattern and cleanup result in tracked docs. Do not record existing alert ids or raw alert payloads.

## Validation and Acceptance

Acceptance for the code change is:

- `tv alert create` validates non-finite price and unsupported condition before CDP connection as before.
- API-backed success includes the requested price, existing user-facing condition string, message, `price_set: true`, `created: true`, `source` identifying the API path, and the new alert id when available.
- API-backed create does not report success unless post-create alert list readback confirms a new matching alert.
- API pre-mutation unavailability can fall back to the existing DOM path.
- API post-check failure does not fall back to DOM and does not risk duplicate alert creation.
- Existing `alert list`, `alert delete --id`, and `alert delete --all` behavior remains compatible, with delete now using the endpoint shape verified during live cleanup.

Validation commands:

    cargo test alert -- --nocapture
    cargo test --test cli_contract alert -- --nocapture
    cargo fmt --check
    cargo clippy --all-targets --all-features -- -D warnings
    cargo test
    git diff --check
    git grep -nE '(/Users/|C:\\|USER;|sessionid|cookie|authorization|bearer)' -- README.md CHANGELOG.md docs .agents/skills || true

Validation result: all commands above passed on 2026-04-27. The tracked-doc
grep returned only existing validation-command examples and public-safe policy
language, not live account identifiers or credentials.

## Idempotence and Recovery

Alert creation is not idempotent. A retry may create another alert if the previous create request succeeded but the CLI did not observe the readback. For that reason, post-mutation ambiguity must return an error and must not run DOM fallback. Live smoke must delete the returned test alert id in the same run. If cleanup fails, record only the new alert's minimal identifying message and returned id in `CONTINUITY.md`, not in tracked docs.

## Artifacts and Notes

Public-safe evidence gathered before implementation:

    alert list endpoint: reachable with explicit chart target
    active chart metadata: symbol, resolution, and currency summary fields available
    upstream create endpoint evidence: cross-origin POST to create_alert with plain string JSON body and no Content-Type

## Interfaces and Dependencies

The implementation should add this helper in `src/ops/alert.rs`:

    pub async fn alert_create_via_api(
        runtime: &mut impl RuntimeEvaluator,
        price: f64,
        condition: &str,
        message: Option<&str>,
    ) -> Result<Value, AppError>

No new crate dependencies are required. The helper uses `RuntimeEvaluator::evaluate` with `await_promise: true` to run `fetch()` inside the authenticated TradingView page context.

## Open Questions

There are no unresolved questions for this slice. Broader alert edit/pause/resume behavior remains future feature research and is not part of this plan.
