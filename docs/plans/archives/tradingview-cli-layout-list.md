# Add saved layout list command

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document follows `.agents/PLANS.md`.

## Purpose / Big Picture

After this change, a user can run `tv layout list` to read saved TradingView chart layouts from the current page session. This closes the read-only half of the old JavaScript `layout list/switch` CLI surface that was missing from the Rust migration inventory.

The command is intentionally separate from `tv pane layout <LAYOUT>`. `pane layout` changes the current chart grid. `layout list` reads saved chart layouts. The mutation command `layout switch` remains deferred for a later ExecPlan.

## Progress

- [x] (2026-04-25 02:05Z) Compared the old JavaScript `layout list/switch` CLI and `core/ui.js` implementation with the current Rust CLI surface.
- [x] (2026-04-25 02:05Z) Created this ExecPlan.
- [x] (2026-04-25 02:14Z) Add `tv layout list` CLI surface and dispatch.
- [x] (2026-04-25 02:14Z) Implement read-only saved layout listing.
- [x] (2026-04-25 02:14Z) Add operation and CLI contract tests.
- [x] (2026-04-25 02:24Z) Update README, AGENTS, migration inventory, contract notes, deferred audit, handoff note, and chart-analysis skill.
- [x] (2026-04-25 02:31Z) Run automated validation.
- [x] (2026-04-25 02:33Z) Run read-only live smoke.
- [x] (2026-04-25 02:38Z) Commit the completed slice.

## Surprises & Discoveries

The old JavaScript CLI had a top-level `layout` group that was distinct from `pane layout`. Rust had implemented pane grid layout mutation, but the saved chart layout list/switch group was missing from the durable migration inventory.

## Decision Log

- Decision: Implement only `layout list` in this slice.
  Rationale: It is read-only and preserves useful old CLI information. `layout switch` loads a saved chart layout and can trigger unsaved-changes UI, so it needs a separate mutation plan and smoke recovery story.
  Date/Author: 2026-04-25 / Codex.

- Decision: Keep saved layout operations in a separate module from pane layout operations.
  Rationale: `src/ops/layout.rs` already owns pane/watchlist behavior. Saved chart layouts are a different capability and should not make that module more ambiguous.
  Date/Author: 2026-04-25 / Codex.

## Outcomes & Retrospective

Implemented `tv layout list` as a read-only saved chart layout inventory command. The command preserves old CLI practical fields under the Rust `data` envelope and keeps `layout switch` deferred.

## Context and Orientation

The Rust binary is named `tv`. Command-line shape is defined in `src/cli.rs`. Runtime dispatch is in `src/main.rs`. Operation functions are re-exported through `src/ops.rs`.

The old JavaScript CLI registered:

    tv layout list
    tv layout switch <NAME_OR_ID>

The old `layout list` implementation called `window.TradingViewApi.getSavedCharts(callback)` and returned `layout_count`, `source`, `layouts`, and optional `error`. The Rust implementation should keep that practical information under the Rust `data` envelope.

## Plan of Work

Add a top-level `Layout` command group with a `List` subcommand. Dispatch it to a new `ops::saved_layout_list` operation.

Implement `saved_layout_list` in `src/ops/saved_layout.rs`. It should evaluate a promise around `window.TradingViewApi.getSavedCharts`, normalize each saved chart into `{ id, name, symbol, resolution, modified }`, and return `layout_count`, `source: "internal_api"`, `layouts`, and optional `error`. If `getSavedCharts` is missing, times out, or returns non-array data, return a successful read payload with an empty `layouts` array and `error`, matching the old read command posture.

Update durable docs to mark `layout list` implemented and `layout switch` deferred. Do not add `layout switch` to Rust help in this slice.

## Concrete Steps

Run commands from the repository root.

Targeted validation while implementing:

    cargo test ops::saved_layout::tests::saved_layout_list -- --nocapture
    cargo test --test cli_contract layout -- --nocapture

Full validation before commit:

    cargo fmt --check
    cargo clippy --all-targets --all-features -- -D warnings
    cargo test
    git diff --check
    rg -n '(/[U]sers/|[C]:\\\\)' README.md AGENTS.md docs .agents/skills || true

## Validation and Acceptance

Automated acceptance is that tests prove help output, connection-attempt behavior, normalized layout payloads, empty/error payloads, and absence of `loadChartFromServer` from the list command expression.

Live smoke is read-only:

    cargo run --quiet -- layout list

Record `layout_count`, `source`, whether `error` is present, and one representative layout row if available.

## Idempotence and Recovery

Source and docs edits are ordinary additive changes and can be rerun. Automated tests use fake runtime responses and do not require TradingView Desktop. Live smoke is read-only and does not mutate chart, account, Pine, drawing, alert, replay, tab, watchlist, or saved layout state.

## Artifacts and Notes

- Targeted validation passed: `cargo test ops::saved_layout::tests::saved_layout_list -- --nocapture`.
- Targeted CLI validation passed: `cargo test --test cli_contract layout -- --nocapture`.
- Full validation passed: `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`, and `git diff --check`.
- Tracked docs local absolute path scan passed with `rg -n '(/[U]sers/|[C]:\\\\)' README.md AGENTS.md docs .agents/skills || true`.
- Chart-analysis skill validation passed: `python .../skill-creator/scripts/quick_validate.py .agents/skills/chart-analysis`.
- Live smoke passed: `cargo run --quiet -- layout list` returned `layout_count: 5`, `source: "internal_api"`, and a first layout row named `Default 2` with id `98948100`, symbol `BATS:ASTS`, resolution `1D`, modified `2026-04-24`.

## Interfaces and Dependencies

At completion, the CLI exposes:

    tv layout list

At completion, `src/ops/saved_layout.rs` exposes:

    pub async fn saved_layout_list(runtime: &mut impl RuntimeEvaluator) -> Result<Value, AppError>

No new crates are required.

## Open Questions

No unresolved critical questions remain for this slice.
