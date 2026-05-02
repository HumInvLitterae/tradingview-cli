# Quote chart source stable readiness

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document follows `.agents/PLANS.md` from the repository root. It is self-contained so a new contributor can finish the chart-source quote hardening without prior chat context.

## Purpose / Big Picture

`tv quote <SYMBOL> --source chart` switches the selected TradingView Desktop chart, reads the current chart quote, and restores the original symbol. A downstream workflow reported that, even after the earlier stale-bar guard, a symbol-targeted chart quote could still occasionally return previous-symbol data. This slice treats that as a focused `v0.5.1` patch candidate before broader `v0.6.0` roadmap work.

After this change, chart-source quote succeeds only after the requested symbol is visible both in the quote payload and in the chart API state, bar values are available, switched-symbol bars differ from the original chart bars, and two consecutive ready samples have been observed. If the CLI cannot prove that the data belongs to the requested symbol, it returns a structured readiness error instead of asking downstream agents to add sleep or double-call workarounds.

## Progress

- [x] (2026-05-02) Archived the completed `v0.6` roadmap/source-taxonomy plan.
- [x] (2026-05-02) Added this ExecPlan for the chart-source quote patch.
- [x] (2026-05-02) Strengthened chart-source quote readiness to require chart-symbol match, quote-symbol match, bar availability, changed switched-symbol bar signature, and two consecutive ready samples.
- [x] (2026-05-02) Added focused unit tests for chart/quote symbol mismatch and non-consecutive ready samples.
- [x] (2026-05-02) Updated docs and runtime skills to explain that downstream agents should read `freshness_check` instead of adding sleep or double-call workarounds.
- [x] (2026-05-02) Ran focused quote tests, CLI contract quote tests, full Rust baseline, skill validation, packaging script syntax check, diff check, and hygiene grep.
- [x] (2026-05-02) Live-smoked `quote PLUG --source chart`, `quote AAPL --source chart`, and `quote PLUG --source auto`; all returned `stable_samples_seen: 2` and restored the original chart symbol.
- [ ] Commit the related changes.

## Surprises & Discoveries

- Observation: the prior guard already retried once and required a changed bar signature, but it accepted the first ready-looking sample.
  Evidence: `crates/cli/src/ops/market/quote.rs` used `symbol_matches && bar_values_available && bar_signature_changed` as the immediate success condition.

## Decision Log

- Decision: Keep `tv quote <SYMBOL>` defaulting to scanner and only harden `--source chart` / chart side of `--source auto`.
  Rationale: Desktop-free convenience and batch quote behavior should not change for a patch release.
  Date/Author: 2026-05-02 / Codex

- Decision: Require two consecutive ready samples before chart-source quote success.
  Rationale: a single ready-looking sample can still be a transient chart/quote mismatch. Two consecutive samples provide a bounded confirmation without requiring downstream sleeps.
  Date/Author: 2026-05-02 / Codex

- Decision: Do not scanner-fallback after chart mutation readiness failure.
  Rationale: after the chart has been switched, returning scanner data would hide an unknown Desktop-backed state. The command should fail with structured diagnostics instead.
  Date/Author: 2026-05-02 / Codex

## Outcomes & Retrospective

Chart-source quote now requires both the quote payload symbol and current chart
symbol to match the requested symbol, bar values to be available, switched
symbol bars to differ from the original chart bars, and two consecutive ready
samples before success. Focused tests cover stale quote symbol, stale chart
symbol, non-consecutive ready samples, retry behavior, and restore behavior.
Live smoke succeeded without adding downstream sleep or double-call
workarounds.

## Context and Orientation

`tv quote <SYMBOL>` defaults to the Desktop-free scanner REST path. `tv quote <SYMBOL> --source chart` is different: it is a Desktop-backed chart operation because it temporarily changes the selected chart symbol, reads chart data, then restores the original symbol. The chart path is useful when the user wants the selected Desktop chart feed, but it must never return data from a previous symbol as a successful response.

Relevant files:

- `crates/cli/src/ops/market/quote.rs` owns chart-source quote switching, readiness, restore, and unit tests.
- `README.md`, `docs/internal-tradingview-apis.md`, `docs/command-source-taxonomy.md`, and `docs/v0.6-roadmap.md` describe source and freshness boundaries.
- `.agents/skills/market-data-interpretation/SKILL.md`, `.agents/skills/chart-analysis/SKILL.md`, and `.agents/skills/multi-symbol-scan/SKILL.md` guide downstream agent behavior.

## Plan of Work

Strengthen the chart-source readiness loop in `quote.rs`. Introduce an internal quote sample that extracts the quote payload symbol, chart API symbol, and bar signature from the same read. A poll is ready only when the quote symbol and chart symbol both match the requested symbol, bar values are available, and the bar signature differs from the original symbol when a switch occurred. Success requires two consecutive ready samples.

Keep existing bounded behavior: one requested-symbol switch attempt, readiness polling, one retry after timeout/readiness failure, and original-symbol restore. If readiness still fails, return `internal_api_unavailable` with public-safe details: requested symbol, observed quote symbol, chart symbol, original symbol, attempts, elapsed time, restore state, `freshness_check`, and `next_action_hint`.

Update docs and runtime skills to describe chart-source quote as self-contained readiness polling. Downstream workflows should inspect `freshness_check` and readiness details rather than adding manual sleep or double-call workarounds.

## Validation and Acceptance

Run:

    cargo test -p tradingview-cli market::quote -- --nocapture
    cargo test -p tradingview-cli --test cli_contract quote -- --nocapture
    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1
    git diff --check
    bash -n scripts/stage-release-package-files.sh
    python3 "$HOME/.codex/skills/.system/skill-creator/scripts/quick_validate.py" .agents/skills/market-data-interpretation
    python3 "$HOME/.codex/skills/.system/skill-creator/scripts/quick_validate.py" .agents/skills/chart-analysis
    python3 "$HOME/.codex/skills/.system/skill-creator/scripts/quick_validate.py" .agents/skills/multi-symbol-scan
    rg -n '(/Users/|C:\\|USER;|sessionid|cookie|authorization|bearer)' README.md CHANGELOG.md docs .agents/skills packaging scripts || true

If live TradingView Desktop is available, smoke:

    target/debug/tv quote PLUG --source chart
    target/debug/tv quote AAPL --source chart
    target/debug/tv quote PLUG --source auto

Live target ids, account-local metadata, and raw chart payloads must not be written to tracked docs.

## Idempotence and Recovery

Re-running the tests and smoke checks is safe. If chart-source live smoke fails, use the structured readiness details to determine whether it is target ambiguity, chart API unavailability, restore failure, or genuine timeout. Do not add unbounded retry or scanner fallback after chart mutation.

## Interfaces and Dependencies

No new CLI option, command, or Rust crate is added. The CLI JSON payload remains additive-only: existing quote fields stay, and readiness metadata may include additional public-safe fields.

## Open Questions

None.
