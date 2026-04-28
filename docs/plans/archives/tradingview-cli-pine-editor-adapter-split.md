# Split Pine Editor operation adapter modules

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This plan follows `.agents/PLANS.md`.

## Purpose / Big Picture

This refactor splits the Pine Editor operation adapter without changing user-visible `tv pine` behavior. The current `crates/cli/src/ops/pine/editor.rs` file mixes source get/set/new, saved script list/open/save, compile/raw-compile/errors/console, Monaco editor discovery, keyboard dispatch, JavaScript snippets, and tests. After this change, `editor.rs` remains the facade used by `crates/cli/src/ops/pine.rs`, while the implementation lives in focused submodules.

The visible result should be no behavior change. Users should see the same commands, JSON payloads, validation errors, and exit codes. The maintainability result is that Pine Editor follows the same facade-plus-submodule direction as Screener, Alert, and Layout.

## Progress

- [x] (2026-04-28T12:02Z) Confirmed `pine/editor.rs` was the largest remaining unsplit Pine Editor adapter file and mixed source, saved-script, compile, runtime, and tests.
- [x] (2026-04-28T12:02Z) Archived the completed Layout adapter split ExecPlan.
- [x] (2026-04-28T12:02Z) Split Pine Editor into facade plus `runtime`, `source`, `scripts`, and `compile` modules.
- [x] (2026-04-28T12:02Z) Moved focused tests into the nearest Pine Editor submodule while preserving behavior.
- [x] (2026-04-28T12:02Z) Updated architecture, development, roadmap, changelog, and plans index docs.
- [x] (2026-04-28T12:02Z) Ran full validation, behavior smoke, metadata, whitespace, and hygiene checks.

## Surprises & Discoveries

- Observation: `pine new --type study --dry-run` is not a valid current CLI smoke command.
  Evidence: Pine `new` accepts the current script-type vocabulary and does not provide a dry-run flag, so this plan uses help, file validation, and CDP-unavailable smokes instead.

## Decision Log

- Decision: Keep Pine Editor operations in the CLI package and do not create a new workspace crate.
  Rationale: Desktop-free Pine static analysis and Pine facade checks already live in `tradingview_pine`. The Editor operations still depend on CDP, Monaco, visible TradingView UI state, and keyboard dispatch, so they remain operation adapters.
  Date/Author: 2026-04-28 / Codex.

- Decision: Use `runtime.rs` for shared Monaco/editor helpers rather than another `engine.rs`.
  Rationale: The shared code here is specifically Editor runtime discovery, JavaScript wrapping, waits, and input dispatch. `runtime` names that role more clearly than a generic engine module.
  Date/Author: 2026-04-28 / Codex.

## Outcomes & Retrospective

Pine Editor now follows the same adapter split direction as Screener, Alert, and Layout. The facade preserves existing operation exports, while source editing, saved-script operations, compile/reporting operations, and Monaco/CDP runtime helpers live in separate modules with focused tests.

Validation passed with `cargo fmt --check`,
`cargo clippy --workspace --all-targets --all-features -- -D warnings`,
`cargo test --workspace`, `cargo test -p tradingview-cli pine::editor -- --nocapture`,
`cargo test -p tradingview-cli --test cli_contract pine -- --nocapture`,
focused `pine::editor::source`, `pine::editor::scripts`,
`pine::editor::compile`, and `pine::editor::runtime` test filters,
`cargo metadata --no-deps --format-version 1`, `git diff --check`, and the
planned behavior smoke commands. The tracked-doc hygiene grep returned only
existing policy text and archived validation-command examples; no new live ids,
local paths, credentials, webhook URLs, or raw payloads were introduced.

## Context and Orientation

The `tradingview-cli` package lives under `crates/cli/`. Operation adapters are exposed through `crates/cli/src/ops.rs`. Pine is exposed through `crates/cli/src/ops/pine.rs`, which combines Desktop-free Pine helpers from the internal `tradingview_pine` crate with CDP-dependent Pine Editor operations.

In this repository, "Pine Editor operations" means commands that need a running TradingView page and the on-page Pine Editor or Monaco editor: `pine get`, `pine set`, `pine new`, `pine open`, `pine save`, `pine compile`, `pine raw-compile`, `pine errors`, `pine console`, and `pine list`. These are different from Desktop-free static source analysis, which already lives in `crates/pine/`.

## Plan of Work

Turn `crates/cli/src/ops/pine/editor.rs` into a facade with submodules under `crates/cli/src/ops/pine/editor/`.

