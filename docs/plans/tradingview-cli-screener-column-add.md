# Screener column add

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This plan follows `.agents/PLANS.md`. It is self-contained so a new contributor can understand and continue the work from this file alone.

## Purpose / Big Picture

Users can already inspect, remove, and reorder saved Stock Screener columns on prepared test screens. The missing cleanup path is the inverse of remove: after deleting a disposable column, an operator needs a CLI way to insert a known storage column id and params back into the saved screen. This slice adds `tv screener columns add --id <COLUMN_ID> [--params-json <JSON>] [--after-index <N>] [--dry-run]` as a low-level, storage-backed mutation.

This is not a generic display-name column catalog. The command inserts a known TradingView storage column id, with optional JSON object params, into the active saved Screener screen. A user can see it working by dry-running the expected storage order, then running a remove-and-add smoke on a test screen and confirming the storage order returns to the expected 13 columns.

## Progress

- [x] (2026-04-27 13:20Z) Added CLI surface and pre-CDP validation for `tv screener columns add --id <COLUMN_ID> [--params-json <JSON>] [--after-index <N>] [--dry-run]`.
- [x] (2026-04-27 13:35Z) Implemented storage-backed add in `src/ops/screener.rs`, including dry-run expected order, test-screen guard for normal mutation, storage save, and post-save storage order check.
- [x] (2026-04-27 13:45Z) Added operation and CLI contract tests for blank id, invalid params JSON, non-object params JSON, out-of-range `--after-index`, dry-run output, non-test screen refusal, normal save, post-check behavior, and help text.
- [x] (2026-04-27 13:55Z) Ran focused tests: `cargo test screener_column -- --nocapture` and `cargo test --test cli_contract screener -- --nocapture`.
- [x] (2026-04-27 14:05Z) Updated README, CHANGELOG, contract notes, Screener feasibility notes, upstream PR triage, and handoff notes.
- [x] (2026-04-27 14:15Z) Ran live smoke on the full-page test Screener target: read config, dry-run add, remove `TechnicalRating`, add it back, and confirm the final 13-column order.
- [x] (2026-04-27 14:30Z) Full validation baseline passed: `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`, `git diff --check`, and tracked-doc local-path / `USER;` grep with only existing validation-command examples.
- [x] (2026-04-27 14:35Z) Updated `CONTINUITY.md` with the column add state, live smoke result, validation evidence, and remaining Screener gaps.
- [x] (2026-04-27 14:45Z) Ready to commit the related implementation and docs as `feat(screener): Add column storage insertion`.

## Surprises & Discoveries

- Observation: The prior column storage slice showed `window.initData` did not expose a reliable generic column catalog or default column set.
  Evidence: The previous read-only evidence found only high-level Screener storage keys, not a complete column catalog. This slice therefore implements id-based insertion only and keeps display-name add plus reset out of scope.
- Observation: Duplicate storage ids must remain allowed in the initial implementation.
  Evidence: TradingView can represent practical columns such as moving averages with the same base id and different params. Blocking exact duplicates would be a separate policy decision and would make remove restoration harder.
- Observation: The remove-then-add live smoke restored the expected storage order on the prepared test screen.
  Evidence: Final `columns config` reported 13 columns, with `RatingMa` at index 11 and `TechnicalRating` at index 12 with params `{"resolution":"TimeResolution1D"}`.

## Decision Log

- Decision: Implement `columns add` as a low-level storage id insertion command rather than a display-name catalog command.
  Rationale: The current evidence has no reliable generic column catalog. A known id plus params is exact, post-checkable, and useful as an operator cleanup path after `columns remove`.
  Date/Author: 2026-04-27 / Codex
- Decision: Default omitted `--params-json` to an empty JSON object and reject non-object JSON.
  Rationale: Storage columns carry params as objects. Accepting arrays, strings, or numbers would create payloads the saved-screen API may not understand.
  Date/Author: 2026-04-27 / Codex
- Decision: Keep normal mutation limited to screen names containing `CLI-Test` or `テスト`.
  Rationale: `columns add` edits saved Screener cloud state. The same guard is already used for storage-backed remove and reorder.
  Date/Author: 2026-04-27 / Codex
- Decision: Continue deferring `columns reset`.
  Rationale: Reset needs a trustworthy default source. Without one, the command would guess at TradingView's defaults and could silently damage a saved screen.
  Date/Author: 2026-04-27 / Codex

## Outcomes & Retrospective

`columns add` now restores a removed `TechnicalRating` storage column with params `{"resolution":"TimeResolution1D"}` on the prepared `米国株（テスト用）` screen. Dry-run shows the expected insertion order without mutation, and normal mode saved the active test screen only after the existing `テクニカル評価` column was removed. The final live config returned to 13 columns with `RatingMa` at index 11 and `TechnicalRating` at index 12. Full validation passed. Remaining Screener work after this slice is `columns reset` feasibility, generic non-numeric filter editing, and later stabilization of the broader Screener UI mutation surface.

## Context and Orientation

The Rust CLI is implemented as the `tv` binary. The command parser lives in `src/cli.rs`, dispatch lives in `src/main.rs`, operation exports live in `src/ops.rs`, and Screener behavior lives in `src/ops/screener.rs`.

Screener commands talk to TradingView Desktop through the Chrome DevTools Protocol. A "full-page Screener target" is a separate TradingView page target whose URL is a Screener page rather than a chart page. `tv tab list` reports these targets under `screener_targets`, and a user can run follow-up commands against one by setting `TV_CDP_TARGET_ID`.

