# Smoke Rust drawing lifecycle for upstream PR #105

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document follows `.agents/PLANS.md`.

## Purpose / Big Picture

After this work, maintainers can tell whether upstream PR #105 exposes an equivalent Rust CLI drawing issue. Upstream PR #105 fixes the old JavaScript `draw_list`, `draw_get_properties`, `draw_remove_one`, and `draw_clear` commands after a JavaScript dependency-injection refactor left them calling undefined helper names. Rust drawing commands are implemented independently, so this slice should smoke-test Rust `tv draw shape/list/get/remove/clear` before making any code change.

The visible proof is a live round trip: create one disposable drawing, list it, get its properties, remove it, and confirm it no longer appears. If the chart starts with zero drawings, also prove `draw clear --dry-run` and normal `draw clear` using another disposable drawing.

## Progress

- [x] (2026-04-26 07:49Z) Checked working tree, continuity ledger, upstream PR #105 body and diff, Rust drawing implementation, and initial live `tv draw list`.
- [x] (2026-04-26 07:54Z) Ran disposable drawing shape/list/get/remove smoke.
- [x] (2026-04-26 07:55Z) Because initial drawing count was zero, ran disposable `draw clear --dry-run` and normal `draw clear` smoke.
- [x] (2026-04-26 07:55Z) Confirmed no Rust code fix is needed for upstream PR #105-equivalent behavior.
- [x] (2026-04-26 07:58Z) Updated upstream PR triage and related docs with the outcome.
- [x] (2026-04-26 08:00Z) Ran focused validation and docs checks.
- [ ] Commit tracked changes.

## Surprises & Discoveries

- Observation: Upstream PR #105 is a JavaScript wrapper bug, not a TradingView chart API bug.
  Evidence: the PR changes bare `getChartApi()` and `evaluate()` calls to `_getChartApi()` and `_evaluate()` in `src/core/drawing.js`.

- Observation: Rust `tv draw list` already succeeds against the selected live target before creating any disposable drawing.
  Evidence: `TV_CDP_TARGET_ID=D202CA6B22895C82C0437F0F9FC6A7BC target/debug/tv draw list` returned `success: true`, `count: 0`, and `shapes: []`.

- Observation: Rust `draw shape/list/get/remove` works on a disposable live drawing.
  Evidence: `draw shape` created entity `JMm48n`, `draw list` returned that id, `draw get JMm48n` returned properties including the smoke text, `draw remove JMm48n` returned `removed: true`, and the following `draw list` returned `count: 0`.

- Observation: Rust `draw clear --dry-run` and normal `draw clear` work when the chart starts with no existing drawings.
  Evidence: `draw shape` created disposable entity `AUVh8U`, `draw clear --dry-run` reported `would_clear_count: 1`, normal `draw clear` reported `cleared: true` and `after_count: 0`, and the following `draw list` returned `count: 0`.

## Decision Log

- Decision: Do not cherry-pick PR #105 into Rust.
  Rationale: Rust has no JavaScript dependency-injection wrapper layer with the same helper-name regression. The right Rust follow-up is smoke evidence and a targeted fix only if the live commands fail.
  Date/Author: 2026-04-26 / Codex.

- Decision: Run normal `draw clear` only when the initial drawing count is zero.
  Rationale: `draw clear` removes all chart drawings. If the chart already has user drawings, this smoke must not delete them.
  Date/Author: 2026-04-26 / Codex.

## Outcomes & Retrospective

Rust does not show an equivalent issue to upstream PR #105 in the selected live TradingView Desktop target. The lifecycle commands `draw shape`, `draw list`, `draw get`, `draw remove`, `draw clear --dry-run`, and normal `draw clear` all worked on disposable drawings. No Rust code change is needed; this slice is documentation and evidence only. Focused validation passed; commit is still pending.

## Context and Orientation

Rust drawing command parsing lives in `src/cli.rs`, dispatch lives in `src/main.rs`, and operation code lives in `src/ops/drawing.rs`. The command group contains `tv draw shape`, `tv draw position`, `tv draw list`, `tv draw get <ENTITY_ID>`, `tv draw remove <ENTITY_ID>`, and `tv draw clear [--dry-run]`. Successful command payloads live under the Rust `data` envelope.

An entity id is the TradingView drawing identifier returned by `tv draw shape` or `tv draw position`. This slice uses one generated entity id at a time so the disposable drawing can be removed directly.

## Plan of Work

