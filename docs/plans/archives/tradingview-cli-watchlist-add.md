# Add watchlist add command

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document follows `.agents/PLANS.md` from the repository root.

## Purpose / Big Picture

The Rust `tv` CLI already reads the visible TradingView watchlist with `tv watchlist get`, but downstream operator workflows still have an old JavaScript CLI dependency for adding missing symbols. After this change, a user or downstream adapter can run `tv watchlist add <SYMBOL>` to add a symbol through the current TradingView Desktop session without returning to the JavaScript bridge.

The command is an explicit operator mutation. It changes the active TradingView account watchlist, so it must be small, documented, and easy to validate. The Rust JSON envelope remains unchanged, and the practical old CLI fields `symbol` and `action` remain available under `data`.

## Progress

- [x] (2026-04-24 00:00Z) Confirmed the worktree was clean before starting.
- [x] (2026-04-24 00:00Z) Ran `cargo test`; 46 unit tests and 18 CLI contract tests passed before implementation.
- [x] (2026-04-24 00:00Z) Add CDP input primitives for text insertion and key dispatch.
- [x] (2026-04-24 00:00Z) Add `tv watchlist add <SYMBOL>` CLI and operation implementation.
- [x] (2026-04-24 00:00Z) Update tests and durable docs.
- [x] (2026-04-24 00:00Z) Run the full validation baseline: `cargo fmt --check`, `cargo clippy --all-targets --all-features`, `cargo test`, `git diff --check`, and the tracked-doc local absolute path scan passed.
- [x] (2026-04-24 00:00Z) Run live smoke with `cargo run -- watchlist add NASDAQ:AAPL`; the command reached TradingView but failed with `internal_api_unavailable` because the watchlist button was not found in the current UI state.
- [x] (2026-04-24 00:00Z) Fix the live-smoke blocker by treating visible watchlist rows as proof that the watchlist panel is already open.
- [x] (2026-04-24 00:00Z) Re-ran live smoke with `cargo run -- watchlist add NASDAQ:AAPL`; the command succeeded with `action: "added"`.
- [ ] Commit implementation and docs in sensible batches.

## Surprises & Discoveries

- Observation: No unexpected behavior has been discovered yet.
  Evidence: Initial `cargo test` passed before implementation.

- Observation: The command can be tested without TradingView Desktop by extending the fake runtime to record CDP input text and key events.
  Evidence: The post-implementation `cargo test` run passed 49 unit tests and 19 CLI contract tests.

- Observation: The live TradingView UI in the smoke session did not expose the expected watchlist button selector.
  Evidence: `cargo run -- watchlist add NASDAQ:AAPL` returned `internal_api_unavailable` with message `Watchlist button not found`.

- Observation: `watchlist get` can read the active watchlist even when the widgetbar button selector is unavailable.
  Evidence: A later read returned `source: "data_attributes"` with 31 symbols and `ui-state` reported `right_panel.open: true`; the add flow now accepts visible `[data-symbol-full]` rows as an already-open watchlist.

## Decision Log

- Decision: Implement `watchlist add` before CI setup.
  Rationale: The current project priority is finishing old CLI migration surface. CI remains valuable, but local baseline validation is already being run consistently.
  Date/Author: 2026-04-24 / Codex

- Decision: Treat `watchlist add` as the next migration slice.
  Rationale: A sibling downstream workflow already invokes old CLI `watchlist add`; the command is small, pairs with existing `watchlist get`, and has a smaller side-effect surface than `alert create` or pane mutation.
  Date/Author: 2026-04-24 / Codex

- Decision: Use CDP `Input.insertText` and `Input.dispatchKeyEvent` rather than embedding the symbol into JavaScript source.
  Rationale: User input should be treated as data. The old CLI uses CDP input events after opening the watchlist add-symbol UI, and the Rust implementation should preserve that safety boundary.
  Date/Author: 2026-04-24 / Codex

## Outcomes & Retrospective

The implementation is complete and behavior-preserving outside the new command surface. The CLI now exposes `tv watchlist add <SYMBOL>`, and the operation uses DOM panel controls plus CDP input events rather than interpolating the symbol into JavaScript source. Automated validation passed. The first live smoke exposed a selector gap, and the follow-up fix now treats visible watchlist rows as an already-open panel instead of failing before the add button search.

## Context and Orientation

This repository implements a Rust-native command-line binary named `tv`. The entrypoint in `src/main.rs` parses commands, connects to TradingView Desktop through Chrome DevTools Protocol, calls operation functions from `src/ops.rs`, and prints JSON envelopes.

Chrome DevTools Protocol, abbreviated CDP, is the local debugging protocol exposed by TradingView Desktop when it runs with a remote debugging port. Runtime JavaScript evaluation is abstracted by the `RuntimeEvaluator` trait in `src/cdp.rs`; unit tests use `src/ops/test_support.rs` fake runtimes so tests do not require TradingView Desktop.