Move shared runtime behavior into `runtime.rs`: Monaco discovery, Pine panel opening, editor-open state, JavaScript wrapper expansion, array/button normalization, keyboard dispatch, compile/save waits, and runtime-focused tests.

Move source behavior into `source.rs`: `pine_get`, `pine_set`, `pine_new`, `validate_pine_script_type`, source setting, template generation, and source-focused tests.

Move saved-script behavior into `scripts.rs`: `pine_list`, `pine_open`, `pine_save`, saved script list/open expressions, save preflight/post-shortcut expressions, save error mapping, and scripts-focused tests.

Move compile/reporting behavior into `compile.rs`: `pine_compile`, `pine_raw_compile`, `pine_errors`, `pine_console`, study-count detection, compile/raw-compile button discovery, marker/console expressions, and compile-focused tests.

Keep all existing exported function names available from `ops::pine::editor` and from `ops::pine`. Do not change CLI dispatch, JSON payload field names, validation error messages, CDP fallback behavior, or Pine command exit codes.

## Concrete Steps

Run commands from the repository root.

1. Archive the completed Layout adapter split plan:

        git mv docs/plans/tradingview-cli-layout-adapter-split.md docs/plans/archives/tradingview-cli-layout-adapter-split.md

2. Split Pine Editor implementation into:

        crates/cli/src/ops/pine/editor.rs
        crates/cli/src/ops/pine/editor/runtime.rs
        crates/cli/src/ops/pine/editor/source.rs
        crates/cli/src/ops/pine/editor/scripts.rs
        crates/cli/src/ops/pine/editor/compile.rs

3. Update docs:

        docs/architecture.md
        docs/development.md
        docs/v0.3-roadmap.md
        CHANGELOG.md
        docs/plans/README.md
        CONTINUITY.md

4. Validate:

        cargo fmt --check
        cargo clippy --workspace --all-targets --all-features -- -D warnings
        cargo test --workspace
        cargo test -p tradingview-cli pine::editor -- --nocapture
        cargo test -p tradingview-cli --test cli_contract pine -- --nocapture
        cargo metadata --no-deps --format-version 1
        git diff --check

5. Run focused tests:

        cargo test -p tradingview-cli pine::editor::source -- --nocapture
        cargo test -p tradingview-cli pine::editor::scripts -- --nocapture
        cargo test -p tradingview-cli pine::editor::compile -- --nocapture
        cargo test -p tradingview-cli pine::editor::runtime -- --nocapture

6. Run behavior smoke:

        target/debug/tv pine --help
        target/debug/tv pine get --help
        target/debug/tv pine set --file target/missing-pine-smoke.pine
        TV_CDP_PORT=9 target/debug/tv pine get
        TV_CDP_PORT=9 target/debug/tv pine compile

## Validation and Acceptance

Acceptance requires all workspace tests and Pine contract tests to pass. Focused module tests should run for `pine::editor::source`, `pine::editor::scripts`, `pine::editor::compile`, and `pine::editor::runtime`; if an exact module filter changes, record the actual command in this plan.

Behavior smoke should prove that help still renders, file validation still happens before CDP connection where applicable, and CDP-dependent reads still return structured connection errors when pointed at an unavailable port. JSON envelope, field names, and exit codes must not change.

## Idempotence and Recovery

This split is mechanical. If compilation fails because a moved helper is used by multiple modules, keep it in `runtime.rs` only when it is truly shared; otherwise keep the helper in the operation-specific module. If behavior output changes, restore the previous payload shape rather than updating tests.

## Artifacts and Notes

This slice should not require live TradingView mutation smoke. Do not record live saved-script names, saved-script ids, chart target ids, cookies, tokens, or local absolute paths in tracked docs.

## Interfaces and Dependencies

At completion, `crates/cli/src/ops/pine/editor.rs` continues to expose the same adapter functions. The implementation modules become:

- `runtime.rs`: shared Pine Editor runtime and Monaco helpers
- `source.rs`: source get/set/new operations
- `scripts.rs`: saved script list/open/save operations
- `compile.rs`: compile, raw compile, errors, and console operations

No new workspace crate is introduced. `tradingview_pine` remains the crate for Desktop-free Pine static analysis, alertcondition discovery, and Pine facade checks.

## Open Questions

No blocking questions. After this split, inspect whether `drawing.rs`, `replay.rs`, or chart-dependent `market.rs` is the next best adapter cleanup target.
