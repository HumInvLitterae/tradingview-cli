# v0.4 market data lane review

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This repository uses `.agents/PLANS.md` as the ExecPlan standard. This document follows that standard and is self-contained so a future contributor can resume the work without reading the chat history.

## Purpose / Big Picture

The `v0.4.0` roadmap has several market-data ideas: Desktop-free quotes, scanner reads, field metadata, batch quotes, and possible Desktop-free historical bars. This plan clarifies which parts are already complete, which parts are intentionally still research-only, and what the next implementation candidate should be.

After this change, a contributor can read `docs/v0.4-roadmap.md` and know that scanner REST watchlist-style reads are sufficient for current known needs, while Desktop-free historical bars are not canceled but remain an undocumented WebSocket research candidate. No new CLI command is added in this slice.

## Progress

- [x] (2026-04-30T00:00:00Z) Confirmed the working tree was clean before implementation.
- [x] (2026-04-30T00:00:00Z) Archived the completed market/scanner typed API docs plan.
- [x] (2026-04-30T00:00:00Z) Reviewed the current roadmap, internal API reference, operation adapter boundary reference, and plan index.
- [x] (2026-04-30T00:00:00Z) Updated durable docs to mark scanner REST watchlist-style reads as sufficient for current known needs and Desktop-free bars as a research candidate.
- [x] (2026-04-30T00:00:00Z) Ran read-only scanner/quote smoke commands to confirm the existing Desktop-free market read lane still works.
- [x] (2026-04-30T00:00:00Z) Ran docs validation, hygiene grep, and status checks.

## Surprises & Discoveries

- Observation: The main facts were already present in separate docs, but `docs/v0.4-roadmap.md` still presented another small scanner REST polish and a lab-gated bars prototype as peers without a clear recommendation.
  Evidence: The roadmap already described scanner scan, scanner metainfo, batch quotes, and WebSocket bars research, while the "Likely next slices" section still listed both scanner polish and bars prototype as open candidates.

- Observation: The current scanner REST lane works without TradingView Desktop for the representative read-only checks in this plan.
  Evidence: `target/debug/tv scanner metainfo --market america --field close --field premarket_close --field postmarket_close`, `target/debug/tv scanner scan --limit 3 --columns name,close,premarket_close,postmarket_close`, and `target/debug/tv quotes AAPL MSFT NYSE:IONQ` returned successful JSON envelopes.

## Decision Log

- Decision: Treat scanner REST watchlist-style reads as "sufficient for current known needs" rather than continuing to search for small scanner additions by default.
  Rationale: `scanner scan`, explicit extended-hours columns, `scanner metainfo`, batch quotes, and typed API docs cover the practical read-only workflow currently identified. Further scanner work should be driven by a concrete operator need and endpoint evidence.
  Date/Author: 2026-04-30 / Codex

- Decision: Keep Desktop-free historical bars as `research_candidate`, not `safe now`, and do not change `tv ohlcv`.
  Rationale: The only comparable evidence is an undocumented TradingView WebSocket chart-session protocol. It may be useful, but it needs lab gating, bounded requests, freshness wording, and structured protocol errors before becoming a CLI feature.
  Date/Author: 2026-04-30 / Codex

- Decision: If the next market-data implementation proceeds, prefer a separate lab-gated `tv bars <SYMBOL> --timeframe <TIMEFRAME> --count <N>` prototype over extending `tv ohlcv`.
  Rationale: `tv ohlcv` currently means selected-chart bars through the Desktop chart API. A browserless symbol-targeted bars command would have different source, freshness, entitlement, and protocol risks.
  Date/Author: 2026-04-30 / Codex

## Outcomes & Retrospective

Completed. The docs now answer the user's two status questions directly: Desktop-free historical bars feasibility is done but stable implementation is deferred as a research candidate, while scanner REST watchlist-style reads are mostly complete for the known practical lane. The recommended next implementation candidate, if market data expansion continues, is a separate lab-gated bars prototype plan rather than more generic scanner REST polishing.

## Context and Orientation

This repository is a Rust-native TradingView CLI workspace. The `tv` binary lives in the `tradingview-cli` package under `crates/cli/`. Desktop-free market reads live in separate internal crates:

- `crates/market/` provides symbol search, symbol metadata, single-symbol quotes, and batch quotes through TradingView scanner or search endpoints.
- `crates/scanner/` provides scanner hotlists, table scans, and field metadata reads through TradingView scanner endpoints.

"Desktop-free" means the command does not connect to TradingView Desktop through Chrome DevTools Protocol. "Scanner REST" means TradingView's scanner HTTP endpoints. These reads are useful for screening, but price freshness can depend on exchange rules, feed selection, and TradingView account entitlements.

