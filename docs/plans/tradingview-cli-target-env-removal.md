# Remove target id environment fallback

This ExecPlan is a living document. The sections `Progress`, `Surprises &
Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to
date as work proceeds.

This document follows `.agents/PLANS.md` from the repository root. It is
self-contained so a reader can understand and continue the work without chat
history.

## Purpose / Big Picture

The CLI now has a direct global target selector: `tv --target-id <ID> ...`.
Keeping the old `TV_CDP_TARGET_ID` fallback and `target_env` payload creates two
ways to express the same user intent. After this change, explicit target
selection is simpler: `--target-id` is the only public handoff, and `tv tab
list` / `tv tab switch` expose `target_cli_args` as the retry path.

## Progress

- [x] (2026-04-28 02:20Z) Confirmed the working tree was clean and found all
  non-archived `TV_CDP_TARGET_ID` / `target_env` references in code, docs, and
  skills.
- [x] (2026-04-28 02:40Z) Removed the env fallback and `target_env` payloads.
- [x] (2026-04-28 02:45Z) Updated docs, runtime skills, and `CONTINUITY.md`.
- [x] (2026-04-28 03:00Z) Ran focused tests, skill validation, full validation baseline, and hygiene checks. Ready to commit.

## Surprises & Discoveries

- Observation: The previous target-selection slice intentionally left
  `TV_CDP_TARGET_ID` in active docs and payloads as a temporary fallback.
  Evidence: `src/transport.rs` still reads `TV_CDP_TARGET_ID`, and README plus
  runtime skills describe `target_env.TV_CDP_TARGET_ID` as a v0.2.x fallback.

## Decision Log

- Decision: Remove `TV_CDP_TARGET_ID` completely from the public target
  selection contract now instead of carrying it until v0.3.0.
  Rationale: `--target-id` is already available, release adoption is still
  early, and removing the fallback now avoids a long-lived duplicate interface.
  Date/Author: 2026-04-28 / Codex.
- Decision: Keep `TV_CDP_HOST` and `TV_CDP_PORT`.
  Rationale: They select the CDP endpoint rather than the page target and remain
  useful for local launch/debug configuration.
  Date/Author: 2026-04-28 / Codex.
- Decision: Do not mix this cleanup with Electron/CDP transport compatibility.
  Rationale: `localhost` versus `127.0.0.1`, CDP domain enable hangs, file URL
  target matching, and Windows MSIX launch policy are separate compatibility
  questions.
  Date/Author: 2026-04-28 / Codex.

## Outcomes & Retrospective

Implemented. `--target-id` is now the only explicit target selection path.
`TransportConfig` no longer reads `TV_CDP_TARGET_ID`, and target handoff
payloads no longer include `target_env`. `target_cli_args` remains the structured
handoff for `target_ambiguous`, `tab list`, `tab switch`, and full-page Screener
targets. Focused tests, runtime skill validation, full Rust validation, and
whitespace checks passed. The active-file grep for `TV_CDP_TARGET_ID` /
`target_env` now returns only removal notes and tests that assert `target_env` is
absent; archived historical plans still preserve old slice evidence.

## Context and Orientation

`src/transport.rs` owns CDP endpoint configuration and target discovery.
`src/ops/tab.rs` exposes target handoff payloads for follow-up commands. The
previous slice added `target_cli_args`; this slice removes the older environment
handoff.

## Plan of Work

First, simplify transport configuration. Remove `TargetIdSource::Env`, stop
reading `TV_CDP_TARGET_ID`, and keep only the CLI-provided target id. Preserve
`target_selected_by: "cli_option"` when a target id is explicitly passed.

Second, remove `target_env` from JSON handoff. `target_ambiguous`, `tab list`,
`screener_targets`, and `tab switch` should retain `target_cli_args` and
`next_action_hint` but no longer include `target_env`.

Third, update active docs and runtime skills. New examples should use only
`tv --target-id <ID> ...`. Historical archived plans may keep old evidence.

## Concrete Steps

Run commands from the repository root.

1. Edit `src/transport.rs`, `src/ops/status.rs`, `src/ops/tab.rs`, and any
   `TransportConfig` literals.
2. Update affected unit and CLI contract tests.
3. Update README, CHANGELOG, stable docs, notes, runtime skills, and
   `CONTINUITY.md`.
4. Run:

       cargo test transport -- --nocapture
       cargo test tab -- --nocapture
       cargo test --test cli_contract tab -- --nocapture
       cargo test --test cli_contract quote -- --nocapture
       cargo test --test cli_contract symbol -- --nocapture
       cargo fmt --check
       cargo clippy --all-targets --all-features -- -D warnings
       cargo test
       git diff --check

5. Check active files for remaining old handoff references:

       rg -n "TV_CDP_TARGET_ID|target_env" README.md CHANGELOG.md docs .agents/skills src tests -g '*.md' -g '*.rs' -g '!docs/plans/archives/**'

## Validation and Acceptance

The change is accepted when explicit target selection still works through
`--target-id`, target ambiguity details and tab payloads include
`target_cli_args`, and non-archived code/docs no longer contain
`TV_CDP_TARGET_ID` or `target_env` except in this plan as historical context.

## Idempotence and Recovery

The work is a contract cleanup and does not mutate TradingView state. If a test
fails because a caller still expects `target_env`, update that caller to use
`target_cli_args`.

## Artifacts and Notes

Do not copy live target ids, account ids, cookies, or raw TradingView payloads
into tracked docs.

## Interfaces and Dependencies

`TransportConfig::from_env_with_target_id(target_id)` should continue reading
`TV_CDP_HOST` and `TV_CDP_PORT`, but `target_id` must come only from the CLI
argument.

## Open Questions

None.
