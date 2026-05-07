# v0.9.0 pre-release completion and refactor audit

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document follows `.agents/PLANS.md` from the repository root. It is self-contained and records the final audit before `v0.9.0` release readiness.

## Purpose / Big Picture

`v0.9.0` added Desktop-free multi-symbol comparison through `tv compare <SYMBOL>...` and an opt-in ignored live contract smoke for that compare packet.

Before release readiness, this slice stops new feature work and asks two questions:

1. Is any `v0.9.0` roadmap work still required before release prep?
2. Is there any small refactor, contract mismatch, or documentation mismatch that should be fixed before release?

The expected outcome is a clear decision to proceed to `v0.9.0` release readiness, or a narrowly scoped blocker fix if validation finds one.

## Progress

- [x] (2026-05-07T12:19Z) Created this ExecPlan and archived the completed compare live-smoke plan.
- [x] (2026-05-07T12:19Z) Reviewed current `v0.9.0` roadmap state, active plans index, compare implementation, compare live smoke, README, and observation workflow docs.
- [x] (2026-05-07T12:35Z) Ran audit searches, focused compare contract checks, package script syntax check, and Rust baseline.
- [x] (2026-05-07T12:35Z) Recorded completion, deferral, and refactor findings.
- [x] (2026-05-07T12:35Z) Committed the audit slice.

## Surprises & Discoveries

- `tv compare <SYMBOL>...` is already represented in README, `docs/observation-workflows.md`, `docs/command-source-taxonomy.md`, and runtime skills as a Desktop-free multi-symbol evidence packet.
- The compare live smoke follows the existing snapshot live-smoke pattern and reports public-safe summaries instead of raw JSON payloads.
- The TODO / panic audit found assertion-style `panic!` calls in ignored live smoke tests, one Pine template TODO string, archived validation examples, and this plan's validation command. No release-blocking TODO, FIXME, `unimplemented!`, or `todo!` marker was found.
- The deferred-surface grep found expected references to compare, snapshot, observation, lab bars, diagnose, binary split, MCP, daemon, and realtime boundaries. Current docs keep broader surfaces deferred rather than presenting them as implemented workflow steps.
- The public-doc hygiene grep reported existing safety policy wording and archived validation examples. No new machine-specific path, credential, raw target id, account-local metadata, or raw live payload was added.

## Decision Log

- Decision: Treat `tv compare <SYMBOL>...` plus its ignored live contract smoke as the intended `v0.9.0` feature scope unless validation exposes a blocker.
  Rationale: The remaining roadmap ideas are broader surfaces such as ranking/scoring, chart-backed compare, realtime feed work, watch/JSONL behavior, or stable browserless bars. Those should be based on downstream evidence after the initial comparison packet is released.
  Date/Author: 2026-05-07 / Codex.

- Decision: Do not refactor compare internals before `v0.9.0` unless validation exposes a concrete release blocker.
  Rationale: Pre-release structural churn should be limited to warnings, dead code, test-contract mismatches, or clear docs/behavior contradictions.
  Date/Author: 2026-05-07 / Codex.

## Outcomes & Retrospective

The audit found no release-blocking implementation, contract, documentation, or refactor issue. `v0.9.0` is complete enough to move to release readiness.

The following work is complete for `v0.9.0`:

- `tv compare <SYMBOL>...` as a Desktop-free multi-symbol comparison packet;
- compare docs and runtime skill alignment through existing observation workflow guidance;
- opt-in `live_compare` ignored integration smoke for live contract evidence.

The following work remains deferred after `v0.9.0`:

- chart-backed compare sources;
- ranking, scoring, or recommendation options;
- batch snapshot, watch, JSONL, or daemon-style compare behavior;
- realtime multi-symbol feed work;
- stable browserless bars or browserless streaming;
- standalone `tv events`;
- `tv diagnose`;
- binary split;
- MCP server work;
- trading bots, dashboards, cookie/session import, and raw endpoint primitives.

No compare internals were refactored. The existing compare orchestration and typed result are small enough for this release, and validation provides better risk reduction than pre-release structural churn.

Validation passed:

