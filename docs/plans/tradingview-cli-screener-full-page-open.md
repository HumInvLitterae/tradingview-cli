# Screener full-page open

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This plan follows `.agents/PLANS.md` in this repository.

## Purpose / Big Picture

`tv screener open` currently opens the Stock Screener drawer inside a chart page. That is useful for visible UI operations, but storage-backed Screener work is more reliable against a full-page Screener target, which appears as its own Chrome DevTools Protocol page with a `tradingview.com/screener` URL. After this change, an operator can run `tv screener open --full-page`, receive `target_cli_args`, and then run commands such as `tv --target-id <ID> screener filters list` without manually preparing the full-page Screener tab.

The existing `tv screener open` behavior must remain unchanged unless `--full-page` is specified.

## Progress

- [x] (2026-04-29) Created this ExecPlan and archived the completed Screener filter storage mutation audit plan.
- [x] (2026-04-29) Added local CDP target creation helpers and shared full-page Screener target recognition.
- [x] (2026-04-29) Added `tv screener open --full-page` with existing-target reuse and CDP target creation attempt.
- [x] (2026-04-29) Live-probed CDP target creation and confirmed TradingView Desktop returns `Could not create new page`.
- [x] (2026-04-29) Updated README, internal API reference, roadmap, changelog, plans index, and local continuity ledger.
- [x] (2026-04-29) Ran focused tests, full workspace validation, hygiene grep, and bounded live smoke.
- [ ] Commit the implementation and docs update.

## Surprises & Discoveries

- Observation: The existing `screener open` implementation only clicks `[data-name="screener-dialog-button"]` inside a chart page.
  Evidence: `crates/cli/src/ops/screener/state.rs` uses `SCREENER_OPEN_EXPRESSION`, checks `button_found`, and then requires the drawer to be open.

- Observation: `tab list` already has full-page Screener URL detection.
  Evidence: `crates/cli/src/ops/tab.rs` classifies page targets whose URL contains `tradingview.com/screener` as `screener_targets`.

- Observation: TradingView Desktop's local CDP endpoint does not currently create a Screener page target through the standard new-target endpoint.
  Evidence: live probes against `/json/new` returned HTTP 500 with body `Could not create new page`.

- Observation: The app-tab UI can create a blank TradingView Desktop app tab, but that blank tab did not expose a new chart or Screener CDP page target.
  Evidence: `tv tab new --from 0` increased `app_tabs` but `tab list` still showed no `screener_targets`.

## Decision Log

- Decision: Add `--full-page` to the existing `open` command instead of adding a new subcommand.
  Rationale: The command remains conceptually "open Screener"; the flag chooses drawer versus full-page target. This preserves the old default while making the desired target explicit.
  Date/Author: 2026-04-29 / Codex.

- Decision: Prefer CDP HTTP target creation over app-tab UI automation.
  Rationale: Opening a local CDP page target through the DevTools endpoint is a narrower and more deterministic operation than clicking TradingView Desktop's app tab UI. UI fallback should not be added unless live evidence proves the CDP target creation endpoint is unavailable.
  Date/Author: 2026-04-29 / Codex.

- Decision: Do not add a generic UI fallback in this slice after CDP new-target failure.
  Rationale: Live evidence showed that app-tab UI creation produces a blank Desktop tab without a Screener CDP target, and app-window menu exploration did not reveal a stable Screener command. The command should reuse existing full-page targets and fail clearly when automatic creation is unavailable rather than pretending to open one.
  Date/Author: 2026-04-29 / Codex.

## Outcomes & Retrospective

Partial implementation completed. The command now supports `tv screener open --full-page` and reuses an existing full-page Screener target when one is already open. It also attempts CDP target creation and returns a structured error with a manual-open hint when TradingView Desktop rejects the standard new-target endpoint. Automatic full-page creation remains unavailable in the current Desktop evidence.

The durable value of this slice is still useful for operator workflow: once a full-page Screener tab is open, the CLI can rediscover it, activate it, and return `target_cli_args` for subsequent `tv --target-id <ID> screener ...` commands. The command no longer leaves this handoff entirely manual. The non-goal is also clearer: without a stable Desktop app-window route, the CLI should not add fragile generic UI fallback just to create the tab.

## Context and Orientation

The `tv` binary parses command-line flags in `crates/cli/src/cli.rs`. Application dispatch lives in `crates/cli/src/app/dispatch.rs`. Screener operations live under `crates/cli/src/ops/screener/`; `state.rs` owns `screener_status`, `screener_open`, `screener_get`, and `screener_close`.

The `tradingview-cdp` crate in `crates/cdp/` owns local Chrome DevTools Protocol target discovery. It already has `TransportConfig::list_url()` and `TransportConfig::activate_url()`. This plan adds a small local CDP helper for creating a new page target and a reusable helper for recognizing full-page Screener targets. This is local Desktop target management, not a TradingView account API.

