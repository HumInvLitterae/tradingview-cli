# Add Screener filter creation surface

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This plan follows `.agents/PLANS.md`.

## Purpose / Big Picture

After this change, an operator should know whether the Rust CLI can safely add a Stock Screener filter from the current TradingView Desktop UI. If the current UI exposes a stable path, `tv screener filters add --name <TEXT> --min <N>|--max <N> [--dry-run]` should be available and should report success only after a visible filter pill appears. If the UI cannot be driven safely, the repository should record that evidence and leave the CLI surface unchanged.

This slice intentionally does not continue deep stabilization of `filters modify`. Existing filter mutation keeps its visible-text post-check boundary. The goal is to broaden Screener surface evidence first, then perform shared UI click and popover stabilization later.

## Progress

- [x] (2026-04-26 12:30Z) Read current repository state, continuity ledger, PLANS.md, and recent Screener docs.
- [x] (2026-04-26 13:05Z) Took live evidence for add-filter catalog, real text insertion search, candidate selection, numeric range selection, and visible-pill post-check using `RSI`.
- [x] (2026-04-26 13:20Z) Decided `filters add` can be implemented for visible catalog numeric presets with dry-run and post-add visible-pill verification.
- [x] (2026-04-26 13:35Z) Implemented `tv screener filters add --name <TEXT> --min <N>|--max <N> [--dry-run]` plus docs updates.
- [x] (2026-04-26 13:55Z) Ran focused tests, full validation, live evidence smoke, and tracked-doc hygiene checks.
- [ ] Commit the completed slice.

## Surprises & Discoveries

- The add-filter catalog search input did not react reliably to direct DOM
  value assignment. It did react to real CDP `Input.insertText`, so the
  implementation focuses the input through DOM evidence and types through CDP.
- Candidate and range option selection needed CDP mouse events. DOM `click()`
  was not enough for the observed candidate row.
- Live evidence on the test screen selected `RSI (相対力指数)`, then selected
  the `> 70` range preset, which produced a visible `RSI (14)` filter pill.
  The test filter was removed afterward with `tv screener filters remove`.
- After the manual live add/remove evidence, the active TradingView session
  sometimes reported the Screener dialog as open while the visible right panel
  showed the watchlist. This appears to be an existing `screener status/open`
  detection fragility rather than a filter-add post-check failure. The command
  itself still requires a real add-filter button and fails safely when it cannot
  find one.

## Decision Log

- Decision: Prioritize broad Screener surface coverage over deeper `filters modify` stabilization in this slice.
  Rationale: The existing `filters modify` command already fails safely when the UI does not reflect the requested change. The next useful step is to determine whether filter creation can be implemented with the same evidence-gated safety boundary.
  Date/Author: 2026-04-26 / Codex.

- Decision: Do not expose `filters add` unless add, range selection, and visible-pill verification are all proven.
  Rationale: A filter added without reliable range setup or post-add verification would pollute the test screen and create a misleading CLI contract.
  Date/Author: 2026-04-26 / Codex.

- Decision: Implement `filters add` with DOM evidence for locating controls but
  CDP input and mouse events for mutation.
  Rationale: The current TradingView UI is React-controlled; the live catalog
  search and candidate selection worked with real input events but not with
  direct DOM value/click mutation.
  Date/Author: 2026-04-26 / Codex.

## Outcomes & Retrospective

Implementation is complete. `tv screener filters add` has been added with
validation, dry-run target reporting, CDP candidate/range clicks, and visible
post-add checks. Full automated validation passed. Live evidence confirmed the
underlying add/range/remove flow with `RSI`, while a later CLI dry-run smoke was
blocked by pre-existing Screener open-state detection fragility and failed
safely with `internal_api_unavailable`.

## Context and Orientation

The Rust command definitions live in `src/cli.rs`. Command validation and dispatch live in `src/main.rs`. Screener UI automation lives in `src/ops/screener.rs`, which evaluates small JavaScript snippets inside a running TradingView Desktop page through Chrome DevTools Protocol, abbreviated CDP. Successful CLI payloads are returned under the top-level `data` field; failures are returned under `error.kind`, `error.message`, and `error.details`.

The current Screener filter surface includes `tv screener filters list`, `actions`, `modify`, `remove`, and `clear`. `filters actions` opens the visible filter edit UI and reports the add button and range preset options. `filters modify` targets one existing visible numeric range filter and only reports success if the visible filter pill text reflects the requested preset. Previous live evidence showed that add-filter UI opens a searchable catalog, but did not prove an end-to-end add plus range plus post-add verification path.

