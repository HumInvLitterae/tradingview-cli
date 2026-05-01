# Fundamentals read and Desktop capability gap audit

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document follows `.agents/PLANS.md` from the repository root. It is self-contained so a future contributor can continue from this file alone.

## Purpose / Big Picture

After this change, a user can run `tv fundamentals <SYMBOL>` to fetch a small, public-safe set of scanner-backed fundamental fields without opening TradingView Desktop. The first useful case is earnings timing: the scanner metadata confirms fields such as `earnings_release_next_date`, `earnings_release_date`, and `earnings_release_next_time` exist. This is a read-only data feature, not investment analysis.

The slice also records a TradingView Desktop capability gap audit so the project does not assume the current CLI surface covers every useful Desktop feature. The audit is a prioritization note, not a promise to reimplement all of TradingView Desktop.

## Progress

- [x] (2026-05-02) Created this ExecPlan and archived the completed Codex app Computer Use skill research plan.
- [x] (2026-05-02) Confirmed scanner metainfo exposes earnings date/time fields and that existing scanner scan allowlist blocks them.
- [x] (2026-05-02) Added `tv fundamentals <SYMBOL>` and typed market API support.
- [x] (2026-05-02) Extended scanner scan supported columns with earnings date/time fields.
- [x] (2026-05-02) Added the Desktop capability gap audit note.
- [x] (2026-05-02) Updated README, internal API docs, v0.5 roadmap, skills, changelog, and local continuity.
- [x] (2026-05-02) Ran focused, full, skill, packaging, smoke, and hygiene validation.
- [ ] Commit the related change as `feat(market): Add fundamentals read`.

## Surprises & Discoveries

- Observation: `tv scanner metainfo --market america --field earnings_release_next_date --field earnings_release_date --field earnings_release_next_time` returns all requested fields as public-safe scanner metadata.
  Evidence: the command returned `type: "time"` for the date fields and `type: "number"` for `earnings_release_next_time`.
- Observation: `tv scanner scan --columns name,earnings_release_next_date` currently fails before network because the explicit scanner scan allowlist does not include the earnings fields.
  Evidence: the CLI returned `Unsupported scanner column: earnings_release_next_date`.

## Decision Log

- Decision: Add a new top-level `tv fundamentals <SYMBOL>` command rather than extending `tv info`.
  Rationale: `info` is symbol metadata, while fundamentals include financial and event fields. A separate command keeps the public surface understandable.
  Date/Author: 2026-05-02 / Codex.
- Decision: Keep the first implementation Desktop-free and scanner REST only.
  Rationale: the relevant fields are discoverable through scanner metainfo and do not require chart state. Chart fallback would blur provenance and freshness boundaries.
  Date/Author: 2026-05-02 / Codex.
- Decision: Preserve raw scanner values for earnings date/time fields and avoid timezone or before/after-market interpretation.
  Rationale: TradingView field semantics are not fully documented here. Returning provenance and raw values is safer than guessing.
  Date/Author: 2026-05-02 / Codex.

## Outcomes & Retrospective

Implemented. The CLI now has a Desktop-free `fundamentals` command backed by scanner REST, scanner scan accepts earnings date/time columns, and durable docs record that broader Desktop capability work should be driven by concrete operator value rather than parity. The command returns raw scanner values for earnings fields and does not infer timezone or before/after-market semantics.

## Context and Orientation

The repository is a Rust workspace. The CLI package lives in `crates/cli`. Desktop-free symbol, quote, and batch quote reads live in `crates/market`, and scanner table/metainfo reads live in `crates/scanner`.

`scanner REST` means an unauthenticated TradingView scanner HTTP endpoint already used by `tv quote`, `tv quotes`, `tv scanner scan`, and `tv scanner metainfo`. It is read-only and does not connect to TradingView Desktop or Chrome DevTools Protocol.

The command envelope remains `{ success, command, data }` for success and `{ success, command, error }` for failures. The new command name is `fundamentals`.

## Plan of Work

First, add a typed fundamentals result to `crates/market`. Create a new module that queries `https://scanner.tradingview.com/america/scan` with a name filter and optional exchange filter, similar to scanner-backed quotes. It must validate that the symbol is non-empty, the requested fields are supported, and the returned scanner row matches the requested bare symbol. It must return validation errors with symbol-search candidates for no-row, ambiguous, and mismatch cases.

Second, add a top-level `Fundamentals` CLI command in `crates/cli/src/cli.rs` and dispatch it through `crates/cli/src/app/dispatch.rs` to a thin operation wrapper in `crates/cli/src/ops/market`. The command takes a required `SYMBOL` and repeatable `--field <FIELD>`.

Third, extend `crates/scanner/src/scan.rs` so scanner scan explicitly accepts the earnings date/time columns that fundamentals uses. Do not change default scanner scan columns.