## Plan of Work

First, add a CDP transport helper that can build and call the local DevTools new-target endpoint for a URL. The function should be named `new_target_url(config: &TransportConfig, url: &str) -> Result<Target, AppError>`. It should send a `PUT` request to `/json/new?<encoded-url>`, parse the returned target, and return a structured `Connection` or `InternalApiUnavailable` error if the endpoint fails or returns an unusable target. Add `TransportConfig::new_target_url(&self, url: &str) -> String`.

Second, move full-page Screener URL recognition into `tradingview-cdp` as `is_screener_target(target: &Target) -> bool`. Update `tab list` to use this helper so the recognition logic is not duplicated.

Third, change `ScreenerCommand::Open` in `crates/cli/src/cli.rs` from a unit variant to `Open { full_page: bool }`. In `app/dispatch`, route `Open { full_page: false }` through the existing runtime-backed drawer open path and route `Open { full_page: true }` to a new config-backed operation, `ops::screener_open_full_page(config)`.

Fourth, implement `screener_open_full_page` in `crates/cli/src/ops/screener/state.rs`. It should fetch targets, reuse and activate the first existing full-page Screener target if present, or call `new_target_url` with `https://www.tradingview.com/screener/` and then poll `fetch_targets` briefly until a full-page Screener target appears. The success payload must include `source`, `action: "open_full_page"`, `full_page: true`, `created`, `reused`, `target_id`, `target_cli_args`, `url`, and `title`. If target creation fails before a full-page target exists, return a structured error that includes a manual-open hint. Do not add UI fallback without a stable app-window command path.

Fifth, update README, `docs/internal-tradingview-apis.md`, `docs/v0.3-roadmap.md`, `CHANGELOG.md`, and `docs/plans/README.md`. Update `CONTINUITY.md` as a local ledger but do not stage it.

## Concrete Steps

Run these commands from the repository root:

    cargo test -p tradingview-cdp transport -- --nocapture
    cargo test -p tradingview-cli screener::state -- --nocapture
    cargo test -p tradingview-cli tab -- --nocapture
    cargo test -p tradingview-cli --test cli_contract screener -- --nocapture
    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1
    git diff --check

For live smoke, use:

    target/debug/tv tab list
    target/debug/tv screener open --full-page
    target/debug/tv --target-id <screener-target> screener status
    target/debug/tv --target-id <screener-target> screener filters list

Do not write the live target id into tracked docs.

## Validation and Acceptance

This slice is accepted when `tv screener open` still opens the chart drawer with its existing payload shape, while `tv screener open --full-page` returns a full-page Screener target handoff when such a target already exists. If no full-page target exists and the CDP new-target endpoint succeeds, the command must report `created: true`. If CDP target creation fails or a new target cannot be found after creation, the command must fail with a structured error and must not report success.

Tests must cover help visibility, existing-target reuse, new-target success payload, and failure when no Screener target appears after creation.

Validation run on 2026-04-29:

    cargo test -p tradingview-cdp transport -- --nocapture
    cargo test -p tradingview-cli screener::state -- --nocapture
    cargo test -p tradingview-cli tab -- --nocapture
    cargo test -p tradingview-cli --test cli_contract screener -- --nocapture
    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1
    git diff --check

All passed. The tracked-doc hygiene grep also passed with only existing policy/archive references.

## Idempotence and Recovery

Running `tv screener open --full-page` repeatedly should be idempotent once a full-page Screener target exists because the command reuses the existing target. The command may create an app tab when none exists; that tab is not account data and can be closed manually or with `tv tab close` if needed. If target creation fails, the command should leave no saved Screener state behind.

## Artifacts and Notes

Record only scrubbed live evidence. Use `<screener-target>` in this plan for target ids. Do not include raw target ids, account-local ids, local absolute paths, cookies, tokens, or raw live payloads in tracked docs.

## Interfaces and Dependencies

The end state should include:

    impl TransportConfig {
        pub fn new_target_url(&self, url: &str) -> String;
    }

    pub async fn new_target_url(config: &TransportConfig, url: &str) -> Result<Target, AppError>;
    pub fn is_screener_target(target: &Target) -> bool;

    pub async fn screener_open_full_page(config: &TransportConfig) -> Result<Value, AppError>;

`screener_open_full_page` should use only `tradingview-cdp` target management helpers and should not use generic UI automation.

## Open Questions

- Confirmed current local evidence: TradingView Desktop returned HTTP 500 for `PUT /json/new` full-page Screener creation.
- UNCONFIRMED: Whether another stable app-window command or future Desktop build can open a full-page Screener target without manual setup.
