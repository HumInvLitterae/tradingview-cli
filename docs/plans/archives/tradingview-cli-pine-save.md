# Add Pine save command

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document follows `.agents/PLANS.md`.

## Purpose / Big Picture

After this change, a user can run `tv pine save` to save the currently open saved Pine Script in TradingView. This closes the next Pine workflow gap after Rust already learned how to read, set, compile, analyze, check, create, open, and list Pine scripts.

This command writes to TradingView cloud state, so it is intentionally isolated from `tv pine compile`. Compile must continue to avoid save-related buttons. `pine save` is the only command in this slice that persists a Pine script.

## Progress

- [x] (2026-04-24 18:49Z) Read `.agents/PLANS.md`, current Pine CLI dispatch, Pine operation code, contract tests, Pine skill, and remaining deferred surface audit.
- [x] (2026-04-24 18:49Z) Created this ExecPlan.
- [x] (2026-04-24 19:02Z) Add `tv pine save [--name <NAME>]` CLI surface and dispatch.
- [x] (2026-04-24 19:02Z) Implement Pine save operation with existing-save and new-name flows.
- [x] (2026-04-24 19:02Z) Add unit and CLI contract tests.
- [x] (2026-04-24 19:08Z) Update README, AGENTS, migration inventory, contract notes, handoff note, remaining deferred audit, and Pine skill.
- [x] (2026-04-24 19:17Z) Run automated validation and skill validation.
- [x] (2026-04-24 19:08Z) Live smoke initially skipped because it would persist TradingView cloud state and no explicit save-smoke approval was given for this slice.
- [x] (2026-04-24 19:31Z) Live smoke attempted after explicit user approval; named save did not pass in the current TradingView Desktop session.
- [x] (2026-04-24 19:20Z) Commit the completed slice.
- [x] (2026-04-25 09:45Z) Narrow public contract by removing `--name` after live smoke showed the naming dialog is not reliably available through CDP.

## Surprises & Discoveries

- `pine save` is more safety-sensitive than the prior Pine editor-buffer mutations because even the smoke path can leave a saved script behind. Automated fake-runtime coverage is the default acceptance path unless the user explicitly approves a live cloud-state smoke.
- Current TradingView Desktop shows the unsaved-script naming dialog in the desktop UI/accessibility tree, but the dialog was not visible to the CDP page context used by the Rust CLI. A keyboard fallback is unsafe because focus can remain in Monaco and insert the requested name into the script body.

## Decision Log

- Decision: Implement `pine save` as a dedicated persistent command and keep `pine compile` non-persistent.
  Rationale: The Rust CLI previously refused save-related compile buttons to avoid accidental TradingView cloud writes. Persistence should require an explicit `pine save` command.
  Date/Author: 2026-04-24 / Codex.

- Decision: Support `--name <NAME>` for new unsaved buffers, but reject name conflicts.
  Rationale: The user selected named save support, and rejecting existing names avoids accidental overwrite. Existing saved scripts can be opened and saved with plain `tv pine save`.
  Date/Author: 2026-04-24 / Codex.

- Decision: Remove `--name <NAME>` from the public CLI and defer named new-save for unsaved scripts.
  Rationale: Live smoke showed the TradingView naming dialog can be outside the CDP page target. A keyboard fallback can type into Monaco instead of the dialog, so exposing named new-save is less safe than failing when an unsaved-script naming dialog appears.
  Date/Author: 2026-04-25 / Codex.

- Decision: Do not implement `pine raw-compile` in this slice.
  Rationale: The old raw compile can click save/add actions without the Rust safety checks, and the remaining deferred surface audit classified it as likely no-direct-clone.
  Date/Author: 2026-04-24 / Codex.

## Outcomes & Retrospective

Implemented `tv pine save` as an explicit Pine persistence command for the current saved script. If an unsaved-script naming dialog appears, the command fails instead of typing into an unverified focus target. Named new-save remains deferred. The command reports save state under `data` and keeps `pine compile` non-persistent.

## Context and Orientation

The Rust binary is named `tv`. Command-line shape is defined in `src/cli.rs`. Runtime dispatch is in `src/main.rs`. Pine operations are grouped under `src/ops/pine.rs`, with Pine Editor behavior implemented in `src/ops/pine/editor.rs`. The Pine Editor is the TradingView code editor backed by Monaco. The current helper `ensure_pine_editor_open` opens the Pine panel and waits until Monaco can be reached through the page's JavaScript state.