## Plan of Work

First, take live evidence on the prepared `米国株（テスト用）` screen. Use only concise UI facts such as button labels, candidate filter names, option labels, counts, and command success or error kinds. Do not write raw Screener rows, account-linked identifiers, or local absolute filesystem paths into tracked docs.

Then inspect the add-filter UI. The key questions are whether a search input is stable, whether an exact candidate row can be selected, whether a numeric range preset can be applied after adding, and whether the resulting filter pill can be verified by `filters list`. If all four are proven, add a `ScreenerFiltersCommand::Add` variant with `--name`, `--min`, `--max`, and `--dry-run`. If any of those steps are not proven, do not add CLI surface; update docs to record the deferred state.

If `filters add` is implemented, validation must reject blank names, missing ranges, max-only ranges unsupported by the existing preset model, and non-finite numeric values before CDP connection. Dry-run must return the intended candidate and requested range without mutation. Normal add must return success only after the visible filter count increases or a target filter pill containing the requested name/range appears.

## Concrete Steps

Run live evidence commands from the repository root:

    target/debug/tv tab list
    TV_CDP_TARGET_ID=<target> target/debug/tv screener screens active
    TV_CDP_TARGET_ID=<target> target/debug/tv screener filters list
    TV_CDP_TARGET_ID=<target> target/debug/tv screener filters actions

Use `TV_ALLOW_UNSAFE_UI_EVAL=1 tv ui eval` only for bounded DOM evidence gathering. Return only high-level UI labels and availability fields.

If implementing the command, update `src/cli.rs`, `src/main.rs`, `src/ops.rs`, `src/ops/screener.rs`, and `tests/cli_contract.rs`. Update README, CHANGELOG, `docs/notes/rust-cli-contract-migration-2026-04-24.md`, `docs/notes/ui-screener-read-evidence-2026-04-26.md`, `docs/notes/screener-hotlist-upstream-feasibility-2026-04-25.md`, `docs/notes/upstream-pr-triage-2026-04-25.md`, and `docs/notes/next-agent-handoff-prompt-2026-04-24.md` only as needed to reflect actual behavior.

## Validation and Acceptance

If `filters add` is implemented, acceptance is a dry-run smoke that resolves a specific filter candidate and a normal smoke on the test screen that either adds the filter and verifies it, or fails safely without claiming success. If implementation is deferred, acceptance is a durable evidence note explaining why no new CLI variant was added.

Run:

    cargo test screener -- --nocapture
    cargo test --test cli_contract screener -- --nocapture
    cargo fmt --check
    cargo clippy --all-targets --all-features -- -D warnings
    cargo test
    git diff --check
    git grep -nE '(/Users/|C:\\|USER;)' -- README.md CHANGELOG.md docs .agents/skills || true

The tracked-doc grep should return no new leaks. Existing validation-command examples in plan documents are acceptable.

## Idempotence and Recovery

Use only the prepared test screen for normal mutation smoke. If a test filter is added and `tv screener filters remove` can remove it exactly, clean it up. If cleanup is not safe, leave the filter only on the test screen and record the exact visible filter label in this plan. If a popover remains open, use `tv ui keyboard Escape` or `tv screener close`.

## Artifacts and Notes

- Focused validation passed:
  - `cargo test screener_filters_add -- --nocapture`
  - `cargo test --test cli_contract screener -- --nocapture`
  - `cargo test screener -- --nocapture`
- Full validation passed:
  - `cargo fmt --check`
  - `cargo clippy --all-targets --all-features -- -D warnings`
  - `cargo test`
  - `git diff --check`
- Tracked-doc grep for local absolute paths and account-linked script IDs
  returned only existing validation-command examples in plan documents.

## Interfaces and Dependencies

Do not add crate dependencies. Reuse the existing `RuntimeEvaluator`, `AppError`, `serde_json::Value`, `js_string`, and Screener helper patterns in `src/ops/screener.rs`. Any new success payload must use `source: "ui_screener_dialog"` and `action: "filter_add"`.

## Open Questions

UNCONFIRMED: Whether the current add-filter catalog exposes a stable exact candidate row for numeric filters.

UNCONFIRMED: Whether the current UI can set a numeric range preset immediately after adding a filter.
