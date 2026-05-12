# `tv quote --source quote-data` regular-session semantics plan

This ExecPlan is a living document. Keep `Progress`, `Surprises &
Discoveries`, `Decision Log`, and `Outcomes & Retrospective` updated as work
continues.

## Purpose

Clarify how `tv quote <SYMBOL> --source quote-data` should be interpreted
during regular session, especially when the current `quote_data.v1` success
condition does not see a matching non-null `qsd.rtc` field inside the bounded
wait.

This slice does not implement a new quote field or source. It records the
`v0.16.0` roadmap direction and prepares the first evidence slice so a later
implementation can decide whether to keep quote-data `rtc`-centered, add an
additive regular-session readback, or strengthen diagnostics only.

## Background

`tv quote <SYMBOL> --source quote-data` is an explicit Desktop-backed
WebSocket quote-data readback. It is separate from scanner REST, chart-source
main-series quote, and `--source auto`.

The current implementation reports success only when a bounded WebSocket
observation sees a TradingView quote-data `qsd` message for the requested
symbol with a non-null `rtc`. That was added because after-hours evidence
showed `qsd.rtc` matching the visible after-hours panel more closely than
chart main-series quote or scanner delayed REST.

During regular session, however, TradingView may expose regular quote-like
fields such as `lp` or `regular_close` while no matching `qsd.rtc` arrives in
the bounded window. That should not be described as a confirmed Desktop API
limitation. It is a field-semantics and availability boundary of the current
contract.

## Progress

- [x] Added `docs/v0.16-roadmap.md` with the quote-data regular-session
  semantics theme.
- [x] Created this ExecPlan as the first `v0.16.0` implementation candidate.
- [x] Archived the completed `v0.15.0` release readiness plan.
- [x] Updated plan index, `v0.15` roadmap, changelog, docs, and runtime skills
  to point at the `v0.16.0` direction.
- [ ] Add or extend an opt-in ignored smoke that summarizes regular-session
  matching-symbol `qsd` fields without raw frames.
- [ ] Run the smoke during regular session and record only public-safe
  summary.
- [ ] Decide whether a later implementation should add regular-session
  quote-data readback or keep the `rtc` success condition unchanged.

## Surprises & Discoveries

- The current quote-data unavailable result is not proof that the Desktop API
  cannot provide any regular-session price information. It means the bounded
  quote-data read did not see the success field required by the current
  contract: matching non-null `qsd.rtc`.
- Prior after-hours evidence made `qsd.rtc` the strongest current candidate
  for the visible after-hours panel value, but that does not automatically
  make `rtc` the regular-session price field.

## Decision Log

- Keep `tv quote <SYMBOL> --source quote-data` as an explicit Desktop-backed
  source. Do not add it to `--source auto`.
- Do not mix scanner `extended_hours`, chart main-series OHLCV, and quote-data
  fields into one synthetic quote.
- Do not describe regular-session quote-data unavailable as "symbol has no
  price" or as a confirmed API-wide limitation.
- Treat `lp`, `regular_close`, `rtc`, `rch`, `rchp`, `current_session`,
  `market_phase`, and `update_mode` as evidence fields to compare before
  changing the public payload.
- Keep raw WebSocket frames, raw live payloads, target ids, account-local
  metadata, credentials, and local validation details out of tracked docs.

## Plan Of Work

1. Extend or add an opt-in ignored smoke for regular-session quote-data field
   evidence.
   - Require an explicit environment flag.
   - Respect existing target selection behavior, including test-only target
     selection where applicable.
   - Keep the normal workspace test run compile-only.

2. Produce a compact public-safe summary for matching-symbol `qsd` frames.
   - Count WebSocket events, WebSocket frames, quote-data messages,
     matching-symbol messages, matching messages with `rtc`, and matching
     messages without `rtc`.
   - Summarize presence or selected values for `lp`, `regular_close`, `rtc`,
     `rch`, `rchp`, `current_session`, `market_phase`, and `update_mode`.
   - Do not output raw frames, raw payloads, raw DOM, target ids, or
     account-local identifiers.

3. Compare source semantics without using source mixing.
   - Scanner REST can be used as a separate freshness reference.
   - Chart-source quote can be used as selected chart main-series context.
   - Quote-data remains its own source; the smoke should not synthesize a
     single price.

4. Record the evidence outcome.
   - If matching-symbol regular-session `qsd` fields expose a stable regular
     readback, create a follow-up plan for additive payload support.
   - If fields are unstable or unavailable, strengthen docs and diagnostics
     without adding public price fields.
   - If the smoke cannot observe enough data, improve the evidence tooling
     before changing public contract semantics.

## Validation

Docs-only planning validation:

- `git diff --check`
- `bash -n scripts/stage-release-package-files.sh`
- `rg -n "v0\\.16|quote-data|qsd\\.rtc|regular session|source_availability|unavailable_reason|lp|regular_close|extended_hours|auto fallback|realtime|binary split|MCP|daemon" README.md CHANGELOG.md docs .agents/skills packaging/agent/AGENTS.md`
- `rg -n '(/Users/|C:\\|USER;|sessionid|cookie|authorization|bearer|raw live payload|raw WebSocket|account-local|target id|downstream-private)' README.md AGENTS.md CLAUDE.md CHANGELOG.md docs .agents/skills packaging scripts crates || true`

If Rust test helpers are added later, also run:

- `cargo test -p tradingview-cli --test live_quote_data_source`
- `cargo fmt --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace`
- `cargo metadata --no-deps --format-version 1`

## Acceptance Criteria

- `v0.16.0` has a durable roadmap focused on quote-data regular-session
  semantics and source availability clarity.
- The next implementation slice has clear evidence goals for regular-session
  `qsd` fields.
- Docs and runtime skills explain that regular-session quote-data unavailable
  is a current `qsd.rtc` availability condition, not proof of price absence or
  an API-wide impossibility.
- No new public command, option, payload field, dependency, or version bump is
  added in this planning slice.

## Outcomes & Retrospective

Planning completed with the regular-session evidence question separated from
quote-data public payload support. The next contributor can implement the
ignored smoke or choose a docs-only clarification follow-up without reopening
the scanner/chart/quote-data source boundary.
