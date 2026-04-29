# Desktop-free historical bars feasibility

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This repository uses `.agents/PLANS.md` as the ExecPlan standard. This document follows that standard and is self-contained so a future contributor can resume the work without reading the chat history.

## Purpose / Big Picture

The CLI now has useful Desktop-free market reads: `tv info <SYMBOL>`, `tv quote <SYMBOL>`, `tv quotes <SYMBOL>...`, `tv scanner scan`, and `tv scanner metainfo`. Historical bars are the next tempting target because they would let downstream workflows fetch recent OHLCV without preparing a TradingView Desktop chart.

This slice deliberately does not add a new command. It answers whether a Desktop-free historical bars path is safe enough for the Rust CLI now. The outcome is a documented boundary: keep `tv ohlcv` as the current-chart CDP read for now, treat TradingView WebSocket bars as a lab/research candidate, and require a separate implementation ExecPlan before adding a future `tv bars`-style command.

## Progress

- [x] (2026-04-29T18:59:25Z) Confirmed the working tree was clean and archived the completed batch quotes plan.
- [x] (2026-04-29T18:59:25Z) Reviewed current Rust `tv ohlcv` implementation and docs; it reads the active chart target's main-series bars through CDP and returns structured readiness errors when bars are unavailable.
- [x] (2026-04-29T18:59:25Z) Reviewed local `fiale-plus/tradingview-mcp-server` WebSocket implementation and PR #47 live metadata.
- [x] (2026-04-29T18:59:25Z) Updated durable docs with the feasibility decision and validation evidence.
- [x] (2026-04-29T18:59:25Z) Ran `git diff --check`, tracked-doc hygiene grep, and `git status --short`.

## Surprises & Discoveries

- Observation: The comparable fiale-plus implementation is browserless but still explicitly experimental and environment-gated.
  Evidence: PR #47 is open and describes `experimental_get_bars`, `experimental_stream_quotes`, `experimental_stream_bars`, and `TV_EXPERIMENTAL_ENABLED=1`.

- Observation: The WebSocket path has an anonymous fallback but also supports session-cookie-derived authentication knobs.
  Evidence: The local implementation defaults to an anonymous token when no session value is configured, while also reading optional session-related environment values for authenticated access.

- Observation: The WebSocket bars protocol is promising but undocumented and session-oriented.
  Evidence: It opens a TradingView data WebSocket, creates a chart session, resolves a symbol, creates a series, parses `timescale_update` messages, and waits for `series_completed` or timeout.

## Decision Log

- Decision: Do not implement Desktop-free bars in this slice.
  Rationale: The only strong evidence found is an undocumented WebSocket protocol with lab gating, optional session-cookie configuration, and no current Rust live smoke. That is useful research material, not stable-enough CLI surface.
  Date/Author: 2026-04-29 / Codex

- Decision: Keep `tv ohlcv` as the selected-chart bars command.
  Rationale: Existing `tv ohlcv` semantics are tied to the active chart target, chart resolution, visible page-session state, and readiness diagnostics. Replacing that with browserless symbol bars would change the command meaning.
  Date/Author: 2026-04-29 / Codex

- Decision: If implemented later, use a separate command shape such as `tv bars <SYMBOL> --timeframe <TIMEFRAME> --count <N>`.
  Rationale: A symbol-targeted browserless bars read is a different operation from reading the selected chart's current bars. A separate command avoids silent behavior drift.
  Date/Author: 2026-04-29 / Codex

## Outcomes & Retrospective

Research-only slice completed. Desktop-free historical bars are classified as `research_candidate`, not `safe now`. The fiale-plus PR #47 implementation is a helpful reference for framing, session lifecycle, bounded streaming, parsing, and lab gating, but it should not be copied into the stable Rust CLI without a new plan, live smoke, and a clear unauthenticated freshness/entitlement boundary.

The next implementation-oriented slice should not modify `tv ohlcv`. If the project chooses to continue this lane, create a new ExecPlan for a lab-gated `tv bars` prototype, with no cookie/session import, bounded requests, explicit freshness wording, and tests proving malformed protocol responses become structured failures.

Validation passed for this docs-only slice. The hygiene grep reported existing policy language, archived validation-command examples, and this plan's safety wording; it did not reveal any newly added secret, account-local id, or machine-specific local path.

## Context and Orientation

`tv ohlcv` currently lives in `crates/cli/src/ops/market/ohlcv.rs`. It evaluates JavaScript inside the selected TradingView Desktop chart target, reads the chart's main-series bars collection, and returns either raw bars or a summary. It depends on `tradingview_cdp::RuntimeEvaluator`, so it cannot run without a CDP target.

