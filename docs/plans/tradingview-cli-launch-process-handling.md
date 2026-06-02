# Improve `tv launch` process handling

This ExecPlan is a living document. The sections `Progress`, `Surprises &
Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up
to date as work proceeds. This document follows `.agents/PLANS.md`.

## Purpose / Big Picture

The first `v0.24.0` slice investigates and hardens `tv launch` process
handling. Downstream reports that launching through `tv` can cause
TradingView Desktop to exit soon after launch, while manually starting the app
does not. The issue is therefore process handling, not merely failure
detection.

The goal is to make `tv launch` safer for agent-driven setup and clearer when
the launch path cannot make CDP ready. This slice does not add daemon
behavior, automatic restart, or implicit process killing.

## Progress

- [x] (2026-06-03) Create this ExecPlan.
- [ ] Inspect current `tv launch` direct-spawn, macOS `open`, existing-CDP,
  and `--kill-existing` behavior.
- [ ] Decide whether macOS should prefer `open -a TradingView --args ...`
  before direct app-binary spawn.
- [ ] Improve additive launch readback, warnings, and docs.
- [ ] Validate focused launch tests, docs, and release package script syntax.

## Surprises & Discoveries

- Observation: current `tv launch` tries direct binary spawn first, then
  macOS `open` fallback only if CDP does not become ready. It drops the child
  handle after `try_wait` with the expectation that the app keeps running.
  This may be the wrong process-lifetime model for some agent or sandboxed
  execution contexts.

## Decision Log

- Decision: Treat this as a process-handling problem, not only a readiness
  reporting problem.
  Rationale: the user clarified that the concern is `tv launch` appearing to
  launch the app in a way that makes it exit, while manual launch is stable.
  Date/Author: 2026-06-03 / Codex.

- Decision: Keep `--kill-existing` explicit.
  Rationale: launch hardening should not silently terminate a user's existing
  TradingView Desktop session.
  Date/Author: 2026-06-03 / Codex.

## Outcomes & Retrospective

Pending.

## Context and Orientation

Current launch behavior is in `crates/cli/src/ops/launch.rs`. The command
already reports `launch_method`, `resolved_by`, `fallback_used`,
`used_existing`, `kill_existing`, `pid`, and `cdp_ready`. It also has a macOS
`open` fallback. This slice should preserve those existing fields and improve
the actual launch path and readback only additively.

## Plan of Work

First, inspect the current launch path and tests. Then decide the safest
process-handling adjustment. The likely implementation is to prefer or more
strongly use macOS app launching (`open -a TradingView --args ...`) instead of
direct app-binary spawn when no explicit path is provided, while preserving
direct spawn for explicit binaries and platforms where it is the only viable
path.

Improve readback and docs so agents understand whether `tv launch` reused an
existing CDP session, started through an app launcher, attempted direct spawn,
or failed to make CDP ready.

## Concrete Steps

Run all commands from the repository root.

Inspect current launch code and tests:

    rg -n "launch|direct_spawn|macos_open|kill_existing|try_wait|spawn" crates/cli/src/ops/launch.rs crates/cli/src/cli.rs
    cargo test -p tradingview-cli ops::launch -- --nocapture
    cargo test -p tradingview-cli --test cli_contract_desktop launch -- --nocapture

Implement launch hardening:

- preserve existing launch payload fields;
- prefer a safer app-launch method where appropriate;
- add public-safe readback only if needed to show process handling and next
  action;
- keep `--kill-existing` opt-in only;
- do not add daemon, monitor, or restart behavior.

Update docs and runtime guidance:

- README quick start / Desktop setup wording;
- `docs/getting-started.md`;
- `docs/ja/getting-started.md`;
- `packaging/agent/AGENTS.md`;
- relevant runtime skills if launch guidance appears there.

## Validation and Acceptance

Acceptance requires:

- `tv launch` no longer uses a process handling path that is likely to make
  TradingView Desktop exit in common agent-run contexts;
- existing-CDP reuse continues to return success without launching another
  process;
- `--kill-existing` remains opt-in;
- payload fields remain backward-compatible;
- warnings and next-action hints explain manual launch, explicit path retry,
  or readiness checks without exposing local paths or target ids;
- focused launch tests and full baseline pass.

Run:

    cargo test -p tradingview-cli ops::launch -- --nocapture
    cargo test -p tradingview-cli --test cli_contract_desktop launch -- --nocapture
    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1
    git diff --check
    bash -n scripts/stage-release-package-files.sh

Optional smoke, only if safe:

    target/debug/tv launch
    target/debug/tv readiness

Record only public-safe summary such as launch method, CDP readiness, warning
class, and whether manual launch was needed. Do not record local absolute
paths, raw target ids, account-local metadata, or raw payloads in tracked docs.

## Idempotence and Recovery

This slice is safe to rerun. If `tv launch` is tested live and leaves
TradingView Desktop running, do not kill it unless the user explicitly asks or
`--kill-existing` is being tested with approval.

## Interfaces and Dependencies

No new command or dependency is planned. Existing payload fields must remain
available. Any new fields must be additive.

## Open Questions

- Whether macOS should always prefer `open -a TradingView --args ...` when no
  explicit binary path is supplied, or only when direct spawn fails. The
  implementation should decide based on the current code and tests.

## Change Note

Planned behavior change: safer `tv launch` process handling and clearer
public-safe launch readback. No remote release action.
