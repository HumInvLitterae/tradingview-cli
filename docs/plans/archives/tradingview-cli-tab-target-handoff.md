# Strengthen tab switch target handoff

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document follows `.agents/PLANS.md` from the repository root.

## Purpose / Big Picture

The Rust CLI already supports `tv tab switch <INDEX>`, but a switched tab is not automatically selected for later commands when multiple TradingView chart targets remain open. After this change, `tv tab switch` tells the operator exactly which `TV_CDP_TARGET_ID` to use for follow-up chart commands. This addresses the Rust-relevant part of upstream PR #40 without introducing a persistent CDP client or guessing the active tab in later commands.

## Progress

- [x] (2026-04-25T11:57:25Z) Confirmed the working tree was clean and inspected the current tab, transport, README, and upstream PR #40 context.
- [x] (2026-04-25T12:00:18Z) Added additive target handoff fields to the `tv tab switch` success payload and covered them with a unit test.
- [x] (2026-04-25T12:01:20Z) Updated durable docs for the target handoff contract.
- [x] (2026-04-25T12:02:00Z) Ran validation and bounded live smoke.
- [x] (2026-04-25T12:02:10Z) Prepared the completed work for commit.

## Surprises & Discoveries

- No surprises yet.

## Decision Log

- Decision: Keep target selection explicit through `TV_CDP_TARGET_ID` instead of auto-selecting the active TradingView tab after `tab switch`.
  Rationale: Rust commands reconnect per process, so the upstream stale-client bug does not directly apply. Auto-detecting the active tab would change current ambiguity semantics and could silently run commands against the wrong chart when multiple chart targets exist.
  Date/Author: 2026-04-25 / Codex

- Decision: Add `target_id`, `target_env`, and `next_command_hint` while preserving existing `tab_id`.
  Rationale: Existing consumers keep working, while humans and downstream adapters get an unambiguous next-command handoff value.
  Date/Author: 2026-04-25 / Codex

## Outcomes & Retrospective

The target handoff is now explicit. `tv tab switch` still returns the original `tab_id`, and it now also returns `target_id`, `target_env.TV_CDP_TARGET_ID`, and `next_command_hint`. This keeps old consumers compatible while giving operators and downstream adapters a direct value for follow-up chart-specific commands.

Automated validation passed:

    cargo fmt --check
    cargo clippy --all-targets --all-features -- -D warnings
    cargo test
    git diff --check
    git grep -n 'USER'';' -- README.md docs .agents/skills || true
    git grep -nE '(/U[s]ers/|[A-Z]:\\)' -- README.md docs .agents/skills || true

Live smoke used the already-active tab index `2`. `tv tab switch 2` returned a `target_env.TV_CDP_TARGET_ID` value, and running `tv state` with that environment value succeeded without target ambiguity.

## Context and Orientation

TradingView Desktop exposes open pages through Chrome DevTools Protocol targets. This repository calls those pages "targets." The Rust `tv` command normally discovers one chart target through `src/transport.rs`. If more than one chart target is open, discovery deliberately returns a target ambiguity error unless `TV_CDP_TARGET_ID` is set. Tab commands live in `src/ops/tab.rs`; `tv tab switch <INDEX>` activates a target by index through `/json/activate/<target_id>`.

Upstream PR #40 fixed a Node implementation problem where `switchTab()` activated a tab but kept evaluating JavaScript through the old WebSocket connection. Rust does not keep a persistent WebSocket across commands, so the direct fix is not needed. The Rust-relevant usability gap is that the successful switch output should make the next target id obvious.

## Plan of Work

Update `src/ops/tab.rs` so the `tab_switch` success payload includes the selected target id in a next-command-oriented shape. Keep `tab_id` for compatibility and add `target_id`, `target_env`, and `next_command_hint`. Add a unit test around a pure payload helper so the contract is covered without making network calls.

Update durable docs to explain that `tab switch` now returns the target handoff. The contract note should mention the additive fields. The upstream PR triage note should classify PR #40 as addressed for Rust by explicit target handoff, not by persistent reconnect logic. README should continue to tell users to set `TV_CDP_TARGET_ID` when multiple chart targets exist and can mention that `tab switch` now returns the env value to copy.

## Concrete Steps

From the repository root, run:

    cargo fmt --check
    cargo clippy --all-targets --all-features -- -D warnings
    cargo test
    git diff --check
    git grep -n 'USER'';' -- README.md docs .agents/skills || true
    git grep -nE '(/U[s]ers/|[A-Z]:\\)' -- README.md docs .agents/skills || true

If TradingView Desktop is available, run bounded smoke:

    tv tab list
    tv tab switch <INDEX>
    TV_CDP_TARGET_ID=<target_id-from-switch> tv state

Prefer switching to the tab that is already active, or switch back to the original tab after the smoke. Do not close tabs, create tabs, switch saved layouts, or mutate chart state.

## Validation and Acceptance

Automated acceptance is that `cargo test` includes a unit test proving `tab_switch` exposes `tab_id`, `target_id`, `target_env.TV_CDP_TARGET_ID`, and `next_command_hint` for the same selected target. Existing tab validation behavior must not change.

Behavioral acceptance is that live smoke can run `tv tab switch <INDEX>`, read the returned `target_env.TV_CDP_TARGET_ID`, and use that value in `TV_CDP_TARGET_ID=<id> tv state` without a target ambiguity error.

## Idempotence and Recovery

The change is additive and does not create, close, or modify chart resources. Live smoke may bring a different tab to the front. If that happens, use the original tab index recorded from `tv tab list` to switch back.

## Artifacts and Notes

Relevant upstream evidence:

    PR #40 says the old Node `switchTab()` activated a target but kept the CDP client connected to the previous target. Rust commands reconnect per process, so this plan improves the handoff value instead of adding persistent reconnect behavior.

## Interfaces and Dependencies

No new crate dependency is required. No new public command or flag is introduced. The existing command remains:

    tv tab switch <INDEX>

The success payload keeps existing fields and adds:

    target_id: string
    target_env: { "TV_CDP_TARGET_ID": string }
    next_command_hint: string

## Open Questions

No critical questions are open.
