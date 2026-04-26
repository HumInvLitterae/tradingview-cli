# Add guarded Screener screen delete

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This plan follows `.agents/PLANS.md`.

## Purpose / Big Picture

After this change, an operator can remove disposable Stock Screener screens from the Rust `tv` CLI instead of leaving test screens behind after `screens create` or `screens save-as` smoke tests. The command remains guarded: normal deletion requires `--confirm-delete`, only test/disposable names containing `CLI-Test` or `テスト` are accepted, and success is reported only after the target no longer appears in the saved screen list.

The visible proof is `tv screener screens delete --name <TEST_SCREEN> --dry-run` resolving the exact target without mutation, followed by `tv screener screens delete --name <TEST_SCREEN> --confirm-delete` returning `deleted: true` and an after-count one lower than the before-count.

## Progress

- [x] (2026-04-27 14:30Z) Confirmed the working tree was clean and read the current Screener delete implementation, ExecPlan requirements, and recent Screener target evidence.
- [x] (2026-04-27 14:40Z) Took live full-page Screener evidence for `status`, `screens active`, `screens list`, `screens list --catalog`, and `screens delete --dry-run`.
- [x] (2026-04-27 14:55Z) Confirmed the TradingView standalone Screener bundle uses the saved-screen storage API for `removeScreen`.
- [x] (2026-04-27 15:10Z) Reworked `tv screener screens delete` to use the storage API with exact name resolution and post-delete absence verification.
- [x] (2026-04-27 15:17Z) Added focused tests for dry-run, normal delete, and active-screen refusal.
- [x] (2026-04-27 15:25Z) Ran live smoke: `CLI-Test-Codex-426A` was already absent; `CLI-Test-Codex-DelSmoke2` was deleted and verified absent.
- [x] (2026-04-27 15:45Z) Full validation baseline passed: `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`, `git diff --check`, and tracked-doc grep for local absolute paths or `USER;` with only existing validation-command examples found.
- [ ] Commit the completed slice.

## Surprises & Discoveries

- Observation: The originally planned cleanup target `CLI-Test-Codex-426A` was already absent from the saved-screen storage list.
  Evidence: `tv screener screens delete --name CLI-Test-Codex-426A --dry-run` returned a validation error with the current saved-screen names, none of which matched that target.

- Observation: Full-page `screens list --catalog` works, but catalog-backed `screens delete --dry-run` could still time out through the old DOM path.
  Evidence: `screens list --catalog` returned five saved screens, while the pre-change delete dry-run timed out during the catalog/restore flow.

- Observation: TradingView's standalone Screener frontend removes custom screens through a storage API rather than a unique DOM-only path.
  Evidence: the loaded standalone Screener bundle contains a `removeScreen` path that calls `DELETE` on the saved-screen storage endpoint with `credentials: "include"`.

- Observation: Live normal delete worked against a disposable screen.
  Evidence: deleting `CLI-Test-Codex-DelSmoke2` returned HTTP status 204 from the storage API and the post-check saved-screen count changed from five to four with that name absent.

## Decision Log

- Decision: Implement `screens delete` through the page-session storage API instead of the saved-screen catalog DOM.
  Rationale: The storage API is the path TradingView's own Screener frontend uses, supports exact ids after exact name resolution, avoids fragile per-row hover/overflow controls, and can be post-checked by re-fetching saved screens.
  Date/Author: 2026-04-27 / Codex.

- Decision: Keep the existing delete safety contract.
  Rationale: Deleting saved screens mutates TradingView cloud state. `--confirm-delete`, test-name guards, and exact target resolution remain necessary even though the implementation path changed.
  Date/Author: 2026-04-27 / Codex.

- Decision: Refuse normal deletion of the active screen.
  Rationale: Deleting the currently active screen can cause implicit fallback navigation. Operators can switch away first, and inactive exact deletion is simpler to verify safely.
  Date/Author: 2026-04-27 / Codex.

## Outcomes & Retrospective

The slice makes `tv screener screens delete` a real guarded cleanup command instead of a dry-run-only boundary. It uses the same authenticated page session that TradingView Desktop already has open, resolves saved screens by exact name, deletes by storage id, and verifies absence before returning success. Full automated validation passed.

The live cleanup target from the earlier plan, `CLI-Test-Codex-426A`, was already gone. One disposable screen, `CLI-Test-Codex-DelSmoke2`, was deleted during smoke. `CLI-Test-Codex-DelSmoke` remains as disposable test data.

## Context and Orientation

The command parser is in `src/cli.rs`, dispatch is in `src/main.rs`, and Screener operations live in `src/ops/screener.rs`. Screener commands run JavaScript inside the current TradingView Desktop page through Chrome DevTools Protocol, abbreviated CDP. The current full-page Screener tab is exposed as a separate CDP target; operators can discover it with `tv tab list` and then set `TV_CDP_TARGET_ID` to the returned Screener target id.

Before this slice, `tv screener screens delete --name <NAME> --dry-run` opened the saved-screen catalog and resolved a target, but normal delete always failed with `internal_api_unavailable`. The old catalog path was also brittle on the full-page Screener target because it could time out while opening or closing catalog UI.

