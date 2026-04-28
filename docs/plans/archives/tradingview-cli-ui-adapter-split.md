# Generic UI operation adapter split

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This plan follows `.agents/PLANS.md` in this repository. It is self-contained and describes a behavior-preserving refactor of the generic `tv ui` compatibility adapter.

## Purpose / Big Picture

The `tv ui` command group is the old-CLI-compatible generic UI automation surface. It can click DOM elements, dispatch keyboard and mouse input, search visible elements, open panels, and run explicitly unsafe JavaScript evaluation when the process-level gate allows it. Before this change, all of those behaviors lived in one file, which made the safety boundary around `ui eval` and the input-event helpers harder to scan.

After this change, `crates/cli/src/ops/ui.rs` is a facade and the implementation lives in focused modules. The public CLI behavior, JSON payloads, exit codes, and `TV_ALLOW_UNSAFE_UI_EVAL` gate behavior stay unchanged. A user can see the refactor is safe by running the same `tv ui` help, validation, unsafe-gate, and bad-CDP-port checks before and after the split.

## Progress

- [x] (2026-04-29) Inspected `ui.rs`, dispatch, CLI contract tests, and current docs to confirm `ui` is the next meaningful single-file adapter split.
- [x] (2026-04-29) Moved `ui.rs` behind a facade and created `dom`, `input`, `selectors`, and `eval` modules.
- [x] (2026-04-29) Kept `TV_ALLOW_UNSAFE_UI_EVAL` in the application safety/dispatch layer and moved only the gated evaluation operation body to `eval.rs`.
- [x] (2026-04-29) Ran focused `ui` tests and CLI contract tests.
- [x] (2026-04-29) Updated durable architecture, development, roadmap, changelog, plans index, and local continuity notes.
- [x] (2026-04-29) Ran full validation and behavior smoke.
- [x] (2026-04-29) Ran final whitespace and tracked-doc hygiene checks.
- [ ] Commit the related changes as one refactor.

## Surprises & Discoveries

- Observation: `ui_element_coordinates` tests use `unwrap_err`, so the successful result type must implement `Debug`.
  Evidence: The first focused `cargo test -p tradingview-cli ui -- --nocapture` failed until `ElementCoordinates` derived `Debug`.
- Observation: The initial smoke example for `ui mouse` used `--x` / `--y`, but the CLI takes `X` and `Y` as positional arguments.
  Evidence: `target/debug/tv ui mouse --x NaN --y 10` returns a clap usage error; `target/debug/tv ui mouse NaN 10` returns the intended finite-number validation error before CDP connection.

## Decision Log

- Decision: Split `ui` into `dom`, `input`, `selectors`, and `eval`.
  Rationale: This keeps DOM search/click helpers, CDP input events, shared selector helpers, and unsafe eval behavior visually separate while preserving the old compatibility surface.
  Date/Author: 2026-04-29 / Codex.
- Decision: Keep the unsafe eval gate outside `ops/ui/eval.rs`.
  Rationale: `TV_ALLOW_UNSAFE_UI_EVAL` is process-level safety policy in the application dispatch layer. The operation module should only evaluate expressions after dispatch has allowed it.
  Date/Author: 2026-04-29 / Codex.
- Decision: Do not create a `tradingview-ui` workspace crate.
  Rationale: Generic UI automation depends on CDP, visible DOM state, and unsafe compatibility policy. It is not a reusable domain layer.
  Date/Author: 2026-04-29 / Codex.

## Outcomes & Retrospective

At completion, `crates/cli/src/ops/ui.rs` should only declare modules and re-export public operations. The operation bodies should live under `crates/cli/src/ops/ui/`, focused tests should pass, and the unsafe eval gate should still reject `tv ui eval` before CDP connection unless `TV_ALLOW_UNSAFE_UI_EVAL=1` is set.

## Context and Orientation

The repository is a Rust workspace. The CLI package lives under `crates/cli/`, and operation adapters live under `crates/cli/src/ops/`.

An operation adapter is a command-facing implementation module. The generic UI adapter is unusual because it is not a TradingView domain feature like alerts or drawings. It is a compatibility surface that can automate arbitrary visible UI or evaluate JavaScript. The `ui eval` command is intentionally guarded by `TV_ALLOW_UNSAFE_UI_EVAL=1` in application dispatch before CDP connection.

The files relevant to this plan are:

- `crates/cli/src/ops/ui.rs`, the public facade.
- `crates/cli/src/ops/ui/dom.rs`, for DOM click/find/panel/fullscreen operations.
- `crates/cli/src/ops/ui/input.rs`, for keyboard, type, hover, scroll, and mouse input.
- `crates/cli/src/ops/ui/selectors.rs`, for selector validation, element coordinates, and numeric field helpers.
- `crates/cli/src/ops/ui/eval.rs`, for gated expression evaluation after dispatch allows it.

Rust 2024 is used in this repository. Do not introduce `mod.rs`.

## Plan of Work

Create this plan and archive `docs/plans/tradingview-cli-medium-adapter-split.md` under `docs/plans/archives/`.

Keep `crates/cli/src/ops/ui.rs` as a facade. It should declare `dom`, `eval`, `input`, and `selectors`, then re-export `ui_click`, `ui_find`, `ui_fullscreen`, `ui_panel`, `ui_eval`, `ui_hover`, `ui_keyboard`, `ui_mouse`, `ui_scroll`, and `ui_type`.

Move DOM-oriented operations into `dom.rs`: `ui_click`, `ui_find`, `ui_panel`, and `ui_fullscreen`. Their tests should move with them.

