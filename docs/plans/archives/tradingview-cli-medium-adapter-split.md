# Medium operation adapter split

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This plan follows `.agents/PLANS.md` in this repository. It is self-contained and describes a behavior-preserving refactor: a future contributor should be able to understand why the files moved, how to validate that the CLI still behaves the same, and where to continue after this slice.

## Purpose / Big Picture

The `tv` CLI has already moved from oversized operation files toward facade modules with same-named implementation directories. Screener, Alert, Layout, and Pine Editor now follow that pattern. The remaining medium-sized operation adapters for Drawing, Replay, and chart-dependent Market code are still single files, so related validation, read, mutation, and payload logic are harder to scan than necessary.

After this change, Drawing, Replay, and Market keep the exact same public CLI behavior, JSON envelopes, and exit codes, but their implementation is easier to maintain. A user can verify that by running the same `tv draw`, `tv replay`, `tv quote`, and `tv ohlcv` commands before and after the refactor and seeing the same success or structured error shapes.

## Progress

- [x] (2026-04-28) Read the current Drawing, Replay, Market, and UI adapter shapes and chose Drawing/Replay/Market for this batch while leaving generic `ui.rs` for a later slice.
- [x] (2026-04-28) Split `crates/cli/src/ops/drawing.rs` into a facade plus `drawing/validation.rs`, `drawing/create.rs`, `drawing/read.rs`, and `drawing/lifecycle.rs`.
- [x] (2026-04-28) Split `crates/cli/src/ops/replay.rs` into a facade plus `replay/validation.rs`, `replay/control.rs`, `replay/autoplay.rs`, `replay/trade.rs`, `replay/status.rs`, and `replay/payload.rs`.
- [x] (2026-04-28) Split `crates/cli/src/ops/market.rs` into a facade plus `market/direct.rs`, `market/quote.rs`, and `market/ohlcv.rs`.
- [x] (2026-04-28) Archived the completed Pine Editor adapter split plan.
- [x] (2026-04-28) Update durable architecture, development, roadmap, changelog, plans index, and local continuity notes.
- [x] (2026-04-28) Run focused and full validation, including command-contract tests and behavior smoke.
- [ ] Commit the related changes as one refactor.

## Surprises & Discoveries

- Observation: Moving `PositionDirection` into Drawing validation made methods that were private inside the original single file inaccessible to the create module.
  Evidence: `cargo test -p tradingview-cli drawing -- --nocapture` initially failed with private method errors for `shape_name` and `as_str`. The methods were changed to `pub(super)` because they are only needed by sibling Drawing modules.
- Observation: The chart quote module still needed `BARS_PATH` after the split.
  Evidence: The same initial compile reported `cannot find value BARS_PATH in this scope` in `crates/cli/src/ops/market/quote.rs`; importing it from common preserved the original JavaScript expression.

## Decision Log

- Decision: Split Drawing, Replay, and Market in one batch rather than one file per plan.
  Rationale: The work is mechanically similar and the user explicitly asked to avoid overly fine-grained slices when the remaining tasks share the same shape.
  Date/Author: 2026-04-28 / Codex.
- Decision: Keep `ui.rs` out of this batch.
  Rationale: Generic UI automation has a different safety model because it is gated by unsafe UI-eval behavior and raw input events. Mixing it into this batch would make the refactor harder to validate and reason about.
  Date/Author: 2026-04-28 / Codex.
- Decision: Do not create new workspace crates for Drawing, Replay, or chart-dependent Market code.
  Rationale: These adapters depend on CDP page state, chart APIs, replay APIs, or visible TradingView state. The useful boundary today is an internal CLI-package adapter split, not a reusable domain crate.
  Date/Author: 2026-04-28 / Codex.

## Outcomes & Retrospective

At completion, the repository should have facade files for Drawing, Replay, and Market that preserve existing exports used by `crates/cli/src/ops.rs` and application dispatch. The implementation bodies should live in same-named directories, focused tests should still pass, and full workspace validation should show no behavior changes. Any future slice can then decide whether `ui.rs`, smaller adapters, or deeper helper extraction is worth doing.

The implementation now has that shape. Full workspace validation, focused module tests, command-contract tests, and non-mutating/error-path smoke checks passed. The one adjustment to the original smoke command is that `tv draw position` takes `DIRECTION` as a positional argument, so the validation smoke used `tv draw position long ...` instead of `--direction long`.

## Context and Orientation

The repository is a Cargo workspace. The CLI package lives under `crates/cli/`; the `tv` binary and application dispatch use operation functions re-exported from `crates/cli/src/ops.rs`.

An operation adapter is a command-facing implementation module. It turns CLI requests into TradingView reads or mutations, often through CDP, page-session JavaScript, or direct HTTP helper crates. A facade file is a Rust file such as `crates/cli/src/ops/drawing.rs` that declares submodules and re-exports the public adapter functions. A same-named directory such as `crates/cli/src/ops/drawing/` contains the implementation modules.

Before this plan, the medium adapters were single files:

