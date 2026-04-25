# Add Pine source set command

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document follows `.agents/PLANS.md`.

## Purpose / Big Picture

After this change, operators can push Pine Script source text into the open TradingView Pine Editor through the Rust-native `tv` CLI. This replaces the old JavaScript CLI's practical `tv pine set` editing surface while keeping the first mutation small: the command changes only the editor buffer. It does not compile, save, create a script, open a saved script, or add a study to the chart.

The behavior is observable by running `tv pine set --file <path>` or piping source into `tv pine set`, then running `tv pine get` and confirming the editor text matches.

## Progress

- [x] (2026-04-24 15:46Z) Read current Pine operation code, CLI dispatch, CLI contract tests, Pine skill, and old JavaScript Pine command shape.
- [x] (2026-04-24 15:46Z) Created this ExecPlan.
- [x] (2026-04-24 15:46Z) Add `tv pine set [--file <PATH>]` CLI and dispatch.
- [x] (2026-04-24 15:46Z) Implement source reading from `--file` or stdin with validation before CDP connection.
- [x] (2026-04-24 15:46Z) Implement `pine_set` in `src/ops/pine.rs`.
- [x] (2026-04-24 15:46Z) Add operation and CLI contract tests.
- [x] (2026-04-24 15:46Z) Update README, AGENTS, migration inventory, contract notes, handoff note, and Pine skill mapping.
- [x] (2026-04-24 15:46Z) Run automated validation, skill validation, and live smoke with source restoration.
- [x] (2026-04-24 15:46Z) Commit the completed slice.

## Surprises & Discoveries

- Observation: No existing command reads stdin, so `pine set` needs the first small input-reading helper in `src/main.rs`.
  Evidence: Searching `src` found screenshot file writes and CDP input primitives, but no stdin source-reader for command payloads.

- Observation: Shell `cmp` against `jq -r` process substitution is a poor live-smoke verifier for Pine source because it can blur trailing-newline behavior.
  Evidence: `pine set` succeeded and restoration succeeded, but the first `cmp` pipeline exited non-zero. A Python JSON parse comparison against the exact `pine get` source confirmed `temporary_source_verified= True` and `original_source_restored= True`.

## Decision Log

- Decision: Implement only `pine set` in this slice.
  Rationale: It is the smallest useful Pine mutation after the read slice. `compile`, `save`, `new`, and `open` have stronger chart/account side effects and need separate safety decisions.
  Date/Author: 2026-04-24 / Codex.

- Decision: Prefer `--file` over stdin when both are present.
  Rationale: The old JavaScript CLI accepted `--file` or stdin. File precedence is deterministic and lets scripted callers avoid accidental piped input changing the requested source.
  Date/Author: 2026-04-24 / Codex.

- Decision: Verify `setValue` by reading `getValue()` immediately after setting.
  Rationale: The operation depends on TradingView internals. Post-set verification makes success payloads mean the editor buffer actually changed.
  Date/Author: 2026-04-24 / Codex.

## Outcomes & Retrospective

Implemented `tv pine set` as a buffer-only Pine Editor mutation. The command reads source from `--file` or stdin, validates non-empty input before connecting to CDP, opens the Pine Editor if needed, writes the source through Monaco `setValue`, and verifies the editor buffer by reading `getValue` immediately after the set.

The slice does not compile, save, create, or open Pine scripts. Those commands remain deferred for separate safety planning.

## Context and Orientation

The Rust CLI is a single binary named `tv`. Command-line shape lives in `src/cli.rs`; dispatch and output envelopes live in `src/main.rs`; TradingView operations live in modules under `src/ops/`, with `src/ops.rs` acting as a facade. Pine Editor reads already live in `src/ops/pine.rs`, including `ensure_pine_editor_open`, which opens the Pine Editor panel when needed and locates Monaco, the editor component TradingView uses for Pine source text.

The old JavaScript CLI exposed `tv pine set` with `--file` or stdin source input. The Rust command should preserve the practical result of that command: replacing the Pine Editor source and returning how many lines were set. The Rust JSON envelope remains `{ success, command, data }`; do not copy the old top-level wire shape.