Move CDP input-event operations into `input.rs`: `ui_keyboard`, `ui_type`, `ui_hover`, `ui_scroll`, and `ui_mouse`, plus key mapping, modifier mapping, and mouse-click dispatch helpers. Their tests should move with them.

Move shared selector helpers into `selectors.rs`: selector strategy validation, element coordinate lookup, numeric field extraction, and the `ElementCoordinates` type. Keep helpers `pub(super)` so only sibling UI modules can use them.

Move only the operation body of `ui_eval` into `eval.rs`. Do not move `require_unsafe_ui_eval_enabled` or change dispatch behavior.

Update stable docs and the local continuity ledger. `CONTINUITY.md` is gitignored and should not be committed.

Run validation, behavior smoke, and commit the tracked changes in one batch.

## Concrete Steps

Run all commands from the repository root.

After code movement, run:

    cargo fmt
    cargo test -p tradingview-cli ui -- --nocapture
    cargo test -p tradingview-cli --test cli_contract ui -- --nocapture

Run focused checks:

    cargo test -p tradingview-cli ui::selectors -- --nocapture
    cargo test -p tradingview-cli ui::input -- --nocapture
    cargo test -p tradingview-cli ui::dom -- --nocapture
    cargo test -p tradingview-cli ui::eval -- --nocapture

Run full validation:

    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1
    git diff --check

Run behavior smoke:

    target/debug/tv ui --help
    target/debug/tv ui click --help
    target/debug/tv ui eval "1+1"
    TV_ALLOW_UNSAFE_UI_EVAL=1 TV_CDP_PORT=9 target/debug/tv ui eval "1+1"
    TV_CDP_PORT=9 target/debug/tv ui find "Indicators"
    target/debug/tv ui mouse NaN 10

Before committing, run:

    git grep -nE '(/Users/|C:\\|USER;|sessionid|cookie|authorization|bearer)' -- README.md CHANGELOG.md docs .agents/skills packaging scripts || true

Commit:

    git add crates/cli/src/ops/ui.rs crates/cli/src/ops/ui docs CHANGELOG.md
    git commit -m "refactor(ui): Split generic automation adapter"

## Validation and Acceptance

The change is accepted when all validation commands pass and behavior smoke confirms these outcomes:

- `tv ui eval "1+1"` without `TV_ALLOW_UNSAFE_UI_EVAL=1` fails before CDP connection with the same safety-gate behavior.
- `TV_ALLOW_UNSAFE_UI_EVAL=1 TV_CDP_PORT=9 tv ui eval "1+1"` attempts CDP connection and returns a structured connection error.
- `TV_CDP_PORT=9 tv ui find "Indicators"` returns a structured connection error because it needs CDP.
- `tv ui mouse NaN 10` fails validation before CDP connection.
- `tv ui --help` and `tv ui click --help` still show the existing command surface.

## Idempotence and Recovery

This is a behavior-preserving file split. If compilation fails, restore missing imports from the original single-file context and keep helper visibility narrow. If a focused module test path does not match exactly, run the nearest matching `cargo test -p tradingview-cli ui -- --nocapture` and record the actual command in this plan.

The smoke commands are safe. They use help output, validation failure, unsafe-gate failure, or a deliberately bad CDP port. They should not mutate a live TradingView UI.

## Artifacts and Notes

Initial focused evidence:

    cargo test -p tradingview-cli ui -- --nocapture
    result: 35 passed; 0 failed

    cargo test -p tradingview-cli --test cli_contract ui -- --nocapture
    result: 20 passed; 0 failed

Full validation evidence:

    cargo fmt --check
    result: passed

    cargo clippy --workspace --all-targets --all-features -- -D warnings
    result: passed

    cargo test --workspace
    result: passed

    cargo test -p tradingview-cli ui::selectors -- --nocapture
    result: 3 passed; 0 failed

    cargo test -p tradingview-cli ui::input -- --nocapture
    result: 5 passed; 0 failed

    cargo test -p tradingview-cli ui::dom -- --nocapture
    result: 5 passed; 0 failed

    cargo test -p tradingview-cli ui::eval -- --nocapture
    result: 1 passed; 0 failed

    cargo metadata --no-deps --format-version 1
    result: passed

Behavior smoke evidence:

    target/debug/tv ui --help
    result: passed

    target/debug/tv ui click --help
    result: passed

    target/debug/tv ui eval "1+1"
    result: validation error before CDP connection, exit 1

    TV_ALLOW_UNSAFE_UI_EVAL=1 TV_CDP_PORT=9 target/debug/tv ui eval "1+1"
    result: structured connection error, exit 2

    TV_CDP_PORT=9 target/debug/tv ui find "Indicators"
    result: structured connection error, exit 2

    target/debug/tv ui mouse NaN 10
    result: validation error before CDP connection, exit 1

Final hygiene evidence:

    git diff --check
    result: passed

    rg -n '(/Users/|C:\\|USER;|sessionid|cookie|authorization|bearer)' README.md CHANGELOG.md docs .agents/skills packaging scripts || true
    result: only existing policy text and validation-command examples, including this plan's hygiene command

## Interfaces and Dependencies

The public adapter exports through `crates/cli/src/ops.rs` must remain:

- `ui_click`
- `ui_keyboard`
- `ui_type`
- `ui_hover`
- `ui_scroll`
- `ui_mouse`
- `ui_find`
- `ui_eval`
- `ui_panel`
- `ui_fullscreen`

No new dependencies should be added. The module continues to use `tradingview_cdp::RuntimeEvaluator`, CDP key/mouse event types, `tradingview_core::AppError`, and `serde_json::Value`.

## Open Questions

No critical open question blocks this plan. Smaller adapters such as tab, launch, and saved layout can remain single files until a later plan proves they need the facade pattern.
