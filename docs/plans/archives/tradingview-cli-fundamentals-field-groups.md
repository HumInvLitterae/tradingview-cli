# Fundamentals field groups

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document follows `.agents/PLANS.md` from the repository root. It must remain self-contained enough for a new contributor to understand and finish the work without prior chat context.

## Purpose / Big Picture

After the previous slice, users can run `tv fundamentals <SYMBOL>` to read a curated set of scanner-backed fundamental fields without TradingView Desktop. This slice makes that command more useful by adding repeatable field groups such as `--group earnings` and `--group dividends`. A user can request a coherent bundle of fundamentals without memorizing every scanner field name, while the command still returns raw TradingView scanner values and does not infer investment meaning, timezone, or before/after-market labels.

## Progress

- [x] (2026-05-02 00:00Z) Confirmed scanner metainfo exposes candidate earnings, valuation, dividend, and financial fields without Desktop/CDP.
- [x] (2026-05-02 00:10Z) Added `--group` parsing and group expansion for `tv fundamentals`.
- [x] (2026-05-02 00:30Z) Updated durable docs, skills, and field-evidence notes.
- [x] (2026-05-02 00:50Z) Ran focused tests, full workspace validation, skill validation, packaging syntax check, read-only smoke, metadata, diff, and hygiene checks.
- [x] (2026-05-02 01:00Z) Committed the related changes as `feat(market): Add fundamentals field groups`.

## Surprises & Discoveries

- Observation: Scanner metainfo currently reports all candidate group fields tested for this slice.
  Evidence: `tv scanner metainfo --market america --field ...` returned `missing_fields: []` for earnings, valuation, dividend, and financial candidates.

## Decision Log

- Decision: Add field groups to `tv fundamentals` instead of adding a new `tv events` or `tv financials` command.
  Rationale: The fields are scanner field bundles for the same single-symbol fundamentals read. A new command would imply a richer event or financial statement model that this slice does not implement.
  Date/Author: 2026-05-02 / Codex

- Decision: Keep group names as plain CLI strings and validate them in application output rather than using a clap enum.
  Rationale: Existing invalid fundamentals input returns structured JSON error envelopes. Validating after parse preserves that contract for unknown groups.
  Date/Author: 2026-05-02 / Codex

- Decision: Do not infer timezone, before/after-market meaning, or financial analysis from returned fields.
  Rationale: TradingView scanner returns raw field values. The CLI is a data read tool and should not add unverified semantics.
  Date/Author: 2026-05-02 / Codex

## Outcomes & Retrospective

Implemented and validated. Users can now run `tv fundamentals <SYMBOL> --group earnings`, `--group valuation`, `--group dividends`, or `--group financials` to request curated scanner field bundles. The command remains Desktop-free, default field behavior is unchanged, and raw scanner values remain under `field_values`.

## Context and Orientation

The repository is a Rust workspace. The `tv` binary and CLI surface live in `crates/cli`. Desktop-free market reads live in `crates/market`. Scanner table and metadata reads live in `crates/scanner`.

`scanner REST` means TradingView scanner HTTP endpoints used without TradingView Desktop, CDP, cookies, or account mutation. `metainfo` means the scanner endpoint that returns public field metadata such as field names and simple types. `fundamentals` means a single-symbol scanner read that returns selected fields under `field_values`.

The current command is `tv fundamentals <SYMBOL> [--field <FIELD>]...`. It returns the JSON envelope `{ success, command, data }`, with `data.field_values` as the source of truth. This slice adds `--group <GROUP>` while keeping `--field` and the default behavior intact.

## Plan of Work

Update `crates/market/src/fundamentals.rs` so field selection accepts both groups and explicit fields. The supported groups are `earnings`, `valuation`, `dividends`, and `financials`. Group fields are expanded first, explicit fields are appended after that, and duplicates are removed while preserving first occurrence order.

Update `crates/market/src/types.rs` to include `requested_groups` only when group expansion was used. Keep existing fields such as `source`, `requested_symbol`, `symbol`, `observed_symbol`, `market`, `fields`, `field_values`, `missing_fields`, and `non_mutating`.

