# Add scanner metainfo read

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This repository uses `.agents/PLANS.md` as the ExecPlan standard. This document follows that standard and is self-contained so a future contributor can resume the work without reading the chat history.

## Purpose / Big Picture

Users can run `tv scanner scan --columns ...` with known field names, but they currently need documentation or prior knowledge to discover field metadata. After this change, users can run `tv scanner metainfo --market america --field close --field premarket_close` without TradingView Desktop and receive normalized scanner field metadata from TradingView's scanner metainfo endpoint.

This is a metadata read, not a price read. It does not guarantee quote freshness or realtime entitlement. The repository should document that credential-free scanner price reads such as `tv quote <SYMBOL>` and `tv scanner scan` may depend on TradingView's public scanner feed, exchange rules, and subscription state.

## Progress

- [x] (2026-04-29 18:15Z) Confirmed the working tree was clean before implementation.
- [x] (2026-04-29 18:15Z) Confirmed read-only `POST https://scanner.tradingview.com/america/metainfo` returns `200 OK` and a `fields` array with compact field metadata.
- [x] (2026-04-29 18:18Z) Added `tradingview-scanner` metainfo request, response normalization, and unit tests.
- [x] (2026-04-29 18:20Z) Added `tv scanner metainfo [--market america] [--field <FIELD>]...` and CLI contract help coverage.
- [x] (2026-04-29 18:25Z) Updated README, changelog, internal API reference, roadmap, plan index, and local continuity with scanner metainfo and Desktop-free price freshness boundaries.
- [x] (2026-04-29 18:35Z) Ran focused tests, full workspace validation, read-only smoke, and hygiene checks.

## Surprises & Discoveries

- Observation: The metainfo endpoint returns compact scanner metadata without authentication.
  Evidence: A read-only probe to `https://scanner.tradingview.com/america/metainfo` returned status `200 OK` with top-level keys including `financial_currency` and `fields`; field entries used compact names such as `n`, `t`, and `r`.
- Observation: The metainfo endpoint does not necessarily include every `scanner scan` column-like value.
  Evidence: `tv scanner metainfo --market america --field name --field close --field premarket_close` succeeded without CDP and returned `close` and `premarket_close`, while `name` was reported in `missing_fields`.

## Decision Log

- Decision: Add a normalized `scanner metainfo` command and omit `--raw`.
  Rationale: Raw metainfo payloads may vary and can be large. A normalized shape is safer for public CLI users and keeps tracked docs public-safe.
  Date/Author: 2026-04-29 / Codex
- Decision: Keep market support to `america` in the first implementation.
  Rationale: Existing `scanner scan` already uses only `america`; adding other markets should be an explicit later compatibility slice.
  Date/Author: 2026-04-29 / Codex
- Decision: Do not use metainfo dynamically to validate `scanner scan --columns` in this slice.
  Rationale: Dynamic validation would add network dependence to a currently local validation path and could change error timing. This slice is field discovery only.
  Date/Author: 2026-04-29 / Codex

## Outcomes & Retrospective

Implemented. `tv scanner metainfo` reads normalized scanner field metadata without TradingView Desktop, reports unknown requested fields in `missing_fields`, and rejects unsupported markets before network access. The implementation intentionally keeps `scanner scan --columns` validation static for now; metainfo is a discovery command, not a dynamic validator.

Read-only smoke confirmed `close`, `premarket_close`, and `postmarket_close` field metadata can be discovered. A request for `banana` succeeded with `banana` under `missing_fields`, and `--market global` returned a validation error before network access. A request including `name` showed that this endpoint can omit scanner identity columns that are still valid in `scanner scan`, so callers should treat `missing_fields` as endpoint metadata absence rather than proof that a scan column can never be requested.

## Context and Orientation

The `tradingview-scanner` crate owns Desktop-free scanner reads. `crates/scanner/src/scan.rs` implements `tv scanner scan`, while `crates/scanner/src/hotlist.rs` implements `tv scanner hotlist`. The CLI package re-exports scanner functions through `crates/cli/src/ops/scanner.rs` and dispatches `ScannerCommand` variants from `crates/cli/src/app/dispatch.rs`.

The scanner metainfo endpoint is an unauthenticated HTTP read at `https://scanner.tradingview.com/{market}/metainfo`. It returns field metadata, not price rows. The response may contain an array or object of fields. Compact array entries commonly use `n` for field name, `t` for type, and `r` for range.

## Plan of Work

Add `crates/scanner/src/metainfo.rs` with `ScannerMetainfoRequest` and `scanner_metainfo`. Validate `market` locally and allow only `america`. Normalize repeated `--field` values by trimming and de-duplicating them; reject blank field values before network access.

In `scanner_metainfo`, POST to the metainfo URL. If fields were requested, send `{ "fields": [...] }`; otherwise send an empty POST request. Normalize the response into `source`, `market`, `requested_fields`, `field_count`, `fields`, `missing_fields`, and optional `financial_currency`. Each field object should include `name`, `type`, and optional safe metadata such as `label` and `range`.

Expose the function from `crates/scanner/src/lib.rs`, re-export it from `crates/cli/src/ops/scanner.rs`, add a `Metainfo` variant under `ScannerCommand`, and dispatch it without connecting to CDP.

Update docs to explain both the new metainfo command and the freshness boundary for scanner REST price reads.

## Concrete Steps

Run commands from the repository root.

Focused validation:

    cargo test -p tradingview-scanner metainfo -- --nocapture
    cargo test -p tradingview-cli --test cli_contract scanner -- --nocapture

Full validation:

    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1
    git diff --check

Read-only smoke:

    target/debug/tv scanner metainfo --market america --field close --field premarket_close --field postmarket_close
    target/debug/tv scanner metainfo --market america --field banana
    target/debug/tv scanner metainfo --market global

The first command should succeed without TradingView Desktop. The second should succeed with `banana` under `missing_fields`. The third should fail with a validation error before network access.

## Validation and Acceptance

Acceptance is met when `tv scanner metainfo` can discover scanner fields without CDP, unknown requested fields are visible in `missing_fields`, unsupported markets fail before network access, existing scanner hotlist/scan behavior remains unchanged, and docs explain that credential-free scanner price reads are not realtime-entitlement guarantees.

## Idempotence and Recovery

This is a read-only additive feature. Tests and smoke commands can be rerun safely. If TradingView changes the metainfo response shape, update the normalizer and record the observed high-level shape here without storing raw live payloads.

## Artifacts and Notes

Do not paste raw metainfo responses, cookies, tokens, account-local identifiers, or local absolute paths into tracked docs. It is safe to record endpoint category, field-name examples, and high-level response shape.

## Interfaces and Dependencies

The new public CLI interface is `tv scanner metainfo [--market <MARKET>] [--field <FIELD>]...`. No existing command payload is changed. `scanner scan` keeps its local supported-column allowlist.

## Open Questions

None.