Historical bars are different from quotes. The existing `tv ohlcv` command reads the currently selected chart's main-series bars through a live TradingView Desktop chart target. Browserless historical bars would require a different source. Current comparable evidence points to an undocumented WebSocket protocol, so it should not silently replace `tv ohlcv`.

## Plan of Work

First, create this plan under `docs/plans/` and archive the completed `docs/plans/tradingview-cli-market-scanner-typed-api-docs.md` plan under `docs/plans/archives/`.

Second, update `docs/v0.4-roadmap.md` so the market data lane has a clear current disposition. The document should say that scanner REST watchlist-style reads are sufficient for current known needs and that further scanner additions require concrete operator demand and endpoint evidence. It should also say that Desktop-free historical bars are feasibility-complete, not implemented, and still a research candidate.

Third, update `docs/internal-tradingview-apis.md` and `docs/operation-adapter-boundaries.md` only where needed so they match the same boundary. These docs should keep raw protocol details out of tracked files and should not include live payloads.

Fourth, update `docs/plans/README.md` and `CHANGELOG.md` so the active plan index and changelog reflect this docs/research slice. Update `CONTINUITY.md` as the local ledger, but do not include it in the commit.

Finally, run read-only market data checks and docs validation. This slice should not change code or add CLI behavior.

## Concrete Steps

Run commands from the repository root.

Archive the completed plan and create this plan:

    mv docs/plans/tradingview-cli-market-scanner-typed-api-docs.md docs/plans/archives/tradingview-cli-market-scanner-typed-api-docs.md

Read-only confirmation:

    target/debug/tv scanner metainfo --market america --field close --field premarket_close --field postmarket_close
    target/debug/tv scanner scan --limit 3 --columns name,close,premarket_close,postmarket_close
    target/debug/tv quotes AAPL MSFT NYSE:IONQ

Validation:

    git diff --check
    rg -n '(/Users/|C:\\|USER;|sessionid|cookie|authorization|bearer)' README.md CHANGELOG.md docs .agents/skills packaging scripts || true
    git status --short

Completed validation:

    target/debug/tv scanner metainfo --market america --field close --field premarket_close --field postmarket_close
    target/debug/tv scanner scan --limit 3 --columns name,close,premarket_close,postmarket_close
    target/debug/tv quotes AAPL MSFT NYSE:IONQ
    git diff --check
    rg -n '(/Users/|C:\\|USER;|sessionid|cookie|authorization|bearer)' README.md CHANGELOG.md docs .agents/skills packaging scripts || true
    git status --short

## Validation and Acceptance

Acceptance is met when the roadmap clearly answers these questions:

- Desktop-free historical bars / OHLCV are not implemented as a stable feature. They remain a research candidate based on undocumented WebSocket evidence.
- `tv ohlcv` remains a selected-chart Desktop/CDP command.
- Scanner REST watchlist-style reads have reached a practical completion point for current needs through `scanner scan`, extended-hours columns, `scanner metainfo`, batch quotes, and typed API docs.
- The next implementation-oriented market data candidate is a separate lab-gated `tv bars` prototype only if the project accepts that experimental WebSocket boundary.

No CLI command, JSON payload, or Rust API should change in this slice.

## Idempotence and Recovery

This is a docs-only plan and is safe to retry. If the completed plan file has already been archived, leave it in `docs/plans/archives/` and continue. If read-only smoke fails because network access or TradingView's scanner endpoint is unavailable, record the failure in this plan and keep the docs distinction intact; do not add fallback code in this slice.

## Artifacts and Notes

Do not write raw scanner responses, WebSocket frames, cookies, authorization headers, account identifiers, chart target ids, or local absolute filesystem paths into tracked docs. It is safe to mention public command names, public example symbols, and high-level endpoint categories.

## Interfaces and Dependencies

No Rust interfaces change in this plan. The existing commands used as evidence are:

    tv scanner metainfo --market america --field close
    tv scanner scan --limit 3 --columns name,close,premarket_close,postmarket_close
    tv quotes AAPL MSFT NYSE:IONQ
    tv ohlcv --count 1

The future candidate is not implemented here, but if later accepted it should be planned as a separate command such as:

    tv bars <SYMBOL> --timeframe <TIMEFRAME> --count <N>

That future command should be lab-gated, bounded by count or duration, and explicit about freshness and entitlement limitations.

## Open Questions

- UNCONFIRMED: Whether the project should actually implement a lab-gated browserless bars prototype next. This plan only identifies it as the next coherent market-data expansion candidate.