TradingView's own standalone Screener frontend uses a saved-screen storage API. In the page session, `window.initData.SCREENER_STORAGE_URL` provides the base URL, `window.initData.standalone_type` provides the Screener key such as `stock`, and `window.initData.screener_storage_release_version` provides the storage schema version. The CLI should use those values from the page rather than hardcoding a URL or version.

## Plan of Work

In `src/ops/screener.rs`, extend the internal `ScreenerScreenTarget` with optional storage metadata: `id`, `owner`, and `shared`. Keep these fields internal to target resolution and include them in JSON payloads so operators can see what exact saved screen was targeted.

Change `screener_screens_delete` so it no longer opens the catalog DOM. It should read the current Screener state only to know the active title, then evaluate a small JavaScript snippet that fetches custom saved screens from the storage API with `credentials: "include"`. Resolve the requested screen by exact `name`. In dry-run mode, return the exact target and full candidate list without mutation. In normal mode, require the already validated confirmation and test-name guard, refuse active targets, call `DELETE` on the target storage id, fetch the saved-screen list again, and return success only if the deleted name is absent.

Keep the existing CLI shape unchanged: `tv screener screens delete --name <NAME> [--dry-run] --confirm-delete`. Do not add new flags. Do not change `screens list --catalog` or `screens switch --catalog` in this slice.

Update README and CHANGELOG so users no longer see normal delete described as disabled. Update `CONTINUITY.md` after validation with the live smoke result. Do not write raw storage ids, local absolute paths, or account-linked identifiers into tracked docs.

## Concrete Steps

From the repository root, use the full-page Screener target when available:

    target/debug/tv tab list
    TV_CDP_TARGET_ID=<screener-target> target/debug/tv screener status
    TV_CDP_TARGET_ID=<screener-target> target/debug/tv screener screens active
    TV_CDP_TARGET_ID=<screener-target> target/debug/tv screener screens list
    TV_CDP_TARGET_ID=<screener-target> target/debug/tv screener screens list --catalog

Run dry-run before mutation:

    TV_CDP_TARGET_ID=<screener-target> target/debug/tv screener screens delete --name CLI-Test-Codex-426A --dry-run

If that exact target is absent, choose one existing disposable `CLI-Test` screen for smoke. Do not create a new screen just to test delete. Run:

    TV_CDP_TARGET_ID=<screener-target> target/debug/tv screener screens delete --name <disposable-test-screen> --dry-run
    TV_CDP_TARGET_ID=<screener-target> target/debug/tv screener screens delete --name <disposable-test-screen> --confirm-delete

Acceptance for live smoke is a success payload with `scope: "screen_storage_api"`, `deleted: true`, and the target name absent from the returned `screens` array.

## Validation and Acceptance

Run focused tests:

    cargo test screener_screen -- --nocapture
    cargo test --test cli_contract screener -- --nocapture

Run the full baseline:

    cargo fmt --check
    cargo clippy --all-targets --all-features -- -D warnings
    cargo test
    git diff --check
    git grep -nE '(/Users/|C:\\|USER;)' -- README.md CHANGELOG.md docs .agents/skills || true

The change is accepted when invalid deletes still fail before CDP connection, dry-run resolves an exact storage target without mutation, normal delete refuses active screens, normal delete verifies post-delete absence, and live smoke deletes only a disposable test screen.

## Idempotence and Recovery

Dry-run is repeatable and non-mutating. Normal delete is not repeatable for the same target: the second run should fail validation because the target name is no longer present. If a screen is accidentally active, switch away before deletion; the command itself refuses active targets. If a UI popover remains open from evidence gathering, press Escape or run `tv screener close`.

## Artifacts and Notes

Live evidence from this slice:

    screens list --catalog returned five saved screens before delete.
    CLI-Test-Codex-426A was already absent.
    CLI-Test-Codex-DelSmoke2 dry-run resolved an inactive disposable target.
    CLI-Test-Codex-DelSmoke2 normal delete returned deleted: true.
    The post-check saved-screen count changed from five to four.
    The full-page Screener target was navigated back to 米国株（テスト用） after smoke.

## Interfaces and Dependencies

No new crate dependency is introduced. Reuse `RuntimeEvaluator::evaluate`, `AppError`, `serde_json::Value`, and the existing `js_string` helper.

At completion, `src/ops/screener.rs` contains these internal helpers:

    async fn fetch_screener_storage_screens(runtime: &mut impl RuntimeEvaluator, active_title: Option<&str>) -> Result<Value, AppError>;
    async fn delete_screener_storage_screen(runtime: &mut impl RuntimeEvaluator, target: &ScreenerScreenTarget) -> Result<Value, AppError>;

The public operation signature remains:

    pub async fn screener_screens_delete(runtime: &mut impl RuntimeEvaluator, name: &str, dry_run: bool, confirm_delete: bool) -> Result<Value, AppError>;

## Open Questions

No blocker remains for guarded inactive test-screen deletion. Column normal mutation, column add/reorder/reset, and broad non-numeric filter editing remain separate Screener follow-up topics.
