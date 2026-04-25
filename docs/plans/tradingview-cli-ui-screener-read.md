# Add UI Screener read commands

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document follows `.agents/PLANS.md`.

## Purpose / Big Picture

After this change, a user can inspect TradingView's built-in Stock Screener dialog from the Rust `tv` CLI without using the unsafe generic `tv ui eval` command or manually scraping the UI. The new commands are `tv screener status`, `tv screener open`, `tv screener get [--limit <N>]`, and `tv screener close`. They are intentionally read-oriented: they may open or close the visible Screener dialog, but they must not remove filters, save screens, change columns, or persist TradingView account state.

This is separate from `tv scanner hotlist`, which reads a TradingView scanner preset REST endpoint. The new `tv screener` commands read localized display text from the currently visible TradingView Desktop UI through Chrome DevTools Protocol, abbreviated CDP. CDP is the local debugging interface exposed by TradingView Desktop when launched with remote debugging enabled.

## Progress

- [x] (2026-04-26 02:10Z) Read `.agents/PLANS.md`, existing scanner implementation, CLI dispatch, contract notes, and the live UI Screener evidence note.
- [x] (2026-04-26 02:22Z) Add `tv screener status/open/get/close` CLI surface and operation module.
- [x] (2026-04-26 02:25Z) Add unit and CLI contract tests.
- [x] (2026-04-26 02:30Z) Update README and migration/upstream notes.
- [x] (2026-04-26 02:44Z) Run automated validation and live smoke.
- [x] (2026-04-26 02:48Z) Record outcomes and prepare the completed slice for commit.

## Surprises & Discoveries

- Observation: The 2026-04-26 live evidence pass found that upstream PR #66's `[class*="screenerContainer"]` selector did not match the current TradingView Desktop UI.
  Evidence: `docs/notes/ui-screener-read-evidence-2026-04-26.md` records that `[class*="screener"]`, visible Screener heading text, Screener `data-name` attributes, and table presence did work.

- Observation: The first focused unit test run caught a restore-path test fixture mistake: `screener_get` calls `screener_close`, and `screener_close` first reads current state before sending Escape.
  Evidence: `cargo test screener -- --nocapture` initially failed because the fake runtime returned closed state too early; adding an explicit open-state response made the intended close path test pass.

- Observation: Live `get` needed to wait for visible row text after the dialog became open.
  Evidence: The first live `tv screener get --limit 3` smoke opened and restored the dialog but returned empty row cells. After adding a short row-text wait, live smoke returned three populated rows and restored the initial closed state.

## Decision Log

- Decision: Implement only `status`, `open`, `get`, and `close`.
  Rationale: These commands unblock useful read workflows while avoiding filter, screen, and column mutations that can persist TradingView account or screen state.
  Date/Author: 2026-04-26 / Codex.

- Decision: Put implementation in `src/ops/screener.rs` and expose it as a top-level `tv screener` command group.
  Rationale: `src/ops.rs` is a thin facade by repository convention, and `tv scanner hotlist` already owns REST scanner presets. A separate `screener` group keeps UI dialog reads distinct from REST scanner data.
  Date/Author: 2026-04-26 / Codex.

- Decision: `get` may temporarily open the Screener dialog and should restore the original open/closed state.
  Rationale: This is convenient for users, and the live evidence showed that opening the dialog is visible UI state only. Restoring the initial state keeps smoke tests and operator workflows tidy.
  Date/Author: 2026-04-26 / Codex.

- Decision: `close` uses Escape rather than trying to click the right-toolbar Screener button.
  Rationale: The live evidence pass showed that clicking the toolbar button did not close the dialog in that session, while Escape restored the original closed state safely.
  Date/Author: 2026-04-26 / Codex.

- Decision: `get` waits for row text, not only table presence.
  Rationale: TradingView can render the Screener table before the virtualized row text is populated. Waiting for non-empty row text prevents successful but useless empty row payloads.
  Date/Author: 2026-04-26 / Codex.

## Outcomes & Retrospective

Implemented. The Rust CLI now exposes `tv screener status`, `tv screener open`,
`tv screener get [--limit <N>]`, and `tv screener close`. The commands use the
standard Rust JSON envelope and read localized display data from the visible
TradingView Stock Screener dialog. Filter, screen, and column mutation remains
deferred.

Automated validation passed with `cargo fmt --check`, `cargo clippy
--all-targets --all-features -- -D warnings`, `cargo test`, and `git diff
--check`. The tracked-doc secret/path grep only matched existing validation
command examples. Live smoke against a TradingView Desktop target passed for
`status`, `get --limit 3`, `open`, and `close`; the dialog started closed and
ended closed.

## Context and Orientation

The command-line parser lives in `src/cli.rs`. Top-level commands are variants of `Command`, and grouped subcommands use a dedicated enum such as `ScannerCommand`.

The CLI dispatch lives in `src/main.rs`. Commands that need the TradingView page call `connect_runtime().await?`, then pass the connected runtime to an operation function under `src/ops/`.

