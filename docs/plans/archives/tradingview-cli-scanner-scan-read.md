# Add a read-only scanner scan command

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document follows `.agents/PLANS.md`.

## Purpose / Big Picture

After this change, a user can run a small TradingView Stock Screener-style REST scan without opening TradingView Desktop, connecting to CDP, or mutating any chart/account state. The new command is `tv scanner scan`; it complements the narrower `tv scanner hotlist` preset command and avoids the higher-risk UI Screener filter/screen/column mutation surface.

The observable behavior is a JSON success envelope with `command: "scanner"` and normalized symbols under `data.symbols`.

## Progress

- [x] (2026-04-25 17:06Z) Read current scanner implementation, CLI dispatch, README examples, and contract notes.
- [x] (2026-04-25 17:10Z) Added `tv scanner scan` CLI surface and scanner REST implementation.
- [x] (2026-04-25 17:11Z) Added unit and CLI contract tests.
- [x] (2026-04-25 17:13Z) Updated README, changelog, contract notes, upstream notes, and handoff material.
- [x] (2026-04-25 17:16Z) Ran automated validation and read-only live smoke.
- [x] (2026-04-25 17:16Z) Recorded outcomes and prepared the completed slice for commit.

## Surprises & Discoveries

- Observation: The `america/scan` endpoint accepts a compact JSON request with `columns`, `filter`, `sort`, and `range`, and returns rows with `s` and `d`.
  Evidence: A read-only curl probe returned `totalCount: 7550` and a first row shaped like `{ "s": "NASDAQ:NVDA", "d": ["NVDA", 208.27, ...] }`.

- Observation: The implemented command does not require TradingView Desktop or CDP.
  Evidence: `target/debug/tv scanner scan --exchange NASDAQ --exchange NYSE --sort market_cap_basic --desc --limit 3` returned a success envelope directly from the REST endpoint.

## Decision Log

- Decision: Add `tv scanner scan` under the existing `scanner` command group rather than `screener`.
  Rationale: This is a REST market-discovery read, not a UI Screener dialog read, and it should remain separate from DOM-dependent `tv screener` commands.
  Date/Author: 2026-04-25 / Codex.

- Decision: Start with safe CLI-expressible filters and no arbitrary JSON filter input.
  Rationale: Arbitrary filter JSON would be powerful but would shift schema and safety decisions onto the caller. A small first slice is easier to validate and document.
  Date/Author: 2026-04-25 / Codex.

## Outcomes & Retrospective

Implemented `tv scanner scan` as a read-only TradingView scanner REST command.
It supports the `america` market, repeatable exchange filters, column selection,
sort direction, limit, and basic numeric filters. The response is normalized
under `data` with `source: "scanner_scan_rest"`, named `field_values`, and raw
compact values preserved under `values`.

Validation passed:

- `cargo fmt --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test`
- `git diff --check`
- `git grep -nE '(/Users/|C:\\|USER;)' -- README.md AGENTS.md docs .agents/skills || true`

The grep command reported only tracked command examples that intentionally show
the grep pattern itself; no live account identifiers or machine-specific paths
were added.

Read-only live smoke passed. The command returned `success: true`,
`command: "scanner"`, `source: "scanner_scan_rest"`, `count: 3`,
`total_count: 7550`, and named field values for the first symbol.

## Context and Orientation

The command-line parser lives in `src/cli.rs`. It already has `ScannerCommand::Hotlist`. Command dispatch lives in `src/main.rs`; current scanner commands do not call `connect_runtime()` because they use network REST reads instead of TradingView Desktop CDP.

Scanner operations live in `src/ops/scanner.rs`. The existing hotlist command reads `https://scanner.tradingview.com/presets/US_<slug>` and normalizes compact scanner rows into `symbols` with both raw `values` and named `field_values`.

The new command should use `https://scanner.tradingview.com/america/scan`. This endpoint is undocumented. Treat it as a read-only external dependency that may change, and report malformed responses as `internal_api_unavailable`.

## Plan of Work

Add `ScannerCommand::Scan` to `src/cli.rs`. The command accepts `--market`, repeatable `--exchange`, `--columns`, `--sort`, `--asc`, `--desc`, `--limit`, `--min-price`, `--max-price`, `--min-volume`, and `--min-market-cap`.

Add a scanner request type and `scanner_scan` operation in `src/ops/scanner.rs`. Validate all CLI inputs before network access. Only support `market: "america"` in this first slice. Default columns are `name,description,close,change,volume,market_cap_basic`; default sort is `market_cap_basic desc`; default limit is 20 and maximum limit is 100.

Build the REST request body with `columns`, `filter`, `sort`, and `range`. Map exchanges to an `exchange in_range` filter. Map numeric filters to `close` and volume/market-cap threshold filters. Normalize response rows shaped as `{ s, d }` into the Rust success payload under `data`.

Update docs to describe the new read-only REST command and keep UI Screener mutation deferred.

## Concrete Steps

Run commands from the repository root.

1. Edit `src/cli.rs`, `src/main.rs`, `src/ops.rs`, and `src/ops/scanner.rs`.
2. Add focused tests in `src/ops/scanner.rs` and `tests/cli_contract.rs`.
3. Update `README.md`, `docs/notes/rust-cli-contract-migration-2026-04-24.md`, `docs/notes/screener-hotlist-upstream-feasibility-2026-04-25.md`, `docs/notes/upstream-pr-triage-2026-04-25.md`, and `docs/notes/next-agent-handoff-prompt-2026-04-24.md`.
4. Run:

        cargo fmt --check
        cargo clippy --all-targets --all-features -- -D warnings
        cargo test
        git diff --check
        git grep -nE '(/Users/|C:\\|USER;)' -- README.md AGENTS.md docs .agents/skills || true

5. Run read-only live smoke:

        target/debug/tv scanner scan --exchange NASDAQ --exchange NYSE --sort market_cap_basic --desc --limit 3

## Validation and Acceptance

The change is accepted when `tv scanner --help` lists `scan`, invalid market/column/sort/limit values fail before network access, and the live smoke returns a success envelope with `source: "scanner_scan_rest"`, `count <= 3`, `total_count`, `columns`, and normalized `symbols`.

Automated tests must prove request construction, validation, and compact row normalization. The command must not require TradingView Desktop or change chart, watchlist, layout, alert, drawing, Pine, replay, or tab state.

## Idempotence and Recovery

The command is read-only and repeatable. If the REST endpoint is unavailable or changes shape, the command should fail without changing local or TradingView state. Re-running after a transient network failure is safe.

## Artifacts and Notes

Do not paste raw live scanner payloads or long symbol lists into tracked docs. Summaries such as `count`, `total_count`, and field names are acceptable.

## Interfaces and Dependencies

At completion, `src/ops/scanner.rs` must expose:

    pub async fn scanner_scan(request: ScannerScanRequest) -> Result<Value, AppError>;

No new crate dependencies are required.

## Open Questions

None.
