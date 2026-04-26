# Add position drawing command

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document follows `.agents/PLANS.md` from the repository root.

## Purpose / Big Picture

TradingView has native Long Position and Short Position drawing tools that show entry, stop, target, risk/reward, and optional money-management values on the chart. The Rust CLI already has generic shape drawing, listing, getting, removing, and clearing. After this change, a user can run `tv draw position ...` to create one native position drawing from price levels, receive its entity id, and remove it afterward with `tv draw remove <ENTITY_ID>`.

## Progress

- [x] (2026-04-25T10:49:37Z) Confirmed the working tree was clean and inspected current drawing command structure plus upstream PR #60 evidence.
- [x] (2026-04-25T10:57:33Z) Added the `tv draw position` command, operation implementation, and automated tests.
- [x] (2026-04-25T10:57:33Z) Updated README, contract notes, lifecycle notes, and upstream PR triage.
- [x] (2026-04-25T10:57:33Z) Ran validation and live smoke that removed the created drawing.
- [ ] Commit the completed work.

## Surprises & Discoveries

- Observation: The live smoke target list was ambiguous without `TV_CDP_TARGET_ID`.
  Evidence: `target/debug/tv status` reported three TradingView chart page targets.

- Observation: `tv draw position` created exactly one native position drawing and `tv draw remove` removed the returned id.
  Evidence: On the explicit target for `BATS:AAOI`, `tv draw position long --entry-price 162.17 --stop-loss 150 --take-profit 185 --risk 1` returned `entity_id: "HKty2q"`, `before_count: 0`, and `after_count: 1`; `tv draw remove HKty2q` returned `removed: true` and `remaining_shapes: 0`.

## Decision Log

- Decision: Implement the command as `tv draw position <long|short>` rather than a top-level `draw-position`.
  Rationale: Rust groups drawing lifecycle commands under `tv draw`, and this is a chart-local drawing creation command with the same cleanup path as `draw shape`.
  Date/Author: 2026-04-25 / Codex

- Decision: Validate price ordering and finite values before connecting to CDP.
  Rationale: Invalid long/short geometry is a caller error and should not require a TradingView session to report.
  Date/Author: 2026-04-25 / Codex

- Decision: Keep smoke cleanup to `draw remove <ENTITY_ID>` and never use `draw clear`.
  Rationale: `draw clear` may delete pre-existing user drawings, while a returned entity id lets the smoke clean up only what it created.
  Date/Author: 2026-04-25 / Codex

## Outcomes & Retrospective

The `tv draw position` slice is complete. The CLI can create native Long/Short position drawings from entry, stop, and target prices, reports the new `entity_id`, and keeps cleanup scoped to `draw remove <ENTITY_ID>`. Automated validation passed, and live smoke created and removed one test position drawing without using `draw clear`.

## Context and Orientation

The Rust CLI command surface is defined in `src/cli.rs`. Command dispatch lives in `src/main.rs`. Drawing operations live in `src/ops/drawing.rs` and are re-exported through `src/ops.rs`. The existing `draw shape` command already creates chart-local drawings through TradingView's chart API and returns a new `entity_id`; `draw remove` can delete one drawing by id.

A position drawing in this plan means TradingView's native `long_position` or `short_position` drawing shape. The user provides actual price levels. TradingView expects stop/profit levels in internal ticks, so the implementation must read the current symbol's `pricescale` and convert price distances to integer `stopLevel` and `profitLevel`.

## Plan of Work

Add a `Position` variant to `DrawingCommand` in `src/cli.rs`. It must accept a positional `direction` value (`long` or `short`) and the required flags `--entry-price`, `--stop-loss`, and `--take-profit`. It should also accept optional `--entry-time`, `--account-size`, `--risk`, and `--lot-size`.

In `src/main.rs`, validate that all numeric values are finite, that optional account/risk/lot values are positive, and that price ordering matches the direction before connecting to CDP. Then build a `DrawingPositionRequest` and call `ops::drawing_position`.

In `src/ops/drawing.rs`, add `DrawingPositionRequest`, `PositionDirection`, and `drawing_position`. The operation should read `pricescale`, use visible range `to` when `entry_time` is absent, create `long_position` or `short_position` with `{ stopLevel, profitLevel, accountSize, risk, lotSize }` overrides, wait briefly, diff `getAllShapes()` before and after creation, and return a payload containing the new entity id and the practical request/result fields.

Update repository docs so `draw position` is clearly marked as a chart-local mutation with cleanup via `draw remove <ENTITY_ID>`. Update the upstream PR triage note so PR #60 becomes addressed.

## Concrete Steps

From the repository root, implement the code and docs, then run:

    cargo fmt --check
    cargo clippy --all-targets --all-features -- -D warnings
    cargo test
    git diff --check
    git grep -n 'USER'';' -- README.md docs .agents/skills || true
    git grep -nE '(/U[s]ers/|[A-Z]:\\)' -- README.md docs .agents/skills || true

If TradingView Desktop is available, run a bounded smoke:

    target/debug/tv state
    target/debug/tv quote
    target/debug/tv draw position long --entry-price <ENTRY> --stop-loss <LOWER> --take-profit <HIGHER>
    target/debug/tv draw remove <ENTITY_ID>

Use `TV_CDP_TARGET_ID` if multiple chart targets are open. Record the created entity id and removal result in this plan. Do not use `draw clear`.

Validation transcript from this implementation:

    cargo fmt --check
    cargo clippy --all-targets --all-features -- -D warnings
    cargo test
    git diff --check
    git grep -n 'USER'';' -- README.md docs .agents/skills || true
    git grep -nE '(/U[s]ers/|[A-Z]:\\)' -- README.md docs .agents/skills || true

All commands completed successfully. Live smoke used an explicit target because multiple chart targets were open:

    TV_CDP_TARGET_ID=... target/debug/tv draw position long --entry-price 162.17 --stop-loss 150 --take-profit 185 --risk 1
    TV_CDP_TARGET_ID=... target/debug/tv draw remove HKty2q

The create command returned one new entity id, and the remove command returned `removed: true` with `remaining_shapes: 0`.

## Validation and Acceptance

Automated acceptance is that tests prove validation, help output, expression generation, entity id detection, and failure mapping. Behavioral acceptance is that live smoke creates exactly one position drawing, returns an entity id, and removes that same id afterward without changing layouts or deleting unrelated drawings.

## Idempotence and Recovery

The implementation is additive. Tests do not require TradingView Desktop. Live smoke mutates the chart only by creating one drawing and immediately deleting it. If deletion fails, record the returned entity id and leave the user enough information to remove the single leftover drawing manually.

## Artifacts and Notes

Relevant upstream evidence:

    #60 adds draw_position to create native TradingView Long/Short position drawings with entry, stop-loss, take-profit, optional account/risk/lot values, and risk/reward reporting.

## Interfaces and Dependencies

No new crate dependency is required. The new public CLI interface is:

    tv draw position <long|short> --entry-price <PRICE> --stop-loss <PRICE> --take-profit <PRICE> [--entry-time <UNIX_SECONDS>] [--account-size <NUMBER>] [--risk <NUMBER>] [--lot-size <NUMBER>]

The new Rust operation interface is:

    pub async fn drawing_position(
        runtime: &mut impl RuntimeEvaluator,
        request: DrawingPositionRequest,
    ) -> Result<Value, AppError>

## Open Questions

No critical questions are open.
