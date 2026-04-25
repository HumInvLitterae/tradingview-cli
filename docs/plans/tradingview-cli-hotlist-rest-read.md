# Add a read-only scanner hotlist command

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document follows `.agents/PLANS.md` from the repository root.

## Purpose / Big Picture

After this change, a user can ask the Rust `tv` CLI for TradingView Hotlist preset rows such as volume gainers without opening TradingView Desktop, connecting to CDP, or mutating any chart/account state. The observable behavior is a new command, `tv scanner hotlist <SLUG> [--limit <N>]`, that returns the standard Rust JSON success envelope with normalized hotlist data under `data`.

This is not a full Stock Screener UI automation port. It implements only the low-risk read-only Hotlist REST surface identified in the upstream PR research notes.

## Progress

- [x] (2026-04-25T14:00Z) Created this ExecPlan and fixed the command namespace and output contract.
- [x] (2026-04-25T14:08Z) Added the CLI subcommand and dispatch without creating a CDP runtime.
- [x] (2026-04-25T14:08Z) Added the scanner operation module, validation helpers, response normalization, and unit tests.
- [x] (2026-04-25T14:12Z) Updated user-facing and agent-facing docs to mark Hotlist REST reads as implemented.
- [x] (2026-04-25T14:15Z) Ran baseline checks and a read-only live smoke.
- [x] (2026-04-25T14:19Z) Committed the completed slice as `feat(scanner): Add hotlist read command`.

## Surprises & Discoveries

- Observation: The Hotlist endpoint accepted a plain GET without custom headers.
  Evidence: `cargo run -- scanner hotlist volume_gainers --limit 3` exited zero and returned three normalized rows.

## Decision Log

- Decision: Use `tv scanner hotlist` rather than `tv screener hotlist`.
  Rationale: The REST preset endpoint is a scanner-style market-discovery read. Keeping it under `scanner` avoids confusing it with future UI Screener dialog automation.
  Date/Author: 2026-04-25 / Codex

- Decision: Treat `--limit 0` as validation error and clamp values above 20 to 20.
  Rationale: Zero rows is not a useful user request, while the upstream endpoint returns only a single small preset page and the existing research note records a maximum of 20.
  Date/Author: 2026-04-25 / Codex

- Decision: Do not include raw scanner rows in tracked docs or CLI output.
  Rationale: The user needs practical information, not the compact upstream wire shape. Normalized fields reduce downstream coupling and avoid documenting raw live payloads.
  Date/Author: 2026-04-25 / Codex

## Outcomes & Retrospective

- Implemented `tv scanner hotlist <SLUG> [--limit <N>]` as a read-only, non-CDP command. The command validates a fixed Hotlist slug whitelist, rejects zero limits, caps larger limits at 20, and returns normalized scanner rows under the Rust `data` envelope. UI Screener automation remains separate and deferred.

## Context and Orientation

The Rust CLI entry point is `src/main.rs`. It parses `src/cli.rs`, dispatches to functions exported from `src/ops.rs`, and prints success payloads under the top-level `data` field through `src/output.rs`. Commands that use TradingView Desktop connect to CDP with `connect_runtime()`, but this new command must not do that because it uses a plain HTTPS endpoint.

The upstream research in `docs/notes/screener-hotlist-upstream-feasibility-2026-04-25.md` found that TradingView Hotlist presets can be read from `https://scanner.tradingview.com/presets/US_<slug>?label-product=right-hotlists`. The same note says the endpoint returns top-level `fields`, `symbols`, `time`, and `totalCount`, and each compact symbol row uses `s` for the symbol and `f` for field values. The raw payload is intentionally not stored in repository docs.

## Plan of Work

Add `ScannerCommand` to `src/cli.rs` with one subcommand: `hotlist`. The command takes a required `slug` and an optional `--limit` / `-n` value. Add `Command::Scanner` to the command enum and return `"scanner"` from `Command::name()`.

Add a `Command::Scanner` dispatch arm in `src/main.rs`. This arm must call the scanner operation directly and must not call `connect_runtime()`. It should delegate slug and limit validation to the operation layer so the same helpers can be tested without launching the binary.

