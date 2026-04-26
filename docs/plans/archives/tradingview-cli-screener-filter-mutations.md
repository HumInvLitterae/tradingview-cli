# Add Screener filter mutations

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document follows `.agents/PLANS.md`.

## Purpose / Big Picture

After this change, `tv screener filters remove` can remove one visible filter from the currently active TradingView Stock Screener dialog, and `tv screener filters clear` can remove all visible filters only when the operator explicitly confirms that destructive action. This gives operators a narrow cleanup command for test or disposable Screener screens without implementing the broader screen save/switch/delete or column-management surfaces from upstream PR #66.

The user prepared a test Stock Screener screen named `米国株（テスト用）`. Live mutation smoke may remove filters only when that test screen is active or when the command is run in `--dry-run` mode.

## Progress

- [x] (2026-04-25 18:24Z) Checked working tree, current Screener implementation, existing notes, and upstream PR #66 boundary.
- [x] (2026-04-25 18:24Z) Took live DOM evidence for visible Screener filters and the per-filter remove button.
- [x] (2026-04-25 18:35Z) Added `tv screener filters remove` and `tv screener filters clear` CLI surface and dispatch.
- [x] (2026-04-25 18:35Z) Implemented dry-run target resolution, single-filter removal, and confirmed clear-all behavior.
- [x] (2026-04-25 18:35Z) Added unit and CLI contract tests.
- [x] (2026-04-25 18:35Z) Updated README, changelog, contract notes, and upstream triage notes.
- [x] (2026-04-25 18:37Z) Ran focused tests, live smoke, and full validation baseline.
- [x] (2026-04-25 18:37Z) Recorded outcomes, committed tracked changes, and updated local continuity ledger.

## Surprises & Discoveries

- Observation: The live target with visible Screener filters was `D202CA6B22895C82C0437F0F9FC6A7BC`; other chart targets either returned target ambiguity or incomplete Screener state.
  Evidence: `tv tab list` reported three chart targets, and `tv screener filters list` on `D202CA6B22895C82C0437F0F9FC6A7BC` returned 19 visible filters.

- Observation: The active screen title read by the current CLI was `米国株`, not `米国株（テスト用）`.
  Evidence: `tv screener screens active` returned `screen_title: "米国株"` for the visible Screener target. Destructive live smoke must therefore remain dry-run unless a later read confirms the test screen is active.

- Observation: A visible filter pill can be opened by its `data-name`, and the resulting popover exposes a visible button whose class starts with `removeButton-`.
  Evidence: opening the `PER` filter pill and evaluating visible popover buttons returned a button with class `removeButton-YamCOOSc`.

- Observation: Clicking the popover remove button inside an awaited CDP promise can remove the filter but still return `Promise was collected`.
  Evidence: the first live `tv screener filters remove --text PER` removed `PER` from the test screen, but the command returned an internal CDP `Promise was collected` error.

- Observation: Scheduling the remove-button click and then waiting from Rust avoids the CDP promise collection failure.
  Evidence: the second live `tv screener filters remove --text PEG` returned success with `before_filter_count: 18`, `after_filter_count: 17`, and `removed: true`.

## Decision Log

- Decision: Implement filter removal by clicking the target visible filter pill and then clicking the popover `button[class*="removeButton"]`.
  Rationale: This matches the current live TradingView DOM and avoids inventing a broader Screener automation abstraction.
  Date/Author: 2026-04-25 / Codex.

- Decision: Require `--confirm-clear` for non-dry-run `filters clear`.
  Rationale: Removing all visible filters can significantly change a saved or active Screener screen, so an explicit confirmation flag is necessary even though mutation commands are allowed in this phase.
  Date/Author: 2026-04-25 / Codex.

- Decision: Support `--index` or `--text` for single-filter removal and make them mutually exclusive.
  Rationale: Index is precise for scripted operation, while text is easier for manual use. Ambiguous text matches must fail rather than guessing.
  Date/Author: 2026-04-25 / Codex.

- Decision: Do not run destructive live smoke for `filters clear`.
  Rationale: Unit tests cover the repeated removal loop and live dry-run proves target enumeration. Removing every visible filter is unnecessary when single-filter live mutation already proved the current DOM path.
  Date/Author: 2026-04-25 / Codex.

## Outcomes & Retrospective

Implementation is complete and has passed focused automated tests, full validation, and live smoke. `tv screener filters remove` supports exact index targeting or unique text targeting, dry-run reporting, and before/after verification. `tv screener filters clear` supports dry-run target reporting and requires `--confirm-clear` before removing all visible filters.

Live smoke used the prepared `米国株（テスト用）` screen. The first smoke removed `PER` but exposed a `Promise was collected` bug, which was fixed by scheduling the remove click and verifying afterward from Rust. The successful follow-up removed `PEGレシオ`, reducing the visible filter count from 18 to 17. `filters clear --dry-run` reported 17 target filters and did not mutate.

## Context and Orientation

Screener commands live in `src/ops/screener.rs`. Existing read commands can open the Stock Screener dialog, read visible filter pills, and restore the original open/closed state for read operations. A filter pill is a visible TradingView button whose `data-name` begins with `screener-filter-pill-`. The current implementation reports each filter with an `index`, `text`, `data_name`, and `visible` flag.

This plan adds mutations to the existing `tv screener filters` command group. A mutation here means a command that changes the active TradingView Screener UI state and may persist through TradingView's own saved-screen behavior.