Update `crates/cli/src/cli.rs`, `crates/cli/src/app/dispatch.rs`, and the market operation wrapper so `tv fundamentals` accepts repeatable `--group` and passes groups to `tradingview-market`. Unknown groups must fail before network access with a JSON validation error that includes `supported_groups`.

Update documentation and runtime skills to show `tv fundamentals NYSE:IONQ --group earnings`, explain the scanner field bundle boundary, and remind agents that fundamentals values are observed data rather than analysis or recommendations.

## Concrete Steps

Work from the repository root.

Run read-only field discovery before finalizing groups:

    target/debug/tv scanner metainfo --market america --field earnings_release_next_date --field earnings_release_next_time --field price_earnings_ttm

Implement group expansion and tests. Then validate with:

    cargo test -p tradingview-market fundamental -- --nocapture
    cargo test -p tradingview-cli --test cli_contract fundamentals -- --nocapture

After docs and skills are updated, run the full validation listed below.

## Validation and Acceptance

Acceptance is met when:

- `tv fundamentals --help` shows `--group`.
- `tv fundamentals NYSE:IONQ --group earnings` returns a successful scanner-backed payload without CDP.
- `tv fundamentals AAPL --group earnings --field price_earnings_ttm` returns earnings group fields plus the explicit field, without duplicate field names.
- `tv fundamentals NYSE:IONQ --group banana` fails with a JSON validation error and does not attempt a network read.
- `tv fundamentals NYSE:IONQ` without groups or fields keeps the previous default field set.

Run:

    cargo test -p tradingview-market fundamental -- --nocapture
    cargo test -p tradingview-cli --test cli_contract fundamentals -- --nocapture
    cargo test -p tradingview-cli --test cli_contract scanner -- --nocapture
    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1
    git diff --check

Run smoke checks:

    target/debug/tv scanner metainfo --market america --field earnings_release_next_date --field earnings_release_next_time --field price_earnings_ttm
    target/debug/tv fundamentals NYSE:IONQ --group earnings
    target/debug/tv fundamentals AAPL --group earnings --field price_earnings_ttm
    target/debug/tv fundamentals AAPL --group valuation --group dividends
    TV_CDP_PORT=9 target/debug/tv fundamentals NYSE:IONQ --group earnings

Validate changed skills with the repo-local skill validator and check packaging syntax:

    python3 "$HOME/.codex/skills/.system/skill-creator/scripts/quick_validate.py" .agents/skills/market-data-interpretation
    python3 "$HOME/.codex/skills/.system/skill-creator/scripts/quick_validate.py" .agents/skills/multi-symbol-scan
    bash -n scripts/stage-release-package-files.sh

Do not write raw scanner responses, account-local identifiers, cookies, tokens, or local machine paths into tracked docs.

## Idempotence and Recovery

The change is additive. Re-running tests and smoke commands is safe. If a TradingView scanner field disappears, remove that field from the affected group before release or record the group as deferred. If a smoke command fails because a symbol has missing values, distinguish missing values from command failure; null field values are acceptable.

## Artifacts and Notes

Public-safe evidence from metainfo:

    `market_cap_basic`, `price_earnings_ttm`, `price_earnings_forward_fy`,
    `earnings_per_share_basic_ttm`, `earnings_per_share_forecast_next_fq`,
    `dividend_yield_recent`, `dividends_yield_current`,
    `dividend_ex_date_upcoming`, `dividend_payment_date_upcoming`,
    `total_revenue_ttm`, `total_revenue_fq`, `net_income_ttm`, `net_income_fq`,
    `revenue_forecast_next_fq`, and `revenue_forecast_next_fy` were visible
    through scanner metainfo during this slice.

## Interfaces and Dependencies

`tradingview-market` exposes the existing `fundamentals_symbol` and `fundamentals_symbol_typed` APIs for field-only callers. This slice adds `fundamentals_symbol_with_groups` and `fundamentals_symbol_with_groups_typed` so the CLI can request group expansion without breaking existing Rust callers.

The CLI interface is:

    tv fundamentals <SYMBOL> [--group <GROUP>]... [--field <FIELD>]...

Supported groups are:

    earnings
    valuation
    dividends
    financials

## Open Questions

None. If field evidence changes during implementation, defer the affected group rather than guessing.
