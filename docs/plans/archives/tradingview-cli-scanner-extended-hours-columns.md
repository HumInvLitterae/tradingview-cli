# Add extended-hours columns to scanner scan

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This repository uses `.agents/PLANS.md` as the ExecPlan standard. This document follows that standard and is self-contained so a future contributor can resume the work without reading the chat history.

## Purpose / Big Picture

Users can already run `tv quote <SYMBOL>` without TradingView Desktop and see premarket and postmarket values when TradingView returns them. Users should also be able to run broader scanner reads such as `tv scanner scan --columns name,close,premarket_close,postmarket_close` without opening a chart. After this change, `tv scanner scan` accepts the same confirmed `premarket_*` and `postmarket_*` scanner REST column names through its existing `--columns` option.

The change is additive. The default scanner scan columns do not change, and scanner rows continue to expose requested fields through `symbols[].field_values`.

## Progress

- [x] (2026-04-29 17:57Z) Confirmed fiale-plus PR #47 is still open on branch `feat/lab-path` and remains an experimental WebSocket implementation, not a main-branch stable feature.
- [x] (2026-04-29 18:00Z) Confirmed `crates/scanner/src/scan.rs` owns scanner scan column validation and that scanner rows already map arbitrary requested columns into `field_values`.
- [x] (2026-04-29 18:03Z) Added confirmed extended-hours scanner columns to the `tv scanner scan` supported column allowlist.
- [x] (2026-04-29 18:03Z) Added a unit test proving explicit extended-hours columns are accepted while default scan columns stay unchanged.
- [x] (2026-04-29 18:08Z) Updated roadmap, README, changelog, internal API reference, plan index, and local continuity.
- [x] (2026-04-29 18:13Z) Focused scanner tests, full workspace validation, read-only smoke, and diff checks passed.
- [x] (2026-04-29 18:15Z) Tracked-doc hygiene grep reported only existing policy language, archived validation-command examples, and this plan's public-safe secret-safety wording.

## Surprises & Discoveries

- Observation: fiale-plus main has help and argument parsing for experimental bars and streaming, while the implementation body lives in open PR #47.
  Evidence: `gh pr view 47 -R fiale-plus/tradingview-mcp-server --json number,title,state,headRefName,updatedAt,url` returned an open PR titled `feat(lab): experimental WebSocket tools — bars, stream-quotes, stream-bars` on `feat/lab-path`.
- Observation: No response-shaping change is needed for scanner scan extended-hours fields.
  Evidence: `normalize_scan_symbol` already maps any requested column names to row values through `field_values_object(columns, values)`.

## Decision Log

- Decision: Add extended-hours columns only to the `--columns` allowlist and do not change `DEFAULT_SCAN_COLUMNS`.
  Rationale: Existing `tv scanner scan` callers should not see wider payloads unless they ask for them. Extended-hours fields are useful for targeted scans but can be `null` outside active sessions.
  Date/Author: 2026-04-29 / Codex
- Decision: Keep scanner scan extended-hours fields in `symbols[].field_values` rather than creating a nested `extended_hours` object.
  Rationale: `scanner scan` is a table-style reader for arbitrary columns. Reshaping only some columns into nested objects would make scanner output less predictable and less consistent with the existing `--columns` contract.
  Date/Author: 2026-04-29 / Codex
- Decision: Treat fiale-plus PR #47 as a later research input, not part of this implementation.
  Rationale: The PR uses an undocumented TradingView WebSocket protocol and an experimental environment gate. It is promising for Desktop-free bars and streaming, but it should be evaluated after the stable scanner REST roadmap items.
  Date/Author: 2026-04-29 / Codex

## Outcomes & Retrospective

Implementation is complete. `tv scanner scan --columns name,close,premarket_close,premarket_volume,postmarket_close,postmarket_volume --limit 3` succeeded through scanner REST without TradingView Desktop, and the output kept extended-hours values under each row's `field_values`. `tv scanner scan --columns name,banana` failed with a validation error and listed the supported fields before any CDP connection was involved.

