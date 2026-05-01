# v0.5 Desktop readiness diagnostics plan

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds. This document follows `.agents/PLANS.md`.

## Purpose / Big Picture

This plan prepares the first `v0.5.0` implementation slice. The goal is to make TradingView Desktop-backed operations easier for agents to use safely by improving readiness and diagnostic information before adding more UI automation or mutation features. After this slice, an agent should be able to inspect existing `tv` command output and decide whether it has a usable chart target, whether chart data is ready, whether it should use scanner or chart source, and whether Computer Use visual confirmation is needed.

This is not a plan to implement every diagnostic idea at once. It is the first decision-complete slice for reviewing and tightening the existing Desktop readiness surface.

## Progress

- [x] (2026-05-01) Created this ExecPlan as the first `v0.5.0` implementation candidate.
- [ ] Review existing readiness and diagnostic payloads for `status`, `tab list`, `state`, `quote --source chart`, and `ohlcv`.
- [ ] Decide whether the implementation should add payload fields to existing commands, add a small `tv diagnose` / `tv readiness` command, or stay as docs/skills only.
- [ ] Implement the chosen smallest behavior change.
- [ ] Update runtime skills so agents do not paper over readiness failures with unbounded sleep, repeated retries, or blind Computer Use.
- [ ] Validate with focused tests, CLI contract tests, docs hygiene, and a live smoke if TradingView Desktop is available.
- [ ] Commit the related changes.

## Surprises & Discoveries

- None yet.

## Decision Log

- Decision: Start `v0.5.0` from Desktop-backed readiness diagnostics instead of Desktop-free historical bars.
  Rationale: `v0.4.0` already made Desktop-free market reads substantially more useful, while `v0.4.1` exposed the practical importance of chart readiness and stale-data protection for Desktop-backed commands.
  Date/Author: 2026-05-01 / Codex

- Decision: Do not assume a new `tv diagnose` command is required.
  Rationale: Existing commands already return some diagnostic data. The first implementation should identify the smallest additive change that improves agent operation without creating a vague catch-all command.
  Date/Author: 2026-05-01 / Codex

## Outcomes & Retrospective

- Pending implementation.

## Context and Orientation

TradingView Desktop-backed commands are commands that connect to the user's running TradingView Desktop app through Chrome DevTools Protocol, usually called CDP in this repository. They differ from Desktop-free scanner reads because their correctness depends on a selected app target, visible chart state, page-session JavaScript objects, or rendered UI.

The relevant existing surfaces are:

- `tv status`, which checks whether the local CDP endpoint is reachable.
- `tv tab list`, which lists chart and Screener page targets and returns `target_cli_args`.
- `tv state`, which reads active chart state from the selected target.
- `tv quote <SYMBOL> --source chart`, which may temporarily switch the chart symbol and now includes a `freshness_check`.
- `tv ohlcv`, which reads current chart bars and already returns chart-bars readiness details on failure.
- runtime skills such as `chart-analysis`, `market-data-interpretation`, `multi-symbol-scan`, and `screener-workflow`, which guide downstream agents through source selection and recovery.

The repository already treats `tv` and Computer Use as complementary. `tv` is best at structured reads, target handoff, error envelopes, and post-checks. Computer Use is best at visual confirmation and one-off UI interaction when structured state cannot explain the situation.

## Plan of Work

First, review the current JSON payloads and error details for the existing diagnostic commands. Use source inspection and tests rather than changing behavior immediately. Look for duplicated concepts such as target selection, active target, chart symbol, chart bars availability, quote freshness, Screener full-page readiness, and next-action hints.

Second, choose the smallest implementation shape. Prefer additive fields on existing payloads if they solve the agent problem. Add a new command only if the same readiness summary would otherwise require agents to run many commands and merge results themselves. If a new command is needed, it should be narrow, read-only, and named around readiness rather than broad debugging.

Third, implement the chosen shape with structured errors and tests. Do not add unbounded retries. Do not hide readiness failures by falling back to Computer Use or DOM actions. If readiness cannot be established, return a clear error or diagnostic payload with a next action.

Fourth, update runtime skills so they teach the new behavior. Skills should say when to rely on `tv`, when to switch target with `--target-id`, when to use scanner source, and when Computer Use visual confirmation is appropriate.

## Concrete Steps

Run all commands from the repository root.

1. Inspect the existing diagnostics:
   - `rg -n "freshness_check|readiness|next_action_hint|target_cli_args|ohlcv|status|state" crates docs .agents/skills`
   - `cargo test -p tradingview-cli market::quote -- --nocapture`
   - `cargo test -p tradingview-cli --test cli_contract status tab quote ohlcv -- --nocapture` if the test filter shape is valid; otherwise run the nearest focused contract commands separately.

2. If the implementation is additive to existing payloads, update the affected operation modules and contract tests. Likely candidates are chart state, tab/status diagnostics, chart-source quote, and OHLCV failure details. Keep existing fields and exit codes compatible.

3. If the implementation needs a new command, add a read-only command that reports Desktop readiness without mutating chart or account state. It must include target handoff, selected source readiness, chart symbol, chart bars availability, and next-action hints when possible.

4. Update runtime skills and docs:
   - `docs/v0.5-roadmap.md`
   - `docs/operation-adapter-boundaries.md`
   - `docs/internal-tradingview-apis.md` if non-public API boundaries change
   - relevant runtime skills under `.agents/skills/`
   - `CHANGELOG.md`

## Validation and Acceptance

Validation must prove that existing command behavior remains compatible and that the new diagnostics help without relying on manual sleep or blind retries.

Run:

- `cargo fmt --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace`
- focused CLI contract tests for the touched commands
- `git diff --check`
- `rg -n '(/Users/|C:\\|USER;|sessionid|cookie|authorization|bearer)' README.md CHANGELOG.md docs .agents/skills packaging scripts || true`

If TradingView Desktop is available, run a read-only or restore-safe smoke:

- `target/debug/tv status`
- `target/debug/tv tab list`
- `target/debug/tv state`
- `target/debug/tv quote PLUG --source chart`
- `target/debug/tv ohlcv --count 1`

Do not record live target ids, account-local values, raw payloads, cookies, tokens, or local absolute paths in tracked docs.

Acceptance criteria:

- Existing JSON envelope shape and exit codes remain compatible.
- Agents can distinguish at least these situations from structured output: no CDP endpoint, ambiguous target, chart target selected, chart bars unavailable, chart-source quote freshness passed or failed.
- Runtime skills describe the new readiness behavior and when Computer Use should be used as visual confirmation rather than as a replacement for structured checks.

## Idempotence and Recovery

This slice should be additive. If a new diagnostic field causes contract churn, keep the field optional or only add it to failure details. If a new command feels too broad during implementation, stop and narrow the plan rather than shipping a vague diagnostic surface. All live smokes should be read-only or restore-safe.

## Artifacts and Notes

Do not paste live TradingView target ids, account metadata, raw page-session payloads, screenshots with private account data, cookies, tokens, or local absolute paths into this plan.

## Interfaces and Dependencies

No stable public Rust API change is required. Any CLI change must be additive and read-only. If a new command is added, it must live in the `tradingview-cli` package and use existing `tradingview-cdp` primitives rather than adding a new workspace crate.

## Open Questions

The implementation must answer one design question during the review step: are additive payload fields enough, or is a small read-only readiness command justified? Record the decision in this plan before editing code.