Fourth, add the Desktop capability gap audit note under `docs/notes/`. Keep it high-level and public-safe. Classify missing or partial areas without recording account-local data or raw live payloads.

Finally, update README, `docs/internal-tradingview-apis.md`, `docs/v0.5-roadmap.md`, `docs/plans/README.md`, `CHANGELOG.md`, and the market-data skills. Update `CONTINUITY.md` locally but do not commit it.

## Concrete Steps

Work from the repository root.

Run focused implementation checks while editing:

    cargo test -p tradingview-market fundamental -- --nocapture
    cargo test -p tradingview-scanner scan -- --nocapture
    cargo test -p tradingview-cli --test cli_contract fundamentals -- --nocapture
    cargo test -p tradingview-cli --test cli_contract scanner -- --nocapture

Then run the full baseline:

    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1
    git diff --check

Run read-only smoke after building:

    target/debug/tv fundamentals NYSE:IONQ
    target/debug/tv fundamentals AAPL --field earnings_release_next_date --field earnings_release_next_time
    target/debug/tv scanner scan --limit 3 --columns name,earnings_release_next_date,earnings_release_date,price_earnings_ttm
    TV_CDP_PORT=9 target/debug/tv fundamentals NYSE:IONQ

Observed result: the smoke commands succeeded without CDP, returned
`source: "scanner_fundamentals_rest"` for `fundamentals`, and allowed
`earnings_release_next_date` / `earnings_release_date` through scanner scan
columns.

Validate changed skills and packaging:

    python3 "${CODEX_HOME:-$HOME/.codex}/skills/.system/skill-creator/scripts/quick_validate.py" .agents/skills/market-data-interpretation
    python3 "${CODEX_HOME:-$HOME/.codex}/skills/.system/skill-creator/scripts/quick_validate.py" .agents/skills/multi-symbol-scan
    bash -n scripts/stage-release-package-files.sh

Run the tracked-doc hygiene check:

    rg -n '(/Users/|C:\\|USER;|sessionid|cookie|authorization|bearer)' README.md CHANGELOG.md docs .agents/skills packaging scripts || true

Only existing policy text and validation-command examples should match.

## Validation and Acceptance

The feature is accepted when `tv fundamentals NYSE:IONQ` succeeds without a Desktop/CDP connection and returns `source: "scanner_fundamentals_rest"`, `requested_symbol`, `symbol`, `observed_symbol`, `market`, `fields`, `field_values`, `missing_fields`, and `non_mutating: true`.

Unknown fields must fail before network with a validation error listing supported fields. Exchange-mismatched input must not fall back to chart state. `tv scanner scan --columns name,earnings_release_next_date` must pass validation, while existing scanner defaults remain unchanged.

Validation completed successfully:

    cargo test -p tradingview-market fundamental -- --nocapture
    cargo test -p tradingview-scanner scan -- --nocapture
    cargo test -p tradingview-cli --test cli_contract fundamentals -- --nocapture
    cargo test -p tradingview-cli --test cli_contract scanner -- --nocapture
    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1
    git diff --check
    python3 "${CODEX_HOME:-$HOME/.codex}/skills/.system/skill-creator/scripts/quick_validate.py" .agents/skills/market-data-interpretation
    python3 "${CODEX_HOME:-$HOME/.codex}/skills/.system/skill-creator/scripts/quick_validate.py" .agents/skills/multi-symbol-scan
    bash -n scripts/stage-release-package-files.sh

## Idempotence and Recovery

All edits are additive or documentation updates. If a smoke command fails because TradingView changes scanner field availability, keep the code tests authoritative and record the live mismatch in the plan before deciding whether to narrow the supported default fields. Do not add chart fallback to work around scanner failures.

## Artifacts and Notes

Public-safe evidence gathered before implementation:

    tv scanner metainfo --market america --field earnings_release_next_date --field earnings_release_date --field earnings_release_next_time

returned all fields, including date fields typed as `time`.

## Interfaces and Dependencies

In `crates/market`, expose:

    pub async fn fundamentals_symbol(symbol: &str, fields: Vec<String>) -> Result<serde_json::Value, AppError>;
    pub async fn fundamentals_symbol_typed(symbol: &str, fields: Vec<String>) -> Result<Fundamentals, AppError>;

The typed `Fundamentals` struct must serialize into the CLI payload shape and include `source`, `requested_symbol`, `symbol`, `observed_symbol`, `market`, `fields`, `field_values`, `missing_fields`, and `non_mutating`.

In `crates/cli`, expose a top-level command:

    tv fundamentals <SYMBOL> [--field <FIELD>]...

No new dependency is required.

## Open Questions

No critical open questions remain. The chosen public surface is a new top-level `fundamentals` command, and the initial implementation is scanner REST only.