Operation modules live under `src/ops/`. `src/ops.rs` declares modules and re-exports public operation functions. Tests for operation behavior usually live in the same module under `#[cfg(test)]` and use `src/ops/test_support.rs::FakeRuntime`.

The JSON output envelope is produced by `src/output.rs`. Operation functions return only the successful payload that appears under top-level `data`.

## Plan of Work

First, add `ScreenerCommand` to `src/cli.rs` with `status`, `open`, `get`, and `close`. `get` takes optional `--limit` / `-n`.

Next, add a `Command::Screener` dispatch arm in `src/main.rs`. It must validate `--limit 0` before connecting to CDP. It should connect to CDP for all Screener subcommands because they read or alter visible TradingView UI state.

Then create `src/ops/screener.rs`. It should expose `screener_status`, `screener_open`, `screener_get`, `screener_close`, and `validate_screener_limit`. The module should use JavaScript serialization helpers from `src/ops/common.rs` where user input is inserted into JavaScript. The implementation should not use `tv ui eval` or require `TV_ALLOW_UNSAFE_UI_EVAL`.

The status/read JavaScript should detect the Screener dialog using multiple live-backed indicators rather than only the stale upstream selector: visible Screener heading text, `[class*="screener"]`, Screener `data-name` attributes, and table presence. It should read the screen title, visible filter pill texts, visible table headers, and visible row cells. Row parsing should prefer table cell structure over whole-row text when cells are available.

Finally, update user-facing and agent-facing docs to describe the new read-only surface and the remaining deferred mutation surfaces.

## Concrete Steps

Run commands from the repository root.

1. Edit `src/cli.rs`, `src/main.rs`, `src/ops.rs`, and new `src/ops/screener.rs`.
2. Add tests in `src/ops/screener.rs` and `tests/cli_contract.rs`.
3. Update `README.md`, `docs/notes/rust-cli-contract-migration-2026-04-24.md`, `docs/notes/screener-hotlist-upstream-feasibility-2026-04-25.md`, `docs/notes/upstream-pr-triage-2026-04-25.md`, and `docs/notes/next-agent-handoff-prompt-2026-04-24.md`.
4. Run:

        cargo fmt --check
        cargo clippy --all-targets --all-features
        cargo test
        git diff --check
        git grep -nE '(/Users/|C:\\|USER;)' -- README.md AGENTS.md docs .agents/skills || true

5. If TradingView Desktop is available, run a restore-safe smoke:

        tv tab list
        TV_CDP_TARGET_ID=<target> tv screener status
        TV_CDP_TARGET_ID=<target> tv screener open
        TV_CDP_TARGET_ID=<target> tv screener get --limit 3
        TV_CDP_TARGET_ID=<target> tv screener close

If the initial `status` says the Screener is already open, leave it open after the smoke. If it says closed, verify that the final `status` is closed.

## Validation and Acceptance

The change is accepted when `tv --help` lists `screener`, `tv screener --help` lists `status`, `open`, `get`, and `close`, and `tv screener get --limit 0` fails with a validation error before CDP connection.

Operation tests must prove that closed and open state are mapped correctly, `get` normalizes headers, filters, and rows, and a temporary open caused by `get` is closed again. Automated baseline must pass. Live smoke, when available, must show a successful `get --limit 3` envelope with `source: "ui_screener_dialog"` and must restore the initial dialog open state.

## Idempotence and Recovery

The commands are designed to be repeatable. Running `tv screener open` when already open should be a no-op. Running `tv screener close` when already closed should be a no-op. If `get` opens the dialog and then fails, the implementation should still attempt to close it when the original state was closed.

If live smoke leaves the dialog open unexpectedly, run `tv screener close` or press Escape in TradingView Desktop.

## Artifacts and Notes

Validation output summary:

    cargo fmt --check
    # passed

    cargo clippy --all-targets --all-features -- -D warnings
    # passed

    cargo test
    # passed: 242 unit tests and 71 CLI contract tests

    git diff --check
    # passed

Live smoke summary:

    tv screener status
    # returned open: false, button_found: true

    tv screener get --limit 3
    # returned source: "ui_screener_dialog", row_count: 3, opened_for_read: true,
    # restored_open_state: false

    tv screener open
    tv screener close
    tv screener status
    # open and close succeeded; final status returned open: false

Raw Screener row payloads are intentionally not pasted into this tracked plan.

## Interfaces and Dependencies

At completion, `src/ops/screener.rs` must expose:

    pub fn validate_screener_limit(limit: Option<usize>) -> Result<usize, AppError>;
    pub async fn screener_status(runtime: &mut impl RuntimeEvaluator) -> Result<Value, AppError>;
    pub async fn screener_open(runtime: &mut impl RuntimeEvaluator) -> Result<Value, AppError>;
    pub async fn screener_get(runtime: &mut impl RuntimeEvaluator, limit: Option<usize>) -> Result<Value, AppError>;
    pub async fn screener_close(runtime: &mut impl RuntimeEvaluator) -> Result<Value, AppError>;

No new crate dependencies are required.

## Open Questions

None.
