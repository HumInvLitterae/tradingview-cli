# Add Screener screen lifecycle commands

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This plan follows `.agents/PLANS.md`.

## Purpose / Big Picture

After this change, an operator should be able to manage disposable Stock Screener screens from the Rust `tv` CLI when the current TradingView Desktop UI exposes a stable, observable path. This matters because future Screener filter and column mutation smoke tests need safe test screens that can be created, renamed, saved as copies, and deleted without touching important layouts or non-test screens.

The visible proof is intentionally narrow. `tv screener screens actions` should reveal which lifecycle actions are available from the active screen menu. Commands such as `tv screener screens create --name <NAME> --dry-run`, `rename`, `delete`, or `save-as` should be added only when live evidence shows a stable dialog flow and a post-check that proves the requested screen state changed. If a flow is not stable, the command must not be exposed.

## Progress

- [x] (2026-04-26 15:31Z) Read `.agents/PLANS.md`, continuity rules, current Screener CLI definitions, dispatch, operation implementation, and contract tests.
- [x] (2026-04-26 15:50Z) Gathered live evidence for screen lifecycle actions in the active TradingView Desktop Screener UI.
- [x] (2026-04-26 16:20Z) Implemented evidence-backed create/rename/save-as commands and dry-run delete target resolution.
- [x] (2026-04-26 16:24Z) Added focused operation and CLI contract tests.
- [x] (2026-04-26 16:50Z) Updated README, CHANGELOG, contract notes, Screener evidence notes, upstream triage, next-agent handoff, and this plan.
- [x] (2026-04-26 17:00Z) Ran automated validation and safe live smoke.
- [x] (2026-04-26 17:03Z) Updated `CONTINUITY.md`.
- [ ] Commit the completed slice.

## Surprises & Discoveries

- The active Screener screen menu exposed save, share, copy, rename, CSV
  download, create, recent, and open actions. On the built-in `すべての株式`
  screen, save/share/rename were disabled while copy and create were enabled.
- Create opens a name dialog whose visible Japanese label is
  `スクリーンを作成`. Rename opens `スクリーン名の変更`. Copy/save-as opens
  `スクリーンのコピーを作成`.
- `tv screener screens create --name CLI-Test-Codex-426B --dry-run` succeeded
  without mutation and reported the create action plus a visible name input.
- Earlier smoke on a prepared test screen showed save-as and rename dry-run
  dialogs. A later save-as dry-run from `すべての株式` timed out, so save-as
  remains guarded by live dialog evidence and post-checks rather than treated as
  universally stable.
- `tv screener screens create --name CLI-Test-Codex-426A` created the
  disposable screen but the first run returned `internal_api_unavailable`
  because the active-title wait was too short. The wait loop was extended. The
  disposable screen `CLI-Test-Codex-426A` remained visible because normal delete
  was not verified.
- Saved-screen catalog evidence is sufficient to resolve an exact delete target
  in dry-run mode. It is not sufficient to click a normal delete action safely:
  the current catalog exposes delete controls, but the exact per-screen action
  and confirmation path were not verified.

## Decision Log

- Decision: Treat screen lifecycle commands as evidence-gated mutations on test screens only.
  Rationale: Create, rename, delete, and save-as can change TradingView cloud state. The user wants mutation candidates considered, but existing important layout/screen state must not be damaged.
  Date/Author: 2026-04-26 / Codex.

- Decision: Do not expose a lifecycle subcommand unless its dialog path and post-check are both observed.
  Rationale: The Screener UI is DOM-fragile. A CLI command that opens a dialog but cannot verify the resulting screen state would create a misleading contract and could leave account state dirty.
  Date/Author: 2026-04-26 / Codex.

- Decision: Expose `screens delete` only as exact-target dry-run resolution for
  now; normal delete must fail with `internal_api_unavailable`.
  Rationale: Operators need to see whether a disposable screen can be targeted,
  but deleting the wrong cloud-backed screen is worse than leaving a test screen
  behind. The current live UI did not prove a safe exact-screen delete action.
  Date/Author: 2026-04-26 / Codex.

## Outcomes & Retrospective

This slice added guarded Screener screen lifecycle commands. `create`,
`rename`, and `save-as` support dry-run dialog reporting and normal test-screen
mutations with active-title post-checks. `delete` resolves an exact catalog
target in dry-run mode but normal deletion remains unsupported until a safe
exact-screen delete path is proven.

The live smoke intentionally avoided broad normal mutation. One disposable test
screen, `CLI-Test-Codex-426A`, remains visible from the earlier create smoke.
This is acceptable test data but should be cleaned up manually or by a future
verified delete path.

## Context and Orientation

The Rust CLI command definitions live in `src/cli.rs`. Command dispatch lives in `src/main.rs`. Operation functions are re-exported from `src/ops.rs`, and the Stock Screener implementation lives in `src/ops/screener.rs`. Screener commands use Chrome DevTools Protocol, abbreviated CDP, to evaluate small JavaScript snippets inside the running TradingView Desktop page and dispatch mouse or keyboard events.

The existing Screener surface supports opening and closing the dialog, reading rows and metadata, listing and switching saved screens from the visible menu or catalog, saving the active screen, adding/modifying/removing/clearing filters, inspecting columns, and the guarded screen lifecycle commands added by this slice.

Successful CLI responses use the Rust JSON envelope with command-specific payload under top-level `data`. Screener payloads use `source: "ui_screener_dialog"`.

## Plan of Work