Validation run:

    cargo test -p tradingview-scanner scan -- --nocapture
    cargo test -p tradingview-cli --test cli_contract scanner -- --nocapture
    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1
    git diff --check

Read-only smoke succeeded for default scanner scan, explicit extended-hours columns, and invalid-column validation.

## Context and Orientation

The `tradingview-scanner` crate owns Desktop-free scanner reads. `crates/scanner/src/scan.rs` validates requested scanner fields against `SUPPORTED_SCAN_COLUMNS`, builds a request for `https://scanner.tradingview.com/america/scan`, and normalizes compact scanner rows into JSON. Scanner rows contain a full symbol under `s` and a compact value array under `d`; `normalize_scan_symbol` maps the requested column list onto `d` and exposes the result as `field_values`.

The earlier extended-hours quote change proved that TradingView scanner REST accepts compact column names such as `premarket_close` and `postmarket_volume`. This plan extends scanner scan to accept those column names directly. It does not add a WebSocket client, streaming mode, or new command.

## Plan of Work

Edit `crates/scanner/src/scan.rs`. Add the confirmed extended-hours scanner columns to `SUPPORTED_SCAN_COLUMNS`: `premarket_open`, `premarket_high`, `premarket_low`, `premarket_close`, `premarket_change`, `premarket_change_abs`, `premarket_gap`, `premarket_volume`, `postmarket_open`, `postmarket_high`, `postmarket_low`, `postmarket_close`, `postmarket_change`, `postmarket_change_abs`, and `postmarket_volume`.

Add a unit test near the existing scan request normalization tests. The test should build a `ScannerScanRequest` with explicit extended-hours columns and `sort: Some("premarket_volume")`, assert the normalized columns and request body include those fields, and then assert a separate default request still uses `DEFAULT_SCAN_COLUMNS`.

Update `docs/v0.4-roadmap.md` to record the fiale-plus PR #47 finding and keep WebSocket bars and streaming as a later research lane. Update README scanner examples to show explicit extended-hours columns. Update `docs/internal-tradingview-apis.md` to say scanner scan can request those columns through `--columns` while scanner quote reshapes them into `extended_hours`. Update `CHANGELOG.md` under `Unreleased`. Update `docs/plans/README.md` so this plan appears as current.

## Concrete Steps

Run commands from the repository root.

First run focused scanner validation:

    cargo test -p tradingview-scanner scan -- --nocapture
    cargo test -p tradingview-cli --test cli_contract scanner -- --nocapture

Then run the full baseline:

    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1
    git diff --check

Finally run read-only smoke when network access is available:

    target/debug/tv scanner scan --limit 3
    target/debug/tv scanner scan --limit 3 --columns name,close,premarket_close,premarket_volume,postmarket_close,postmarket_volume
    target/debug/tv scanner scan --limit 3 --columns name,banana

The first two commands should succeed through scanner REST without TradingView Desktop or CDP. The invalid column command should fail with a validation error before network access.

## Validation and Acceptance

Acceptance is met when explicit scanner scan extended-hours columns are accepted, default scanner scan output remains unchanged, invalid columns still fail validation, focused and workspace tests pass, and read-only smoke proves the new column set works without TradingView Desktop.

## Idempotence and Recovery

This is an additive read-only change. Tests and smoke commands are safe to rerun. If TradingView removes or rejects a column, remove that specific column from the allowlist, record the observation here, and keep the rest of the accepted columns.

## Artifacts and Notes

Do not paste raw live scanner responses, cookies, tokens, account-local identifiers, or local absolute paths into tracked docs. It is safe to record public column names and high-level PR observations.

## Interfaces and Dependencies

No new CLI flag or command is added. The existing interface is `tv scanner scan --columns <CSV>`. The public scanner output remains under `data.symbols[].field_values`, where each key is one requested column name.

## Open Questions

None.
