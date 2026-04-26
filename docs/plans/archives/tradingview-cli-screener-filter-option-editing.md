# Add Screener filter option editing

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document follows `.agents/PLANS.md`. It is self-contained so a future contributor can continue the work from this file and the current repository tree alone.

## Purpose / Big Picture

TradingView Stock Screener filters are not all numeric ranges. Users also need to edit visible option-style filters such as analyst rating without manually clicking the Screener UI. This change extends the existing guarded `tv screener filters modify` command with a minimal option editing form:

    tv screener filters modify --index <N>|--text <TEXT> --option <TEXT> [--dry-run]

The command should only operate on an existing visible filter pill. It must resolve exactly one target filter, resolve exactly one visible option from that filter's own popover, and report success only after the visible filter text reflects the requested option. It is intentionally not a complete generic filter editor.

## Progress

- [x] (2026-04-26T16:17Z) Read the current filter modification implementation in `src/ops/screener.rs`, CLI dispatch in `src/cli.rs` and `src/main.rs`, and CLI contract tests.
- [x] (2026-04-26T16:17Z) Re-confirmed the full-page Screener target from `tv tab list`: `米国株（テスト用）`.
- [x] (2026-04-26T16:17Z) Captured live evidence that the visible `アナリストの評価` filter opens an option popover with `強い売り`, `売り`, `中立`, `買い`, `強い買い`, `評価なし`, and `すべて選択`.
- [x] (2026-04-26T16:17Z) Captured live evidence that selecting `買い` changes the visible pill text to `アナリストの評価 買い`.
- [x] (2026-04-26T16:30Z) Implemented `--option` validation and CLI dispatch.
- [x] (2026-04-26T16:45Z) Implemented option dry-run and normal mutation with visible-text post-check.
- [x] (2026-04-26T16:55Z) Updated README, CHANGELOG, contract notes, Screener evidence notes, upstream PR triage, and handoff notes.
- [x] (2026-04-26T17:05Z) Full automated validation and final grep checks passed.
- [x] (2026-04-26T17:10Z) Committed tracked changes as `feat(screener): Add filter option editing`.

## Surprises & Discoveries

- Observation: The `セクター` filter exposes many options, but selecting one during early exploration did not change the visible pill text. That makes it a poor first target because post-check would be ambiguous.
  Evidence: `tv screener filters list` still reported the visible text as `セクター` after the exploratory click.

- Observation: The `アナリストの評価` filter is a better first target because its popover is scoped and visible-text post-check is clear.
  Evidence: A read-only DOM scan found the popover text `アナリストの評価 強い売り 売り 中立 買い 強い買い 評価なし すべて選択`; selecting `買い` changed the filter list entry to `アナリストの評価 買い`.

- Observation: `アナリストの評価` is multi-select, not single-select. Clicking `強い買い` while `買い` was selected changed the visible text to `アナリストの評価2`, so a blind "click requested option" algorithm is not sufficient.
  Evidence: The first normal CLI attempt failed safely with `internal_api_unavailable` after the post-check saw `アナリストの評価2`.

- Observation: The option popover exposes selected state through `aria-selected=true`, so the command can perform a minimal replacement by clearing other selected options before selecting the requested option.
  Evidence: A DOM scan after the `アナリストの評価2` state showed `買い` and `強い買い` with `aria-selected=true`; after the replacement implementation, normal mutation changed `アナリストの評価2` to `アナリストの評価強い買い`, and a second normal mutation changed it back to `アナリストの評価買い`.

- Observation: One restore attempt timed out while a transient popover state was active. Closing transient popups and retrying succeeded.
  Evidence: `filters modify --index 7 --option "買い"` first returned a CDP timeout, then the same command succeeded after popups were closed and dry-run was re-run.

## Decision Log

- Decision: Extend `filters modify` with `--option` instead of adding a separate command.
  Rationale: The existing command already means "change an existing visible filter"; an option edit is another edit mode. Keeping it under `modify` also reuses target resolution, dry-run, and post-check concepts.
  Date/Author: 2026-04-26 / Codex.

- Decision: Make `--option` mutually exclusive with `--min` and `--max`.
  Rationale: Numeric range preset editing and option selection use different UI surfaces and different post-check expectations. Combining them would make success semantics unclear.
  Date/Author: 2026-04-26 / Codex.

- Decision: Require visible option resolution and visible filter text post-check for normal mode.
  Rationale: Screener UI automation is fragile. A click that cannot be verified must fail safely rather than pretending the account state changed.
  Date/Author: 2026-04-26 / Codex.

- Decision: Treat option editing as single-option replacement, not additive multi-select editing.
  Rationale: The first stable operator use case is "set this visible option filter to one option". Broader add/remove/replace modes need separate semantics and should not be smuggled into this small surface.
  Date/Author: 2026-04-26 / Codex.

## Outcomes & Retrospective

Implementation, validation, live smoke, and commit are complete. The slice adds `tv screener filters modify --option` for existing visible option filters, with dry-run option reporting, selected-option cleanup when TradingView exposes selection state, and visible-text post-checks. Broader multi-option workflow semantics and free-text filter editors remain deferred.

Validation passed with `cargo test screener_filter -- --nocapture`, `cargo test --test cli_contract screener -- --nocapture`, `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`, `git diff --check`, and the tracked-doc local path / `USER;` grep. The grep returned only existing validation-command examples in plan documents.

## Context and Orientation