The existing watchlist read command lives in `src/ops/layout.rs` beside `pane_list`. `watchlist get` is read-only. `watchlist add` is different: it opens the watchlist panel if needed, clicks the add-symbol button, types the requested symbol, confirms the first result with Enter, and closes the search UI with Escape. This mirrors the old JavaScript implementation in `../tradingview-mcp/src/core/watchlist.js`.

## Plan of Work

First, extend `RuntimeEvaluator` in `src/cdp.rs` with the minimal input methods needed by this command: insert text and dispatch a keyboard event. The concrete `CdpClient` should map these to `Input.insertText` and `Input.dispatchKeyEvent`. Add small unit coverage for the dispatch payload mapping where practical, and extend `FakeRuntime` so operation tests can observe inserted text and key events.

Next, extend the CLI surface in `src/cli.rs`. Add `WatchlistCommand::Add { symbol: String }`. Update `src/main.rs` dispatch to validate that `symbol.trim()` is not empty before connecting, then call `ops::watchlist_add`.

Then, implement `watchlist_add` in `src/ops/layout.rs`. The operation should first evaluate JavaScript that opens the right-side watchlist panel if it is closed. If the watchlist button is missing, return `AppError::new(ErrorKind::InternalApiUnavailable, ...)`. It should then evaluate JavaScript that clicks the visible add-symbol button inside the watchlist panel. If no add button is found, return the same error kind. After that, call `insert_text(symbol)`, dispatch Enter key down/up, then dispatch Escape key down/up. Return a success payload containing at least `symbol` and `action: "added"`, plus `requested_symbol`, `source: "dom_input"`, and selector or method information from the clicked button when available.

Finally, update tests and documentation. Unit tests should prove the expected JavaScript selectors are evaluated, the symbol is inserted as CDP text, and Enter/Escape key events are sent. CLI contract tests should cover help, missing symbol validation, and structured connection errors. Documentation should move `watchlist add` from deferred migration backlog to implemented surface while keeping larger mutation surfaces deferred.

## Concrete Steps

From the repository root, the pre-implementation baseline is:

    cargo test

Expected result:

    46 unit tests and 18 CLI contract tests pass.

After implementing the code and docs, run:

    cargo fmt --check
    cargo clippy --all-targets --all-features
    cargo test
    git diff --check
    tracked-doc local absolute path scan

If `cargo fmt --check` fails only because of formatting, run `cargo fmt` and repeat the baseline.

If a TradingView Desktop CDP session is available, run a smoke test with a low-risk symbol that is already acceptable in the active watchlist:

    cargo run -- watchlist add NASDAQ:AAPL

Record the observed JSON or blocker in this plan. The live smoke is useful but not required for automated acceptance because it mutates the user's account watchlist.

## Validation and Acceptance

The change is accepted when `tv watchlist add <SYMBOL>` appears in help, rejects a missing or empty symbol before connecting, returns structured connection errors when CDP is unavailable, and passes all unit and CLI contract tests.

The success JSON must use the Rust envelope:

    {
      "success": true,
      "command": "watchlist",
      "data": {
        "symbol": "NASDAQ:AAPL",
        "action": "added"
      }
    }

Additional fields may be present under `data`, but old practical information must not be removed.

## Idempotence and Recovery

Code changes are safe to rerun. The automated tests do not require TradingView Desktop and do not mutate a real watchlist.

The live smoke is not idempotent from the user's account perspective because it may add a symbol to a watchlist. Prefer a harmless or already-present symbol and record the chosen symbol. If the UI selector fails during smoke, do not keep retrying blindly; record the blocker and keep automated validation as the acceptance gate.

## Artifacts and Notes

Initial baseline:

    cargo test
    result: ok. 46 unit tests and 18 CLI contract tests passed.

Post-implementation test check:

    cargo test
    result: ok. 49 unit tests and 19 CLI contract tests passed.

Final validation:

    cargo fmt --check
    result: passed

    cargo clippy --all-targets --all-features
    result: passed

    cargo test
    result: ok. 49 unit tests and 19 CLI contract tests passed.

    git diff --check
    result: passed

    tracked-doc local absolute path scan
    result: no matches

Live smoke:

    cargo run -- watchlist add NASDAQ:AAPL
    result: exit code 3, internal_api_unavailable, Watchlist button not found

Follow-up selector fix:

    cargo test ops::layout::tests::watchlist_add
    result: ok. 4 watchlist add tests passed.

Follow-up live smoke:

    cargo run -- watchlist add NASDAQ:AAPL
    result: success true, action added, add_button selector [data-name="add-symbol-button"], opened_panel false.

## Interfaces and Dependencies

At the end of the implementation, the following command must exist:

    tv watchlist add <SYMBOL>

The operation facade must expose:

    pub async fn watchlist_add(runtime: &mut impl RuntimeEvaluator, symbol: &str) -> Result<Value, AppError>

No new third-party Rust dependencies are required.

## Open Questions

There are no unresolved critical questions. This slice intentionally does not add dry-run mode, watchlist removal, alert creation, pane mutation, or launch automation.
