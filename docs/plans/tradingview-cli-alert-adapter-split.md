# Split Alert operation adapter modules

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This plan follows `.agents/PLANS.md`.

## Purpose / Big Picture

This refactor splits the large Alert operation adapter without changing user-visible `tv alert` behavior. The current `alert.rs` mixes alert list/delete, normal alert creation, indicator alert creation, payload sanitization, API fallback policy, and tests in one file. This slice turns `alert.rs` into a facade and moves implementation into focused modules.

The visible result should be no behavior change. Users should see the same alert commands, JSON payloads, validation errors, and exit codes. The maintainability result is that Alert follows the same facade-plus-submodule direction as Screener.

## Progress

- [x] (2026-04-28T10:55Z) Confirmed `alert.rs` was the largest remaining single operation adapter at about 3,069 lines.
- [x] (2026-04-28T11:05Z) Archived the completed Screener engine split ExecPlan.
- [x] (2026-04-28T11:15Z) Split Alert into facade plus `list`, `create`, `indicator`, `delete`, and `payload` modules.
- [x] (2026-04-28T11:15Z) Moved focused tests into the nearest Alert submodule while preserving test behavior.
- [x] (2026-04-28T11:35Z) Updated architecture, development, roadmap, changelog, and plans index docs for the Alert adapter split.
- [x] (2026-04-28T11:50Z) Completed workspace, Alert-focused, contract, smoke, metadata, whitespace, and hygiene validation.

## Surprises & Discoveries

- Observation: `alert_create_via_api` was public before the split but is not used by current internal dispatch.
  Evidence: facade re-export needed an explicit unused-import allowance to preserve that public surface without tripping workspace `-D warnings`.

- Observation: payload sanitization is shared by all alert read/write paths.
  Evidence: list, normal create, indicator create, delete, and delete-all all normalize through the same public-safe alert value sanitizers.

## Decision Log

- Decision: Keep Alert inside the CLI package instead of creating a workspace crate.
  Rationale: Alert operations still depend on page-session APIs, active chart metadata, Pine helper integration, and CLI-facing fallback behavior. The boundary is not yet a stable reusable Rust API.
  Date/Author: 2026-04-28 / Codex.

- Decision: Preserve the `alert_create_via_api` re-export even though current dispatch does not use it directly.
  Rationale: It was a public function on the previous `alert.rs` module. Keeping it avoids an avoidable internal API break during a behavior-preserving refactor.
  Date/Author: 2026-04-28 / Codex.

## Outcomes & Retrospective

Alert now follows the same adapter split direction as Screener without changing
the public CLI contract. The facade preserves existing operation exports, while
the implementation is easier to scan by behavior: endpoint list reads, normal
alert creation, indicator alert creation, delete operations, and shared
sanitized payload normalization.

Validation passed with `cargo fmt --check`,
`cargo clippy --workspace --all-targets --all-features -- -D warnings`,
`cargo test --workspace`, `cargo test -p tradingview-cli alert -- --nocapture`,
`cargo test -p tradingview-cli --test cli_contract alert -- --nocapture`,
focused `alert::create`, `alert::indicator`, and `alert::delete` test filters,
`cargo metadata --no-deps --format-version 1`, `git diff --check`, and the
planned behavior smoke commands. The tracked-doc hygiene grep returned only
existing policy text, archived validation-command examples, and public-safe
references to forbidden payload categories; no new live ids, local paths,
credentials, or webhook values were introduced.

## Context and Orientation

The `tradingview-cli` package lives under `crates/cli/`. Operation adapters are exposed through `crates/cli/src/ops.rs`. Alert was a single `crates/cli/src/ops/alert.rs` file before this slice.

Alert operations include `alert list`, `alert create`, `alert create-indicator`, `alert delete --id`, and `alert delete --all`. Some paths use page-session TradingView alert endpoints, some have DOM fallback, and indicator alert creation also uses Desktop-free Pine alertcondition discovery plus page-session saved-script metadata.

## Plan of Work

Turn `crates/cli/src/ops/alert.rs` into a facade with submodules under `crates/cli/src/ops/alert/`.

Move list behavior into `list.rs`, normal alert creation and condition validation into `create.rs`, indicator alertcondition creation into `indicator.rs`, delete and delete-all behavior into `delete.rs`, and shared public-safe sanitization/normalization into `payload.rs`.

Keep all existing exported function/type names available from `ops::alert`: `alert_list`, `alert_create`, `alert_create_via_api`, `validate_alert_condition`, `alert_create_indicator`, `IndicatorAlertRequest`, `alert_delete`, and `alert_delete_all`.

Move unit tests to the nearest submodule. Do not change alert command behavior, API fallback boundaries, indicator alert metadata rules, payload field names, or error kinds.

## Concrete Steps

Run commands from the repository root.

1. Archive the completed Screener engine split plan:

        git mv docs/plans/tradingview-cli-screener-engine-split.md docs/plans/archives/tradingview-cli-screener-engine-split.md

2. Split Alert implementation into:

        crates/cli/src/ops/alert.rs
        crates/cli/src/ops/alert/list.rs
        crates/cli/src/ops/alert/create.rs
        crates/cli/src/ops/alert/indicator.rs
        crates/cli/src/ops/alert/delete.rs
        crates/cli/src/ops/alert/payload.rs

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
        cargo test -p tradingview-cli alert -- --nocapture
        cargo test -p tradingview-cli --test cli_contract alert -- --nocapture
        cargo metadata --no-deps --format-version 1
        git diff --check

5. Run focused tests:

        cargo test -p tradingview-cli alert::create -- --nocapture
        cargo test -p tradingview-cli alert::indicator -- --nocapture
        cargo test -p tradingview-cli alert::delete -- --nocapture

6. Run behavior smoke:

        target/debug/tv alert --help
        target/debug/tv alert create --price NaN
        target/debug/tv alert delete --id ""
        target/debug/tv alert create-indicator --help
        TV_CDP_PORT=9 target/debug/tv alert list

## Validation and Acceptance

Acceptance requires all workspace tests and Alert contract tests to pass. Focused module tests should run for create, indicator, and delete; if an exact module filter changes, record the actual command in this plan.

Behavior smoke should prove that help still renders, validation failures still happen before CDP connection, and CDP-dependent reads still return structured connection errors when pointed at an unavailable port. JSON envelope and field names must not change.

## Idempotence and Recovery

This split is mechanical. If compilation fails because a moved module needs shared sanitization, keep that code in `payload.rs`. If a helper is clearly specific to normal create or indicator create, keep it in that module instead of moving it to a generic helper. If behavior output changes, restore the previous payload shape rather than updating tests.

## Artifacts and Notes

This slice should not require live TradingView mutation smoke. Do not record live alert ids, saved script ids, raw alert payloads, cookies, tokens, webhook URLs, or local absolute paths in tracked docs.

## Interfaces and Dependencies

At completion, `crates/cli/src/ops/alert.rs` continues to expose the same adapter functions and types. The implementation modules become:

- `list.rs`: alert endpoint list read
- `create.rs`: normal alert create and condition validation
- `indicator.rs`: Pine `alertcondition()` alert creation
- `delete.rs`: single-alert and all-alert cleanup
- `payload.rs`: public-safe sanitization and payload normalization

## Open Questions

No blocking questions. After this split, inspect whether Alert payload normalization should stay private to Alert or whether a later page-session API helper boundary is useful.
