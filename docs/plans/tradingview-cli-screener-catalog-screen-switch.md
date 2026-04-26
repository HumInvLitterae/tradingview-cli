# Add Screener catalog screen list and switch

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document follows `.agents/PLANS.md`.

## Purpose / Big Picture

After this work, an operator can ask the Rust `tv` CLI to read saved Stock Screener screens from the Screener catalog and switch between prepared test screens by exact name. The existing `tv screener screens list` and `tv screener screens switch` commands only use entries visible in the screen-title menu; live smoke showed that this menu-visible path can target rows but fail to activate a different screen. The new catalog path adds an explicit `--catalog` flag so the old behavior remains available while a more complete saved-screen flow can be tested and improved.

The visible proof is `tv screener screens list --catalog` returning `scope: "screen_catalog"` with saved-screen rows, and `tv screener screens switch --name <NAME> --catalog --dry-run` resolving an exact target without mutating. Normal switching is allowed only for prepared test screens and is successful only if the post-click active screen title equals the requested name.

## Progress

- [x] (2026-04-26 07:59Z) Read continuity state, existing Screener implementation, upstream PR #66, and the accepted plan.
- [x] (2026-04-26 08:10Z) Added CLI flags and Screener operation support for catalog list/switch.
- [x] (2026-04-26 08:12Z) Added operation and CLI contract tests; focused Screener tests passed.
- [x] (2026-04-26 08:17Z) Updated README, changelog, contract, feasibility, upstream triage, and handoff docs.
- [x] (2026-04-26 08:32Z) Ran safe live smoke for catalog list and dry-run switch.
- [x] (2026-04-26 08:38Z) Ran automated validation.
- [ ] Commit tracked changes.

## Surprises & Discoveries

- Observation: Existing Rust `screens list/switch` is intentionally scoped to the title menu.
  Evidence: `src/ops/screener.rs` returns `scope: "screen_title_menu"` and resolves only `collectScreenerScreenEntries(menu, activeTitle)`.

- Observation: Upstream PR #66 did not implement most catalog mutation flows either.
  Evidence: the PR body describes save-as, add filter, and add column flows as `not_implemented_yet` stubs.

- Observation: The initial code path can be extended additively by keeping default menu behavior and routing only `--catalog` through the saved-screen catalog.
  Evidence: `cargo test screener -- --nocapture` passed 23 Screener-focused tests, and `cargo test --test cli_contract screener -- --nocapture` passed 4 CLI contract tests after adding the catalog flag.

- Observation: The saved-screen catalog did not open from an in-page synthetic JavaScript click, but it did open from a CDP mouse event at the catalog menu row.
  Evidence: early `screens list --catalog` attempts returned a CDP timeout or `catalog_not_found`; after changing catalog-open to use `Input.dispatchMouseEvent`, live `screens list --catalog` returned `scope: "screen_catalog"` with two screens.

- Observation: The live catalog includes built-in popular screens after the user's saved screens, so the command should return only the `My screens` / `マイスクリーン` section.
  Evidence: a live catalog probe initially found 45 entries including popular presets; after filtering to the saved-screen section, live `screens list --catalog` returned only `米国株（テスト用）` and `米国株`.

## Decision Log

- Decision: Add catalog support behind `--catalog` instead of changing the default `screens list/switch` behavior.
  Rationale: Existing callers may rely on the current menu-visible semantics and `scope: "screen_title_menu"` payload. A flag makes the broader and more fragile catalog UI path explicit.
  Date/Author: 2026-04-26 / Codex.

- Decision: Keep screen switching exact-name only and require post-click title verification.
  Rationale: Screener screen switching is a UI mutation. Partial matching or optimistic success could switch the wrong saved screen or hide a failed click.
  Date/Author: 2026-04-26 / Codex.

- Decision: Return only the saved-screen section from `--catalog`.
  Rationale: The catalog also contains TradingView popular presets; including them would make a saved-screen operator command look broader and more mutation-prone than intended.
  Date/Author: 2026-04-26 / Codex.

## Outcomes & Retrospective

Implementation is in progress. The additive Screener catalog path is implemented in code, safe live smoke passed for catalog list plus dry-run switching, and full automated validation passed. Commit is still pending.

## Context and Orientation

The Rust CLI command definitions live in `src/cli.rs`. Command dispatch lives in `src/main.rs`. Stock Screener UI automation lives in `src/ops/screener.rs`; this module evaluates JavaScript inside the TradingView Desktop page through Chrome DevTools Protocol, often abbreviated as CDP. CDP is the local debugging protocol used here to inspect and click the running TradingView Desktop window.

The existing `tv screener screens list` command opens the Stock Screener dialog if needed, opens the active screen-title menu, and reads entries visible in that small menu. The existing `tv screener screens switch --name <NAME> [--dry-run]` clicks one of those visible menu entries and reports success only if the active screen title changes to `<NAME>`. Both commands return the Rust JSON envelope, where the command-specific payload is under top-level `data`.

In this plan, a "catalog" is the larger saved-screen dialog opened from the title menu entry named `Open screen` or `スクリーンを開く`. The catalog can show prepared saved screens such as `CLI-Test1`, `CLI-Test2`, or localized test screen names.

## Plan of Work