## Plan of Work

Add `Remove` and `Clear` variants to `ScreenerFiltersCommand` in `src/cli.rs`. `Remove` accepts exactly one target selector, `--index <N>` or `--text <TEXT>`, plus `--dry-run`. `Clear` accepts `--dry-run` and `--confirm-clear`.

In `src/main.rs`, validate the remove selector before connecting when possible, validate that non-dry-run clear has `--confirm-clear`, then dispatch into `src/ops/screener.rs`.

In `src/ops/screener.rs`, reuse the existing open/read helper pattern. Read the current filters, resolve a target by exact index or unique case-insensitive substring text match, and return the target in dry-run mode. For non-dry-run remove, click the resolved pill by `data_name`, click the visible popover remove button, wait until the filter count decreases or the target `data_name` is gone, and return before/after counts plus the removed target. For confirmed clear, repeatedly remove the first visible filter until no filters remain, returning all removed filters and counts.

## Concrete Steps

Run commands from the repository root.

1. Edit `src/cli.rs`, `src/main.rs`, and `src/ops/screener.rs`.
2. Add focused tests in `src/ops/screener.rs` and `tests/cli_contract.rs`.
3. Update `README.md`, `CHANGELOG.md`, `docs/notes/rust-cli-contract-migration-2026-04-24.md`, `docs/notes/screener-hotlist-upstream-feasibility-2026-04-25.md`, and `docs/notes/upstream-pr-triage-2026-04-25.md`.
4. Run focused tests:

        cargo test screener -- --nocapture
        cargo test --test cli_contract screener -- --nocapture

5. Run live smoke with an explicit target id:

        TV_CDP_TARGET_ID=<target> target/debug/tv screener screens active
        TV_CDP_TARGET_ID=<target> target/debug/tv screener filters remove --index 0 --dry-run
        TV_CDP_TARGET_ID=<target> target/debug/tv screener filters clear --dry-run

   Run destructive remove smoke only if the active screen is the prepared test screen `米国株（テスト用）`. Do not run destructive clear smoke unless explicitly needed, because single-filter live mutation plus clear dry-run is enough evidence for this slice.

6. Run full validation:

        cargo fmt --check
        cargo clippy --all-targets --all-features -- -D warnings
        cargo test
        git diff --check
        git grep -nE '(/Users/|C:\\|USER;)' -- README.md AGENTS.md docs .agents/skills || true

## Validation and Acceptance

The change is accepted when help output lists `filters remove` and `filters clear`, tests prove validation and payload behavior, dry-run smoke reports a target without mutation, and destructive smoke is either successful on the prepared test screen or intentionally skipped because the active screen title is not the test screen.

## Idempotence and Recovery

Dry-run commands are safe to repeat. Single-filter remove is not automatically reversible because the CLI does not implement filter creation. Clear-all is intentionally gated by `--confirm-clear`. If a destructive smoke removes filters from the prepared test screen, record the exact screen title and removed filter texts in this plan; do not attempt to recreate filters unless a separate verified UI path exists.

## Artifacts and Notes

Record only command summaries, target filter text, counts, and whether destructive smoke was skipped or performed. Do not paste raw Screener table rows or account-linked identifiers into tracked docs.

Focused tests passed:

    cargo test screener -- --nocapture
    cargo test --test cli_contract screener -- --nocapture

Full validation passed:

    cargo fmt --check
    cargo clippy --all-targets --all-features -- -D warnings
    cargo test
    git diff --check
    git grep -nE '(/Users/|C:\\|USER;)' -- README.md AGENTS.md docs .agents/skills || true

The grep command returned only existing validation-command examples in plan documents, not live account identifiers or newly introduced machine-specific paths.

Live smoke summary:

    TV_CDP_TARGET_ID=D202CA6B22895C82C0437F0F9FC6A7BC target/debug/tv screener screens active

returned `screen_title: "米国株（テスト用）"`.

    TV_CDP_TARGET_ID=D202CA6B22895C82C0437F0F9FC6A7BC target/debug/tv screener filters remove --text PER

removed `PER` but returned `Promise was collected`, so the implementation was changed to schedule the remove click and verify afterward.

    TV_CDP_TARGET_ID=D202CA6B22895C82C0437F0F9FC6A7BC target/debug/tv screener filters remove --text PEG

returned success and removed `PEGレシオ`, changing `before_filter_count: 18` to `after_filter_count: 17`.

    TV_CDP_TARGET_ID=D202CA6B22895C82C0437F0F9FC6A7BC target/debug/tv screener filters clear --dry-run

returned `before_filter_count: 17`, `after_filter_count: 17`, and `cleared: false`.

## Interfaces and Dependencies

Expose these operation functions through `src/ops.rs`:

    pub async fn screener_filters_remove(runtime: &mut impl RuntimeEvaluator, selector: ScreenerFilterSelector, dry_run: bool) -> Result<Value, AppError>;
    pub async fn screener_filters_clear(runtime: &mut impl RuntimeEvaluator, dry_run: bool, confirm_clear: bool) -> Result<Value, AppError>;

Define `ScreenerFilterSelector` in `src/ops/screener.rs` with index and text variants. No new crate dependencies are required.

## Open Questions

None.

Revision note: updated after implementation and live smoke to record the command surface, CDP promise-collection fix, test evidence, and intentionally skipped destructive clear smoke.
