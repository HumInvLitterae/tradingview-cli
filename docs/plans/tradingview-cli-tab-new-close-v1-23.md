# Add TradingView tab new and close commands

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document follows `.agents/PLANS.md`. It is intentionally self-contained so a future contributor can continue the work from this file and the repository tree alone.

## Purpose / Big Picture

After this change, operators can use the Rust-native `tv` CLI to open a new TradingView Desktop app tab and close a specific TradingView Desktop app tab without falling back to the old JavaScript CLI. This completes the low-risk tab lifecycle surface around the existing `tv tab list` and `tv tab switch` commands.

The observable behavior is:

- `tv tab new --from 0` activates chart tab index `0`, clicks the TradingView Desktop app tab-strip new-tab button, waits briefly, and reports the increased app-tab count plus the new app-tab candidate.
- `tv tab close <INDEX>` closes the requested TradingView Desktop app tab, waits briefly, and reports the decreased app-tab count.
- `tv tab close` requires an explicit index and refuses to close the last remaining TradingView app tab. This is a Rust-native safety deviation from the old CLI, which closed the current tab.

## Progress

- [x] (2026-04-24 00:00Z) Read current `src/ops/tab.rs`, `src/cdp.rs`, `src/cli.rs`, `src/main.rs`, and `src/ops.rs` to confirm the existing tab list/switch implementation and CDP keyboard event shape.
- [x] (2026-04-24 00:00Z) Created this ExecPlan with the intended CLI, safety rules, test scope, and validation commands.
- [x] (2026-04-24 00:00Z) Add `modifiers` support to CDP keyboard events and update existing call sites.
- [x] (2026-04-24 00:00Z) Implement `tv tab new [--from <INDEX>]` and `tv tab close <INDEX>` in the tab operation module.
- [x] (2026-04-24 00:00Z) Add unit and CLI tests for source selection, close safety, help output, and connection errors.
- [x] (2026-04-24 00:00Z) Update README, repository guide, migration inventory, contract notes, lifecycle audit, and next-agent handoff note.
- [x] (2026-04-24 00:00Z) Run formatting, linting, tests, diff checks, and tracked-doc local path scan. The full baseline passed.
- [x] (2026-04-24 00:00Z) Run live TradingView Desktop smoke for `tab list`, `tab new --from 0`, and closing the newly created tab.
- [x] (2026-04-24 00:00Z) Commit the completed implementation.

## Surprises & Discoveries

- Observation: The existing CDP `KeyEvent` type did not include modifier keys even though the old JavaScript `tab new` and `tab close` commands depend on Cmd or Ctrl shortcuts.
  Evidence: `src/cdp.rs` sends `type`, `key`, `code`, and `windowsVirtualKeyCode` only.

- Observation: The existing tests use `AppError` fields directly rather than accessor methods.
  Evidence: New tab helper tests assert `error.kind` and `error.message`.

- Observation: The first live smoke attempt on TradingView Desktop did not open a chart target with either Cmd or Ctrl keyboard shortcuts, even though the old JavaScript CLI sent those shortcuts.
  Evidence: `cargo run --quiet -- tab new --from 0` returned `internal_api_unavailable` with `tabs_before: 1` and `tabs_after: 1`.

- Observation: TradingView Desktop has a separate app-window target whose DOM contains the visible tab strip. Clicking `button.create-new-tab-button` creates a visible app tab, but that blank app tab does not become a `tradingview.com/chart` CDP page target even after waiting 10 seconds.
  Evidence: live probing showed app tabs `[LWLG..., 新規タブ]` while `tv tab list` still reported one chart target.

## Decision Log

- Decision: `tv tab close` takes an explicit zero-based app-tab index and refuses to close the final app tab.
  Rationale: Closing the current tab is easy to misfire in automation. An explicit target and last-tab guard provide a safer Rust-native contract while still replacing the useful old CLI behavior.
  Date/Author: 2026-04-24 / Codex.

- Decision: `tv tab new` allows omitted `--from` only when there is exactly one chart tab.
  Rationale: With multiple chart tabs, implicit source selection is ambiguous. Requiring `--from` avoids opening from the wrong chart context.
  Date/Author: 2026-04-24 / Codex.