Create one disposable horizontal line with a unique text marker. Record the returned `entity_id`. Run `tv draw list` and confirm the id appears. Run `tv draw get <ENTITY_ID>` and confirm properties are returned rather than an internal helper error. Run `tv draw remove <ENTITY_ID>` and confirm `removed: true`. Run `tv draw list` again and confirm the id is gone.

If the initial `draw list` count is zero, create a second disposable drawing and run `tv draw clear --dry-run` followed by normal `tv draw clear`. Accept normal clear only if it reports all drawings cleared and a final list returns count zero.

If a Rust command fails, inspect the error details and patch only the failing Rust operation. Do not add new drawing command surface in this slice.

## Concrete Steps

Run commands from the repository root. Use an explicit target id when multiple TradingView chart targets are open.

1. Confirm initial drawing state:

        TV_CDP_TARGET_ID=<target> target/debug/tv draw list

2. Read quote metadata for a reasonable drawing price/time:

        TV_CDP_TARGET_ID=<target> target/debug/tv quote

3. Create a disposable drawing:

        TV_CDP_TARGET_ID=<target> target/debug/tv draw shape --type horizontal_line --price <PRICE> --time <TIME> --text tv-draw-pr105-smoke-<timestamp>

4. Exercise list/get/remove:

        TV_CDP_TARGET_ID=<target> target/debug/tv draw list
        TV_CDP_TARGET_ID=<target> target/debug/tv draw get <ENTITY_ID>
        TV_CDP_TARGET_ID=<target> target/debug/tv draw remove <ENTITY_ID>
        TV_CDP_TARGET_ID=<target> target/debug/tv draw list

5. If initial count was zero, exercise clear with another disposable drawing:

        TV_CDP_TARGET_ID=<target> target/debug/tv draw shape --type horizontal_line --price <PRICE> --time <TIME> --text tv-draw-pr105-clear-smoke-<timestamp>
        TV_CDP_TARGET_ID=<target> target/debug/tv draw clear --dry-run
        TV_CDP_TARGET_ID=<target> target/debug/tv draw clear
        TV_CDP_TARGET_ID=<target> target/debug/tv draw list

6. Run validation:

        cargo test drawing -- --nocapture
        cargo test --test cli_contract draw -- --nocapture
        git diff --check
        git grep -nE '(/Users/|C:\\|USER;)' -- README.md AGENTS.md docs .agents/skills || true

If code changes are made, also run:

        cargo fmt --check
        cargo clippy --all-targets --all-features -- -D warnings
        cargo test

## Validation and Acceptance

The work is accepted when live smoke proves Rust `draw list`, `draw get`, and `draw remove` work on a disposable drawing without a PR #105-equivalent failure. If the initial chart has no drawings, `draw clear --dry-run` and normal `draw clear` must also work on a disposable drawing and leave the chart at count zero. Automated validation must pass the focused drawing tests and docs checks, plus full baseline if code changes were required.

## Idempotence and Recovery

Every disposable drawing must be tracked by its returned `entity_id`. If any smoke command fails after creation, run `tv draw remove <ENTITY_ID>` before continuing. Do not run normal `draw clear` when initial `draw list` shows existing drawings, because that could remove user state.

## Artifacts and Notes

Record only command names, counts, entity ids created for this smoke, and high-level result fields. Do not paste large raw drawing property payloads into tracked docs.

Live smoke summary with explicit target `D202CA6B22895C82C0437F0F9FC6A7BC`:

        initial draw list: count 0
        draw shape: created JMm48n with text tv-draw-pr105-smoke-20260426T0749Z
        draw list: count 1, id JMm48n
        draw get JMm48n: returned properties with smoke text
        draw remove JMm48n: removed true, remaining_shapes 0
        draw list: count 0
        draw shape: created AUVh8U with text tv-draw-pr105-clear-smoke-20260426T0749Z
        draw clear --dry-run: would_clear_count 1, target AUVh8U
        draw clear: cleared true, after_count 0
        final draw list: count 0

Focused validation summary:

        cargo test drawing -- --nocapture: 26 passed
        cargo test --test cli_contract draw -- --nocapture: 4 passed
        git diff --check: passed
        tracked-doc local path / USER; grep: only validation command examples in plan docs

## Interfaces and Dependencies

No new public CLI interface is planned. If a fix is required, keep existing commands and payload shape compatible.

## Open Questions

No critical open question blocks execution. The only branch is whether live smoke exposes a Rust issue; if it does not, this slice is documentation and evidence only.
