# Add practical scanner scan filters

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document follows `.agents/PLANS.md`.

## Purpose / Big Picture

After this change, `tv scanner scan` can express a more useful read-only stock screen without falling back to UI Screener automation or arbitrary JSON. A user can filter by stock type, sector, industry, relative volume, price change, and valuation while still using the existing REST scanner endpoint and Rust JSON envelope.

This slice is read-only. It does not open TradingView Desktop, connect to CDP, save Screener screens, remove filters, change columns, or add workflow scanner packs.

## Progress

- [x] (2026-04-25 17:31Z) Read current scanner scan implementation, CLI dispatch, docs, and module split state.
- [x] (2026-04-25 17:44Z) Added CLI flags and request fields for practical scanner filters.
- [x] (2026-04-25 17:45Z) Extended scanner request builder, validation, unit tests, and CLI contract tests.
- [x] (2026-04-25 17:46Z) Updated README, changelog, and contract notes.
- [x] (2026-04-25 17:55Z) Ran focused tests, REST smoke, and full validation baseline.
- [x] (2026-04-25 17:56Z) Recorded outcomes and prepared the completed slice for commit.

## Surprises & Discoveries

- Observation: The current scanner REST endpoint accepts sector, type/subtype, relative volume, change, and price-to-earnings filters.
  Evidence: Read-only curl probes against `https://scanner.tradingview.com/america/scan` returned success payloads for those fields.

## Decision Log

- Decision: Add explicit CLI flags rather than arbitrary filter JSON.
  Rationale: The current Rust CLI should keep the scanner surface safe and easy to validate; arbitrary JSON would expose undocumented endpoint details directly to callers.
  Date/Author: 2026-04-25 / Codex.

- Decision: Keep `america` as the only supported market in this slice.
  Rationale: Market expansion requires separate endpoint and field validation. This change focuses only on making the existing market scan more useful.
  Date/Author: 2026-04-25 / Codex.

## Outcomes & Retrospective

Completed. `tv scanner scan` now supports repeatable string filters for `--sector`, `--industry`, `--type`, and `--subtype`, plus numeric bounds for `--min-change`, `--max-change`, `--min-relative-volume`, and `--max-pe`. The command remains read-only and keeps the existing `scanner_scan_rest` payload envelope under `data`.

Focused tests covered filter JSON construction, blank string validation before network access, and CLI help visibility. REST smoke returned successful `scanner_scan_rest` payloads for a technology relative-volume scan and a positive-change price-to-earnings scan. Full validation passed with `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`, and `git diff --check`. The repository grep for local absolute paths and `USER;` produced only existing validation-command examples in plan documents.

## Context and Orientation

The scanner CLI surface is declared in `src/cli.rs` as `ScannerCommand::Scan`. `src/main.rs` converts the CLI fields into `ops::ScannerScanRequest`, then calls `ops::scanner_scan`. The scanner operations are split under `src/ops/scanner/`; `src/ops/scanner.rs` is a facade, `src/ops/scanner/hotlist.rs` owns Hotlist preset reads, and `src/ops/scanner/scan.rs` owns generic scanner REST reads.

The scanner scan command currently supports exchanges, columns, sorting, limit, price bounds, volume minimum, and market-cap minimum. It sends JSON to TradingView's undocumented REST scanner endpoint and returns normalized results under `data`.

## Plan of Work

Add CLI fields to `ScannerCommand::Scan` for `--sector`, `--industry`, `--type`, `--subtype`, `--min-change`, `--max-change`, `--min-relative-volume`, and `--max-pe`. Repeatable string filters should allow multiple values and use the endpoint's `in_range` operation. Numeric filters should continue to use `greater` or `less` operations.

Add matching fields to `ScannerScanRequest` and map them in `src/main.rs`. In `src/ops/scanner/scan.rs`, add `type` and `subtype` to `SUPPORTED_SCAN_COLUMNS`; the other planned fields are already supported. Reject blank string filter values, reject non-finite numeric values, and keep the existing non-negative numeric policy.

Update docs to describe the additional filters. Do not change the envelope shape, `source: "scanner_scan_rest"`, default columns, default sort, or max limit.

## Concrete Steps

Run commands from the repository root.

1. Edit `src/cli.rs`, `src/main.rs`, and `src/ops/scanner/scan.rs`.
2. Add tests in `src/ops/scanner/scan.rs` and `tests/cli_contract.rs`.
3. Update `README.md`, `CHANGELOG.md`, and `docs/notes/rust-cli-contract-migration-2026-04-24.md`.
4. Run:

        cargo test scanner -- --nocapture
        cargo test --test cli_contract scanner -- --nocapture

5. Run read-only REST smoke:

        target/debug/tv scanner scan --type stock --sector "Technology Services" --min-relative-volume 1.5 --sort relative_volume_10d_calc --desc --limit 3
        target/debug/tv scanner scan --type stock --min-change 2 --max-pe 50 --sort change --desc --limit 3

6. Run:

        cargo fmt --check
        cargo clippy --all-targets --all-features -- -D warnings
        cargo test
        git diff --check
        git grep -nE '(/Users/|C:\\|USER;)' -- README.md AGENTS.md docs .agents/skills || true

## Validation and Acceptance

The change is accepted when help output lists the new flags, request-builder tests show the intended filter JSON, invalid blank string filters and invalid numeric values fail before network access, and REST smoke succeeds with `source: "scanner_scan_rest"` and the requested filters reflected in `data.filters`.

No TradingView Desktop session is required, and no chart, watchlist, alert, layout, drawing, Pine, replay, or tab state should change.

## Idempotence and Recovery

The command is read-only and repeatable. If the endpoint rejects a field or changes shape, the command should fail without local or TradingView state changes. Re-running after transient network failure is safe.

## Artifacts and Notes

Do not paste raw scanner payloads or long symbol lists into tracked docs. Record only command success, source values, counts, and relevant filter summaries.

## Interfaces and Dependencies

`ScannerScanRequest` gains fields for the new filters. `scanner_scan` keeps the same public function signature. No new crate dependencies are required.

## Open Questions

None.