Create `src/ops/scanner.rs` and export `scanner_hotlist` from `src/ops.rs`. The module must whitelist exactly these slugs: `volume_gainers`, `percent_change_gainers`, `percent_change_losers`, `percent_range_gainers`, `percent_range_losers`, `gap_gainers`, `gap_losers`, `percent_gap_gainers`, and `percent_gap_losers`. It must reject empty or unknown slugs with `ErrorKind::Validation` before network access. It must reject `Some(0)` as validation error, default `None` to 20, and clamp values above 20 to 20.

The operation must build the URL from the validated slug, send a GET request with `reqwest`, map network failures and non-success status codes to `ErrorKind::Connection`, and map JSON parse or unexpected shape failures to `ErrorKind::InternalApiUnavailable`.

The success payload must contain `source: "scanner_preset_rest"`, `region: "US"`, `slug`, `limit`, `count`, `total_count`, `fields`, and `symbols`. Each normalized symbol row must contain `symbol`, `values`, and `field_values`. `field_values` maps field names to values when both sides are present at the same index. `values` keeps all compact values so information is not lost if fields and values do not line up perfectly.

Update `README.md`, `docs/notes/rust-cli-contract-migration-2026-04-24.md`, `docs/notes/upstream-pr-triage-2026-04-25.md`, `docs/notes/screener-hotlist-upstream-feasibility-2026-04-25.md`, and `docs/notes/next-agent-handoff-prompt-2026-04-24.md` to reflect the new command and the remaining boundary: UI Screener automation is still separate and deferred.

## Concrete Steps

Work from the repository root.

First add the CLI types and dispatch, then add the operation module and tests. Keep the implementation small and avoid introducing a generic scanner framework.

Run targeted tests while developing:

    cargo test scanner
    cargo test --test cli_contract scanner

Run the final baseline:

    cargo fmt --check
    cargo clippy --all-targets --all-features -- -D warnings
    cargo test
    git diff --check

Run a read-only live smoke:

    cargo run -- scanner hotlist volume_gainers --limit 3

Success means the command exits zero and prints a success envelope with command `"scanner"` and at most three entries in `data.symbols`. The smoke does not require TradingView Desktop and does not change user state.

## Validation and Acceptance

The feature is accepted when `tv scanner --help` shows `hotlist`, invalid slugs fail before any network request, `--limit 0` fails before any network request, unit tests prove normalization of compact scanner rows, and the read-only live smoke returns normalized symbols from the Hotlist preset endpoint.

The repository remains acceptable for public release documentation: no raw live scanner payload, machine-specific absolute path, secret, or account-linked identifier is added to tracked docs.

## Idempotence and Recovery

The implementation is additive. Re-running tests and the live smoke is safe. If the endpoint is temporarily unavailable, keep the command implementation and record the observed HTTP or network error in this ExecPlan rather than changing the command contract. If the endpoint requires headers, add only the same conservative `Origin` and `Referer` style already used by `tv search` and record that discovery here.

## Artifacts and Notes

Validation completed:

    cargo fmt --check
    cargo clippy --all-targets --all-features -- -D warnings
    cargo test
    git diff --check

`cargo test` passed 236 unit tests and 69 CLI contract tests. The read-only live smoke `cargo run -- scanner hotlist volume_gainers --limit 3` returned a success envelope with `command: "scanner"`, `count: 3`, and `fields: ["volume"]`. Repository grep checks found no newly introduced machine-specific absolute paths or raw account-local scanner payloads. The completed slice was committed as `feat(scanner): Add hotlist read command`.

## Interfaces and Dependencies

`src/ops/scanner.rs` must expose:

    pub async fn scanner_hotlist(slug: &str, limit: Option<usize>) -> Result<serde_json::Value, AppError>;

The module should keep helper functions private unless tests need `pub(super)` visibility. It uses the existing `reqwest`, `serde_json`, `AppError`, and `ErrorKind` dependencies. No new crate is expected.

## Open Questions

None. The command namespace, validation, output contract, docs scope, and validation commands are decided in this plan.
