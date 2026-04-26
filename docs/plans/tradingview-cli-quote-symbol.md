# Add optional symbol support to quote

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document follows `.agents/PLANS.md`.

## Purpose / Big Picture

After this change, an operator can run `tv quote AAPL` to read a quote for a specific symbol without manually changing the active TradingView chart first. The command keeps the existing `tv quote` behavior for the current chart, and only performs a temporary chart symbol switch when a different symbol is requested. The visible proof is that `tv quote <other-symbol>` returns quote fields for the requested symbol and then `tv symbol` still reports the original chart symbol.

This addresses upstream PR #104 from the original JavaScript repository, which reports that `quote_get(symbol)` accepted a symbol argument but returned data from the active chart. Rust does not need to clone the old wire shape, but it should avoid the same practical bug when adding symbol-targeted quote reads.

## Progress

- [x] (2026-04-26 07:28Z) Checked current working tree, continuity ledger, quote implementation, upstream PR #104, upstream PR #105, and current docs.
- [x] (2026-04-26 07:39Z) Added optional `SYMBOL` parsing, validation, operation support, and tests.
- [x] (2026-04-26 07:47Z) Updated README, CHANGELOG, contract note, upstream PR triage, handoff note, and continuity ledger.
- [x] (2026-04-26 07:52Z) Ran focused quote tests.
- [x] (2026-04-26 08:05Z) Ran full validation.
- [x] (2026-04-26 07:56Z) Ran live smoke with restore verification against an explicit TradingView target.
- [ ] Commit tracked changes.

## Surprises & Discoveries

- Observation: Upstream PR #104 confirms a real old-CLI bug rather than a pure feature request.
  Evidence: The PR body says parallel `quote_get` calls for different symbols returned identical active-chart OHLC with only the requested symbol label changed.

- Observation: Rust `tv quote` currently has no symbol argument, so it has not inherited the exact false-label bug.
  Evidence: `src/cli.rs` defines `Quote` without fields, and `src/ops/market.rs::quote` reads `chart.symbol()` from the active chart.

- Observation: Upstream PR #105 is a JavaScript dependency-injection regression in drawing functions, not an obvious Rust bug.
  Evidence: Rust drawing functions are implemented separately in `src/ops/drawing.rs`; the PR only changes bare JavaScript names such as `getChartApi()` to `_getChartApi()`.

- Observation: Concurrent symbol-targeted quote commands against the same CDP target can race across separate CLI processes.
  Evidence: an accidental parallel smoke caused one command to see `original_symbol: "AAPL"` while the target had started on `BATS:AAOI`; the chart was restored manually with `tv symbol BATS:AAOI` before sequential smoke continued.

## Decision Log

- Decision: Implement `tv quote [SYMBOL]` as an optional positional argument.
  Rationale: It preserves the existing current-chart command while giving callers the explicit symbol-targeted read that upstream PR #104 tried to provide.
  Date/Author: 2026-04-26 / Codex.

- Decision: Use temporary chart switching and restore rather than reverse-engineering TradingView's snapshot service.
  Rationale: Snapshot service support would be a larger protocol investigation. The chart switch path is narrow, observable, and matches existing Rust chart mutation capabilities.
  Date/Author: 2026-04-26 / Codex.

- Decision: Treat restore failure as command failure.
  Rationale: A symbol-targeted quote is allowed to mutate chart state only temporarily. If the CLI cannot verify restoration, it should not report a clean success.
  Date/Author: 2026-04-26 / Codex.

- Decision: Add a short process lock around symbol-targeted quote commands.
  Rationale: Unlike the old Node server, Rust CLI invocations are separate processes. A local lock prevents normal concurrent `tv quote <SYMBOL>` calls from interleaving chart switch and restore steps.
  Date/Author: 2026-04-26 / Codex.

## Outcomes & Retrospective

Implementation is complete at the CLI and operation layer. `tv quote` without arguments still reads the current chart directly. `tv quote <SYMBOL>` validates a non-empty symbol, acquires a short local lock, switches only when the requested symbol differs from the current chart by bare ticker, reads quote data, and verifies restore before success. Focused tests, full validation, and live smoke passed. Commit is still pending.

## Context and Orientation

The command parser lives in `src/cli.rs`, dispatch lives in `src/main.rs`, and quote reads live in `src/ops/market.rs`. The existing `tv quote` command connects to the current Chrome DevTools Protocol target, evaluates JavaScript inside the TradingView page, reads the active chart's last bar, and returns a Rust JSON envelope whose payload is under `data`.

A temporary chart switch means calling TradingView's chart API `setSymbol()` to load a requested symbol, reading the quote from the active chart, then calling `setSymbol()` again with the original symbol. This is a visible UI mutation while it is running, so the command must report whether a switch was performed and whether the chart was restored.

