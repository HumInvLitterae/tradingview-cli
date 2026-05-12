# `tv quote --source quote-data` regular-session semantics plan

This ExecPlan is a living document. Keep `Progress`, `Surprises &
Discoveries`, `Decision Log`, and `Outcomes & Retrospective` updated as work
continues.

## Purpose

Clarify how `tv quote <SYMBOL> --source quote-data` should be interpreted
during regular session, especially when the current `quote_data.v1` success
condition does not see a matching non-null `qsd.rtc` field inside the bounded
wait.

This slice adds additive regular-session readback to the existing
`quote_data.v1` payload so matching quote-data messages with usable regular
`lp` are no longer reported as unavailable only because `rtc` is absent.

## Background

`tv quote <SYMBOL> --source quote-data` is an explicit Desktop-backed
WebSocket quote-data readback. It is separate from scanner REST, chart-source
main-series quote, and `--source auto`.

The current implementation reports success only when a bounded WebSocket
observation sees a TradingView quote-data `qsd` message for the requested
symbol with a non-null `rtc`. That was added because after-hours evidence
showed `qsd.rtc` matching the visible after-hours panel more closely than
chart main-series quote or scanner delayed REST.

During regular session, however, TradingView can expose regular quote-like
fields such as `lp` or `regular_close` while no matching `qsd.rtc` arrives in
the bounded window. Reporting that as unavailable made the explicit
`quote-data` interface hard to use, because the same source could have
matching symbol evidence but still fail only because the after-hours-oriented
field was absent.

## Progress

- [x] Added `docs/v0.16-roadmap.md` with the quote-data regular-session
  semantics theme.
- [x] Created this ExecPlan as the first `v0.16.0` implementation candidate.
- [x] Archived the completed `v0.15.0` release readiness plan.
- [x] Updated plan index, `v0.15` roadmap, changelog, docs, and runtime skills
  to point at the `v0.16.0` direction.
- [x] Added `quote_data.price_readback` so success payloads distinguish
  `rtc` readback from regular `lp` readback.
- [x] Made matching-symbol non-null `lp` a success condition when `rtc` is
  absent.
- [x] Added public-safe wait-summary counters for matching-symbol `lp`,
  `regular_close`, and price-readback observations.
- [ ] Run focused quote-data, diagnose, and CLI contract tests.
- [ ] Run full release-slice validation.

## Surprises & Discoveries

- The current quote-data unavailable result is not proof that the Desktop API
  cannot provide any regular-session price information. It means the bounded
  quote-data read did not see the success field required by the current
  contract: matching non-null `qsd.rtc`.
- Prior after-hours evidence made `qsd.rtc` the strongest current candidate
  for the visible after-hours panel value, but that does not automatically
  make `rtc` the regular-session price field.
- The least confusing compatibility path is to keep `rtc` first, but allow
  matching-symbol `lp` to succeed as a separately labeled `regular_last`
  readback.

## Decision Log

- Keep `tv quote <SYMBOL> --source quote-data` as an explicit Desktop-backed
  source. Do not add it to `--source auto`.
- Do not mix scanner `extended_hours`, chart main-series OHLCV, and quote-data
  fields into one synthetic quote.
- Do not describe regular-session quote-data unavailable as "symbol has no
  price" or as a confirmed API-wide limitation.
- Treat `qsd.v.rtc` and `qsd.v.lp` as two distinct quote-data readbacks.
  `rtc` keeps priority when present. `lp` is returned as
  `price_readback.kind: "regular_last"` when `rtc` is absent.
- Return `regular_close` as additive context when TradingView provides it, but
  do not let `regular_close` alone make a quote-data read successful.
- Keep raw WebSocket frames, raw live payloads, target ids, account-local
  metadata, credentials, and local validation details out of tracked docs.

## Plan Of Work

1. Extend the quote-data observer.
   - Continue parsing only public-safe selected fields from bounded `qsd`
     messages.
   - Prefer matching non-null `rtc`.
   - If `rtc` is absent but matching non-null `lp` is present, return success
     with `price_readback.kind: "regular_last"`.
   - Count matching-symbol `lp`, `regular_close`, and any usable price
     readback in `wait_summary`.

2. Extend the payload additively.
   - Keep all existing `quote_data.v1` fields.
   - Add `quote_data.price_readback`.
   - Add `quote_data.lp`, `quote_data.regular_close`, `quote_data.lp_time`,
     and `quote_data.rt_update_time` as source-labeled readback fields.
   - Add `source_availability.price_readback_observed`.

3. Keep unavailable narrowly scoped.
   - Use `no_rtc` only when matching `qsd` exists but neither `rtc` nor usable
     `lp` is observed.
   - Keep unavailable as structured failure.
   - Do not output raw frames, raw payloads, raw DOM, target ids, or
     account-local identifiers.

4. Sync docs, runtime skills, and help.
   - Explain that quote-data can return either `rtc` or regular `lp` readback.
   - Keep scanner, chart, and quote-data separated.
   - Keep `--source auto` unchanged.

## Validation

- `git diff --check`
- `bash -n scripts/stage-release-package-files.sh`
- `rg -n "v0\\.16|quote-data|qsd\\.rtc|regular session|source_availability|unavailable_reason|lp|regular_close|extended_hours|auto fallback|realtime|binary split|MCP|daemon" README.md CHANGELOG.md docs .agents/skills packaging/agent/AGENTS.md`
- `rg -n '(/Users/|C:\\|USER;|sessionid|cookie|authorization|bearer|raw live payload|raw WebSocket|account-local|target id|downstream-private)' README.md AGENTS.md CLAUDE.md CHANGELOG.md docs .agents/skills packaging scripts crates || true`
- `cargo test -p tradingview-cli market::quote_data -- --nocapture`
- `cargo test -p tradingview-cli --test cli_contract quote -- --nocapture`
- `cargo test -p tradingview-cli --test cli_contract diagnose -- --nocapture`
- `cargo test -p tradingview-cli --test live_quote_data_source`
- `cargo fmt --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace`
- `cargo metadata --no-deps --format-version 1`

## Acceptance Criteria

- `quote-data` success payloads include `quote_data.price_readback`.
- Matching non-null `qsd.v.rtc` still returns `price_readback.kind: "rtc"`.
- Matching non-null `qsd.v.lp` without `rtc` returns
  `price_readback.kind: "regular_last"` instead of unavailable.
- Docs and runtime skills explain that `rtc` and regular `lp` are distinct
  source-labeled quote-data readbacks.
- No new public command, option, dependency, source, automatic fallback, or
  version bump is added in this slice.

## Outcomes & Retrospective

Implementation adds regular-session `lp` readback without changing the
scanner/chart/quote-data source boundary. `rtc` remains the preferred
after-hours/premarket-style candidate, while regular `lp` is explicitly
labeled as `regular_last`.
