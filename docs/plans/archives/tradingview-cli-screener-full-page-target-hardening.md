# Harden full-page Screener target handling

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document follows `.agents/PLANS.md`.

## Purpose / Big Picture

TradingView Desktop can show Stock Screener either inside a chart-side panel or as a full-page Screener tab. The full-page target is more promising for future Screener work because it is exposed as its own Chrome DevTools Protocol page at a `tradingview.com/screener/...` URL and avoids some chart panel noise. After this change, operators can use `tv tab list` to find the full-page Screener target, run `tv screener ...` commands against it with `TV_CDP_TARGET_ID`, and get accurate screen title menu entries instead of shortcut labels such as `⌘ S`.

## Progress

- [x] (2026-04-26 14:31Z) Confirmed `tv tab list` exposes the active full-page Screener target through `screener_targets`.
- [x] (2026-04-26 14:34Z) Confirmed `TV_CDP_TARGET_ID=<screener target> tv screener status/get` succeeds on the full-page Screener tab.
- [x] (2026-04-26 14:39Z) Probed full-page `screens active/actions/list`, `filters list/actions`, `columns list/actions`, and `columns remove --dry-run`.
- [x] (2026-04-26 14:45Z) Fixed `screens list` so shortcut labels in the full-page screen title menu are not reported as saved screen names.
- [x] (2026-04-26 14:49Z) Re-ran focused tests and live smoke for `screens list`.
- [x] (2026-04-26 15:01Z) Full validation baseline passed: `cargo fmt --check`, `cargo test screener_screens_list -- --nocapture`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`, `git diff --check`, and tracked-doc grep for local absolute paths or `USER;` with only existing validation-command examples found.

## Surprises & Discoveries

- Observation: The full-page Screener tab is a normal CDP page target with a Screener URL, while `tv tab list` still preserves chart targets separately.
  Evidence: `tv tab list` returned `screener_targets[0].url` as `https://jp.tradingview.com/screener/...` and `target_env.TV_CDP_TARGET_ID`.

- Observation: Full-page `tv screener status` and `tv screener get --limit 1` work without the chart-side Screener toolbar button.
  Evidence: `status` returned `open: true`, `button_found: false`, `screen_title: "米国株（テスト用）のコピー"` or `"米国株（テスト用)"`, and `get --limit 1` returned 13 columns and 17 filters.

- Observation: The full-page screen title menu includes keyboard shortcut labels that the previous menu-entry collector mistook for saved screen names.
  Evidence: before the fix, `tv screener screens list` returned entries named `⌘ S`, `⇧ N`, and `ドット`; after the fix it returned saved screens such as `米国株（テスト用）`, `米国株`, and `CLI-Test-Codex-DelSmoke`.

- Observation: Full-page catalog paths are still unstable.
  Evidence: `tv screener screens list --catalog` and `tv screener screens switch --name CLI-Test-Codex-DelSmoke --dry-run` timed out. This slice intentionally does not deepen catalog automation.

## Decision Log

- Decision: Prefer full-page Screener targets for future Screener live smoke when available, but do not rewrite all Screener commands around that assumption yet.
  Rationale: Full-page read commands are already more stable, but catalog-backed screen switching and deletion still require separate evidence.
  Date/Author: 2026-04-26 / Codex.

- Decision: Fix only the screen title menu shortcut misclassification in this slice.
  Rationale: It is a small, observable correctness bug caught by live full-page smoke. Catalog timeout and normal delete remain larger UI automation problems.
  Date/Author: 2026-04-26 / Codex.

## Outcomes & Retrospective

The slice adds a narrow hardening fix: `tv screener screens list` no longer treats keyboard shortcut labels as saved screen names on a full-page Screener target. Full-page read and metadata commands are promising, but catalog-backed commands still time out and should remain deferred unless a later plan specifically targets them. The validation baseline passed, and no destructive Screener mutation was run.

## Context and Orientation

The Rust CLI command definitions are in `src/cli.rs`. Tab target discovery and tab-strip operations are in `src/ops/tab.rs`. Screener UI operations are in `src/ops/screener.rs`. A Chrome DevTools Protocol target is a page exposed by TradingView Desktop through its local debugging endpoint. Operators can select a target by setting `TV_CDP_TARGET_ID`.

The previous slice made `tv tab list` expose `screener_targets`. Each screener target has an id and `target_env.TV_CDP_TARGET_ID`, allowing commands such as:

    TV_CDP_TARGET_ID=<screener-target-id> tv screener get --limit 1

This slice only fixes a full-page read accuracy issue. It does not create, rename, delete, or save TradingView screens.

## Plan of Work

Update `src/ops/screener.rs` in the JavaScript helper block used by `tv screener screens list`. Add a helper that recognizes keyboard shortcut labels in the screen title menu and filter those labels out of `collectScreenerScreenEntries`.

Record the live evidence in this plan and update `CONTINUITY.md` so the next contributor knows that full-page Screener targets are preferred for read smoke, while catalog-backed full-page commands still time out.

## Concrete Steps

From the repository root, run:

    tv tab list
    TV_CDP_TARGET_ID=<screener-target-id> target/debug/tv screener status
    TV_CDP_TARGET_ID=<screener-target-id> target/debug/tv screener screens list
    TV_CDP_TARGET_ID=<screener-target-id> target/debug/tv screener columns remove --name "EMA (21)" --dry-run

Expected behavior after the code change is that `screens list` returns real screen names rather than shortcut labels. `columns remove --dry-run` should remain non-mutating and may still report `remove_supported: false`.

## Validation and Acceptance

Run the focused checks:

    cargo fmt --check
    cargo test screener_screens_list -- --nocapture

Run the full baseline before committing:

    cargo clippy --all-targets --all-features -- -D warnings
    cargo test
    git diff --check
    git grep -nE '(/Users/|C:\\|USER;)' -- README.md AGENTS.md docs .agents/skills || true

Live acceptance is that `tv tab list` returns at least one `screener_targets` entry when a full-page Screener tab is open, and `TV_CDP_TARGET_ID=<that id> tv screener screens list` returns saved screen names without `⌘ S`, `⇧ N`, or `ドット`.

## Idempotence and Recovery

The code change is read-only. Re-running the live commands may open and close transient screen title menus, but it should not save or mutate a screen. If a menu remains open, pressing Escape in TradingView Desktop or re-running `tv screener status` on the selected target is safe.

## Artifacts and Notes

Short live evidence after the fix:

    "screen_count": 5,
    "screens": [
      { "name": "米国株（テスト用）", "active": true },
      { "name": "米国株（テスト用）のコピー", "active": false },
      { "name": "米国株", "active": false },
      { "name": "CLI-Test-Codex-DelSmoke2", "active": false },
      { "name": "CLI-Test-Codex-DelSmoke", "active": false }
    ]

Catalog-backed commands remain deferred:

    tv screener screens list --catalog
    tv screener screens switch --name CLI-Test-Codex-DelSmoke --dry-run

Both timed out against the full-page target during this slice.

## Interfaces and Dependencies

At the end of this slice, `src/ops/screener.rs` contains a JavaScript helper named `screenerScreenShortcutText(text)` and `collectScreenerScreenEntries(menu, activeTitle)` calls it before adding a screen entry.

No new Rust public function is introduced. No new CLI flags are introduced.

## Open Questions

- UNCONFIRMED: Whether TradingView exposes a stable full-page saved-screen catalog DOM path that can support catalog-backed switching or delete without timeout.
- UNCONFIRMED: Whether opening a full-page Screener tab can be automated safely from `tv` without relying on generic UI automation. This remains separate from the current read accuracy fix.
