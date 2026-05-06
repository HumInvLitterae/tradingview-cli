# Fundamentals event field enrichment

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document follows `.agents/PLANS.md` from the repository root. It is self-contained and describes a small `v0.7.0` implementation slice for enriching existing scanner-backed fundamentals field groups.

## Purpose / Big Picture

`tv fundamentals --group earnings` and `tv fundamentals --group dividends` already provide Desktop-free scanner-backed fundamentals reads. The recent field evidence note confirmed additional earnings and dividend-adjacent scanner fields. This slice makes those confirmed fields usable through the existing groups and through explicit `--field` / `scanner scan --columns` selection without adding a broader `tv events` command.

After this change, a user can run `tv fundamentals AAPL --group dividends` or `tv fundamentals NYSE:IONQ --group earnings` and receive the expanded scanner field bundle in the existing `fields`, `field_values`, and `missing_fields` payload shape.

## Progress

- [x] (2026-05-06T00:00Z) Created this ExecPlan and archived the completed observation workflow docs plan.
- [x] (2026-05-06T00:00Z) Expanded the existing earnings and dividends fundamentals groups with scanner-metainfo-backed fields.
- [x] (2026-05-06T00:00Z) Added the same fields to explicit fundamentals and scanner scan validation allowlists.
- [x] (2026-05-06T00:00Z) Updated focused tests and CLI contract tests.
- [x] (2026-05-06T00:00Z) Updated docs and skills with the enriched field boundary.
- [x] (2026-05-06T00:00Z) Ran validation.
- [x] (2026-05-06T00:00Z) Committed the slice.

## Surprises & Discoveries

- No new command surface was needed. The strongest evidence fits the existing `earnings` and `dividends` scanner field groups.

## Decision Log

- Decision: Add confirmed scanner fields to `earnings` and `dividends` instead of adding `tv events` or a new `events` group.
  Rationale: The evidence is field-bundle evidence, not a complete event calendar, news feed, or financial statement API.
  Date/Author: 2026-05-06 / Codex.

- Decision: Keep `tv fundamentals` default fields unchanged.
  Rationale: Existing default output should stay compact and compatible. Users who want the expanded event-like fields can ask for the group or explicit fields.
  Date/Author: 2026-05-06 / Codex.

## Outcomes & Retrospective

Expanded the existing Desktop-free fundamentals field bundles without changing the command surface, JSON envelope, or default field set. Additional fields remain raw TradingView scanner values and do not imply timezone, before/after-market, publication-code, or investment-significance interpretation.

## Context and Orientation

`tv fundamentals` is implemented in `crates/market/src/fundamentals/`. Field selection and group expansion live in `crates/market/src/fundamentals/fields.rs`. Scanner table reads live in `crates/scanner/src/scan.rs` and have their own supported column allowlist.

The field evidence note at `docs/notes/fundamentals-events-field-evidence-2026-05-06.md` confirmed additional scanner fields around earnings and dividends. This plan promotes those confirmed field names into the existing allowlists only.

## Plan of Work

Update `crates/market/src/fundamentals/fields.rs` so:

- `EARNINGS_FUNDAMENTAL_FIELDS` also includes `earnings_release_next_trading_date_fq`, `earnings_release_trading_date_fq`, `earnings_release_time`, and `earnings_publication_type_fq`;
- `DIVIDENDS_FUNDAMENTAL_FIELDS` also includes `dividend_amount_recent`, `dividend_amount_upcoming`, `dividend_frequency_recent`, `dividend_frequency_upcoming`, `next_dividend_date`, and `expected_annual_dividends`;
- `SUPPORTED_FUNDAMENTAL_FIELDS` includes the same new fields so explicit `--field` works.

Update `crates/scanner/src/scan.rs` so `scanner scan --columns ...` accepts the same new field names. Do not change default scanner columns.

Update tests to prove group expansion, deduplication, explicit field validation, and scanner column validation. Update docs and runtime skills only where needed to say that the enriched groups are scanner field bundles, not a full event calendar.

## Concrete Steps

Run the focused tests after editing:

    cargo test -p tradingview-market fundamental -- --nocapture
    cargo test -p tradingview-scanner scan -- --nocapture
    cargo test -p tradingview-cli --test cli_contract fundamentals -- --nocapture
    cargo test -p tradingview-cli --test cli_contract scanner -- --nocapture

Then run the full validation baseline:

    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1
    git diff --check

Read-only smoke, when a debug binary is available:

    target/debug/tv fundamentals NYSE:IONQ --group earnings
    target/debug/tv fundamentals AAPL --group dividends
    target/debug/tv fundamentals AAPL --field dividend_amount_recent --field next_dividend_date
    target/debug/tv scanner scan --limit 3 --columns name,earnings_release_next_trading_date_fq,dividend_amount_recent
    TV_CDP_PORT=9 target/debug/tv fundamentals NYSE:IONQ --group earnings

Run packaging and hygiene checks:

    bash -n scripts/stage-release-package-files.sh
    rg -n '(/Users/|C:\\|USER;|sessionid|cookie|authorization|bearer|raw live payload|account-local)' README.md CHANGELOG.md docs .agents/skills packaging scripts || true

## Validation and Acceptance

Acceptance is met when:

- `tv fundamentals <SYMBOL>` without group or field keeps the same default field set;
- `--group earnings` includes the four additional earnings fields;
- `--group dividends` includes the six additional dividend fields;
- explicit `--field` accepts the same added fields;
- `scanner scan --columns` accepts those fields;
- unsupported field and group validation remains unchanged;
- docs do not present these scanner fields as a complete event calendar.

## Idempotence and Recovery

The change is additive and safe to repeat. If a newly added field proves unsupported in validation, remove it from both the group and supported allowlists and record the reason in this plan before continuing. If docs start implying a richer event/calendar API, revise them back to scanner field-bundle wording.

## Interfaces and Dependencies

No new dependency is required. No new command, option, JSON envelope, or crate-level API name is added. Existing typed fundamentals and scanner results simply include additional field names when requested.

## Open Questions

None. `tv events`, news/calendar reads, raw financial statement APIs, authenticated reads, cookie/session import, and stable browserless bars remain deferred.