- Decision: Use CDP keyboard shortcuts rather than a `/json/new` or `/json/close` endpoint.
  Rationale: The old CLI used platform shortcuts, and the current repository already has target activation and keyboard dispatch primitives. The CDP close endpoint would be more direct but changes behavior and may not activate the same TradingView chart context first.
  Date/Author: 2026-04-24 / Codex.

- Decision: Use the TradingView Desktop app-window tab-strip DOM for `tab new` and `tab close`, while preserving chart-target fields in `tab list` and `tab switch`.
  Rationale: Live evidence showed keyboard shortcuts do not create CDP chart targets in this environment, while the app-window DOM exposes stable tab strip controls for creating and closing visible TradingView tabs. The Rust CLI keeps old practical chart-target information and adds app-tab information needed for cleanup.
  Date/Author: 2026-04-24 / Codex.

## Outcomes & Retrospective

Implemented `tv tab new [--from <INDEX>]` and `tv tab close <INDEX>`. The implementation preserves chart-target tab list/switch behavior and adds app-tab awareness for the TradingView Desktop tab strip because live evidence showed newly opened tabs appear first as app tabs, not as `tradingview.com/chart` CDP page targets.

Live smoke opened a new app tab at index `1` with title `新規タブ`, then closed index `1`. The final `tab list` returned to one app tab and one chart target.

## Context and Orientation

This repository implements a Rust-native TradingView Desktop CLI named `tv`. The CLI talks to TradingView Desktop through Chrome DevTools Protocol, abbreviated CDP. CDP exposes browser targets, which are open pages or workers. TradingView chart tabs are CDP page targets whose URL includes `tradingview.com/chart`.

The existing tab surface lives in `src/ops/tab.rs`. It already implements:

- `tab_list(config)`, which fetches CDP targets, filters TradingView chart page targets, assigns zero-based indexes, and returns `tab_count` plus `tabs`.
- `tab_switch(config, index)`, which validates the index and calls the CDP HTTP activation endpoint for that target.

The command-line argument shape lives in `src/cli.rs`, and command dispatch lives in `src/main.rs`. The operation facade in `src/ops.rs` re-exports public operation functions from capability modules.

Keyboard input is represented by `src/cdp.rs::KeyEvent` and sent by `RuntimeEvaluator::dispatch_key_event`. Existing call sites are in `src/ops/layout.rs` for Enter and Escape. The old JavaScript CLI used Cmd+T or Ctrl+T to open a tab, and Cmd+W or Ctrl+W to close a tab. Live testing showed these keyboard shortcuts do not create chart targets in this TradingView Desktop environment, so the implemented tab lifecycle uses the app-window tab-strip DOM instead.

## Plan of Work

First, extend `src/cdp.rs::KeyEvent` with a `modifiers: i64` field and include it in the `Input.dispatchKeyEvent` payload. Update every existing `KeyEvent` construction to set `modifiers: 0`.

Next, update `src/ops/tab.rs`. Add `tab_new(config, from)` and `tab_close(config, index)`. Both commands fetch targets, derive `ChartTab` values with `chart_tabs_from_targets`, and connect to the TradingView Desktop app-window target whose URL contains `/app/window/index.html`. `tab_new` activates the selected source chart target and clicks `button.create-new-tab-button` in the app-window DOM. `tab_close` clicks the close button for the requested app-tab index in the app-window DOM. Both commands wait briefly and verify app-tab counts afterward.

For `tab_new`, resolve the source chart tab as follows. If `--from` is provided, it must be a valid chart tab index. If it is omitted and exactly one chart tab exists, use index `0`. If omitted and multiple chart tabs exist, return a validation error asking for `--from`. After clicking the app tab-strip new-tab button, verify that the app-tab count increased. The JSON payload should include `action`, `source_index`, `source_tab`, `tabs_before`, `tabs_after`, `app_tabs_before`, `app_tabs_after`, `new_app_tabs`, `chart_tabs_before`, `chart_tabs_after`, and `new_tabs`.

For `tab_close`, validate that at least two app tabs exist and that the requested app-tab index exists. After clicking that app tab's close button, verify that the app-tab count decreased. The JSON payload should include `action`, `closed_index`, `closed_tab`, `tabs_before`, `tabs_after`, `app_tabs_before`, `app_tabs_after`, `chart_tabs_before`, and `chart_tabs_after`.

Then update `src/cli.rs`, `src/main.rs`, and `src/ops.rs` to expose the new commands:

