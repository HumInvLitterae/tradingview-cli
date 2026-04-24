# Add pane mutation commands to the Rust CLI

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This plan follows `.agents/PLANS.md` from the repository root.

## Purpose / Big Picture

This change migrates the old JavaScript CLI pane mutation surface into the Rust-native `tv` CLI as a small operator slice. After the change, an operator can inspect panes with `tv pane list`, focus a zero-based pane index with `tv pane focus <INDEX>`, set a symbol in a specific pane with `tv pane symbol <INDEX> <SYMBOL>`, and change the TradingView chart layout with `tv pane layout <LAYOUT>`.

The work matters because multi-pane chart setup is a practical operator workflow. It unblocks repeatable multi-symbol visual review without forcing downstream scripts to know TradingView's private JavaScript objects directly. This is still intentionally smaller than a downstream workflow helper: the CLI exposes direct pane operations only, while orchestration such as saving layouts, restoring a full workspace, or deciding which symbols belong in panes remains outside the core CLI.

## Progress

- [x] (2026-04-24T09:34:36Z) Compared the old JavaScript pane commands and identified the practical fields to preserve: `layout`, `layout_name`, `chart_count`, `panes`, `focused_index`, `total_panes`, `index`, and `symbol`.
- [x] (2026-04-24T09:34:36Z) Added `pane layout`, `pane focus`, and `pane symbol` CLI subcommands and operation functions.
- [x] (2026-04-24T09:34:36Z) Added unit and CLI contract tests for layout aliases, pre-connect validation, CDP connection attempts, focus errors, and JavaScript string serialization.
- [x] (2026-04-24T09:34:36Z) Ran automated validation: `cargo fmt --check`, `cargo clippy --all-targets --all-features`, `cargo test`, `git diff --check`, and tracked-doc absolute path scan.
- [x] (2026-04-24T09:34:36Z) Ran restore-safe live smoke against the current TradingView Desktop session.
- [x] (2026-04-24T09:34:36Z) Prepared the completed slice for commit.

## Surprises & Discoveries

- Observation: The existing Rust `pane list` layout names used short names such as `single` and `4 panes`, while the old JavaScript CLI exposed friendlier names such as `1 chart` and `2x2 grid`.
  Evidence: `src/ops/layout.rs` contained the older Rust local map before this slice, and the old JavaScript CLI used a `LAYOUT_NAMES` table with the practical names.

## Decision Log

- Decision: Implement only direct pane operations in the core CLI and leave multi-step layout orchestration downstream.
  Rationale: The core CLI should replace old practical surface without becoming a workflow helper. Focusing, setting a pane symbol, and setting a layout are the smallest useful primitives.
  Date/Author: 2026-04-24 / Codex

- Decision: Validate `pane layout` before connecting to CDP.
  Rationale: Unknown layout names are local input errors, so users should get a validation envelope and exit code 1 rather than a connection attempt.
  Date/Author: 2026-04-24 / Codex

- Decision: Keep the Rust JSON envelope and place command payloads under `data`.
  Rationale: This follows the project's accepted Rust contract while preserving the old CLI's practical information inside the payload.
  Date/Author: 2026-04-24 / Codex

## Outcomes & Retrospective

The pane mutation slice is implemented and validated. `tv pane --help` now lists `list`, `layout`, `focus`, and `symbol`. The Rust CLI preserves the practical old pane information while continuing to use the Rust envelope with payloads under `data`.

Automated validation passed with `cargo fmt --check`, `cargo clippy --all-targets --all-features`, `cargo test`, `git diff --check`, and the tracked-doc absolute path scan. Restore-safe live smoke also passed against a running TradingView Desktop CDP session by reading the current pane state, focusing the already active pane, setting the already active pane back to its current symbol, reapplying the current layout, and reading the pane state again.

## Context and Orientation

The repository implements a Rust-native TradingView Desktop CLI named `tv`. Commands are parsed in `src/cli.rs`, dispatched in `src/main.rs`, and implemented through operation modules under `src/ops/`. The file `src/ops.rs` is a thin facade that re-exports feature functions. The file `src/ops/layout.rs` currently owns watchlist and pane operations. This is acceptable for this small slice, but future growth should split large sub-surfaces rather than recreating a monolithic operation file.

TradingView is controlled through Chrome DevTools Protocol, abbreviated CDP. CDP lets the CLI evaluate JavaScript inside the currently running TradingView Desktop page. The pane commands use TradingView's private `window.TradingViewApi._chartWidgetCollection` object, the same practical dependency used by the old JavaScript CLI. Because that API is private, failures caused by a missing method or changed page object are reported as `internal_api_unavailable`.

## Plan of Work

In `src/cli.rs`, add `layout`, `focus`, and `symbol` variants under `PaneCommand`. In `src/main.rs`, dispatch those variants to operation functions. `pane layout` validates layout aliases before connecting. `pane symbol` validates that the symbol is not empty before connecting.