Existing Pine commands already use the Rust JSON envelope, where successful command-specific fields live under top-level `data`. The old JavaScript `pine save` sent Ctrl+S and optionally clicked a visible Save button in a dialog. This Rust implementation preserves the practical saving capability for already saved scripts while refusing unsafe named new-save fallback behavior.

## Plan of Work

First add `Save` to `PineCommand` in `src/cli.rs`. Update `src/main.rs` to dispatch valid requests to `ops::pine_save`.

Then implement `pine_save` in `src/ops/pine/editor.rs` and re-export it through `src/ops/pine.rs` and `src/ops.rs`. The operation should call `ensure_pine_editor_open`, evaluate JavaScript for preflight and post-shortcut verification, and map returned `{ error, kind }` objects into `AppError`. The expression should trigger Ctrl+S and fail if a Save Script naming dialog appears, because naming unsaved scripts is deferred until a verified CDP-visible contract exists.

The success payload should include `saved`, `action`, `name`, `dialog_handled`, `source`, `editor_open_before`, `opened_editor`, `dirty_before`, and `dirty_after`. Unknown dirty state should be `null`; a known dirty state that remains true after save should be an `internal_api_unavailable` error.

Finally update tests and durable docs. CLI help should now list `save` while still hiding `raw-compile`. Docs should move only `pine save` to implemented status and keep `pine raw-compile`, bulk destructive commands, and generic UI automation deferred.

## Concrete Steps

Run commands from the repository root.

Targeted validation while implementing:

    cargo test ops::pine::editor::tests::pine_save -- --nocapture
    cargo test --test cli_contract pine_save -- --nocapture

Full validation before commit:

    cargo fmt --check
    cargo clippy --all-targets --all-features -- -D warnings
    cargo test
    git diff --check
    rg -n "(/[U]sers/|[C]:\\\\)" README.md AGENTS.md docs .agents/skills || true

Because `.agents/skills/pine-develop` changes, run the skill validator against that skill before committing.

## Validation and Acceptance

Automated acceptance is that tests prove help output, connection error behavior, existing save payload normalization, unsaved-script dialog rejection, absence of user-provided script-name serialization, and dirty-after-save failure handling.

Live smoke is optional and should not create or overwrite TradingView cloud state without explicit approval. If live smoke is approved for the narrowed contract, open an existing disposable saved script, make a harmless identifiable edit if needed, run `tv pine save`, and record any saved artifact or mutation clearly. Do not smoke `pine raw-compile` or named new-save.

Live smoke after approval:

- Intended disposable names:
  - `Codex Pine Save Smoke 20260424-190422`
  - `Codex Pine Save Smoke 20260424-190635`
  - `Codex Pine Save Smoke 20260424-190739`
- None of those intended disposable names appeared in `tv pine list`.
- A rejected keyboard fallback experiment caused TradingView to save the script under a default dialog name.
- Leftover saved script: one default-named disposable script, with its account-local script id intentionally omitted from this public archive note.
- The original Pine Editor source was restored with `tv pine set --file ...` and verified with `tv pine get`.

## Idempotence and Recovery

Source and docs edits are ordinary additive changes and can be rerun. Automated tests use fake runtime responses and do not require TradingView Desktop. If live smoke creates a disposable saved script, the current Rust CLI has no delete-saved-script command, so the created script name must be recorded for manual cleanup. If a live save changes the current Pine Editor buffer state, `tv pine get` and `tv pine set --file <PATH>` can be used to restore source text from an ignored `target/` backup.

## Artifacts and Notes

- Targeted Pine save unit and CLI contract tests were updated for payload normalization, unsaved-script dialog rejection, no user-provided script-name serialization, dirty-after-save failure, help output, and connection-attempt behavior.
- Validation passed: `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`, `git diff --check`, tracked docs local absolute path scan, and the Pine develop skill validator.
- Live smoke found a named-save blocker in the current TradingView Desktop session and left one identifiable default-named disposable script; the account-local script id is intentionally omitted from this public archive note.

## Interfaces and Dependencies

At completion, the CLI exposes:

    tv pine save

At completion, `src/ops/pine/editor.rs` exposes:

    pub async fn pine_save(runtime: &mut impl RuntimeEvaluator) -> Result<Value, AppError>

No new crates are required.

## Open Questions

No unresolved critical questions remain for this slice.
