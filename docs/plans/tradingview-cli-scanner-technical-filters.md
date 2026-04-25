# Add technical scanner filters

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document follows `.agents/PLANS.md`.

## Purpose / Big Picture

After this change, `tv scanner scan` can express practical technical and momentum screens using the existing read-only TradingView scanner REST endpoint. Users can filter by average volume, weekly/monthly/quarterly performance, RSI, TradingView recommendation score, and signed daily change without falling back to UI Screener automation or downstream workflow packs.

This slice is read-only. It does not open TradingView Desktop, connect to CDP, save Screener screens, mutate filters, or add strategy-specific scanner presets.

## Progress

- [x] (2026-04-25 18:16Z) Checked the current scanner scan implementation, CLI dispatch, docs, and working tree.
- [x] (2026-04-25 18:23Z) Added technical scanner fields and CLI flags.
- [x] (2026-04-25 18:24Z) Split scanner numeric validation by signed, non-negative, RSI, and recommendation ranges.
- [x] (2026-04-25 18:27Z) Added unit and CLI contract tests.
- [x] (2026-04-25 18:29Z) Updated README, changelog, and contract notes.
- [x] (2026-04-25 18:37Z) Ran focused tests, REST smoke, and full validation baseline.
- [x] (2026-04-25 18:38Z) Recorded outcomes and prepared the completed tracked slice for commit.

## Surprises & Discoveries

- Observation: `--max-change -5` currently fails in clap before scanner validation because the negative value is parsed as an unexpected flag.
  Evidence: `target/debug/tv scanner scan --type stock --max-change -5 --sort change --asc --limit 1` returned a usage validation error before reaching the scanner request builder.

- Observation: The scanner REST endpoint currently returns `average_volume_10d_calc`, `Perf.W`, `Perf.1M`, `Perf.3M`, `RSI`, and `Recommend.All`.
  Evidence: Read-only curl probes against `https://scanner.tradingview.com/america/scan` returned successful payloads for those columns and filters.

## Decision Log

- Decision: Keep this as an explicit-filter CLI surface rather than adding arbitrary filter JSON.
  Rationale: The command should remain safe, discoverable, and validation-friendly while the endpoint is undocumented.
  Date/Author: 2026-04-25 / Codex.

- Decision: Treat daily change and performance fields as signed finite percentages, while keeping price, volume, relative volume, PE, and market cap non-negative.
  Rationale: Loss and weakness scans are a normal scanner workflow; rejecting negative change values makes the command artificially one-sided.
  Date/Author: 2026-04-25 / Codex.

## Outcomes & Retrospective

Completed. `tv scanner scan` now supports average-volume, performance, RSI, and recommendation filters on top of the existing read-only scanner REST surface. Daily change and performance filters now accept signed finite values, which enables bearish scans such as `--max-change -5`.

Focused tests passed for scanner request construction and CLI contract behavior. REST smoke succeeded for a bullish technical scan and a bearish signed-change scan, both returning `source: "scanner_scan_rest"` with the requested filters reflected in `data.filters`. Full validation passed with `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`, and `git diff --check`. The repository grep for local absolute paths and `USER;` produced only existing validation-command examples in plan documents.

## Context and Orientation

`src/cli.rs` declares `ScannerCommand::Scan`; `src/main.rs` maps CLI values into `ops::ScannerScanRequest`; `src/ops/scanner/scan.rs` builds and validates the REST scanner request. Scanner operations are already split under `src/ops/scanner/`.

`tv scanner scan` currently supports market, exchange, type/subtype, sector/industry, columns, sort, limit, price, volume, market cap, daily change, relative volume, and PE filters. The payload lives under `data` with `source: "scanner_scan_rest"`.

## Plan of Work

Add CLI flags and request fields for:

- `--min-average-volume`
- `--min-performance-week` / `--max-performance-week`
- `--min-performance-month` / `--max-performance-month`
- `--min-performance-quarter` / `--max-performance-quarter`
- `--min-rsi` / `--max-rsi`
- `--min-recommendation` / `--max-recommendation`

Add `average_volume_10d_calc`, `Perf.W`, `Perf.1M`, `Perf.3M`, `RSI`, and `Recommend.All` to supported scanner columns and sort fields.

Use `allow_hyphen_values = true` for signed CLI options so negative daily change, performance, and recommendation values are parsed as values. Keep non-negative validation for price, volume, market cap, average volume, relative volume, and PE. Validate RSI in `0..=100` and recommendation in `-1..=1`.

## Concrete Steps

Run commands from the repository root.

1. Edit `src/cli.rs`, `src/main.rs`, and `src/ops/scanner/scan.rs`.
2. Add or update tests in `src/ops/scanner/scan.rs` and `tests/cli_contract.rs`.
3. Update `README.md`, `CHANGELOG.md`, and `docs/notes/rust-cli-contract-migration-2026-04-24.md`.
4. Run focused tests:

        cargo test scanner -- --nocapture
        cargo test --test cli_contract scanner -- --nocapture

5. Run read-only REST smoke:

        target/debug/tv scanner scan --type stock --min-average-volume 1000000 --min-performance-week 5 --max-rsi 70 --min-recommendation 0.1 --sort Perf.W --desc --columns name,close,Perf.W,RSI,Recommend.All --limit 3
        target/debug/tv scanner scan --type stock --max-change -5 --sort change --asc --columns name,change,volume --limit 3

6. Run full validation:

        cargo fmt --check
        cargo clippy --all-targets --all-features -- -D warnings
        cargo test
        git diff --check
        git grep -nE '(/Users/|C:\\|USER;)' -- README.md AGENTS.md docs .agents/skills || true

## Validation and Acceptance

The change is accepted when help output lists the new flags, request-builder tests show the intended filter JSON, negative signed filter values reach scanner validation and REST smoke, invalid RSI/recommendation ranges fail before network access, and REST smoke succeeds with `source: "scanner_scan_rest"` and the requested filters reflected in `data.filters`.

No TradingView Desktop session is required, and no chart, watchlist, alert, layout, drawing, Pine, replay, tab, Screener dialog, or account state should change.

## Idempotence and Recovery

The command is read-only and repeatable. If TradingView changes a scanner field or response shape, the command should fail without local or TradingView state changes. Re-running after transient network failure is safe.

## Artifacts and Notes

Do not paste raw scanner payloads or long symbol lists into tracked docs. Record only command success, source values, counts, and relevant filter summaries.

## Interfaces and Dependencies

`ScannerScanRequest` gains fields for the new filters. `scanner_scan` keeps the same public function signature. No new crate dependencies are required.

## Open Questions

None.
