# Add indicator commands

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document follows `.agents/PLANS.md` from the repository root.

## Purpose / Big Picture

The old JavaScript `tv` CLI exposed `tv indicator add/remove/toggle/set/get`, while the Rust CLI currently only exposes indicator reads through `tv data indicator <ENTITY_ID>` and the study list in `tv state`. After this change, a user can add an indicator, inspect it, hide or show it, change known input values, and remove it again from the Rust-native `tv` CLI.

This is the next old CLI migration slice because it is a complete chart-local lifecycle surface. It is smaller and safer than Pine editor, replay, drawing, tab, stream, or generic UI automation surfaces, and it builds on already implemented chart-state and indicator-read commands.

## Progress

- [x] (2026-04-24 00:00Z) Confirmed the worktree was clean before starting.
- [x] (2026-04-24 00:00Z) Inspected old JavaScript CLI indicator commands and core implementation.
- [x] (2026-04-24 00:00Z) Created this ExecPlan and recorded the migration contract.
- [x] (2026-04-24 00:00Z) Added Rust CLI surface and dispatch for `tv indicator add/remove/toggle/set/get`.
- [x] (2026-04-24 00:00Z) Implemented indicator operations in `src/ops/indicator.rs`.
- [x] (2026-04-24 00:00Z) Added unit and CLI contract tests.
- [x] (2026-04-24 00:00Z) Updated README, migration inventory, lifecycle audit, handoff docs, and agent guide.
- [x] (2026-04-24 00:00Z) Ran automated validation baseline: `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`, `git diff --check`, and tracked-doc local absolute path scan passed.
- [x] (2026-04-24 00:00Z) Ran live TradingView Desktop smoke against `Volume`: add, get, toggle hidden, toggle visible, set length, and remove were exercised; the added study was removed.
- [x] (2026-04-24 00:00Z) Committed the completed slice as `feat(cli): Add indicator commands`.

## Surprises & Discoveries

- Observation: `chart.getStudyById(entity_id)` can throw when the study is missing instead of returning null.
  Evidence: The first live smoke removed the newly added `Volume` study but returned an `internal_api_unavailable` envelope during post-delete verification because `getStudyById` threw `Error: There is no such study`.

- Observation: Some study name properties are functions rather than plain strings.
  Evidence: The first successful `indicator remove` smoke returned `"indicator": {}` before the name extraction was changed to call `study.name()` or `study.title()` when those properties are functions.

## Decision Log

- Decision: Implement the old `indicator` CLI surface as a single lifecycle slice.
  Rationale: The old CLI exposes add, remove, toggle, set, and get together. Implementing the surface as a set keeps cleanup and inspection available for live smoke and downstream operator workflows.
  Date/Author: 2026-04-24 / Codex

- Decision: Keep `tv indicator get <ENTITY_ID>` as a migration alias for the practical payload already available from `tv data indicator <ENTITY_ID>`.
  Rationale: The old CLI had `indicator get`, and Rust already has the underlying read. Duplicating the implementation would increase drift.
  Date/Author: 2026-04-24 / Codex

- Decision: Preserve the Rust JSON envelope instead of recreating the old JavaScript top-level payload.
  Rationale: This repository's public contract is `{ success, command, data }` for successes and `{ success, command, error }` for failures. Migration requires information compatibility, not wire-format cloning.
  Date/Author: 2026-04-24 / Codex

## Outcomes & Retrospective

This section will be updated after implementation and validation.
The Rust CLI now implements `tv indicator add/remove/toggle/set/get`. Automated tests cover input parsing, safe JavaScript serialization, missing study handling, post-delete verification, visibility confirmation, and matched versus unmatched input ids. CLI contract tests cover help output, validation before CDP connection, and connection envelopes.

Live smoke added a fresh `Volume` study, read its inputs, hid it, showed it, changed `length` from 20 to 21, removed the returned `entity_id`, and confirmed the added study no longer appeared in `tv state`. The post-delete existence check was hardened after TradingView threw on missing studies.