- `crates/cli/src/ops/drawing.rs` combined drawing request validation, drawing creation, drawing reads, and drawing lifecycle cleanup.
- `crates/cli/src/ops/replay.rs` combined replay validation, control actions, autoplay, trade actions, status reads, and payload normalization.
- `crates/cli/src/ops/market.rs` combined Desktop-free market delegations, chart quote fallback, quote freshness/restore logic, OHLCV reads, and OHLCV summaries.

The desired shape after this plan is:

- `crates/cli/src/ops/drawing.rs` as the facade; implementation under `crates/cli/src/ops/drawing/`.
- `crates/cli/src/ops/replay.rs` as the facade; implementation under `crates/cli/src/ops/replay/`.
- `crates/cli/src/ops/market.rs` as the facade; implementation under `crates/cli/src/ops/market/`.

Rust 2024 is used in this repository. Do not introduce `mod.rs`.

## Plan of Work

First, create this plan and archive the completed Pine Editor plan from `docs/plans/` into `docs/plans/archives/`.

Second, split Drawing. Keep `crates/cli/src/ops/drawing.rs` as a facade that declares `validation`, `create`, `read`, and `lifecycle` modules. Move request types, `PositionDirection`, override parsing, position validation, and validation tests into `drawing/validation.rs`. Move `drawing_shape`, `drawing_position`, and their tests into `drawing/create.rs`. Move `drawing_list`, `drawing_get`, and read tests into `drawing/read.rs`. Move `drawing_remove`, `drawing_clear`, and lifecycle tests into `drawing/lifecycle.rs`. Keep the public exports exactly as before.

Third, split Replay. Keep `crates/cli/src/ops/replay.rs` as a facade that declares `validation`, `control`, `autoplay`, `trade`, `status`, and `payload` modules. Move date parsing, autoplay speed validation, trade action validation, and validation tests into `replay/validation.rs`. Move `replay_start`, `replay_step`, and `replay_stop` into `replay/control.rs`. Move `replay_autoplay` into `replay/autoplay.rs`. Move `replay_trade` into `replay/trade.rs`. Move `replay_status` into `replay/status.rs`. Move payload normalization helpers into `replay/payload.rs`. Keep the public exports exactly as before.

Fourth, split Market. Keep `crates/cli/src/ops/market.rs` as a facade that declares `direct`, `quote`, and `ohlcv` modules. Move `symbol_search`, `symbol_info_direct`, and direct `quote_symbol` delegation into `market/direct.rs`. Move chart current-quote read, symbol-targeted chart fallback quote, quote lock, symbol switch/restore, and freshness checks into `market/quote.rs`. Move `ohlcv_bars`, `ohlcv_summary`, readiness details, and summary helpers into `market/ohlcv.rs`. Keep `tradingview_market` as the owner of Desktop-free HTTP logic; this plan does not move more code into that crate.

Fifth, update durable docs. `docs/architecture.md`, `docs/development.md`, `docs/v0.3-roadmap.md`, `CHANGELOG.md`, `docs/plans/README.md`, and `CONTINUITY.md` should reflect that Drawing, Replay, and chart-dependent Market now follow the facade + submodule pattern. `CONTINUITY.md` is local and gitignored, so it should be updated but not committed.

Finally, run validation and behavior smoke, fix any import or visibility issues, and commit the related changes in one batch.

## Concrete Steps

Run all commands from the repository root.

Create the new plan and archive the completed previous plan:

    git mv docs/plans/tradingview-cli-pine-editor-adapter-split.md docs/plans/archives/tradingview-cli-pine-editor-adapter-split.md

After code movement, format and run focused tests:

    cargo fmt
    cargo test -p tradingview-cli drawing -- --nocapture
    cargo test -p tradingview-cli replay -- --nocapture
    cargo test -p tradingview-cli market -- --nocapture

Run full validation:

    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo test -p tradingview-cli --test cli_contract draw -- --nocapture
    cargo test -p tradingview-cli --test cli_contract replay -- --nocapture
    cargo test -p tradingview-cli --test cli_contract quote -- --nocapture
    cargo test -p tradingview-cli --test cli_contract ohlcv -- --nocapture
    cargo metadata --no-deps --format-version 1
    git diff --check

Run focused module checks where module names match:

    cargo test -p tradingview-cli drawing::validation -- --nocapture
    cargo test -p tradingview-cli drawing::create -- --nocapture
    cargo test -p tradingview-cli drawing::read -- --nocapture
    cargo test -p tradingview-cli drawing::lifecycle -- --nocapture
    cargo test -p tradingview-cli replay::validation -- --nocapture
    cargo test -p tradingview-cli replay::control -- --nocapture
    cargo test -p tradingview-cli replay::autoplay -- --nocapture
    cargo test -p tradingview-cli replay::trade -- --nocapture
    cargo test -p tradingview-cli replay::status -- --nocapture
    cargo test -p tradingview-cli market::quote -- --nocapture
    cargo test -p tradingview-cli market::ohlcv -- --nocapture

