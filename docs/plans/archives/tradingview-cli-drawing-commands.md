# Add drawing commands

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document follows `.agents/PLANS.md` from the repository root.

## Purpose / Big Picture

The old JavaScript `tv` CLI exposed `tv draw shape/list/get/remove/clear`. After this change, the Rust CLI can create a chart drawing, list drawings, inspect one drawing, and remove one drawing by id. The bulk `clear` command remains deferred because it can delete all chart drawings at once.

This is the next old CLI migration slice because it is a complete chart-local lifecycle surface when limited to `shape/list/get/remove`. A user can add a drawing, verify the returned `entity_id`, and remove that exact drawing without broad cleanup.

## Progress

- [x] (2026-04-24 00:00Z) Confirmed the worktree was clean before starting.
- [x] (2026-04-24 00:00Z) Inspected old JavaScript `draw` CLI and core implementation.
- [x] (2026-04-24 00:00Z) Created this ExecPlan and recorded that `draw clear` is deferred.
- [x] (2026-04-24 00:00Z) Added Rust CLI surface and dispatch for `tv draw shape/list/get/remove`.
- [x] (2026-04-24 00:00Z) Implemented drawing operations in `src/ops/drawing.rs`.
- [x] (2026-04-24 00:00Z) Added unit and CLI contract tests.
- [x] (2026-04-24 00:00Z) Updated README, migration inventory, lifecycle audit, handoff docs, and agent guide.
- [x] (2026-04-24 00:00Z) Ran automated validation baseline: `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`, `git diff --check`, and tracked-doc local absolute path scan passed after implementation and live smoke.
- [x] (2026-04-24 00:00Z) Ran live TradingView Desktop smoke with `horizontal_line`: shape, get, list, and remove succeeded; the smoke drawing was removed.
- [x] (2026-04-24 00:00Z) Committed the completed slice as `feat(cli): Add drawing commands`.

## Surprises & Discoveries

- Observation: TradingView normalizes the supplied drawing time to the bar time.
  Evidence: The smoke command used `--time 1776951000`; `draw get DeQmSS` returned the point time as `1776902400`, matching the daily bar anchor.

- Observation: `draw get` can expose richer properties than `draw list`.
  Evidence: `draw list` returned id and name, while `draw get DeQmSS` returned points and properties such as `linecolor`, `linewidth`, `showPrice`, and the smoke text.

## Decision Log

- Decision: Implement `draw shape/list/get/remove` and keep `draw clear` deferred.
  Rationale: `shape/list/get/remove` has an exact cleanup path through `entity_id`. `clear` is a bulk destructive command and should not ride along with the first drawing migration slice.
  Date/Author: 2026-04-24 / Codex

- Decision: Preserve the Rust JSON envelope instead of recreating the old JavaScript top-level payload.
  Rationale: This repository's public contract is `{ success, command, data }` for successes and `{ success, command, error }` for failures. Migration requires information compatibility, not wire-format cloning.
  Date/Author: 2026-04-24 / Codex

## Outcomes & Retrospective

The Rust CLI now implements `tv draw shape/list/get/remove`. Automated tests cover override parsing, finite validation, safe JavaScript serialization, second-point shape creation, missing drawing handling, post-delete verification, and CLI contract behavior. `draw clear` remains deferred and is intentionally absent from help.

Live smoke created a new `horizontal_line` drawing with text `tv-cli-smoke`, inspected it, confirmed it appeared in `draw list`, removed it by returned `entity_id`, and confirmed the drawing count returned to its prior value.

## Context and Orientation

This repository implements a Rust-native command-line binary named `tv`. The entrypoint in `src/main.rs` parses commands, connects to TradingView Desktop through Chrome DevTools Protocol, calls operation functions re-exported from `src/ops.rs`, and prints JSON envelopes.

Chrome DevTools Protocol, abbreviated CDP, is the local debugging protocol exposed by TradingView Desktop when it runs with a remote debugging port. Runtime JavaScript evaluation is abstracted by the `RuntimeEvaluator` trait in `src/cdp.rs`; unit tests use fake runtimes from `src/ops/test_support.rs` so tests do not require TradingView Desktop.

Existing Rust data commands already read drawing-derived Pine graphics through `tv data lines/labels/tables/boxes`, but they do not create or remove TradingView chart drawings. The old JavaScript drawing mutation surface should live in a new `src/ops/drawing.rs` module so it stays separate from read-only Pine graphics summaries.

## Plan of Work

First, extend the CLI surface in `src/cli.rs` with a new top-level `Draw` command and a `DrawingCommand` subcommand enum. The subcommands are `shape`, `list`, `get`, and `remove`. `shape` accepts `--type`, `--price`, `--time`, optional `--price2` and `--time2`, optional `--text`, and optional `--overrides`. `clear` must not be added in this slice.

Next, update `src/main.rs` dispatch. Keep validation there narrow: reject non-finite numeric inputs, require `--price2` and `--time2` together, reject empty entity ids, parse `--overrides` as a JSON object before connecting, and connect to CDP only after validation passes.

Then, create `src/ops/drawing.rs`. Implement `drawing_shape`, `drawing_list`, `drawing_get`, `drawing_remove`, and `parse_drawing_overrides`. Use `serde_json::to_string` and existing helpers such as `js_string` rather than hand-written JavaScript quoting. Every operation should return command payloads under the Rust `data` envelope through normal dispatch.

