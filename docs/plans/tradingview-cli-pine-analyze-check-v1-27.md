# Add Pine analyze and check commands

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document follows `.agents/PLANS.md`.

## Purpose / Big Picture

After this change, operators can run offline Pine static analysis and server-side Pine compilation checks from the Rust-native `tv` CLI without opening or mutating the TradingView Pine Editor. This closes the next safe Pine development gap after `pine get`, `pine set`, and `pine compile`: a user can validate source from a file or stdin before injecting it into the chart, while still leaving save, new, open, and raw compile behavior deferred.

The work is intentionally narrow. `pine analyze` is a local heuristic checker and does not contact TradingView. `pine check` posts source to TradingView's pine-facade compile endpoint and reports errors and warnings, but it does not connect to Chrome DevTools Protocol, modify the chart, save scripts, or open editor UI.

## Progress

- [x] (2026-04-24 17:05Z) Read `.agents/PLANS.md`, Pine skill guidance, current Rust Pine implementation, old JavaScript Pine CLI/core code, and relevant inventory docs.
- [x] (2026-04-24 17:10Z) Split the existing Pine Editor implementation from `src/ops/pine.rs` into `src/ops/pine/editor.rs` with a thin facade, preserving behavior.
- [x] (2026-04-24 17:10Z) Created this ExecPlan.
- [ ] Commit the behavior-preserving Pine module refactor.
- [ ] Add `tv pine analyze` and `tv pine check` CLI and dispatch.
- [ ] Implement local static analysis in `src/ops/pine/analysis.rs`.
- [ ] Implement server-side Pine compile check in `src/ops/pine/check.rs`.
- [ ] Add unit and CLI contract tests.
- [ ] Update README, AGENTS, migration inventory, contract notes, handoff note, and Pine skill mapping.
- [ ] Run automated validation, skill validation, and live/external smoke.
- [ ] Commit the completed feature slice.

## Surprises & Discoveries

- Observation: Moving `src/ops/pine.rs` under `src/ops/pine/editor.rs` required only one test import path adjustment.
  Evidence: `cargo test ops::pine -- --nocapture` passed after changing the fake runtime import from two parent modules to three.

## Decision Log

- Decision: Split Pine operations before adding analyze/check.
  Rationale: `src/ops/pine.rs` was already large after get/set/compile/errors/console/list. Splitting keeps with the repository guideline to avoid module bloat and lets offline analysis and server checks live outside the CDP editor code.
  Date/Author: 2026-04-24 / Codex.

- Decision: Implement `pine analyze` and `pine check` together.
  Rationale: They are the remaining low-mutation Pine development helpers from the old CLI. Both consume Pine source from stdin or `--file`, neither saves scripts, and together they unblock a practical pre-editor validation loop.
  Date/Author: 2026-04-24 / Codex.

## Outcomes & Retrospective

Not completed yet.

## Context and Orientation

The Rust CLI is a single binary named `tv`. Command-line shape lives in `src/cli.rs`; dispatch and JSON envelopes live in `src/main.rs`; operation functions are re-exported through `src/ops.rs`. Pine command logic now has a thin facade at `src/ops/pine.rs`, with TradingView Desktop editor operations in `src/ops/pine/editor.rs`.

Chrome DevTools Protocol, or CDP, is the local debugging protocol used by commands that inspect or mutate the running TradingView Desktop page. The new `pine analyze` command must not use CDP. The new `pine check` command must also avoid CDP; it should use ordinary HTTP through `reqwest` to contact TradingView's pine-facade endpoint.

The old JavaScript CLI accepted Pine source through stdin or `--file` for `pine analyze` and `pine check`. The Rust implementation already has `read_pine_source` in `src/main.rs` for `pine set`; reuse it so source validation and `input_source` naming remain consistent.

## Plan of Work

First complete the behavior-preserving module split. Keep `src/ops/pine.rs` as a facade that declares submodules and re-exports public Pine operations. Keep existing editor behavior in `src/ops/pine/editor.rs`. Validate with `cargo test ops::pine -- --nocapture`, then commit this refactor separately.

Next update the CLI. Add `Analyze { file: Option<PathBuf> }` and `Check { file: Option<PathBuf> }` to `PineCommand` in `src/cli.rs`. In `src/main.rs`, read source with `read_pine_source(file.as_deref())?`, then call `ops::pine_analyze(&source, input_source)` for analyze and `ops::pine_check(&source, input_source).await` for check. These branches must not call `connect_runtime`.