## Plan of Work

First update `src/cli.rs` by adding `Set { file: Option<PathBuf> }` to `PineCommand`, with `--file` and `-f`. Keep `get`, `errors`, `console`, and `list` unchanged.

Next update `src/main.rs`. Import `std::io::{self, IsTerminal, Read}` and `std::path::PathBuf`. Add a private helper that returns a Pine source string and an input source label. If `--file` is present, read the whole file with `std::fs::read_to_string` and return label `file`; map read errors to `ErrorKind::Validation`. If no file is present and stdin is a terminal, return a validation error saying that Pine source is required via stdin or `--file`. If stdin is not a terminal, read it to a string and return label `stdin`; map read errors to validation. Reject empty or whitespace-only source before connecting to CDP.

Then update the `Command::Pine` dispatch branch so only `get`, `errors`, `console`, and `list` connect immediately. For `set`, read and validate source first, then connect and call `ops::pine_set(&mut runtime, &source, input_source)`.

In `src/ops/pine.rs`, add `pub async fn pine_set(runtime: &mut impl RuntimeEvaluator, source: &str, input_source: &str) -> Result<Value, AppError>`. It should call `ensure_pine_editor_open`, evaluate a Monaco expression that calls `m.editor.setValue(<json source>)`, then returns `m.editor.getValue()`. If the returned value is not the exact source, return `internal_api_unavailable` with details. On success, return `lines_set`, `char_count`, `input_source`, `editor_open_before`, and `opened_editor`.

Update tests in `src/ops/pine.rs` and `tests/cli_contract.rs` to cover the new behavior without requiring TradingView Desktop. Update durable docs and `.agents/skills/pine-develop` so future agents know `pine set` exists, while compile/save/new/open/analyze/check remain deferred.

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
    cargo run --quiet -- pine set --file target/pine-set-smoke.pine
    cargo run --quiet -- pine get
    cargo run --quiet -- pine set --file target/pine-set-restore.pine
    cargo run --quiet -- pine get

The smoke should first save the original `pine get` source into an ignored `target/` file, set a tiny temporary script, verify `pine get` matches that script, then restore the original source and verify it matches again. Do not run compile or save during smoke.

## Idempotence and Recovery

The source and docs edits are ordinary additive changes. Running tests repeatedly should not change tracked files. Live smoke mutates only the Pine Editor buffer and writes temporary files under ignored `target/`. If live smoke fails after setting the temporary source, rerun `tv pine set --file target/pine-set-restore.pine` to restore the original editor text.

## Artifacts and Notes

Automated validation completed successfully:

    cargo fmt --check
    cargo clippy --all-targets --all-features -- -D warnings
    cargo test

Targeted tests completed successfully:

    cargo test ops::pine -- --nocapture
    cargo test --test cli_contract pine -- --nocapture

Skill validation completed successfully:

    python3 <skill-creator>/scripts/quick_validate.py .agents/skills/pine-develop

Live smoke completed successfully with TradingView Desktop running:

    temporary_source_verified= True set= {'lines_set': 4, 'char_count': 56, 'input_source': 'file', 'editor_open_before': True, 'opened_editor': False}
    original_source_restored= True set= {'lines_set': 8, 'char_count': 176, 'input_source': 'file', 'editor_open_before': True, 'opened_editor': False}

## Interfaces and Dependencies

At completion, `src/ops/pine.rs` exposes:

    pub async fn pine_set(
        runtime: &mut impl RuntimeEvaluator,
        source: &str,
        input_source: &str,
    ) -> Result<Value, AppError>

At completion, the CLI exposes:

    tv pine set --file <PATH>
    cat script.pine | tv pine set

The operation must use `serde_json::to_string(source)` or equivalent JSON serialization to embed Pine source in JavaScript. It must not hand-roll quote escaping.

## Open Questions

No unresolved critical questions remain for this slice.
