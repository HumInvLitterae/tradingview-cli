# Add Pine new and open commands

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document follows `.agents/PLANS.md`.

## Purpose / Big Picture

After this change, operators can create a fresh Pine Editor buffer from a known template and load an existing saved Pine script into the editor from the Rust-native `tv` CLI. This closes the next Pine development workflow gap after `pine get`, `pine set`, `pine compile`, `pine analyze`, and `pine check` without adding cloud save behavior or raw compile behavior.

The work is intentionally narrow. `pine new` and `pine open` mutate only the current Pine Editor buffer. They do not save scripts to TradingView, compile scripts, add studies to the chart, or open generic UI automation surfaces.

## Progress

- [x] (2026-04-24 18:10Z) Read `.agents/PLANS.md`, current Pine implementation, current CLI contract tests, the old JavaScript `pine new/open` implementation, and current migration inventory docs.
- [x] (2026-04-24 18:10Z) Created this ExecPlan.
- [x] (2026-04-24 18:25Z) Add `tv pine new [indicator|strategy|library]` and `tv pine open <NAME...>` CLI and dispatch.
- [x] (2026-04-24 18:25Z) Implement Pine Editor buffer template creation and saved-script open behavior in `src/ops/pine/editor.rs`.
- [x] (2026-04-24 18:25Z) Add unit and CLI contract tests.
- [x] (2026-04-24 18:35Z) Update README, AGENTS, migration inventory, contract notes, handoff note, and Pine skill mapping.
- [x] (2026-04-24 18:45Z) Run automated validation, skill validation, and live smoke with source restoration.
- [x] (2026-04-24 18:50Z) Commit the completed slice.

## Surprises & Discoveries

- The CLI help negative assertion for an unimplemented `save` subcommand needed to check for the command row shape rather than the substring `save`, because `pine open` help text legitimately mentions saved scripts.
- Live smoke found one saved script available in the current TradingView session, so both `pine new` and `pine open` were exercised. The original editor source was restored after the smoke.

## Decision Log

- Decision: Implement `pine new` and `pine open` together, but leave `pine save` and `pine raw-compile` deferred.
  Rationale: `new` and `open` only replace the local Pine Editor buffer, while `save` persists to TradingView cloud and raw compile may click unsafe save/add buttons. This slice advances old CLI migration while keeping side effects narrow and recoverable.
  Date/Author: 2026-04-24 / Codex.

- Decision: Make partial-name `pine open` safer than the old JavaScript CLI by rejecting ambiguous partial matches.
  Rationale: Opening the wrong saved script would overwrite the current editor buffer. Exact matches are safe; partial matches are useful only when unique.
  Date/Author: 2026-04-24 / Codex.

## Outcomes & Retrospective

Implemented `tv pine new [indicator|strategy|library]` and `tv pine open <NAME...>` as Pine Editor buffer mutations. The slice keeps `pine save` and `pine raw-compile` deferred. `pine new` writes a known template into Monaco and verifies the resulting source; `pine open` fetches saved script metadata through the page session, rejects missing or ambiguous names, loads the chosen script source into Monaco, and verifies the resulting editor source.

## Context and Orientation

The Rust CLI is a single binary named `tv`. Command-line shape lives in `src/cli.rs`; dispatch and JSON envelopes live in `src/main.rs`; operation functions are re-exported through `src/ops.rs`; Pine operation modules are declared and re-exported through `src/ops/pine.rs`. Pine Editor operations that use Chrome DevTools Protocol, abbreviated CDP, live in `src/ops/pine/editor.rs`. CDP lets this CLI evaluate JavaScript inside the running TradingView Desktop page.

The helper `ensure_pine_editor_open` in `src/ops/pine/editor.rs` opens the Pine Editor panel when needed and verifies that Monaco, TradingView's embedded code editor, is available. Existing commands already use this helper for `pine get`, `pine set`, `pine compile`, `pine errors`, and `pine console`.

The old JavaScript CLI implemented `pine new` by setting the Monaco editor source to a simple template. It implemented `pine open` by calling TradingView's `pine-facade/list/?filter=saved` endpoint from the current page session, matching a saved script by name/title, then calling `pine-facade/get/<id>/<version>` and setting that source in Monaco. This Rust slice should keep that practical behavior while using the Rust JSON envelope.

## Plan of Work

First update `src/cli.rs` by adding `New { script_type: Option<String> }` and `Open { name: Vec<String> }` to `PineCommand`. In `src/main.rs`, validate the script type before connecting for `pine new`; accept only `indicator`, `strategy`, and `library`, defaulting to `indicator`. For `pine open`, join the name words with spaces and reject an empty joined name before connecting.