## Context and Orientation

This repository implements a Rust-native command-line binary named `tv`. The entrypoint in `src/main.rs` parses commands, connects to TradingView Desktop through Chrome DevTools Protocol, calls operation functions re-exported from `src/ops.rs`, and prints JSON envelopes.

Chrome DevTools Protocol, abbreviated CDP, is the local debugging protocol exposed by TradingView Desktop when it runs with a remote debugging port. Runtime JavaScript evaluation is abstracted by the `RuntimeEvaluator` trait in `src/cdp.rs`; unit tests use fake runtimes from `src/ops/test_support.rs` so tests do not require TradingView Desktop.

The existing Rust CLI already provides `tv state`, which returns study ids and names, and `tv data indicator <ENTITY_ID>`, which returns indicator visibility and input values for one study. The old JavaScript CLI's `indicator get` command called the same practical read. The new Rust `indicator` mutation operations should live in a new `src/ops/indicator.rs` module so `src/ops.rs` remains a thin facade and no `mod.rs` file is introduced.

## Plan of Work

First, extend the CLI surface in `src/cli.rs` with a new top-level `Indicator` command and an `IndicatorCommand` subcommand enum. The subcommands are `add`, `remove`, `toggle`, `set`, and `get`. `add` accepts one or more words for the indicator name plus optional `--inputs <JSON>`. `set` requires `--inputs <JSON>`. `toggle` accepts `--visible` or `--hidden`; if neither is supplied, it defaults to visible, matching the old CLI behavior.

Next, update `src/main.rs` dispatch. Keep validation there narrow: reject empty indicator names and entity ids, reject `--visible` plus `--hidden`, parse `--inputs` as a non-empty JSON object before connecting when possible, and connect to CDP only after validation passes. Dispatch `get` to the existing `ops::data_indicator` implementation.

Then, create `src/ops/indicator.rs`. Implement `indicator_add`, `indicator_remove`, `indicator_toggle`, `indicator_set`, and `parse_indicator_inputs`. Use `serde_json::to_string` and existing helpers such as `js_string` rather than hand-written JavaScript quoting. Every operation should return command payloads under the Rust `data` envelope through normal dispatch.

For `add`, compare study ids before and after `chart.createStudy()`, wait briefly for TradingView to attach the study, and return the new `entity_id`. If no new id appears, fail with an internal API error instead of returning a false-success payload. For `remove`, verify the study exists, call `chart.removeEntity(entity_id)`, wait briefly, and verify it no longer exists. For `toggle`, call `setVisible(target)` and return the actual `isVisible()` value. For `set`, read current inputs, update only ids present in the user's JSON object, return `updated_inputs` and `unmatched_inputs`, and fail if no provided input id matched the study.

Finally, update tests and durable docs. Unit tests belong under `#[cfg(test)]` in `src/ops/indicator.rs`. CLI contract tests belong in `tests/cli_contract.rs`. Docs should move the old `indicator` lifecycle pair from deferred backlog to implemented surface and list the new commands in README and handoff material.

## Concrete Steps

From the repository root, run the implementation loop:

    cargo fmt --check
    cargo clippy --all-targets --all-features -- -D warnings
    cargo test
    git diff --check
    run the repository's standard tracked-doc local absolute path scan

If `cargo fmt --check` fails only because of formatting, run `cargo fmt` and repeat the baseline.

If TradingView Desktop is available through CDP, run a live smoke with an indicator that is safe to add and remove:

    cargo run --quiet -- state
    cargo run --quiet -- indicator add "Volume"
    cargo run --quiet -- indicator get <ENTITY_ID>
    cargo run --quiet -- indicator toggle <ENTITY_ID> --hidden
    cargo run --quiet -- indicator toggle <ENTITY_ID> --visible
    cargo run --quiet -- indicator remove <ENTITY_ID>
    cargo run --quiet -- state

Only run `indicator set` during live smoke if `indicator get` exposes an obvious safe input id and value for the newly added study. Record the commands and summarized results in this plan. Do not remove indicators that existed before the smoke test.