- `tv tab new [--from <INDEX>]`
- `tv tab close <INDEX>`

Finally, update durable docs to mark `tab new` and `tab close` implemented and to record the explicit-index safety deviation.

## Concrete Steps

Run commands from the repository root.

Edit:

- `src/cdp.rs`
- `src/ops/layout.rs`
- `src/ops/tab.rs`
- `src/cli.rs`
- `src/main.rs`
- `src/ops.rs`
- `README.md`
- `AGENTS.md`
- `docs/notes/legacy-cli-command-migration-inventory-2026-04-24.md`
- `docs/notes/rust-cli-contract-migration-2026-04-24.md`
- `docs/notes/command-lifecycle-balance-audit-2026-04-24.md`
- `docs/notes/next-agent-handoff-prompt-2026-04-24.md`

After edits, run:

    cargo fmt --check
    cargo clippy --all-targets --all-features -- -D warnings
    cargo test
    git diff --check
    git grep -nE '(/Users/|C:\\)' -- README.md AGENTS.md docs .agents/skills || true

For live smoke with TradingView Desktop running and CDP enabled, run:

    cargo run --quiet -- tab list
    cargo run --quiet -- tab new --from 0
    cargo run --quiet -- tab list
    cargo run --quiet -- tab close <NEW_INDEX>
    cargo run --quiet -- tab list

The final tab count should equal the starting tab count. If closing the new tab fails, record the new tab index here and stop without broad UI automation.

## Validation and Acceptance

Automated acceptance:

- `cargo fmt --check` succeeds.
- `cargo clippy --all-targets --all-features -- -D warnings` succeeds.
- `cargo test` succeeds.
- `git diff --check` succeeds.
- The tracked-doc local path scan prints no tracked-document match.

Behavioral acceptance:

- `tv tab --help` lists `list`, `switch`, `new`, and `close`.
- `tv tab new --help` lists `--from`.
- `tv tab close` without an index exits with a validation error from clap.
- With a bad CDP port, `TV_CDP_PORT=9 tv tab new` and `TV_CDP_PORT=9 tv tab close 0` fail as connection errors rather than panicking.
- In a live TradingView Desktop session, `tab new --from 0` increases the app-tab count by at least one, and `tab close <NEW_INDEX>` decreases it.

## Idempotence and Recovery

The code and documentation edits are ordinary source changes and can be rerun safely. The live smoke temporarily opens a new TradingView app tab and then closes the tab that was just opened. If the close step fails, do not attempt generic UI automation; leave the index of the created tab in this plan or the final report so a human can close it intentionally.

`tv tab close` refuses to close the last remaining app tab. This prevents the smoke test or downstream automation from removing the only active TradingView Desktop tab context.

## Artifacts and Notes

Initial automated test run:

    cargo test
    test result: ok. 117 passed; 0 failed
    test result: ok. 39 passed; 0 failed

Final automated baseline:

    cargo fmt --check
    cargo clippy --all-targets --all-features -- -D warnings
    cargo test
    git diff --check
    git grep -nE '(/Users/|C:\\)' -- README.md AGENTS.md docs .agents/skills || true

All commands succeeded. The tracked-doc local path scan printed no matches.

Live smoke summary:

    cargo run --quiet -- tab list
    app_tab_count: 1, tab_count: 1

    cargo run --quiet -- tab new --from 0
    action: new_tab_opened
    tabs_before: 1
    tabs_after: 2
    new_app_tabs[0].index: 1
    new_app_tabs[0].title: 新規タブ

    cargo run --quiet -- tab close 1
    action: tab_closed
    tabs_before: 2
    tabs_after: 1

    cargo run --quiet -- tab list
    app_tab_count: 1, tab_count: 1

## Interfaces and Dependencies

At completion, `src/ops/tab.rs` must expose:

    pub async fn tab_new(config: &TransportConfig, from: Option<usize>) -> Result<Value, AppError>
    pub async fn tab_close(config: &TransportConfig, index: usize) -> Result<Value, AppError>

At completion, `src/cli.rs::TabCommand` must include:

    New { from: Option<usize> }
    Close { index: usize }

At completion, `src/cdp.rs::KeyEvent` must include:

    pub modifiers: i64

The concrete `CdpClient` must pass that field to CDP as `modifiers`.

## Open Questions

No unresolved critical questions remain for this slice.
