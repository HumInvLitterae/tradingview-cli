# Harden data label truncation metadata

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document follows `.agents/PLANS.md` in this repository.

## Purpose / Big Picture

`tv data labels` reads visible Pine label graphics from the active TradingView chart. Before this change, the command returned at most 50 labels by default and silently kept only the newest labels when more were available. That made dense signal indicators look incomplete to downstream callers without a clear indication that older labels were omitted. After this change, callers can run `tv data labels` and see a larger default sample plus explicit metadata that says which limit was applied and whether truncation happened.

This is a read-only hardening slice. It does not change the chart, account, saved layouts, alerts, watchlists, or stream commands.

## Progress

- [x] (2026-04-25T12:30Z) Read the current `tv data labels` implementation and upstream PR #89 audit notes.
- [x] (2026-04-25T12:30Z) Create this ExecPlan with the intended contract and validation steps.
- [x] (2026-04-25T12:35Z) Update `tv data labels` to default to 500 labels and report truncation metadata.
- [x] (2026-04-25T12:35Z) Add unit tests for explicit `--max`, default limit, and default truncation.
- [x] (2026-04-25T12:35Z) Update repository notes that described this as future work.
- [x] (2026-04-25T12:41Z) Run validation and live read-only smoke.
- [x] (2026-04-25T12:43Z) Commit the completed slice.

## Surprises & Discoveries

- Observation: The current Rust implementation already preserves the newest labels when the limit is exceeded.
  Evidence: `src/ops/data/drawings.rs` uses `labels.split_off(labels.len() - limit)` in `summarize_pine_labels`.

## Decision Log

- Decision: Apply this improvement only to request-response `tv data labels`, not `tv stream labels`.
  Rationale: The upstream PR #89 evidence and project notes identify `tv data labels` default/truncation as the near-term Rust candidate. Stream commands have different volume and polling behavior and need their own design if changed.
  Date/Author: 2026-04-25 / Codex

- Decision: Add metadata fields while preserving existing `total_labels`, `showing`, and `labels`.
  Rationale: Existing callers should keep receiving the practical fields they already parse, while new callers can detect omitted labels through additive fields.
  Date/Author: 2026-04-25 / Codex

## Outcomes & Retrospective

The code and notes have been updated. Targeted `cargo test data_labels -- --nocapture` passed with three label tests, including the new default-limit and truncation cases. Full validation and commit remain.

Final validation passed: `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`, `git diff --check`, and repository grep checks for `USER;` and machine-specific absolute paths. Live read-only smoke with an explicit `TV_CDP_TARGET_ID` returned a successful empty labels payload for the selected chart target. The slice was committed as `feat(data): Add label truncation metadata`.

## Context and Orientation

The command-line parser lives in `src/cli.rs`; `tv data labels` already accepts `--filter`, `--max`, and `--verbose`. The operation implementation lives in `src/ops/data/drawings.rs`. It evaluates TradingView chart internals, summarizes label graphics, and returns a JSON value that is wrapped by the top-level CLI envelope elsewhere.

The word "truncation" means the command found more label records than it returned. This command intentionally keeps the newest labels because that is the existing behavior and usually the most useful view for current chart state.

Upstream PR #89 in the original JavaScript project included a larger mixed patch. The Rust audit recorded only one immediate implementation candidate from it: raise the `tv data labels` default cap from 50 to 500 and report whether labels were omitted.

## Plan of Work

In `src/ops/data/drawings.rs`, introduce a private `DEFAULT_LABEL_LIMIT` constant with value `500` and use it when `max_labels` is absent. In `summarize_pine_labels`, count labels after invalid entries are filtered out but before truncation. Return each study with the existing fields plus `available_labels`, `limit`, and `truncated`. `available_labels` is the number of usable labels before truncation. `limit` is the applied cap. `truncated` is true when `available_labels` is larger than the returned label count.

Update the existing label unit test to assert the new metadata for `--max 2`. Add one test that omitting `--max` uses `limit: 500` without truncation for a small input, and another test that 501 usable labels are trimmed to 500 with `truncated: true` while keeping the newest records.

Update `docs/notes/rust-cli-contract-migration-2026-04-24.md` so the JSON contract note mentions the new additive fields and the 500-label default. Update `docs/notes/upstream-pr-triage-2026-04-25.md` and `docs/notes/upstream-pr-89-hidden-surface-audit-2026-04-25.md` so they no longer describe this as pending future work.

## Concrete Steps

From the repository root, inspect the implementation with:

    rg -n "data_labels|summarize_pine_labels" src/ops/data/drawings.rs

Edit the files named in the plan of work. Then run:

    cargo test data_labels -- --nocapture
    cargo fmt --check
    cargo clippy --all-targets --all-features -- -D warnings
    cargo test
    git diff --check
    git grep -n 'USER;' -- README.md docs .agents/skills || true
    git grep -nE '(/U[s]ers/|[A-Z]:\\)' -- README.md docs .agents/skills || true

If TradingView Desktop is available, run the read-only smoke:

    target/debug/tv data labels --max 5

This smoke must not add, delete, or change chart/account state.

## Validation and Acceptance

The new tests pass and prove that `tv data labels` reports the applied limit and truncation status. The full Rust baseline passes. The repository docs contain no machine-specific absolute paths or live TradingView account script identifiers.

Manual acceptance is a JSON payload from `tv data labels --max 5` where each study still contains `total_labels`, `showing`, and `labels`, and additionally contains `available_labels`, `limit`, and `truncated`.

## Idempotence and Recovery

The code change is read-only at runtime and can be tested repeatedly. If a live smoke cannot connect to TradingView Desktop, keep the automated test results as validation and record that live smoke was skipped. If a doc update accidentally adds local paths or account-linked identifiers, remove them before committing.

## Artifacts and Notes

Expected per-study payload shape after the change:

    {
      "name": "Signals",
      "total_labels": 501,
      "available_labels": 501,
      "limit": 500,
      "showing": 500,
      "truncated": true,
      "labels": [...]
    }

## Interfaces and Dependencies

The public command remains `tv data labels [--filter <TEXT>] [--max <N>] [--verbose]`. No new CLI flags are added. The JSON payload gains additive fields under each item in `data.studies`. No external dependency is added.

## Open Questions

None.
