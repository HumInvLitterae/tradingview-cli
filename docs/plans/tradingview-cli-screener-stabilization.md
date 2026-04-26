# Stabilize the implemented Screener surface

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document follows `.agents/PLANS.md`.

## Purpose / Big Picture

The Rust CLI now exposes the main planned Stock Screener surface: screen lifecycle commands, filter add/modify/remove/clear commands, and storage-backed column add/remove/reorder commands. The next useful outcome is not another broad command, but confidence that the existing commands fail safely, avoid stale popover state, and have a repeatable full-page Screener smoke procedure.

After this work, an operator should be able to open a full-page TradingView Screener target, run representative read-only, dry-run, and bounded mutation commands on a prepared test screen, and know whether any disposable test state was left behind.

## Progress

- [x] (2026-04-26 16:52Z) Created this stabilization ExecPlan and the durable Screener completion note.
- [x] (2026-04-26 16:52Z) Identified stale filter popovers as the first narrow reliability issue to address.
- [x] (2026-04-26 17:00Z) Added stale-transient-popup cleanup before opening the option filter popover.
- [x] (2026-04-26 17:00Z) Added a focused test asserting cleanup happens before the target pill click.
- [x] (2026-04-26 17:32Z) Ran the automated Screener validation baseline.
- [x] (2026-04-26 17:05Z) Ran bounded full-page Screener read-only and dry-run smoke.
- [x] (2026-04-26 17:35Z) Updated `CONTINUITY.md` with validation and live-smoke outcomes.
- [ ] Commit the stabilization changes without pushing.

## Surprises & Discoveries

- Observation: The implemented `filters modify --option` path already rejects ambiguous option matches and refuses success without a visible-text post-check.
  Evidence: Existing tests cover dry-run target reporting, ambiguous `買` matching, successful post-check, and failed post-check.

- Observation: Full-page Screener read-only and dry-run commands work on the prepared test screen, but normal option mutation can still time out.
  Evidence: `tv screener status`, `tv screener screens active`, `tv screener filters modify --text "アナリストの評価" --option "買い" --dry-run`, and `tv screener columns reorder --from-index 12 --to-index 11 --dry-run` succeeded on `米国株（テスト用）`. A normal `filters modify --option "強い買い"` attempt returned `timeout`, and a follow-up `filters list` showed the filter remained `アナリストの評価買い`.

## Decision Log

- Decision: Treat the current Screener command set as the planned feature-complete surface for this pass.
  Rationale: `columns reset` and broader multi-option/free-text editors were already checked and remain evidence-gated, while the implemented surface is large enough that reliability now matters more than breadth.
  Date/Author: 2026-04-26 / Codex

- Decision: Stabilize stale popover handling before changing any command contract.
  Rationale: Popover residue can affect filter add/modify operations, and closing transient popups before opening the target popover is a small behavior-preserving reliability improvement.
  Date/Author: 2026-04-26 / Codex

- Decision: Do not keep iterating on normal option mutation after the timeout observed in this slice.
  Rationale: The command failed safely without changing the visible filter state. The user explicitly preferred bounded stabilization over long UI trial-and-error, so deeper click/popover retry work should be a later focused slice if it becomes worth the cost.
  Date/Author: 2026-04-26 / Codex

## Outcomes & Retrospective

Stale popup cleanup was added before option filter popovers are opened. Bounded live smoke confirmed read-only and dry-run Screener commands on the full-page test target, and also confirmed that a normal option mutation can still fail safely with a CDP timeout rather than reporting false success. Full automated validation passed.

## Context and Orientation

The Rust CLI binary is `tv`. Command parsing lives in `src/cli.rs`, dispatch lives in `src/main.rs`, and Screener operation code lives in `src/ops/screener.rs`. Screener commands communicate with TradingView Desktop through Chrome DevTools Protocol, abbreviated CDP. CDP lets the CLI evaluate JavaScript in the authenticated TradingView page and dispatch mouse clicks.

A full-page Screener target is a TradingView Desktop browser target whose page is the Screener itself rather than a chart with a side panel. `tv tab list` reports these targets under `screener_targets`, and each entry includes `target_env.TV_CDP_TARGET_ID`. Use that environment value for live Screener smoke whenever possible.

The current implemented Screener commands are documented in `docs/notes/screener-surface-completion-and-stabilization.md`. Deferred items are intentionally excluded from this stabilization plan: `columns reset`, display-name column catalog insertion, broad multi-option filter workflow semantics, and free-text filter editors.

## Plan of Work

First, update documentation so the repository has one durable note that says the main Screener surface is implemented and that stabilization is now the next phase. Link that note from README and the next-agent handoff so future contributors do not restart the same surface-selection discussion.

Second, make a narrow reliability improvement in `src/ops/screener.rs`: before the option filter editor opens a target filter pill, close existing transient Screener popups and wait briefly. This must not change the public CLI contract. It only makes repeated dry-run and live smoke less likely to read a stale menu.