Run behavior smoke:

    target/debug/tv draw --help
    target/debug/tv replay --help
    target/debug/tv quote NYSE:IONQ
    target/debug/tv draw position long --entry-price NaN --stop-loss 90 --take-profit 120
    target/debug/tv replay start --date 2026-02-31
    TV_CDP_PORT=9 target/debug/tv draw list
    TV_CDP_PORT=9 target/debug/tv replay status
    TV_CDP_PORT=9 target/debug/tv ohlcv --count 1

Before committing, check hygiene:

    git grep -nE '(/Users/|C:\\|USER;|sessionid|cookie|authorization|bearer)' -- README.md CHANGELOG.md docs .agents/skills packaging scripts || true

Commit:

    git add crates/cli/src/ops docs CHANGELOG.md
    git commit -m "refactor(ops): Split medium adapter modules"

## Validation and Acceptance

The change is accepted when the validation commands pass, the behavior smoke shows the same categories of output as before the refactor, and `git diff --check` is clean.

The expected behavior is not new functionality. The important observable result is preservation:

- `tv quote NYSE:IONQ` still succeeds through the Desktop-free direct market path.
- malformed Drawing and Replay requests still fail before CDP connection with structured validation errors.
- commands that require Desktop/CDP still return structured connection errors when `TV_CDP_PORT=9` points to no running CDP endpoint.
- command-contract tests for `draw`, `replay`, `quote`, and `ohlcv` still pass.

## Idempotence and Recovery

The split is file movement and import adjustment. If a test fails, inspect the failing module path, restore missing imports from the original single-file context, and keep helper visibility as narrow as possible. Prefer `pub(super)` for helpers used only by sibling modules. If a submodule split becomes confusing, use `git diff` to compare the facade exports with `crates/cli/src/ops.rs`; the exported names must not change.

The behavior smoke is safe: the drawing and replay examples use invalid input or a bad CDP port for mutation-capable operations, so they should not mutate TradingView state. `tv quote NYSE:IONQ` is a Desktop-free read.

## Artifacts and Notes

Initial focused test evidence after the first import/visibility fixes:

    cargo test -p tradingview-cli drawing -- --nocapture
    result: 26 passed; 0 failed

    cargo test -p tradingview-cli replay -- --nocapture
    result: 24 passed; 0 failed

    cargo test -p tradingview-cli market -- --nocapture
    result: 12 passed; 0 failed

Full validation evidence:

    cargo fmt --check
    result: passed

    cargo clippy --workspace --all-targets --all-features -- -D warnings
    result: passed

    cargo test --workspace
    result: passed; tradingview-cli unit tests reported 300 passed and cli_contract reported 89 passed

    cargo test -p tradingview-cli --test cli_contract draw -- --nocapture
    result: 4 passed; 0 failed

    cargo test -p tradingview-cli --test cli_contract replay -- --nocapture
    result: 4 passed; 0 failed

    cargo test -p tradingview-cli --test cli_contract quote -- --nocapture
    result: 2 passed; 0 failed

    cargo test -p tradingview-cli --test cli_contract ohlcv -- --nocapture
    result: 2 passed; 0 failed

Behavior smoke evidence:

    target/debug/tv draw --help
    result: listed draw subcommands

    target/debug/tv replay --help
    result: listed replay subcommands

    target/debug/tv quote NYSE:IONQ
    result: success through source scanner_scan_rest

    target/debug/tv draw position long --entry-price NaN --stop-loss 90 --take-profit 120
    result: validation error, entry_price must be finite

    target/debug/tv replay start --date 2026-02-31
    result: validation error for invalid YYYY-MM-DD date

    TV_CDP_PORT=9 target/debug/tv draw list
    result: structured connection error

    TV_CDP_PORT=9 target/debug/tv replay status
    result: structured connection error

    TV_CDP_PORT=9 target/debug/tv ohlcv --count 1
    result: structured connection error

## Interfaces and Dependencies

At the end of the plan, these public adapter exports must still exist through `crates/cli/src/ops.rs`:

- Drawing: `DrawingPoint`, `DrawingPositionRequest`, `DrawingShapeRequest`, `PositionDirection`, `drawing_clear`, `drawing_get`, `drawing_list`, `drawing_position`, `drawing_remove`, `drawing_shape`, `parse_drawing_overrides`, `validate_position_request`.
- Replay: `replay_autoplay`, `replay_start`, `replay_status`, `replay_step`, `replay_stop`, `replay_trade`, `validate_replay_autoplay_speed`, `validate_replay_date`, `validate_replay_trade_action`.
- Market: `ohlcv_bars`, `ohlcv_summary`, `quote`, `quote_symbol`, `symbol_info_direct`, `symbol_search`.

No new runtime dependencies should be added. Drawing and Replay remain CLI-package adapters because they rely on CDP page state. Market remains split between direct HTTP reads delegated to `tradingview_market` and chart-dependent quote/OHLCV code in the CLI package.

## Open Questions

No critical open question blocks this plan. A later plan may decide whether generic `ui.rs` should also be split or whether smaller adapters such as tab, launch, and saved layout are already acceptable as single files.
