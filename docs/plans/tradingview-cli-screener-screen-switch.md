# Add Screener screen list and switch

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document follows `.agents/PLANS.md`.

## Purpose / Big Picture

After this change, an operator can use `tv screener screens list` to inspect the Stock Screener screens currently visible in the active screen title menu, then use `tv screener screens switch --name <NAME>` to switch to one of those visible screens by exact name. This is useful for live smoke and daily operation because prepared test screens such as `米国株（テスト用）` can be selected without manually clicking through the TradingView Desktop UI.

This is not full saved-screen catalog management. The command only uses entries visible in the active screen title menu, such as recent screens shown directly in that menu. It does not open the larger `スクリーンを開く...` catalog and does not implement save-as, create, rename, delete, or column mutation.

## Progress

- [x] (2026-04-25 19:15Z) Checked working tree, current Screener CLI/operation code, existing notes, and upstream PR #66 evidence.
- [x] (2026-04-25 19:21Z) Took live DOM evidence from the active Stock Screener screen title menu.
- [x] (2026-04-25 19:31Z) Added `tv screener screens list` and `tv screener screens switch --name <NAME> [--dry-run]` CLI surface and dispatch.
- [x] (2026-04-25 19:31Z) Implemented menu-visible screen extraction, dry-run target reporting, exact-name switching, and post-switch verification.
- [x] (2026-04-25 19:31Z) Added focused operation and CLI contract tests.
- [x] (2026-04-25 19:46Z) Updated README, changelog, contract notes, and upstream triage notes.
- [x] (2026-04-25 20:08Z) Ran live smoke against the prepared test screen; list and dry-run passed, while actual switch failed safely with `internal_api_unavailable`.
- [x] (2026-04-25 20:18Z) Ran full validation and recorded outcomes.
- [ ] Commit tracked changes and update the local continuity ledger.

## Surprises & Discoveries

- Observation: Upstream PR #66 does not provide a working `list` or `switch` implementation for saved Screener screens.
  Evidence: Its README and tests describe `list`, `switch`, `save_as`, `delete`, `rename`, and `create_new` as stretch actions returning `not_implemented_yet`.

- Observation: The current TradingView Desktop UI exposes the active Screener screen title as a visible button.
  Evidence: live DOM inspection found `data-name="screener-topbar-screen-title"` with text `米国株（テスト用）`.

- Observation: Clicking the title button opens a menu containing screen actions plus a recent-screen section.
  Evidence: live DOM text included `スクリーンを保存`, `コピーを作成...`, `最近使用した項目`, `米国株（テスト用）`, `米国株`, and `スクリーンを開く...`.

- Observation: The visible recent-screen entries can be read, but clicking the `米国株` entry did not activate that screen in the current TradingView Desktop session.
  Evidence: `tv screener screens switch --name "米国株"` returned `internal_api_unavailable` after post-click verification, and the final active title remained `米国株（テスト用）`.

- Observation: Opening the full `スクリーンを開く...` dialog exposes `マイスクリーン` rows for `米国株（テスト用）` and `米国株`, but clicking those rows also did not activate `米国株` in this session.
  Evidence: manual CDP mouse clicks on the custom screen dialog row returned success at the input layer, but `tv screener screens active` still returned `米国株（テスト用）`.

## Decision Log

- Decision: Implement only menu-visible screen list and exact-name switch.
  Rationale: This provides the operator workflow needed for prepared test screens while avoiding the larger saved-screen catalog and modal flows that upstream left as stubs.
  Date/Author: 2026-04-25 / Codex.

- Decision: Do not require `--confirm-switch`.
  Rationale: Switching screens changes active Screener UI state but does not delete or persist new objects by itself. Exact-name targeting, `--dry-run`, and post-switch verification provide a better safety shape than a confirmation flag that would make routine screen setup cumbersome.
  Date/Author: 2026-04-25 / Codex.

- Decision: Use exact name matching only.
  Rationale: Screen names may be localized and user-defined. Partial matching could accidentally switch to the wrong saved screen.
  Date/Author: 2026-04-25 / Codex.

- Decision: Keep the non-dry-run `switch` command but require post-click verification and fail with `internal_api_unavailable` when TradingView does not activate the requested screen.
  Rationale: The CLI should not report success for an unobserved UI mutation. Keeping the command shape and safe failure behavior preserves the interface while recording that current live TradingView behavior may need a separate catalog-specific implementation before switch is operational in all sessions.
  Date/Author: 2026-04-25 / Codex.

## Outcomes & Retrospective

Implementation is complete at the CLI and operation layer. `tv screener screens list` can read menu-visible screen entries and `tv screener screens switch --name <NAME> [--dry-run]` validates exact targets, reports dry-run targets, and verifies non-dry-run switching before success.

Live smoke proved `screens list` and `switch --dry-run` on the prepared screen `米国株（テスト用）`. Non-dry-run switching to `米国株` did not change the active screen in the current TradingView Desktop session, so the command correctly failed with `internal_api_unavailable` instead of reporting a false success. A later slice should investigate the full saved-screen catalog workflow if reliable switching is still required.

Full validation passed with `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`, `git diff --check`, and the tracked-doc grep for machine-specific absolute paths or `USER;`. The grep returned only existing validation-command examples in plan documents.

## Context and Orientation

