# Add Pine compile command

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document follows `.agents/PLANS.md`.

## Purpose / Big Picture

After this change, operators can ask the Rust-native `tv` CLI to compile the current Pine Editor source in TradingView Desktop. This closes the next practical Pine workflow gap after `pine get` and `pine set`: a user can push source into the editor, compile it, and read diagnostics without returning to the old JavaScript CLI.

The command is intentionally narrow. It compiles the current editor buffer and reports button action, marker diagnostics, and study-count change. It does not save scripts to TradingView, create or open saved scripts, run offline analysis, or call the server-side check endpoint.

## Progress

- [x] (2026-04-24 16:15Z) Read current Pine operation code, CDP keyboard primitives, fake runtime support, old JavaScript Pine compile implementation, and current docs.
- [x] (2026-04-24 16:15Z) Created this ExecPlan.
- [x] (2026-04-24 16:25Z) Add `tv pine compile` CLI and dispatch.
- [x] (2026-04-24 16:25Z) Implemented safe compile button detection, keyboard fallback, marker read, and study-count reporting in `src/ops/pine.rs`.
- [x] (2026-04-24 16:25Z) Added unit and CLI contract tests.
- [x] (2026-04-24 16:30Z) Updated README, AGENTS, migration inventory, contract notes, handoff note, and Pine skill mapping.
- [x] (2026-04-24 16:40Z) Ran automated validation, skill validation, and live smoke with source restoration.
- [x] (2026-04-24 16:45Z) Commit the completed slice.

## Surprises & Discoveries

- Observation: The current CDP layer already supports keyboard events with modifier bits.
  Evidence: `src/cdp.rs` exposes `RuntimeEvaluator::dispatch_key_event` and `KeyEvent { modifiers }`, added for earlier watchlist/tab work.

- Observation: The old JavaScript `compile` would click a save-related button if it found one.
  Evidence: `tradingview-mcp/src/core/pine.js` accepts `Save and add to chart` and a `saveButton` fallback. This Rust slice intentionally avoids those to keep persistence outside `pine compile`.

- Observation: In the live Japanese TradingView UI, the safe compile action appeared as duplicated text in the DOM. The CLI now normalizes exact repeated button text before returning `button_clicked`.
  Evidence: A follow-up live smoke returned `button_clicked: "チャートに追加"`, `has_errors: true`, `error_count: 1`, `studies_before: 33`, and `studies_after: 34`; the added `Untitled Script` study was then removed with `tv indicator remove`.

## Decision Log

- Decision: Implement only `pine compile` in this slice.
  Rationale: It is the next useful Pine workflow command after source set. Raw compile, save, new/open, analyze, and check have different safety profiles and should stay separate.
  Date/Author: 2026-04-24 / Codex.

- Decision: Do not click save-related Pine buttons.
  Rationale: `pine compile` may add or update a chart-local study, but it must not persist scripts to TradingView cloud or handle save dialogs. Save behavior belongs in a separate explicitly planned slice.
  Date/Author: 2026-04-24 / Codex.

## Outcomes & Retrospective

Completed. `tv pine compile` now compiles the current Pine Editor buffer without save behavior, reports Monaco diagnostics and study-count change under `data`, and keeps raw compile, save, new/open, analyze, and server-side check helpers deferred. Live smoke confirmed an invalid script reports errors, restores the original editor source, and allows the added chart-local study to be identified and removed.

## Context and Orientation

The Rust CLI is a single binary named `tv`. Command-line shape lives in `src/cli.rs`; dispatch and JSON envelopes live in `src/main.rs`; Pine Editor operations live in `src/ops/pine.rs`; `src/ops.rs` re-exports operation functions for dispatch.

Pine Editor source text is hosted by Monaco, the code editor component TradingView uses. `src/ops/pine.rs` already has `ensure_pine_editor_open`, which opens the Pine Editor if needed and finds Monaco through TradingView's React fiber tree. `pine compile` should reuse that helper and the existing marker-reading behavior from `pine errors`.

The old JavaScript CLI exposed both `compile` and `raw-compile`. This slice implements the smarter user-facing `compile` behavior only, but changes the save-related fallback: Rust must refuse to click save buttons in this command.

## Plan of Work