The "storage API" in this plan means TradingView's logged-in page-session saved-screen storage endpoint, accessed from inside the authenticated TradingView page with `fetch`. It is not a public stable API. The command must therefore treat missing storage metadata, failed save requests, or failed post-checks as `internal_api_unavailable` rather than guessing success.

## Plan of Work

In `src/cli.rs`, add an `Add` variant under `ScreenerColumnsCommand` with flags `--id`, `--params-json`, `--after-index`, and `--dry-run`. In `src/main.rs`, validate blank id and params JSON before opening a CDP connection, then dispatch to the Screener operation.

In `src/ops/screener.rs`, add `ScreenerColumnAddRequest` and `validate_screener_column_add_request`. The validator trims id, rejects empty id, parses optional params JSON, accepts only JSON objects, defaults params to `{}`, and carries optional `after_index` plus dry-run state.

Implement `screener_columns_add` by reading the active Screener state, fetching the active saved screen storage config, inserting a storage column with the requested id and params, and re-indexing the expected order. Dry-run returns the target column and expected after-order without saving. Normal mode first verifies that the active screen name is test/disposable, saves the updated `default_custom_column_set`, then re-fetches the storage config and reports success only when id, params, and order match the expected result.

Update docs so `columns add` is listed with the implemented Screener surface, and so the remaining deferred column work is only `columns reset` plus possible future catalog/default-source discovery.

## Concrete Steps

From the repository root, run:

    cargo test screener_column -- --nocapture
    cargo test --test cli_contract screener -- --nocapture
    cargo fmt --check
    cargo clippy --all-targets --all-features -- -D warnings
    cargo test
    git diff --check
    git grep -nE '(/Users/|C:\\|USER;)' -- README.md CHANGELOG.md docs .agents/skills || true

For live smoke, first identify the full-page Screener target:

    tv tab list

Then set `TV_CDP_TARGET_ID` to the Screener target and run read-only plus dry-run commands:

    TV_CDP_TARGET_ID=<screener-target> tv screener screens active
    TV_CDP_TARGET_ID=<screener-target> tv screener columns config
    TV_CDP_TARGET_ID=<screener-target> tv screener columns add --id TechnicalRating --params-json '{"resolution":"TimeResolution1D"}' --after-index 11 --dry-run

Normal smoke should use only the prepared test screen:

    TV_CDP_TARGET_ID=<screener-target> tv screener columns remove --name "テクニカル評価"
    TV_CDP_TARGET_ID=<screener-target> tv screener columns add --id TechnicalRating --params-json '{"resolution":"TimeResolution1D"}' --after-index 11
    TV_CDP_TARGET_ID=<screener-target> tv screener columns config

The final config should have 13 columns, with `RatingMa` at index 11 and `TechnicalRating` at index 12. If add fails, stop deeper smoke attempts and record the remaining column difference in this plan and `CONTINUITY.md`.

## Validation and Acceptance

Acceptance requires focused and full validation commands to pass. The CLI help must list `columns add` and flags `--id`, `--params-json`, `--after-index`, and `--dry-run`. Invalid blank id and invalid or non-object params JSON must fail before CDP connection.

`tv screener columns add --dry-run` must return `action: "columns_add"`, `scope: "screen_storage_api"`, `dry_run: true`, `added: false`, the target storage column, and the expected post-add order. Normal `columns add` must refuse non-test screen names, must save only the active saved screen custom column set, and must not report success unless the re-fetched storage column id/params/order matches the requested result.

## Idempotence and Recovery

Read-only and dry-run commands are safe to repeat. Normal add allows duplicates by design, so repeated normal adds can create repeated columns. Live smoke should avoid repetition and should use a remove-then-add sequence for a known disposable column. If add fails after remove, rerun `tv screener columns config` against the same target to inspect the saved storage state and record the remaining difference.

## Artifacts and Notes

Focused tests after the implementation:

    cargo test screener_column -- --nocapture
    16 passed; 0 failed

    cargo test --test cli_contract screener -- --nocapture
    6 passed; 0 failed

High-level live evidence to capture after smoke:

    Active test Screener screen: 米国株（テスト用）
    add target: TechnicalRating with params {"resolution":"TimeResolution1D"}
    final expected order: RatingMa index 11, TechnicalRating index 12

Live smoke result:

    screens active: 米国株（テスト用）
    columns add dry-run: inserted_index 12, after_column_count 14 because the target column was still present
    columns remove --name "テクニカル評価": removed true, after_column_count 12
    columns add --id TechnicalRating --params-json '{"resolution":"TimeResolution1D"}' --after-index 11: added true, after_column_count 13
    final columns config: 13 columns, RatingMa index 11, TechnicalRating index 12

Do not record raw storage payloads, account-linked identifiers, or local absolute paths in tracked docs.

## Interfaces and Dependencies

At the end of this plan, these interfaces exist:

    pub struct ScreenerColumnAddRequest {
        pub id: String,
        pub params: serde_json::Value,
        pub after_index: Option<usize>,
        pub dry_run: bool,
    }

    pub fn validate_screener_column_add_request(id: &str, params_json: Option<&str>, after_index: Option<usize>, dry_run: bool) -> Result<ScreenerColumnAddRequest, AppError>

    pub async fn screener_columns_add(runtime: &mut impl RuntimeEvaluator, request: ScreenerColumnAddRequest) -> Result<serde_json::Value, AppError>

The payload keeps the Rust envelope convention: the top-level CLI output contains `success`, `command`, and `data`, while the command-specific fields described here live under `data`.

## Open Questions

- UNCONFIRMED: Whether TradingView exposes a reliable generic column catalog suitable for display-name based `columns add`.
- UNCONFIRMED: Whether TradingView exposes a reliable default column set suitable for `columns reset --confirm-reset`.
