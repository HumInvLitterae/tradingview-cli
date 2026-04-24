# Add watchlist remove command

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document follows `.agents/PLANS.md` from the repository root.

## Purpose / Big Picture

The Rust `tv` CLI already implements `tv watchlist add <SYMBOL>`, which can change the user's active TradingView account watchlist. After this change, a user or downstream adapter can run `tv watchlist remove <SYMBOL>` to clean up a specific symbol without leaving the Rust CLI.

This is not a direct old JavaScript CLI migration item. The old JavaScript CLI exposed `watchlist get` and `watchlist add`, but not `watchlist remove`. This command exists because Rust now has a watchlist mutation and needs an operator cleanup path. The command must be stricter than add: it must prove the target symbol exists before deletion and prove it is absent afterward, or fail clearly without touching unrelated rows.

## Progress

- [x] (2026-04-24 00:00Z) Confirmed the worktree was clean before starting.
- [x] (2026-04-24 00:00Z) Inspected existing `watchlist get` and `watchlist add` implementation in `src/ops/layout.rs`.
- [x] (2026-04-24 00:00Z) Created this ExecPlan and recorded the safety contract.
- [x] (2026-04-24 00:00Z) Added `tv watchlist remove <SYMBOL>` CLI and dispatch.
- [x] (2026-04-24 00:00Z) Implemented row-scoped `watchlist_remove`.
- [x] (2026-04-24 00:00Z) Added unit and CLI contract tests.
- [x] (2026-04-24 00:00Z) Updated durable docs for implemented lifecycle balance.
- [x] (2026-04-24 00:00Z) Ran initial validation baseline: `cargo fmt --check`, `cargo clippy --all-targets --all-features`, `cargo test`, `git diff --check`, and tracked-doc local absolute path scan passed before the live-smoke fix.
- [x] (2026-04-24 00:00Z) Ran live smoke with `NASDAQ:AAPL`: add succeeded, the first remove implementation failed safely because the symbol remained, the row-button-only fix succeeded, and a follow-up `watchlist get` no longer showed `NASDAQ:AAPL`.
- [x] (2026-04-24 00:00Z) Re-ran final validation baseline after the live-smoke fix: `cargo fmt --check`, `cargo clippy --all-targets --all-features`, and `cargo test` passed.
- [x] (2026-04-24 00:00Z) Committed implementation and tests as `d96433c feat(cli): Add watchlist remove command`.
- [x] (2026-04-24 00:00Z) Committed docs as `docs(plans): Record watchlist remove slice`.

## Surprises & Discoveries

- Observation: The current localized TradingView UI may expose the watchlist button with a Japanese aria label.
  Evidence: A read-only CDP DOM inspection showed a right-panel button with aria label `ウォッチリスト・詳細・ニュース`.

- Observation: The current live session can have the right panel open without visible watchlist rows.
  Evidence: `cargo run --quiet -- watchlist get` returned `count: 0`, `source: "empty"`, and `symbols: []` while the right panel was visible.

- Observation: A broad context-menu fallback is unsafe for this command.
  Evidence: The first live remove attempt returned `internal_api_unavailable` because `NASDAQ:AAPL` remained present, and DOM inspection showed unrelated visible delete menu items outside the watchlist row. The implementation now ignores context menus and uses only a remove icon inside the matched row.

- Observation: TradingView exposes the matched row's delete icon as a row-local element with a `removeButton` class, but it may fail ordinary visibility checks.
  Evidence: Read-only DOM inspection after hovering the `NASDAQ:AAPL` row showed `<span class="... removeButton-...">` inside the matched row. The final live smoke succeeded with `remove_method: "row_remove_button"`.

## Decision Log

- Decision: Treat `watchlist remove` as a Rust-native operator cleanup command, not old CLI parity.
  Rationale: The old JavaScript CLI did not expose a remove command, but Rust now exposes `watchlist add`; cleanup is needed for safe live smoke and downstream operator workflows.
  Date/Author: 2026-04-24 / Codex

- Decision: Require exact `data-symbol-full` matching before any delete interaction.
  Rationale: Watchlist deletion is account state mutation. Partial matches, display-name matches, or search-style matches could delete the wrong row.
  Date/Author: 2026-04-24 / Codex

- Decision: Do not use bulk delete, generic UI automation, or an unscoped Delete key path.
  Rationale: The command must only affect the requested symbol row. If a stable row-scoped delete path cannot be found, the command should fail rather than risk removing unrelated watchlist entries.
  Date/Author: 2026-04-24 / Codex

- Decision: Do not keep a context-menu fallback.
  Rationale: The context menu is difficult to scope reliably in the live TradingView DOM, and the page may contain unrelated delete menu items. A row-local `removeButton` icon is more conservative and easier to verify.
  Date/Author: 2026-04-24 / Codex

## Outcomes & Retrospective

The implementation adds `tv watchlist remove <SYMBOL>` as an exact-match cleanup command for `watchlist add`. Automated tests cover absent symbols, missing remove controls, failed post-delete verification, and the success payload. Live smoke added `NASDAQ:AAPL`, removed it through the row-local remove icon, and confirmed it no longer appeared in `watchlist get`.

## Context and Orientation

This repository implements a Rust-native command-line binary named `tv`. The entrypoint in `src/main.rs` parses commands, connects to TradingView Desktop through Chrome DevTools Protocol, calls operation functions re-exported from `src/ops.rs`, and prints JSON envelopes.

Chrome DevTools Protocol, abbreviated CDP, is the local debugging protocol exposed by TradingView Desktop when it runs with a remote debugging port. Runtime JavaScript evaluation is abstracted by the `RuntimeEvaluator` trait in `src/cdp.rs`; unit tests use fake runtimes from `src/ops/test_support.rs` so tests do not require TradingView Desktop.