First update `src/cli.rs` by adding `Compile` to `PineCommand`. Update `src/main.rs` so `PineCommand::Compile` connects to the current TradingView runtime and calls `ops::pine_compile`.

In `src/ops.rs`, re-export `pine_compile`. In `src/ops/pine.rs`, import `KeyEvent` and `KeyEventType`. Add `pub async fn pine_compile(runtime: &mut impl RuntimeEvaluator) -> Result<Value, AppError>`. This function should open the Pine Editor, read the current study count, try to click a compile/add/update button, fall back to `Ctrl+Enter` if no safe button exists, wait briefly, read Monaco markers, read the study count again, and return a structured payload.

The button-detection expression should return a JSON object rather than a bare string. It should click only visible buttons whose text matches English or Japanese add/update labels, such as `Add to chart`, `Update on chart`, `チャートに追加`, or an update-on-chart Japanese equivalent. It must not click buttons whose text includes save wording such as `Save`, `保存`, or `Save and add to chart`; if only a save-related compile-looking button is present, return an `internal_api_unavailable` error with details so the operator can choose a future save-specific command.

Add tests in `src/ops/pine.rs` using `FakeRuntime`. Cover safe button click, save-button refusal, keyboard fallback, and malformed marker payload. Add CLI contract tests for `pine compile` help and connection failure.

Update README and docs to list `pine compile` as implemented while leaving raw compile, save, new/open, analyze, and check deferred. Update the Pine skill to say compile verification is now available through `tv pine compile`, but saving and server/offline checks remain unavailable.

## Validation and Acceptance

Automated validation must pass from the repository root:

    cargo fmt --check
    cargo clippy --all-targets --all-features -- -D warnings
    cargo test
    git diff --check
    rg -n "(/[U]sers/|[C]:\\\\)" README.md AGENTS.md docs .agents/skills || true

Because `.agents/skills/pine-develop` changes, validate it with the repo skill validator before committing.

Live smoke should run only against a running TradingView Desktop session:

    cargo run --quiet -- pine get
    cargo run --quiet -- pine set --file target/pine-compile-invalid.pine
    cargo run --quiet -- pine compile
    cargo run --quiet -- pine set --file target/pine-compile-restore.pine
    cargo run --quiet -- pine get

The smoke should first save the original `pine get` source into an ignored `target/` file, set an intentionally invalid small script, run compile, confirm `has_errors: true` and `error_count > 0`, then restore the original source and verify `pine get` matches. The smoke should not run `pine save`.

## Idempotence and Recovery

Source and docs edits are ordinary additive changes. Tests do not require TradingView Desktop. Live smoke mutates only the Pine Editor buffer and may attempt to compile invalid source; it writes temporary files under ignored `target/`. If live smoke fails after changing source, rerun `tv pine set --file target/pine-compile-restore.pine` to restore the original editor buffer.

## Artifacts and Notes

- `cargo fmt --check && cargo clippy --all-targets --all-features -- -D warnings && cargo test` passed.
- `git diff --check` passed.
- `rg -n "(/[U]sers/|[C]:\\\\)" README.md AGENTS.md docs .agents/skills || true` returned no tracked-doc local absolute paths.
- `python3 .../quick_validate.py .agents/skills/pine-develop` passed.
- Live smoke:
  - Saved original Pine Editor source to `target/pine-compile-restore.pine`.
  - Set invalid source from `target/pine-compile-invalid.pine`.
  - Ran `tv pine compile`; result included `button_clicked: "チャートに追加"`, `has_errors: true`, `error_count: 1`, `study_added: true`, `studies_before: 33`, and `studies_after: 34`.
  - Restored the original source and verified `tv pine get` matched the original source.
  - Removed the smoke-added `Untitled Script` study with `tv indicator remove`.

## Interfaces and Dependencies

At completion, `src/ops/pine.rs` exposes:

    pub async fn pine_compile(runtime: &mut impl RuntimeEvaluator) -> Result<Value, AppError>

At completion, the CLI exposes:

    tv pine compile

The operation uses existing CDP `Runtime.evaluate` for DOM and Monaco access, and existing CDP `Input.dispatchKeyEvent` for the `Ctrl+Enter` fallback.

## Open Questions

No unresolved critical questions remain for this slice.