## Plan of Work

Update `src/cli.rs` so `Command::Quote` accepts an optional positional `symbol`. Update `src/main.rs` to validate a non-empty requested symbol before connecting, then pass `Option<&str>` to the operation.

In `src/ops/market.rs`, change `quote` to accept `Option<&str>`. Keep the no-symbol path as a direct active-chart read. For a requested symbol, acquire a short local process lock, read the original symbol, compare bare tickers case-insensitively, switch only if needed, read the quote, and restore the original symbol if a switch occurred. The returned payload should include `requested_symbol`, `original_symbol`, `observed_symbol`, `switch_performed`, and `restored` in addition to the existing quote fields.

Update tests to cover help output, empty symbol validation before CDP connection, no-symbol behavior, same-symbol fast path, switch/read/restore ordering, quote failure with restore attempt, and restore failure mapping to a structured error.

Update docs to explain that `tv quote <SYMBOL>` briefly changes the chart and restores it. Refresh the upstream PR triage note so #104 is addressed by this Rust slice and #105 remains a drawing smoke candidate rather than an immediate Rust code fix.

## Concrete Steps

Run commands from the repository root.

1. Edit `src/cli.rs`, `src/main.rs`, and `src/ops/market.rs`.
2. Add focused operation tests in `src/ops/market.rs` and CLI contract tests in `tests/cli_contract.rs`.
3. Update `README.md`, `CHANGELOG.md`, `docs/notes/rust-cli-contract-migration-2026-04-24.md`, `docs/notes/upstream-pr-triage-2026-04-25.md`, and `docs/notes/next-agent-handoff-prompt-2026-04-24.md`.
4. Run focused tests:

        cargo test quote -- --nocapture
        cargo test --test cli_contract quote -- --nocapture

5. Run live smoke when a TradingView Desktop CDP target is available:

        tv symbol
        tv quote <current-symbol-or-bare-ticker>
        tv quote MSFT
        tv symbol

   The final `tv symbol` should report the same symbol as the first command. If the requested symbol is already current, use a different liquid test symbol.

6. Run full validation:

        cargo fmt --check
        cargo clippy --all-targets --all-features -- -D warnings
        cargo test
        git diff --check
        git grep -nE '(/Users/|C:\\|USER;)' -- README.md AGENTS.md docs .agents/skills || true

## Validation and Acceptance

The change is accepted when `tv quote` without arguments behaves as before, `tv quote <SYMBOL>` returns quote fields for the requested symbol, the payload reports whether a temporary switch occurred, and live smoke confirms the chart returns to the original symbol after a cross-symbol quote. Automated validation must pass the focused quote tests and the full baseline.

## Idempotence and Recovery

The no-symbol path is read-only. The symbol-targeted path is safe to repeat because it restores the original symbol after each run. Symbol-targeted quote commands use a local lock file in the system temporary directory so ordinary concurrent invocations wait instead of interleaving chart mutations. If a live smoke leaves the chart on the requested symbol, immediately run `tv symbol <original-symbol>` and record the restore failure in this plan before investigating.

## Artifacts and Notes

Do not record raw market payloads beyond minimal symbol and boolean metadata needed to prove behavior. Do not write machine-specific paths into tracked docs.

Focused tests passed:

        cargo test quote -- --nocapture
        cargo test --test cli_contract quote -- --nocapture

Live smoke used explicit target `D202CA6B22895C82C0437F0F9FC6A7BC`. Initial `tv symbol` returned `BATS:AAOI`. `tv quote AAOI` returned `switch_performed: false` and `restored: true`. `tv quote AAPL` returned `symbol: "BATS:AAPL"`, `original_symbol: "BATS:AAOI"`, `switch_performed: true`, and `restored: true`. A final `tv symbol` returned `BATS:AAOI`.

Full validation passed:

        cargo fmt --check
        cargo clippy --all-targets --all-features -- -D warnings
        cargo test
        git diff --check
        git grep -nE '(/Users/|C:\\|USER;)' -- README.md AGENTS.md docs .agents/skills || true

The grep returned only existing validation-command examples in plan documents.

## Interfaces and Dependencies

The public CLI interface after this slice is:

    tv quote [SYMBOL]

The operation signature should be:

    pub async fn quote(runtime: &mut impl RuntimeEvaluator, symbol: Option<&str>) -> Result<Value, AppError>

No new external Rust dependency is required.

## Open Questions

No critical open questions block implementation. Multi-symbol parallel quote fan-out is intentionally out of scope for this slice; callers should use scanner REST commands for broad multi-symbol discovery.