In `src/ops/layout.rs`, add a small layout parser that accepts the old JavaScript layout codes and aliases. Supported canonical layouts are `s`, `2h`, `2v`, `2-1`, `1-2`, `3h`, `3v`, `3s`, `4`, `4h`, `4v`, `4s`, `6`, `8`, `10`, `12`, `14`, and `16`. Aliases include `single`, `1`, `1x1`, `2x1`, `1x2`, `2x2`, `grid`, `quad`, `3x1`, and `1x3`.

The new operation functions are `crate::ops::pane_layout`, `crate::ops::pane_focus`, `crate::ops::pane_symbol`, and `crate::ops::validate_pane_layout`. They use the existing `RuntimeEvaluator` trait, serialize user strings with `js_string`, and return structured payloads that preserve the old CLI's practical fields.

Update `tests/cli_contract.rs` and `src/ops/layout.rs` tests. Update `README.md`, `docs/notes/rust-cli-contract-migration-2026-04-24.md`, `docs/notes/legacy-cli-command-migration-inventory-2026-04-24.md`, and `docs/notes/next-agent-handoff-prompt-2026-04-24.md` so the durable project state matches the code.

## Concrete Steps

From the repository root, edit the Rust command surface and operation implementation. Then run:

    cargo fmt --check
    cargo clippy --all-targets --all-features
    cargo test
    git diff --check
    git grep -nE '(/Users/|C:\\)' -- README.md AGENTS.md docs .agents/skills || true

If TradingView Desktop is running with CDP enabled, run a restore-safe smoke:

    cargo run -- pane list
    cargo run -- pane focus <current_active_index>
    cargo run -- pane symbol <current_active_index> <current_symbol>
    cargo run -- pane layout <current_layout>
    cargo run -- pane list

The smoke intentionally focuses the already active pane, sets the already active pane's current symbol back to itself, and reapplies the current layout. It proves the commands can reach TradingView without intentionally changing the user's workspace.

## Validation and Acceptance

Acceptance requires that the automated validation commands pass. The CLI help for `tv pane --help` must list `list`, `layout`, `focus`, and `symbol`. The command `tv pane layout banana` must fail before connecting with `error.kind` set to `validation` and a supported layout list in `error.details.supported`.

When live CDP smoke is possible, `tv pane focus <current_active_index>` should return `success: true` with `data.focused_index` and `data.total_panes`. `tv pane symbol <current_active_index> <current_symbol>` should return `success: true` with `data.index`, `data.symbol`, and `data.focused_index`. `tv pane layout <current_layout>` should return `success: true` with `data.layout`, `data.layout_name`, `data.chart_count`, and `data.panes`.

## Idempotence and Recovery

The automated tests are repeatable and do not require TradingView Desktop. The live smoke is designed to be idempotent because it reapplies the current pane, symbol, and layout rather than introducing a new symbol or layout. If a live command fails with `internal_api_unavailable`, rerun `tv pane list` to confirm the current TradingView page still exposes `window.TradingViewApi._chartWidgetCollection`; record the failure instead of guessing.

## Artifacts and Notes

Important output snippets will be added after validation. Keep snippets short and do not paste machine-specific absolute paths into this tracked document.

Automated validation summary:

    cargo fmt --check
    cargo clippy --all-targets --all-features
    cargo test
    git diff --check
    git grep -nE '(/Users/|C:\\)' -- README.md AGENTS.md docs .agents/skills || true

All commands completed successfully. `cargo test` reported 64 unit tests and 23 CLI contract tests passing.

Restore-safe live smoke summary:

    cargo run --quiet -- pane list
    success: true, layout: s, active_index: 0, symbol: BATS:LWLG

    cargo run --quiet -- pane focus 0
    success: true, focused_index: 0, total_panes: 1

    cargo run --quiet -- pane symbol 0 BATS:LWLG
    success: true, index: 0, symbol: BATS:LWLG, focused_index: 0

    cargo run --quiet -- pane layout s
    success: true, layout: s, layout_name: 1 chart, observed_layout: s

    cargo run --quiet -- pane list
    success: true, layout: s, active_index: 0, symbol: BATS:LWLG

## Interfaces and Dependencies

At completion, `src/ops.rs` re-exports:

    pub use layout::{
        pane_focus, pane_layout, pane_list, pane_symbol, validate_pane_layout, watchlist_add,
        watchlist_get,
    };

The command-line interface accepts:

    tv pane list
    tv pane layout <LAYOUT>
    tv pane focus <INDEX>
    tv pane symbol <INDEX> <SYMBOL>

No new Rust crate dependency is required. The implementation uses existing project modules: `src/cdp.rs` for CDP evaluation, `src/error.rs` for typed errors, `src/output.rs` for JSON envelopes, and `src/ops/common.rs` for shared TradingView JavaScript paths and safe JavaScript string serialization.

## Open Questions

No critical open questions block this slice. Future work should decide whether `src/ops/layout.rs` should be split into separate watchlist and pane modules if either sub-surface grows again.