First, gather live evidence from the current TradingView Desktop session. Run `tv screener status`, `tv screener screens active`, `tv screener screens actions`, and `tv screener screens list --catalog`. Then inspect the active screen menu and catalog dialogs for visible labels and controls related to create, rename, delete, and copy/save-as. Record only high-level labels, availability flags, and command outcomes. Do not record raw Screener table rows, account-linked identifiers, or machine-specific absolute paths.

Then choose the implementation scope from the evidence. If the active screen menu exposes a stable `Create new screen` / `新規スクリーンを作成` flow with a text input and a post-check through active title or catalog entry, add `tv screener screens create --name <NAME> [--dry-run]`. If the menu exposes a stable `Rename` flow for the active or exact target screen, add `tv screener screens rename --name <CURRENT> --to <NEW> [--dry-run]`. If the catalog or menu exposes a stable delete flow and confirmation dialog, add `tv screener screens delete --name <NAME> [--dry-run] --confirm-delete`, with normal deletion restricted to names containing `CLI-Test` or `テスト`. If copy or save-as is clearly exposed and verifiable, add `tv screener screens save-as --name <NAME> [--dry-run]`.

For each implemented command, validate names before CDP connection. Blank names are invalid. Rename must reject identical current and new names. Delete must reject normal mutation without `--confirm-delete`, and must reject non-test names before CDP connection. Dry-run must return the target action and target screen or requested name without mutation. Normal mode must return success only after a post-check confirms the expected title or catalog state.

If live evidence does not prove a flow, leave the normal mutation unsupported
and record the deferred reason here and in the durable notes. `screens delete`
is the one intentionally exposed read/dry-run exception: it resolves an exact
catalog target for operator safety, but normal mode fails until exact deletion is
verified.

## Concrete Steps

Run evidence commands from the repository root. If multiple TradingView targets are open, first run `tv tab list` and use `TV_CDP_TARGET_ID=<target id>` for all live commands.

    target/debug/tv tab list
    TV_CDP_TARGET_ID=<target> target/debug/tv screener status
    TV_CDP_TARGET_ID=<target> target/debug/tv screener screens active
    TV_CDP_TARGET_ID=<target> target/debug/tv screener screens actions
    TV_CDP_TARGET_ID=<target> target/debug/tv screener screens list --catalog

Use `TV_ALLOW_UNSAFE_UI_EVAL=1 tv ui eval` only for bounded DOM evidence gathering. Keep outputs concise and sanitize any live account-linked values before writing tracked docs.

If commands are implemented, update `src/cli.rs`, `src/main.rs`, `src/ops.rs`, `src/ops/screener.rs`, and `tests/cli_contract.rs`. Update documentation only to reflect actual implemented behavior and evidence.

## Validation and Acceptance

Run focused tests:

    cargo test screener -- --nocapture
    cargo test --test cli_contract screener -- --nocapture

Run the full baseline:

    cargo fmt --check
    cargo clippy --all-targets --all-features -- -D warnings
    cargo test
    git diff --check
    git grep -nE '(/Users/|C:\\|USER;)' -- README.md CHANGELOG.md docs .agents/skills || true

Live acceptance depends on what is implemented. Dry-run smoke must never mutate. Normal create/rename/delete/save-as smoke must use only disposable names such as `CLI-Test-Codex-<short suffix>` and must record any remaining test screen name in this plan. Delete smoke must never target non-test screens.

## Idempotence and Recovery

The evidence commands and dry-runs can be repeated. Normal lifecycle commands may change TradingView cloud state, so run them only on disposable test screens. If a dialog remains open, press Escape or run `tv screener close`. If a disposable screen remains, leave its exact visible name in this plan so it can be cleaned up later.

## Artifacts and Notes

- `CLI-Test-Codex-426A` was created during live smoke and remains as disposable
  test data.
- Validation passed:
  - `cargo fmt --check`
  - `cargo test screener -- --nocapture`
  - `cargo test --test cli_contract screener -- --nocapture`
  - `cargo clippy --all-targets --all-features -- -D warnings`
  - `cargo test`
  - `git diff --check`
  - tracked-doc grep for local absolute paths or `USER;`, with only existing
    validation-command examples found

## Interfaces and Dependencies

Do not add crate dependencies. Reuse the existing `RuntimeEvaluator`, `AppError`, `serde_json::Value`, `js_string`, `Input.insertText`, CDP mouse click helpers, `ScreenerMutationSession`, `read_screener_state`, and screen menu/catalog helper patterns in `src/ops/screener.rs`.

At completion, expose only the commands that live evidence supports. Candidate signatures are:

    pub async fn screener_screens_create(runtime: &mut impl RuntimeEvaluator, name: &str, dry_run: bool) -> Result<Value, AppError>;
    pub async fn screener_screens_rename(runtime: &mut impl RuntimeEvaluator, name: &str, new_name: &str, dry_run: bool) -> Result<Value, AppError>;
    pub async fn screener_screens_delete(runtime: &mut impl RuntimeEvaluator, name: &str, dry_run: bool, confirm_delete: bool) -> Result<Value, AppError>;
    pub async fn screener_screens_save_as(runtime: &mut impl RuntimeEvaluator, name: &str, dry_run: bool) -> Result<Value, AppError>;

## Open Questions

UNCONFIRMED: Whether the current TradingView Desktop UI exposes stable create, rename, delete, or save-as flows that can be verified without guessing.

UNCONFIRMED: Whether delete actions are available from the saved-screen catalog, the active screen title menu, or only another per-screen overflow menu.