Screener commands live in `src/ops/screener.rs`. The command parser is in `src/cli.rs`, and dispatch is in `src/main.rs`. Existing Screener commands can open the Stock Screener dialog, read visible rows and metadata, and remove visible filter pills with guards. The Stock Screener dialog is a floating TradingView UI panel, and these commands interact with it through JavaScript evaluated in the authenticated TradingView page context over Chrome DevTools Protocol.

A "menu-visible screen" means a screen name that appears directly in the active screen title menu after clicking the current screen title. This is narrower than the full saved-screen catalog opened by `スクリーンを開く...`.

## Plan of Work

Add `List` and `Switch` variants to `ScreenerScreensCommand` in `src/cli.rs`. `Switch` accepts `--name <NAME>` and `--dry-run`. Validate that `--name` is non-empty before connecting to CDP.

In `src/ops/screener.rs`, add a helper that opens the Screener dialog if needed, clicks the visible screen title button, reads screen names from the opened title menu, then closes the menu. Return only deduplicated screen names from the menu, with an `active` flag based on the current screen title. For switching, open the same menu, find an exact visible text match, click it, then wait until `readScreenerState(0)` reports the requested `screen_title`.

In docs, describe the new commands as menu-visible screen operations and keep full catalog management deferred.

## Concrete Steps

Run commands from the repository root.

1. Edit `src/cli.rs`, `src/main.rs`, `src/ops.rs`, and `src/ops/screener.rs`.
2. Add focused tests in `src/ops/screener.rs` and `tests/cli_contract.rs`.
3. Update `README.md`, `CHANGELOG.md`, `docs/notes/rust-cli-contract-migration-2026-04-24.md`, `docs/notes/screener-hotlist-upstream-feasibility-2026-04-25.md`, `docs/notes/upstream-pr-triage-2026-04-25.md`, and `docs/notes/next-agent-handoff-prompt-2026-04-24.md`.
4. Run focused tests:

        cargo test screener -- --nocapture
        cargo test --test cli_contract screener -- --nocapture

5. Run live smoke with an explicit target id:

        TV_CDP_TARGET_ID=<target> target/debug/tv screener status
        TV_CDP_TARGET_ID=<target> target/debug/tv screener screens active
        TV_CDP_TARGET_ID=<target> target/debug/tv screener screens list
        TV_CDP_TARGET_ID=<target> target/debug/tv screener screens switch --name "米国株（テスト用）" --dry-run

If both `米国株` and `米国株（テスト用）` are visible in `screens list`, attempt to switch to `米国株`. If TradingView does not activate the row, the command should fail with `internal_api_unavailable` and leave the active screen as `米国株（テスト用）`. If switching succeeds, switch back to `米国株（テスト用）`, and verify the final active screen title.

6. Run full validation:

        cargo fmt --check
        cargo clippy --all-targets --all-features -- -D warnings
        cargo test
        git diff --check
        git grep -nE '(/Users/|C:\\|USER;)' -- README.md AGENTS.md docs .agents/skills || true

## Validation and Acceptance

The change is accepted when help output lists `screens list` and `screens switch`, operation tests prove menu-visible listing and exact-name switch behavior, CLI contract tests prove validation happens before CDP connection, live smoke can list the prepared test screen, `switch --dry-run` reports the exact target without mutation, and non-dry-run switch either verifies a changed active title or fails safely with `internal_api_unavailable`.

## Idempotence and Recovery

`screens list` and `screens switch --dry-run` are safe to repeat. Switching to a different visible screen can be undone by switching back to the original visible screen. If the original screen is not visible in the menu, do not perform a destructive smoke switch. If the menu opens but the target is not visible, the command fails without clicking a screen.

## Artifacts and Notes

Record only screen names, counts, and before/after active screen titles. Do not paste raw Screener row payloads or account-linked identifiers into tracked docs.

Focused tests passed:

        cargo test screener -- --nocapture
        cargo test --test cli_contract screener -- --nocapture

Live smoke summary:

        TV_CDP_TARGET_ID=D202CA6B22895C82C0437F0F9FC6A7BC target/debug/tv screener screens list

returned `screen_count: 2` with `米国株（テスト用）` active and `米国株` visible.

        TV_CDP_TARGET_ID=D202CA6B22895C82C0437F0F9FC6A7BC target/debug/tv screener screens switch --name "米国株（テスト用）" --dry-run

returned `already_active: true`, `dry_run: true`, and did not mutate.

        TV_CDP_TARGET_ID=D202CA6B22895C82C0437F0F9FC6A7BC target/debug/tv screener screens switch --name "米国株"

returned `internal_api_unavailable` because the active screen title remained `米国株（テスト用）` after the click attempt.

Full validation passed:

        cargo fmt --check
        cargo clippy --all-targets --all-features -- -D warnings
        cargo test
        git diff --check
        git grep -nE '(/Users/|C:\\|USER;)' -- README.md AGENTS.md docs .agents/skills || true

The grep command returned only existing validation-command examples in plan documents.

## Interfaces and Dependencies

Expose these operation functions through `src/ops.rs`:

    pub fn validate_screener_screen_name(name: &str) -> Result<String, AppError>;
    pub async fn screener_screens_list(runtime: &mut impl RuntimeEvaluator) -> Result<Value, AppError>;
    pub async fn screener_screens_switch(runtime: &mut impl RuntimeEvaluator, name: &str, dry_run: bool) -> Result<Value, AppError>;

No new crate dependencies are required.

## Open Questions

No critical open questions block implementation. Full saved-screen catalog listing remains intentionally out of scope.