For `shape`, compare shape ids before and after `createShape()` or `createMultipointShape()`, wait briefly for TradingView to attach the shape, and return the new `entity_id`. If no new id appears, fail with an internal API error. For `list`, return count and shape summaries. For `get`, return points, properties, and common flags when available. For `remove`, verify the shape exists, call `removeEntity(entity_id)`, and verify it no longer exists.

Finally, update tests and durable docs. Unit tests should prove finite validation and safe serialization behavior at operation level. CLI contract tests should cover help, invalid `--overrides`, missing required ids, and structured connection errors. Docs should move only `draw shape/list/get/remove` to implemented and leave `draw clear` deferred.

## Concrete Steps

From the repository root, run the implementation loop:

    cargo fmt --check
    cargo clippy --all-targets --all-features -- -D warnings
    cargo test
    git diff --check
    run the repository's standard tracked-doc local absolute path scan

If `cargo fmt --check` fails only because of formatting, run `cargo fmt` and repeat the baseline.

If TradingView Desktop is available through CDP, run a live smoke with a drawing that is safe to add and remove:

    cargo run --quiet -- state
    cargo run --quiet -- quote
    cargo run --quiet -- draw shape --type horizontal_line --price <PRICE> --time <VISIBLE_UNIX_SECONDS>
    cargo run --quiet -- draw get <ENTITY_ID>
    cargo run --quiet -- draw list
    cargo run --quiet -- draw remove <ENTITY_ID>
    cargo run --quiet -- draw list

Record the chosen price, time, and returned `entity_id` in this plan. Do not use `draw clear`.

## Validation and Acceptance

The change is accepted when `tv draw --help` lists `shape`, `list`, `get`, and `remove`; `clear` is absent; invalid numeric inputs and invalid `--overrides` fail before CDP connection; connection failures use the structured connection envelope; and all automated tests pass.

The success JSON must use the Rust envelope. For example, `tv draw shape --type horizontal_line --price 100 --time 1700000000` should print a success envelope whose `data` includes `action: "shape"`, `shape`, `entity_id`, and shape counts. `tv draw remove <ENTITY_ID>` should print a success envelope whose `data` includes `action: "remove"`, `entity_id`, `removed`, and remaining count.

## Idempotence and Recovery

Automated tests are safe to rerun and must use fake runtimes. They must not require a running TradingView Desktop.

Live smoke mutates the active chart drawings. It is safe only when the smoke adds a fresh drawing and removes that same returned `entity_id` afterward. If shape creation succeeds but remove fails, record the `entity_id` in this plan and stop; do not use `draw clear` as cleanup.

## Artifacts and Notes

Automated validation before live smoke:

    cargo fmt --check
    result: passed

    cargo clippy --all-targets --all-features -- -D warnings
    result: passed

    cargo test
    result: ok. 90 unit tests and 32 CLI contract tests passed.

Final validation after live smoke and docs updates:

    cargo fmt --check
    result: passed

    cargo clippy --all-targets --all-features -- -D warnings
    result: passed

    cargo test
    result: ok. 90 unit tests and 32 CLI contract tests passed.

    git diff --check
    result: passed

    tracked-doc local absolute path scan
    result: no tracked-doc local absolute paths found

Live smoke:

    cargo run --quiet -- state
    result: success true, symbol BATS:LWLG, resolution 1D, visible range from 1758499200 to 1788825600

    cargo run --quiet -- quote
    result: success true, close 13.6, time 1776951000

    cargo run --quiet -- draw list
    result: success true, count 2, existing drawings dMlruO and vlHUFh

    cargo run --quiet -- draw shape --type horizontal_line --price 13.6 --time 1776951000 --text tv-cli-smoke
    result: success true, entity_id DeQmSS, before_count 2, after_count 3

    cargo run --quiet -- draw get DeQmSS
    result: success true, points included price 13.6 and normalized time 1776902400, properties included text tv-cli-smoke

    cargo run --quiet -- draw list
    result: success true, count 3, DeQmSS present as horizontal_line

    cargo run --quiet -- draw remove DeQmSS
    result: success true, removed true, before_count 3, remaining_shapes 2

    cargo run --quiet -- draw list
    result: success true, count 2, DeQmSS absent

## Interfaces and Dependencies

At the end of the implementation, these commands must exist:

    tv draw shape --type <TYPE> --price <NUMBER> --time <UNIX_SECONDS> [--price2 <NUMBER>] [--time2 <UNIX_SECONDS>] [--text <TEXT>] [--overrides <JSON>]
    tv draw list
    tv draw get <ENTITY_ID>
    tv draw remove <ENTITY_ID>

The operation facade must expose:

    pub async fn drawing_shape(runtime: &mut impl RuntimeEvaluator, request: DrawingShapeRequest) -> Result<Value, AppError>
    pub async fn drawing_list(runtime: &mut impl RuntimeEvaluator) -> Result<Value, AppError>
    pub async fn drawing_get(runtime: &mut impl RuntimeEvaluator, entity_id: &str) -> Result<Value, AppError>
    pub async fn drawing_remove(runtime: &mut impl RuntimeEvaluator, entity_id: &str) -> Result<Value, AppError>
    pub fn parse_drawing_overrides(raw: &str) -> Result<Value, AppError>

No new third-party Rust dependencies are required.

## Open Questions

There are no unresolved critical questions at the plan stage. If live TradingView behavior differs from the old JavaScript assumptions, record the discovery here and choose the safest failing behavior rather than bulk cleanup.