Then update `src/ops/pine/editor.rs`. Add `pub async fn pine_new(runtime: &mut impl RuntimeEvaluator, script_type: &str) -> Result<Value, AppError>`. It should call `ensure_pine_editor_open`, select one of three templates, set the template into Monaco, verify `getValue()` equals the template, and return `type`, `action: "new_script_created"`, `template`, `lines_set`, `char_count`, `editor_open_before`, and `opened_editor`.

Add `pub async fn pine_open(runtime: &mut impl RuntimeEvaluator, name: &str) -> Result<Value, AppError>`. It should call `ensure_pine_editor_open`, evaluate one awaitable JavaScript expression that fetches saved scripts, chooses an exact match on `scriptName` or `scriptTitle`, otherwise permits exactly one partial match, rejects missing or ambiguous matches, fetches source for the chosen script, sets it into Monaco, verifies the editor value, and returns `name`, `script_id`, `version`, `lines`, `source: "internal_api"`, `opened: true`, `editor_open_before`, and `opened_editor`. If JavaScript returns an error object, map missing or ambiguous user input cases to `validation`; map malformed API/editor cases to `internal_api_unavailable`.

Finally re-export the functions through `src/ops/pine.rs` and `src/ops.rs`, add tests, and update durable docs. Docs must move only `pine new/open` to implemented and keep `pine raw-compile` and `pine save` deferred.

## Concrete Steps

Run all commands from the repository root.

Targeted validation while implementing:

    cargo test ops::pine -- --nocapture
    cargo test --test cli_contract pine -- --nocapture

Full validation before commit:

    cargo fmt --check
    cargo clippy --all-targets --all-features -- -D warnings
    cargo test
    git diff --check
    rg -n "(/[U]sers/|[C]:\\\\)" README.md AGENTS.md docs .agents/skills || true

Because `.agents/skills/pine-develop` changes, run the skill validator against that skill before committing.

## Validation and Acceptance

Automated acceptance is that the full Rust baseline passes and tests prove `pine new` validates script type before connecting, `pine open` requires a name before connecting, and both commands preserve the expected Rust JSON envelope. Unit tests should prove template selection, source set verification, exact open match, unique partial open match, ambiguous partial rejection, missing script rejection, and empty fetched source handling.

Live smoke should run only against a running TradingView Desktop session:

    cargo run --quiet -- pine get
    cargo run --quiet -- pine new indicator
    cargo run --quiet -- pine get
    cargo run --quiet -- pine list
    cargo run --quiet -- pine open <SAVED_SCRIPT_NAME>
    cargo run --quiet -- pine set --file target/pine-new-open-restore.pine
    cargo run --quiet -- pine get

The smoke should first save the original editor source into an ignored `target/` file, run `pine new indicator`, verify the indicator template is in the editor, choose a saved script from `pine list`, open it, then restore the original source and verify `pine get` matches. The smoke must not run `pine save` or `pine compile`.

## Idempotence and Recovery

Source and docs edits are ordinary additive changes. The live smoke mutates only the Pine Editor buffer and writes temporary restore files under ignored `target/`. If live smoke fails after changing source, rerun `tv pine set --file target/pine-new-open-restore.pine` to restore the original editor buffer. If no saved scripts are available, record that `pine new` was smoked and `pine open` live smoke was skipped for lack of a saved script.

## Artifacts and Notes

- `cargo fmt --check && cargo clippy --all-targets --all-features -- -D warnings && cargo test` passed.
- `git diff --check` passed.
- `rg -n '(/[U]sers/|[C]:\\\\)' README.md AGENTS.md docs .agents/skills || true` returned no tracked-doc absolute local paths.
- The skill validator passed for `.agents/skills/pine-develop`.
- Live smoke passed against TradingView Desktop: `pine get` saved the original source, `pine new indicator` produced the indicator template, `pine list` returned one saved script, `pine open "<saved script name>"` loaded 115 lines from an account-local saved-script id, and `pine set --file target/pine-new-open-restore.pine` restored the original 8-line source.

## Interfaces and Dependencies

At completion, `src/ops/pine/editor.rs` exposes:

    pub async fn pine_new(runtime: &mut impl RuntimeEvaluator, script_type: &str) -> Result<Value, AppError>
    pub async fn pine_open(runtime: &mut impl RuntimeEvaluator, name: &str) -> Result<Value, AppError>

At completion, the CLI exposes:

    tv pine new [indicator|strategy|library]
    tv pine open <NAME...>

The implementation uses existing CDP `Runtime.evaluate` for Monaco and pine-facade access. It adds no new crates.

## Open Questions

No unresolved critical questions remain for this slice.