Add a `--catalog` boolean flag to `ScreenerScreensCommand::List` and `ScreenerScreensCommand::Switch` in `src/cli.rs`. In `src/main.rs`, pass that flag into the Screener operations.

In `src/ops/screener.rs`, change `screener_screens_list` to accept `catalog: bool`. When `catalog` is false, keep the current title-menu implementation and payload unchanged. When `catalog` is true, open the Screener dialog with restore semantics, open the screen-title menu, click the catalog entry, wait for the catalog UI, collect exact visible screen names, close the catalog UI, and return `scope: "screen_catalog"`.

Change `screener_screens_switch` to accept `catalog: bool`. When `catalog` is false, keep the current menu-visible implementation. When `catalog` is true, open the catalog UI, resolve the exact visible name, support dry-run without clicking, click the target row in normal mode, wait for the active screen title to equal the requested name, close transient UI, restore the original Screener dialog open state, and fail with `internal_api_unavailable` if the active title does not match.

Do not implement screen save, save-as, delete, rename, create, column mutation, or filter add/modify in this slice.

## Concrete Steps

Run commands from the repository root.

1. Edit `src/cli.rs`, `src/main.rs`, and `src/ops/screener.rs` to add the `--catalog` path.
2. Add tests in the existing `#[cfg(test)]` block in `src/ops/screener.rs` for catalog list, catalog dry-run switch, and catalog switch verification.
3. Update `tests/cli_contract.rs` so `screens list --help` and `screens switch --help` expose `--catalog`, and empty `--name` still fails before CDP.
4. Update user-facing and durable docs: `README.md`, `CHANGELOG.md`, `docs/notes/rust-cli-contract-migration-2026-04-24.md`, `docs/notes/screener-hotlist-upstream-feasibility-2026-04-25.md`, `docs/notes/upstream-pr-triage-2026-04-25.md`, and `docs/notes/next-agent-handoff-prompt-2026-04-24.md`.
5. Run validation:

        cargo fmt --check
        cargo clippy --all-targets --all-features -- -D warnings
        cargo test
        git diff --check
        git grep -nE '(/Users/|C:\\|USER;)' -- README.md AGENTS.md docs .agents/skills || true

6. If TradingView Desktop is reachable, run safe live smoke:

        target/debug/tv screener screens active
        target/debug/tv screener screens list --catalog
        target/debug/tv screener screens switch --name "<current screen>" --catalog --dry-run

   Only run normal catalog switching if at least two visible catalog screens contain `CLI` or `テスト` in their names. If normal switching is run, switch from one safe test screen to another and then back, and record the final active screen. In the 2026-04-26 live smoke, only one visible saved screen contained `テスト`, so normal switch smoke was skipped.

## Validation and Acceptance

The implementation is accepted when `tv screener screens list --catalog` returns a successful payload with `scope: "screen_catalog"` when the catalog UI is available, and `tv screener screens switch --name <NAME> --catalog --dry-run` resolves an exact visible catalog target without mutation. Existing non-catalog list/switch behavior must remain compatible and keep `scope: "screen_title_menu"`.

Automated tests must pass for Screener operation behavior and CLI help contracts. Full repository validation must pass because this slice changes Rust code and public CLI behavior.

Live smoke is accepted if catalog list and dry-run target resolution work. Normal switch smoke is optional and should be skipped unless safe test screens are visible; if skipped, document why.

## Idempotence and Recovery

Catalog list and dry-run switch are read-only except for transient UI opening and closing. Normal catalog switch changes only the active Screener screen. Do not run normal switch against non-test screens. If a switch succeeds, switch back to the original test screen before finishing. If a command leaves the catalog or Screener dialog open unexpectedly, run `tv screener close` or press Escape in TradingView Desktop.

## Artifacts and Notes

Record command names, screen names, counts, and high-level result fields only. Do not paste raw Screener table rows, account-linked identifiers, or local absolute paths into tracked docs.

Live smoke summary with explicit target `D202CA6B22895C82C0437F0F9FC6A7BC`:

        screener screens active: screen_title 米国株（テスト用）
        screener screens list --catalog: scope screen_catalog, screen_count 2, screens 米国株（テスト用） and 米国株
        screener screens switch --name "米国株（テスト用）" --catalog --dry-run: already_active true, switched false
        normal catalog switch: skipped because only one visible saved screen name contained テスト

Validation summary:

        cargo fmt --check: passed
        cargo clippy --all-targets --all-features -- -D warnings: passed
        cargo test: 275 unit tests and 78 CLI contract tests passed
        git diff --check: passed
        tracked-doc local path / USER; grep: only validation command examples in plan docs

## Interfaces and Dependencies

The public CLI additions are:

    tv screener screens list --catalog
    tv screener screens switch --name <NAME> --catalog [--dry-run]

The Rust operation signatures should be:

    pub async fn screener_screens_list(runtime: &mut impl RuntimeEvaluator, catalog: bool) -> Result<Value, AppError>
    pub async fn screener_screens_switch(runtime: &mut impl RuntimeEvaluator, name: &str, dry_run: bool, catalog: bool) -> Result<Value, AppError>

No new external crate or network endpoint is required.

## Open Questions

UNCONFIRMED: the current TradingView Desktop catalog DOM may differ by locale or account state. The implementation must therefore fail clearly with `internal_api_unavailable` rather than reporting an empty successful catalog when no catalog UI can be found.
