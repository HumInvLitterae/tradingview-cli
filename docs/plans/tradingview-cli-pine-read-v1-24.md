# Add Pine read commands

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document follows `.agents/PLANS.md`.

## Purpose / Big Picture

After this change, operators can use the Rust-native `tv` CLI to inspect Pine Editor state without returning to the old JavaScript CLI. The first Pine slice is intentionally read-oriented: it can open the Pine Editor panel to make Monaco available, then read current source, markers, console rows, and saved script metadata. It does not write Pine source, save, compile, create, or open scripts.

## Progress

- [x] (2026-04-25 00:00Z) Read the current Rust CLI surface, Pine skill notes, migration inventory, and old JavaScript Pine implementation.
- [x] (2026-04-25 00:00Z) Created this ExecPlan.
- [x] (2026-04-25 00:00Z) Add `tv pine get/errors/console/list` CLI and dispatch.
- [x] (2026-04-25 00:00Z) Implement Pine read operations in `src/ops/pine.rs`.
- [x] (2026-04-25 00:00Z) Add unit and CLI contract tests.
- [x] (2026-04-25 00:00Z) Update README, AGENTS, migration inventory, contract notes, handoff note, and Pine skill mapping.
- [x] (2026-04-25 00:00Z) Run automated validation and skill validation.
- [x] (2026-04-25 00:00Z) Run live Pine smoke against TradingView Desktop.
- [x] (2026-04-25 00:00Z) Commit the completed slice.

## Surprises & Discoveries

- Observation: Existing repo-local `pine-develop` skill still says Pine editor pull/check capabilities are not implemented.
  Evidence: `.agents/skills/pine-develop/SKILL.md` and `references/workflow.md` both mark Pine editor operations as current CLI gaps.

- Observation: TradingView Desktop currently exposes Pine Editor's Monaco environment at `memoizedProps.monacoEnv` in one React fiber path, not only at the older `memoizedProps.value.monacoEnv` location.
  Evidence: Initial live smoke opened the Pine Editor but `pine get/errors/console` returned `internal_api_unavailable` until `FIND_MONACO` checked both locations.

- Observation: Wrapping a multi-line Monaco finder directly after JavaScript `return` triggers automatic semicolon insertion and can make an editor-present check return `undefined`.
  Evidence: Live CDP probing found Monaco and source text, while the Rust helper returned `editor_open_before: false`; changing the readiness body to `var m = __FIND_MONACO__; return m !== null;` made `pine get` pass live smoke.

- Observation: The broad console DOM fallback can accidentally return the entire Pine Editor text when no dedicated console rows are present.
  Evidence: Initial `pine console` live smoke returned one giant entry containing the current Pine source. The fallback now excludes source-looking and very large text nodes, producing an empty console list when no log rows exist.

## Decision Log

- Decision: Implement only `pine get`, `pine errors`, `pine console`, and `pine list`.
  Rationale: These preserve useful old CLI read surfaces while avoiding source mutation, save, compile, script creation, and chart study changes.
  Date/Author: 2026-04-25 / Codex.

- Decision: Allow Pine Editor panel auto-open for `get`, `errors`, and `console`.
  Rationale: The user selected this behavior. It matches the old CLI's practical behavior and makes the commands usable when the editor panel is closed, while still not changing Pine source or chart studies.
  Date/Author: 2026-04-25 / Codex.

## Outcomes & Retrospective

Implemented the first Rust-native Pine read slice:

- `tv pine list` returns saved Pine script metadata through the current TradingView page session.
- `tv pine get` returns current editor source plus line and character counts.
- `tv pine errors` returns Monaco marker diagnostics.
- `tv pine console` returns console-like entries when present and avoids reporting editor source as a log row.

The slice intentionally does not implement Pine source mutation, save, compile, create, open, offline analysis, or server-side check commands.

## Context and Orientation

The Rust CLI uses `src/cli.rs` for clap arguments, `src/main.rs` for dispatch, and `src/ops.rs` as a thin facade over capability modules under `src/ops/`. Pine read operations should live in a new `src/ops/pine.rs` module.

The old JavaScript CLI exposed a broad `pine` group. This slice migrates only the read-oriented subset. Pine Editor source and markers are read from Monaco through TradingView Desktop's page JavaScript. Saved script listing uses the current TradingView page session to call `https://pine-facade.tradingview.com/pine-facade/list/?filter=saved`.

## Plan of Work

Add a `PineCommand` enum with `Get`, `Errors`, `Console`, and `List`, and add a `Command::Pine` dispatch branch. Each subcommand connects to the normal chart runtime. `pine get/errors/console` call Pine Editor helpers that first check whether Monaco is already present, attempt to open the editor through `window.TradingView.bottomWidgetBar` and the Pine button if needed, poll for Monaco, then read the requested data. `pine list` calls the pine-facade list endpoint from the page session and returns an empty list plus `error` when the fetch fails.

Update durable docs and the repo-local `pine-develop` skill so future agents know the read subset is implemented and mutation/compile operations remain backlog.

## Validation and Acceptance

Automated validation must pass:

    cargo fmt --check
    cargo clippy --all-targets --all-features -- -D warnings
    cargo test
    git diff --check
    git grep -nE '(/[U]sers/|[C]:\\)' -- README.md AGENTS.md docs .agents/skills || true

Because a skill changes, validate `.agents/skills/pine-develop` with the repo skill validator before committing.

Live smoke should run:

    cargo run --quiet -- ui-state
    cargo run --quiet -- pine list
    cargo run --quiet -- pine get
    cargo run --quiet -- pine errors
    cargo run --quiet -- pine console

If Pine Editor, Monaco, or pine-facade is unavailable in the live session, record the blocker here and keep automated validation as the merge gate.

## Idempotence and Recovery

The Rust source and docs edits are ordinary additive changes. Live smoke may open the Pine Editor panel but must not change source, save a script, compile, add a study, or close unrelated panels.

## Artifacts and Notes

Automated validation completed successfully:

    cargo fmt --check
    cargo clippy --all-targets --all-features -- -D warnings
    cargo test

Targeted Pine tests completed successfully:

    cargo test ops::pine -- --nocapture
    cargo test --test cli_contract pine -- --nocapture

Skill validation completed successfully:

    python3 <skill-creator>/scripts/quick_validate.py .agents/skills/pine-develop

Live smoke completed successfully with TradingView Desktop running and the Pine Editor visible/openable:

    cargo run --quiet -- ui-state
    cargo run --quiet -- pine list
    cargo run --quiet -- pine get
    cargo run --quiet -- pine errors
    cargo run --quiet -- pine console

Observed live results:

- `pine list`: `count: 1`, `error: null`.
- `pine get`: `line_count: 7`, `char_count: 175`, `editor_open_before: true`, `opened_editor: false`.
- `pine errors`: `has_errors: false`, `error_count: 0`.
- `pine console`: `entry_count: 0` after filtering editor-source fallback noise.

## Interfaces and Dependencies

At completion, `src/ops/pine.rs` exposes:

    pub async fn pine_get(runtime: &mut impl RuntimeEvaluator) -> Result<Value, AppError>
    pub async fn pine_errors(runtime: &mut impl RuntimeEvaluator) -> Result<Value, AppError>
    pub async fn pine_console(runtime: &mut impl RuntimeEvaluator) -> Result<Value, AppError>
    pub async fn pine_list(runtime: &mut impl RuntimeEvaluator) -> Result<Value, AppError>

At completion, the CLI exposes:

    tv pine get
    tv pine errors
    tv pine console
    tv pine list

## Open Questions

No unresolved critical questions remain for this slice.