Third, add a focused test that proves the option-operation script contains the transient popup cleanup before the target pill click. Existing fake-runtime tests already prove post-check semantics and ambiguous-match behavior; do not add broad brittle tests for TradingView DOM details.

Fourth, run the automated validation baseline. If a live TradingView Desktop Screener target is available, run a bounded smoke on a prepared test screen named with `テスト` or `CLI-Test`. Keep mutation smoke small: one screen dry-run, one filter option dry-run or reversible normal mutation, and one column storage dry-run or reversible reorder. Record any residual test state.

Finally, update `CONTINUITY.md` and commit the related tracked changes. Do not push.

## Concrete Steps

From the repository root, inspect the current target state:

    git status --short
    tv tab list

If `tv tab list` reports a Screener target, set the target id for live commands:

    export TV_CDP_TARGET_ID=<screener-target-id>

Run read-only Screener checks first:

    tv screener status
    tv screener screens active
    tv screener filters list
    tv screener columns config

Run dry-run mutation checks before normal mutation:

    tv screener filters modify --text "アナリストの評価" --option "買い" --dry-run
    tv screener columns reorder --from-index 12 --to-index 11 --dry-run

Only if the active screen is a prepared test or disposable screen, run one reversible normal mutation and restore it. For example:

    tv screener filters modify --text "アナリストの評価" --option "買い"
    tv screener filters modify --text "アナリストの評価" --option "強い買い"

If either normal command fails post-check, do not retry indefinitely. Record the failure and leave the implemented guard in place.

## Validation and Acceptance

Automated validation must pass:

    cargo test screener -- --nocapture
    cargo test --test cli_contract screener -- --nocapture
    cargo fmt --check
    cargo clippy --all-targets --all-features -- -D warnings
    cargo test
    git diff --check
    git grep -nE '(/Users/|C:\\|USER;)' -- README.md CHANGELOG.md docs .agents/skills || true

Acceptance for the stabilization code is that existing Screener tests pass, the CLI contract is unchanged, stale popup cleanup happens before the option filter pill is clicked, and normal filter mutations still require visible-text post-check before success.

Acceptance for live smoke is that read-only and dry-run commands succeed on the full-page target, and any normal mutation either succeeds with a verified restore path or fails safely with `internal_api_unavailable` or `timeout` rather than reporting false success.

## Idempotence and Recovery

Documentation and tests are repeatable. Dry-run Screener commands are repeatable and should not mutate TradingView state. Normal Screener mutations are intentionally limited to test or disposable screens and should be restored immediately when a reverse command is known.

If a popover remains open during live smoke, press Escape in TradingView Desktop or run another read-only command that opens and closes the relevant UI. If a test filter or screen remains, record its visible name in this plan and in `CONTINUITY.md` rather than hiding the state.

## Artifacts and Notes

Do not paste raw Screener row payloads, saved-screen storage payloads, account-linked identifiers, or machine-specific local paths into tracked docs. Record only command names, visible screen/filter/column names, counts, and high-level success or failure states.

Live smoke evidence from this slice:

    tv tab list
    # Found one full-page Screener target titled 米国株（テスト用）.

    tv screener status
    # Succeeded with open: true, screen_title: 米国株（テスト用）, 17 filters, 13 columns, and 100 visible rows.

    tv screener filters modify --text "アナリストの評価" --option "買い" --dry-run
    # Succeeded. The matched option was already selected, and no mutation was requested.

    tv screener columns reorder --from-index 12 --to-index 11 --dry-run
    # Succeeded. The expected order would move TechnicalRating before RatingMa without mutation.

    tv screener filters modify --text "アナリストの評価" --option "強い買い"
    # Failed with timeout. A follow-up filters list still showed アナリストの評価買い, so no residual mutation was observed.

Automated validation evidence:

    cargo test screener -- --nocapture
    # 66 passed; 0 failed.

    cargo test --test cli_contract screener -- --nocapture
    # 6 passed; 0 failed.

    cargo fmt --check
    # Passed.

    cargo clippy --all-targets --all-features -- -D warnings
    # Passed.

    cargo test
    # 318 unit tests and 80 CLI contract tests passed.

    git diff --check
    # Passed.

    git grep -nE '(/Users/|C:\\|USER;)' -- README.md CHANGELOG.md docs .agents/skills || true
    # Returned only existing validation-command examples in plan documents.

## Interfaces and Dependencies

No public CLI flags or JSON contract changes are intended in this plan. The relevant internal operation remains:

    pub async fn screener_filters_modify(
        runtime: &mut impl RuntimeEvaluator,
        request: ScreenerFilterModifyRequest,
    ) -> Result<Value, AppError>;

The reliability change stays inside the JavaScript generated by `filter_option_operation` in `src/ops/screener.rs`.

## Open Questions

None for this stabilization slice. Skill updates are intentionally a follow-up phase after this plan is completed.