Desktop-free reads live in separate client crates. `tradingview-market` handles symbol search, metadata, single quote, and batch quote reads. `tradingview-scanner` handles scanner hotlist, scanner scan, and scanner metainfo reads. These paths are read-only and do not use TradingView Desktop.

The comparable project `fiale-plus/tradingview-mcp-server` has an open PR #47 for experimental WebSocket tools. The relevant local modules are `src/ws/client.ts`, `src/ws/session.ts`, `src/ws/protocol.ts`, `src/ws/parser.ts`, `src/tools/bars.ts`, and `src/tools/stream.ts`. They are useful for understanding the protocol shape, but this repository must not record raw live payloads, session cookies, or copy-paste authentication procedures in tracked docs.

## Plan of Work

First, archive `docs/plans/tradingview-cli-batch-quotes.md` because batch quotes are complete. Add this ExecPlan as the current research plan.

Second, document the evidence. Update `docs/v0.4-roadmap.md` to mark Desktop-free historical bars as a completed feasibility pass with a `research_candidate` outcome. Update `docs/internal-tradingview-apis.md` with a high-level WebSocket lab section that names the category, not raw protocol payloads or authentication steps. Update `docs/operation-adapter-boundaries.md` to preserve the current `tv ohlcv` boundary and explain that browserless bars would be a separate future command.

Third, update `docs/plans/README.md` so the active plan list names this feasibility plan and the archived categories include batch quotes. Update `CONTINUITY.md` as the local ledger, but do not commit it.

No Rust code should change in this slice.

## Concrete Steps

Run commands from the repository root.

Evidence gathering:

    rg -n "ohlcv|bars|WebSocket|stream|session|quote session|chart session" docs crates
    rg -n "ohlcv|bars|WebSocket|stream|session|quote session|chart session" <local fiale-plus checkout>
    gh pr view 47 -R fiale-plus/tradingview-mcp-server --json number,title,state,updatedAt,url,body,isDraft,author

Validation:

    git diff --check
    rg -n '(/Users/|C:\\|USER;|sessionid|cookie|authorization|bearer)' README.md CHANGELOG.md docs .agents/skills packaging scripts || true
    git status --short

The second command may report historical safety policy wording or archived validation-command examples. It must not reveal new live credentials, raw account-local identifiers, or machine-specific paths added by this slice.

## Validation and Acceptance

Acceptance is met when the docs clearly state that Desktop-free historical bars are not implemented, `tv ohlcv` remains chart-dependent, WebSocket bars are classified as research candidate, and the next implementer has enough public-safe evidence to design a separate lab-gated prototype without reading this chat.

No Cargo baseline is required because this slice does not touch Rust code. If any Rust code is touched unexpectedly, run `cargo fmt --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, and `cargo test --workspace`.

## Idempotence and Recovery

This is documentation-only research. The grep and GitHub commands are read-only. If GitHub is unavailable, keep the local fiale-plus evidence and mark PR status as not refreshed rather than guessing.

If a later contributor decides to implement a prototype, they must create a new ExecPlan and should not edit this one into an implementation plan.

## Artifacts and Notes

Public-safe evidence from fiale-plus PR #47:

    PR title: feat(lab): experimental WebSocket tools — bars, stream-quotes, stream-bars
    State: open as of 2026-04-29
    Surface: experimental_get_bars, experimental_stream_quotes, experimental_stream_bars
    Boundary: experimental environment gate, bounded limits/durations, WebSocket/session adapter modules

Validation evidence:

    git diff --check
    result: passed

    tracked-doc hygiene grep
    result: only existing policy language, archived validation-command examples, and this plan's safety wording were reported

Do not add raw WebSocket frames, session values, cookies, tokens, or local absolute file paths to tracked docs.

## Interfaces and Dependencies

No new public CLI interface is added by this slice.

A future implementation candidate would likely introduce a separate symbol-targeted command, not modify `tv ohlcv`. The candidate dependency would be a new Rust WebSocket client or a small internal lab module, but it must be proven in a separate plan before any code is added.

## Open Questions

- UNCONFIRMED: Whether anonymous WebSocket bars are consistently available for US equities and whether they share the same freshness or delay boundary as scanner REST reads.
- UNCONFIRMED: Whether extended-session bars through WebSocket are stable enough for user-facing CLI output.
- UNCONFIRMED: Whether a lab-gated Rust prototype should live under `tradingview-market`, a new crate, or the CLI package until the protocol boundary is clearer.