## Validation and Acceptance

The change is accepted when `tv indicator --help` lists `add`, `remove`, `toggle`, `set`, and `get`; missing required arguments fail with structured validation errors; invalid `--inputs` fails before CDP connection; connection failures use the structured connection envelope; and all automated tests pass.

The success JSON must use the Rust envelope. For example, `tv indicator add "Volume"` should print a success envelope whose `data` includes `action: "add"`, `indicator`, `entity_id`, and study counts. `tv indicator set <ENTITY_ID> --inputs '{"length":20}'` should print a success envelope whose `data` includes `updated_inputs` and `unmatched_inputs`.

## Idempotence and Recovery

Automated tests are safe to rerun and must use fake runtimes. They must not require a running TradingView Desktop.

Live smoke mutates the active chart layout. It is safe only when the smoke adds a fresh indicator and removes that same returned `entity_id` afterward. If add succeeds but remove fails, record the `entity_id` in this plan and stop; do not use broad UI automation or remove unrelated studies.

## Artifacts and Notes

Artifacts will be added as implementation and validation proceed.
Automated validation:

    cargo fmt --check
    result: passed

    cargo clippy --all-targets --all-features -- -D warnings
    result: passed

    cargo test
    result: ok. 81 unit tests and 29 CLI contract tests passed.

    git diff --check
    result: passed

    tracked-doc local absolute path scan
    result: no tracked-doc local absolute paths found

Live smoke:

    cargo run --quiet -- state
    result: success true, symbol BATS:LWLG, 33 studies before smoke

    cargo run --quiet -- indicator add "Volume"
    result: success true, entity_id cZaFPi, before_count 33, after_count 34

    cargo run --quiet -- indicator get cZaFPi
    result: success true, inputs included length 20 and col_prev_close false

    cargo run --quiet -- indicator toggle cZaFPi --hidden
    result: success true, visible false

    cargo run --quiet -- indicator toggle cZaFPi --visible
    result: success true, visible true

    cargo run --quiet -- indicator set cZaFPi --inputs '{"length":21}'
    result: success true, updated_inputs length 21

    cargo run --quiet -- indicator remove cZaFPi
    result: first attempt removed the study but failed during post-delete verification because getStudyById threw on absence

    cargo run --quiet -- indicator add "Volume"
    result: success true, entity_id 37Wn9q, before_count 33, after_count 34

    cargo run --quiet -- indicator remove 37Wn9q
    result after getStudyById-safe fix: success true, removed true, before_count 34, after_count 33

    cargo run --quiet -- state | rg '37Wn9q|"Volume"'
    result: no match; the added smoke study was absent

## Interfaces and Dependencies

At the end of the implementation, these commands must exist:

    tv indicator add <INDICATOR_NAME...> [--inputs <JSON>]
    tv indicator remove <ENTITY_ID>
    tv indicator toggle <ENTITY_ID> [--visible | --hidden]
    tv indicator set <ENTITY_ID> --inputs <JSON>
    tv indicator get <ENTITY_ID>

The operation facade must expose:

    pub async fn indicator_add(runtime: &mut impl RuntimeEvaluator, indicator: &str, inputs: Option<&Value>) -> Result<Value, AppError>
    pub async fn indicator_remove(runtime: &mut impl RuntimeEvaluator, entity_id: &str) -> Result<Value, AppError>
    pub async fn indicator_toggle(runtime: &mut impl RuntimeEvaluator, entity_id: &str, visible: bool) -> Result<Value, AppError>
    pub async fn indicator_set(runtime: &mut impl RuntimeEvaluator, entity_id: &str, inputs: &Value) -> Result<Value, AppError>
    pub fn parse_indicator_inputs(raw: &str) -> Result<Value, AppError>

No new third-party Rust dependencies are required.

## Open Questions

There are no unresolved critical questions at the plan stage. If live TradingView behavior differs from the old JavaScript assumptions, record the discovery here and choose the safest failing behavior rather than broad UI automation.