Add `src/ops/pine/analysis.rs` with `pub fn pine_analyze(source: &str, input_source: &str) -> Value`. Port the old static checks: detect arrays created by `array.from(...)` and `array.new_*` with known sizes; report literal-index `array.get` and `array.set` calls that are out of bounds; warn on `.first()` or `.last()` when the array is known to have size zero; error when `strategy.entry` or `strategy.close` appears without a `strategy(...)` declaration; and add an informational diagnostic when the script declares Pine version below 5. Return `input_source`, `issue_count`, `diagnostics`, and a no-issue note.

Add `src/ops/pine/check.rs` with `pub async fn pine_check(source: &str, input_source: &str) -> Result<Value, AppError>` and a pure helper that normalizes the pine-facade JSON response. Post `source` as form data to `https://pine-facade.tradingview.com/pine-facade/translate_light?user_name=Guest&pine_id=00000000-0000-0000-0000-000000000000`. Map HTTP/network failures to `connection`, malformed response shapes to `internal_api_unavailable`, and successful responses to `compiled`, `error_count`, `warning_count`, `errors`, `warnings`, `input_source`, and `source: "pine_facade"`.

Update tests. Unit tests in `analysis.rs` should cover valid array access, positive and negative out-of-bounds indexes, `array.set`, empty-array `first/last`, missing strategy declaration, older version info, and multiple diagnostics. Unit tests in `check.rs` should cover successful compile payload, error payload, warning payload, outer `error`, and malformed payload. CLI contract tests should verify `pine --help` lists `analyze` and `check`, each rejects missing source before connecting, and each attempts no CDP connection when given source.

Finally update durable docs and skills. Mark `pine analyze` and `pine check` implemented in README, AGENTS, migration inventory, contract notes, handoff note, and `.agents/skills/pine-develop`. Keep `pine raw-compile`, `pine save`, `pine new`, and `pine open` deferred.

## Concrete Steps

Run all commands from the repository root.

After the refactor milestone:

    cargo test ops::pine -- --nocapture

After the feature implementation:

    cargo fmt --check
    cargo clippy --all-targets --all-features -- -D warnings
    cargo test
    git diff --check
    rg -n "(/[U]sers/|[C]:\\\\)" README.md AGENTS.md docs .agents/skills || true

Because `.agents/skills/pine-develop` changes, run the skill validator against that skill before committing the feature slice.

## Validation and Acceptance

Automated acceptance is that the full Rust baseline passes and the new tests prove both commands are usable without CDP. `tv pine analyze --file target/pine-analyze-invalid.pine` should print `success: true` with `data.issue_count` greater than zero for a source containing an obvious out-of-bounds array access. `tv pine check --file target/pine-check-valid.pine` should print `success: true` with `data.compiled: true` for a small valid script, if the TradingView pine-facade endpoint is reachable.

External smoke is separate from CI because it depends on TradingView's public pine-facade endpoint. Create temporary files under ignored `target/`, run `cargo run --quiet -- pine analyze --file ...` and `cargo run --quiet -- pine check --file ...`, and record the observed result here. This smoke does not require TradingView Desktop and must not change the user's chart or account state.

## Idempotence and Recovery

The module split and command additions are ordinary source changes. The smoke writes only temporary files under ignored `target/`. If the external pine-facade endpoint is unavailable or rate-limited, record the structured error and keep automated tests as the acceptance source. No cleanup is required in TradingView Desktop because this slice does not open or mutate the editor.

## Artifacts and Notes

Refactor validation:

    cargo test ops::pine -- --nocapture
    test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 117 filtered out

## Interfaces and Dependencies

At completion, `src/ops/pine.rs` re-exports:

    pub fn pine_analyze(source: &str, input_source: &str) -> serde_json::Value
    pub async fn pine_check(source: &str, input_source: &str) -> Result<serde_json::Value, AppError>

At completion, the CLI exposes:

    tv pine analyze [--file <PATH>]
    tv pine check [--file <PATH>]

The implementation uses the existing `reqwest` dependency for HTTP and does not add new crates.

## Open Questions

No unresolved critical questions remain for this slice.
