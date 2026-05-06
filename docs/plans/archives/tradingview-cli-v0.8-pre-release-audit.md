# v0.8.0 pre-release completion and refactor audit

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document follows `.agents/PLANS.md` from the repository root. It is self-contained and describes the completion and refactor audit before `v0.8.0` release readiness.

## Purpose / Big Picture

`v0.8.0` added the first Desktop-free symbol evidence packet through `tv snapshot <SYMBOL>`, aligned workflow docs and runtime skills around snapshot, and added an opt-in live contract smoke. This audit stops new feature work and checks whether the release is complete enough for `v0.8.0` release readiness.

After this change, the project should have a durable record of what is complete for `v0.8.0`, what is deferred, and whether any small release-blocking refactor or contract cleanup remains.

## Progress

- [x] (2026-05-06T00:00Z) Created this ExecPlan and archived the completed snapshot live-smoke plan.
- [x] (2026-05-06T00:00Z) Reviewed snapshot implementation, typed payload, CLI dispatch, README, and observation workflow docs.
- [x] (2026-05-06T00:00Z) Ran audit searches, focused snapshot contract checks, package script syntax check, and Rust baseline.
- [x] (2026-05-06T00:00Z) Recorded completion, deferral, and refactor findings.
- [x] (2026-05-06T00:00Z) Committed the audit slice.

## Surprises & Discoveries

- Snapshot implementation and docs already match the intended boundary: Desktop-free, one-symbol, JSON, no chart mutation, no screenshot, no lab bars inclusion, and section-level errors.
- The TODO / panic audit found only expected assertion-style panics in ignored live smoke tests, a Pine template placeholder string, archived validation examples, and this plan's validation command. No release-blocking TODO, FIXME, `unimplemented!`, or `todo!` marker was found.
- The source / deferred-surface grep found expected references to snapshot, observe, lab bars, events, diagnose, binary split, MCP, and daemon boundaries. Current docs keep those broader surfaces deferred rather than presenting them as implemented workflow steps.
- The public-doc hygiene grep reported existing safety policy wording and archived validation examples. No new machine-specific path, credential, raw target id, account-local metadata, or raw live payload was added.

## Decision Log

- Decision: Treat `tv snapshot <SYMBOL>` plus workflow docs and live contract smoke as sufficient for `v0.8.0`.
  Rationale: The roadmap's remaining snapshot ideas are broader surfaces such as chart-backed snapshot, watch/JSONL, automatic screenshots, or events. They should be based on downstream evidence after the initial release.
  Date/Author: 2026-05-06 / Codex.

- Decision: Do not refactor snapshot internals before `v0.8.0` unless validation exposes a blocker.
  Rationale: `crates/market/src/snapshot.rs` is small, focused, typed, and already tested. Pre-release churn would add risk without improving the public contract.
  Date/Author: 2026-05-06 / Codex.

## Outcomes & Retrospective

The audit found no release-blocking implementation, contract, documentation, or refactor issue. `v0.8.0` is complete enough to move to release readiness.

The following work is complete for `v0.8.0`:

- `tv snapshot <SYMBOL>` as a Desktop-free, one-symbol evidence packet;
- snapshot workflow docs and runtime skill alignment;
- opt-in `live_snapshot` ignored integration smoke for live contract evidence.

The following work remains deferred after `v0.8.0`:

- chart-backed snapshot sources;
- batch, watch, JSONL, or daemon-style snapshot behavior;
- automatic screenshot capture in snapshot;
- stable browserless bars or browserless streaming;
- standalone `tv events`;
- `tv diagnose`;
- binary split;
- MCP server work;
- trading bots, dashboards, cookie/session import, and raw endpoint primitives.

No snapshot internals were refactored. The existing snapshot orchestration and typed result are small and focused enough for this release, and validation provides better risk reduction than pre-release structural churn.

Validation passed:

- `git diff --check`
- `bash -n scripts/stage-release-package-files.sh`
- audit greps for TODO/FIXME/panic markers, deferred-surface references, and public-doc hygiene
- `cargo fmt --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace`
- `cargo metadata --no-deps --format-version 1`
- `cargo test -p tradingview-market snapshot -- --nocapture`
- `cargo test -p tradingview-cli --test cli_contract snapshot -- --nocapture`
- `cargo test -p tradingview-cli --test live_snapshot`
- `target/debug/tv snapshot --help`

The optional live `tv snapshot NASDAQ:AAPL` smoke was not run in this audit. The ignored `live_snapshot` test remains the opt-in path for live scanner contract evidence.

## Context and Orientation

The relevant `v0.8.0` work is:

- `tv snapshot <SYMBOL>` as a Desktop-free single-symbol evidence packet;
- workflow docs and runtime skills that distinguish snapshot, multi-symbol reads, and chart observation;
- opt-in `live_snapshot` ignored integration smoke for live contract evidence.

The snapshot payload is expected to preserve top-level source metadata, section-level `quote` / `info` / `fundamentals` results, section-level errors, and next-action hints. It must not become a batch command, JSONL stream, Desktop-backed chart read, screenshot command, or browserless bars wrapper in this release.

## Plan of Work

Audit completion by checking current docs, runtime skills, and tests against the roadmap. Classify roadmap lanes as complete for `v0.8.0` or deferred after `v0.8.0`.

Audit refactor risk around snapshot orchestration, typed result shape, CLI dispatch, and `live_snapshot`. Only fix a problem if validation reveals a concrete release blocker such as a warning, dead code, contract mismatch, or docs/behavior contradiction.

Update `docs/v0.8-roadmap.md` and `docs/plans/README.md` so this audit is the current plan. Do not add a changelog entry unless the audit causes a user-facing docs or implementation change.

## Validation and Acceptance

Run:

    git diff --check
    bash -n scripts/stage-release-package-files.sh
    rg -n "TODO|FIXME|panic!|unimplemented!|todo!" crates docs README.md AGENTS.md CLAUDE.md packaging/agent/AGENTS.md
    rg -n "snapshot|observe chart|tv bars|tv events|diagnose|binary split|MCP|daemon" README.md CHANGELOG.md docs .agents/skills packaging/agent/AGENTS.md
    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1
    cargo test -p tradingview-market snapshot -- --nocapture
    cargo test -p tradingview-cli --test cli_contract snapshot -- --nocapture
    cargo test -p tradingview-cli --test live_snapshot
    target/debug/tv snapshot --help

Optional read-only smoke:

    target/debug/tv snapshot NASDAQ:AAPL

Do not paste raw live output into tracked docs.

Acceptance is met when validation passes, the audit records no release-blocking refactor need, and the roadmap clearly says the next step is `v0.8.0` release readiness.

## Idempotence and Recovery

This slice is mostly documentation plus validation. If a validation command fails, record the failure in this plan and either make the smallest safe fix or stop before release readiness. If the optional live smoke fails due to network or TradingView availability, do not treat that alone as a release blocker unless the structured failure contradicts the snapshot contract.

## Interfaces and Dependencies

No CLI option, JSON payload, Rust public API, dependency, version, or release package allowlist changes are part of this audit.

## Open Questions

None. Remaining feature ideas are deferred until after `v0.8.0`.