The Rust CLI entry point is `src/main.rs`. The command-line shape is defined in `src/cli.rs` using `clap`. Screener operations live in `src/ops/screener.rs`, which includes JavaScript helper functions embedded in `SCREENER_HELPERS` for reading and cautiously clicking the TradingView page.

`tv screener filters modify` currently supports only numeric range presets. The user passes `--min` and/or `--max`; Rust validates finite supported values before connecting to CDP, maps the numbers to a visible preset label such as `0% 〜 10%`, clicks the visible filter pill, chooses the preset, and waits until the visible filter text contains the requested preset.

For this plan, an "option-style filter" means a filter pill whose edit popover exposes textual choices such as `買い` or `強い買い` rather than numeric range presets. A "post-check" means re-reading `readScreenerState` after mutation and requiring the same filter pill to contain the requested option text.

## Plan of Work

First, update `src/cli.rs` so `ScreenerFiltersCommand::Modify` accepts `--option <TEXT>`. Then update both pre-dispatch validation and runtime dispatch in `src/main.rs` to pass that option into `ops::validate_screener_filter_modify_request`.

Next, update `src/ops/screener.rs`. Replace the current range-only request shape with an edit-mode enum or equivalent structure that can represent either a numeric range preset or a single option. Validation must reject missing selectors, conflicting selectors, blank `--option`, missing edit input, and any use of `--option` together with `--min` or `--max` before CDP connection.

Then add JavaScript helpers for option resolution. The helper should click the target filter pill, find the edit popover related to that pill, collect visible option-like elements in that popover, normalize their text, and require exactly one match for the requested option. Exact normalized matches should be preferred. If multiple visible options match, the command should return a validation-style failure with available options instead of clicking.

Dry-run mode should open and resolve the option but should close transient popups without clicking the option. Normal mode should click the option and then call the same style of wait loop used by numeric range editing, but the expected text is the requested option. Success must include the target filter, the matched option, the after filter, and before/after counts.

Finally, update README, CHANGELOG, contract notes, Screener feasibility notes, upstream PR triage, and the next-agent handoff note so they no longer say generic non-numeric filter editing is entirely deferred. Keep the documented boundary clear: only single visible option selection is supported.

## Concrete Steps

Run all commands from the repository root.

Before editing, confirm the live target:

    target/debug/tv tab list
    TV_CDP_TARGET_ID=<screener target> target/debug/tv screener filters list

After implementation, use a dry-run first:

    TV_CDP_TARGET_ID=<screener target> target/debug/tv screener filters modify --text "アナリストの評価" --option "強い買い" --dry-run

If dry-run resolves the exact option, run one normal mutation on the prepared test screen and then restore it:

    TV_CDP_TARGET_ID=<screener target> target/debug/tv screener filters modify --text "アナリストの評価" --option "強い買い"
    TV_CDP_TARGET_ID=<screener target> target/debug/tv screener filters modify --text "アナリストの評価" --option "買い"

The live smoke for this slice restored the visible filter text to `アナリストの評価買い`, which matches the already-mutated state created during early exploration. It did not restore to the original all-options state because `すべて選択` has different visible-text semantics and is left outside this minimal single-option replacement surface.

## Validation and Acceptance

Run:

    cargo test screener_filter -- --nocapture
    cargo test --test cli_contract screener -- --nocapture
    cargo fmt --check
    cargo clippy --all-targets --all-features -- -D warnings
    cargo test
    git diff --check
    git grep -nE '(/Users/|C:\\|USER;)' -- README.md CHANGELOG.md docs .agents/skills || true

Acceptance requires that `--option` appears in `tv screener filters modify --help`, invalid option/range combinations fail before CDP connection, dry-run reports a matched option without mutation, normal mode does not report success without a visible-text post-check, and live smoke either restores the test screen or records the exact remaining test-screen state.

## Idempotence and Recovery

The implementation is additive. Validation commands can be re-run safely. Live smoke must use only the prepared test Screener screen `米国株（テスト用）`. If a normal option mutation succeeds but restore fails, leave the test screen in the observed state and record the visible filter text. Do not mutate non-test screens.

## Artifacts and Notes

Live evidence summary, without raw DOM payloads or account-linked identifiers:

    tv screener filters list
    filter 7 before: アナリストの評価

    DOM option scan:
    アナリストの評価 options: 強い売り, 売り, 中立, 買い, 強い買い, 評価なし, すべて選択

    exploratory click:
    selected: 買い
    filter 7 after: アナリストの評価 買い

    CLI smoke:
    dry-run selected target option: 強い買い
    normal mutation after text: アナリストの評価強い買い
    restore mutation after text: アナリストの評価買い

## Interfaces and Dependencies

The public CLI interface added by this plan is:

    tv screener filters modify --index <N>|--text <TEXT> --option <TEXT> [--dry-run]

In `src/ops/screener.rs`, validation should still expose:

    pub fn validate_screener_filter_modify_request(
        index: Option<usize>,
        text: Option<&str>,
        min: Option<f64>,
        max: Option<f64>,
        option: Option<&str>,
        dry_run: bool,
    ) -> Result<ScreenerFilterModifyRequest, AppError>

The output remains under the Rust JSON envelope's `data` field. Successful option dry-run and normal mode should use `action: "filter_modify"` and include `requested_option`.

## Open Questions

No critical question blocks implementation. Future work remains for multi-option add/remove/replace semantics and free-text filters.