The watchlist operations live in `src/ops/layout.rs` beside pane operations. `watchlist get` reads visible watchlist rows from the right panel. `watchlist add` opens the watchlist panel if needed, clicks the add-symbol button, types a symbol through CDP input events, and confirms it. `watchlist remove` should live in the same module because it operates on the same right-panel watchlist surface.

## Plan of Work

First, extend the CLI surface in `src/cli.rs` by adding `WatchlistCommand::Remove { symbol: String }`. Update `src/main.rs` dispatch so an empty or whitespace-only symbol returns a validation error before connecting to CDP, then call `ops::watchlist_remove`.

Next, update the operation facade in `src/ops.rs` to re-export `watchlist_remove`. Keep the Rust 2024 module layout unchanged: `src/ops.rs` remains the facade and no `mod.rs` file is introduced.

Then, implement `watchlist_remove` in `src/ops/layout.rs`. Reuse or extract a private helper that opens the watchlist panel. The helper should continue supporting the existing English watchlist selectors and also allow localized labels by matching either `Watchlist` or `ウォッチリスト` in `aria-label`.

The operation should evaluate one row-scoped JavaScript block. It must read the right-panel rows, find a row whose `data-symbol-full` exactly equals the requested symbol, and return a validation error payload if no row matches. If a row matches, it should reveal that row's hover controls and click only a remove/delete control inside that matched row. It must not use context menus, global delete controls, bulk delete, or a generic Delete key. It must then re-read the right-panel rows and succeed only if the requested symbol is absent.

Finally, update tests and documentation. Unit tests should prove that exact-match deletion succeeds, absent symbols are validation errors, missing row-scoped controls are internal API errors, and a failed post-delete verification is an internal API error. CLI contract tests should cover help, missing symbol validation, and structured connection errors. Docs should move the lifecycle note from "gap exists" to "gap resolved by watchlist remove".

## Concrete Steps

From the repository root, run the usual implementation loop:

    cargo fmt --check
    cargo clippy --all-targets --all-features
    cargo test
    git diff --check
    tracked-doc local absolute path scan

If `cargo fmt --check` fails only because of formatting, run `cargo fmt` and repeat the baseline.

If a TradingView Desktop CDP session is available and a row-scoped delete path is implemented, run a live smoke with a symbol that is absent before the test:

    cargo run --quiet -- watchlist get
    cargo run --quiet -- watchlist add <SYMBOL>
    cargo run --quiet -- watchlist get
    cargo run --quiet -- watchlist remove <SYMBOL>
    cargo run --quiet -- watchlist get

Record the chosen symbol and observed JSON in this plan. Do not delete a symbol that existed before the smoke test.

## Validation and Acceptance

The change is accepted when `tv watchlist remove <SYMBOL>` appears in help, rejects a missing or empty symbol before connecting, returns structured connection errors when CDP is unavailable, and passes all unit and CLI contract tests.

The success JSON must use the Rust envelope:

    {
      "success": true,
      "command": "watchlist",
      "data": {
        "symbol": "NASDAQ:AAPL",
        "requested_symbol": "NASDAQ:AAPL",
        "action": "removed",
        "removed": true
      }
    }

Additional fields may be present under `data`, such as `source`, `before_count`, and `after_count`.

## Idempotence and Recovery

Code changes and automated tests are safe to rerun. The automated tests must use fake runtimes and must not mutate a real TradingView watchlist.

The live smoke mutates the active TradingView account watchlist. It is only safe when the symbol was absent before the test and was added specifically for this smoke. If removal fails after add succeeds, record the symbol in this plan and stop; do not try broad UI automation or bulk cleanup.

## Artifacts and Notes

Initial read-only live context:

    cargo run --quiet -- watchlist get
    result: success true, count 0, source empty, symbols []

Read-only DOM inspection:

    result: right panel visible, localized watchlist button aria label includes ウォッチリスト

Live smoke:

    cargo run --quiet -- watchlist remove TVCLI:SMOKE_SENTINEL
    result: validation, Watchlist symbol not found, before_count 31

    cargo run --quiet -- watchlist add NASDAQ:AAPL
    result: success true, action added, source dom_input

    cargo run --quiet -- watchlist get | rg 'NASDAQ:AAPL|"count"|"source"'
    result: count 53, source data_attributes, NASDAQ:AAPL present

    cargo run --quiet -- watchlist remove NASDAQ:AAPL
    result: first implementation failed safely, internal_api_unavailable, matched_after true

    cargo run --quiet -- watchlist remove NASDAQ:AAPL
    result after row-remove-button fix: success true, action removed, removed true, matched_after false, remove_method row_remove_button

    cargo run --quiet -- watchlist get | rg 'NASDAQ:AAPL|"count"|"source"'
    result: count 53, source data_attributes, NASDAQ:AAPL absent

Final validation:

    cargo fmt --check
    result: passed

    cargo clippy --all-targets --all-features
    result: passed

    cargo test
    result: ok. 73 unit tests and 25 CLI contract tests passed.

## Interfaces and Dependencies

At the end of the implementation, the following command must exist:

    tv watchlist remove <SYMBOL>

The operation facade must expose:

    pub async fn watchlist_remove(runtime: &mut impl RuntimeEvaluator, symbol: &str) -> Result<Value, AppError>

No new third-party Rust dependencies are required.

## Open Questions

There are no unresolved critical questions at the plan stage. If implementation cannot find a safe row-scoped delete path, record that as a blocker in `Surprises & Discoveries` and do not ship a risky deletion command.