- `git diff --check`
- `bash -n scripts/stage-release-package-files.sh`
- audit greps for TODO/FIXME/panic markers, deferred-surface references, and public-doc hygiene
- `cargo fmt --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace`
- `cargo metadata --no-deps --format-version 1`
- `cargo test -p tradingview-market compare -- --nocapture`
- `cargo test -p tradingview-cli --test cli_contract compare -- --nocapture`
- `cargo test -p tradingview-cli --test live_compare`
- `target/debug/tv compare --help`

The optional live `tv compare NASDAQ:AAPL NYSE:IONQ` smoke was not run in this audit. The ignored `live_compare` test remains the opt-in path for live scanner contract evidence.

## Context and Orientation

The current `v0.9.0` work is centered on `tv compare <SYMBOL>...`. It is a Desktop-free command, which means it must not connect to TradingView Desktop, use Chrome DevTools Protocol, switch the visible chart, capture screenshots, or include lab-gated browserless bars. It gathers scanner quote, symbol info, and default scanner-backed fundamentals evidence for multiple requested symbols and returns a JSON comparison packet.

The relevant implementation files are:

- `crates/market/src/compare.rs`, which orchestrates the typed Desktop-free compare packet;
- `crates/cli/src/cli.rs` and `crates/cli/src/app/dispatch.rs`, which expose and dispatch the CLI command;
- `crates/cli/tests/live_compare.rs`, which provides opt-in ignored live contract evidence;
- `docs/observation-workflows.md` and runtime skills, which explain when to use `compare` versus `snapshot`, `quotes`, scanner reads, and chart observation.

For this release, `compare` is not a ranking or recommendation engine. It preserves observed evidence, section-level errors, missing-value summaries, source metadata, and next-action hints so humans or agents can compare candidates without hiding data quality differences behind a score.

## Plan of Work

Archive the completed compare live-smoke ExecPlan and make this audit the current plan in `docs/plans/README.md` and `docs/v0.9-roadmap.md`.

Audit completion by checking current docs, runtime skills, CLI help text, compare tests, and market compare implementation against the roadmap. Classify roadmap lanes as complete for `v0.9.0` or deferred after `v0.9.0`.

Audit refactor risk around compare orchestration, typed result shape, CLI dispatch, and `live_compare`. Only fix a problem if validation reveals a concrete release blocker such as a warning, dead code, contract mismatch, or docs/behavior contradiction.

Do not add new command surface, options, payload fields, dependencies, release version changes, or broad refactors in this slice.

## Concrete Steps

Run audit and validation commands from the repository root:

    git diff --check
    bash -n scripts/stage-release-package-files.sh
    rg -n "TODO|FIXME|panic!|unimplemented!|todo!" crates docs README.md AGENTS.md CLAUDE.md packaging/agent/AGENTS.md
    rg -n "compare|snapshot|observe chart|tv bars|diagnose|binary split|MCP|daemon|realtime" README.md CHANGELOG.md docs .agents/skills packaging/agent/AGENTS.md
    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1
    cargo test -p tradingview-market compare -- --nocapture
    cargo test -p tradingview-cli --test cli_contract compare -- --nocapture
    cargo test -p tradingview-cli --test live_compare
    target/debug/tv compare --help

Optional read-only smoke:

    target/debug/tv compare NASDAQ:AAPL NYSE:IONQ

Do not paste raw live output into tracked docs.

## Validation and Acceptance

Acceptance is met when:

- validation passes;
- no release-blocking TODO/FIXME/panic/unimplemented marker is found;
- `v0.9.0` roadmap lanes are classified as complete or deferred;
- no new command, option, payload change, dependency, or large refactor is introduced;
- next step is clearly `v0.9.0` release readiness.

## Idempotence and Recovery

This audit is docs-only unless validation reveals a blocker. If a blocker is found, do not mix a broad refactor into this slice. Either make a minimal fix with focused validation or create a new ExecPlan for the blocker.

The optional live smoke depends on TradingView network availability. If it fails due to network or service availability, do not treat that alone as a release blocker unless the structured failure contradicts the compare contract.

## Interfaces and Dependencies

No public interface changes. No dependency changes. No release version bump in this slice.

## Open Questions

None. Release readiness is the next step if this audit passes.
