# Indicator alertcondition dry-run preview

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document follows `.agents/PLANS.md` from the repository root. It is self-contained so a future contributor can continue from this file without prior chat context.

## Purpose / Big Picture

The CLI can now discover Pine `alertcondition()` candidates locally with `tv pine alertconditions`, but it still does not have a safe bridge from those candidates to TradingView's account alert endpoint. Upstream PR #112 suggests a normal account mutation is possible, but exposing raw saved-script ids and alert payload fields would be risky.

This change adds only a dry-run preview command: `tv alert create-indicator --script <NAME> --file <PATH> --condition-title <TITLE>|--alert-cond-id <ID> --dry-run`. It reads local Pine source, finds a specific alertcondition candidate, checks that exactly one saved Pine script matches the requested script name in the current logged-in TradingView session, and returns a sanitized preview. It does not create an alert, does not output saved script ids, and refuses non-dry-run execution.

## Progress

- [x] (2026-04-28 00:00Z) Read current `alert` and `pine` command definitions, Pine list endpoint normalization, and the previous static discovery plan.
- [x] (2026-04-28 00:00Z) Chose a dry-run-only command shape under `alert`: `create-indicator`.
- [x] (2026-04-28 00:00Z) Implemented typed Pine alertcondition candidates and the dry-run preview operation.
- [x] (2026-04-28 00:00Z) Added CLI contract and operation tests.
- [x] (2026-04-28 00:00Z) Updated README, CHANGELOG, API reference, roadmap, upstream notes, and handoff/contract notes.
- [x] (2026-04-28 00:00Z) Ran focused tests, full Rust baseline, hygiene grep, and dry-run smoke.
- [x] (2026-04-28 00:00Z) Updated `CONTINUITY.md`, recorded final outcomes, and prepared the related changes for commit.

## Surprises & Discoveries

- Observation: `pine list` normalizes saved scripts to `id`, `name`, `title`, `version`, and `modified`.
  Evidence: `src/ops/pine/editor.rs` maps `scriptIdPart` to `id`, but docs prohibit writing live saved-script ids into tracked notes.

- Observation: A live dry-run can succeed without creating an alert when the supplied saved-script display name has exactly one match.
  Evidence: `cargo run --quiet -- alert create-indicator --script <redacted saved script name> --condition-title Long --dry-run < target/pine-alertconditions-smoke.pine` returned `success: true`, `dry_run: true`, and `mutation_supported: false`.

## Decision Log

- Decision: Require `--dry-run` and reject normal `alert create-indicator` execution before connecting.
  Rationale: The safe boundary for this slice is preview only. Normal mutation needs a separate plan for payload construction, readback matching, and cleanup smoke.
  Date/Author: 2026-04-28 / Codex.

- Decision: Require local source through stdin or `--file`, and match the saved script by user-provided display name.
  Rationale: Local source avoids fetching or recording account script source. Saved-script matching proves that the logged-in session has a plausible script without exposing its id.
  Date/Author: 2026-04-28 / Codex.

- Decision: Do not include saved script ids in the dry-run output.
  Rationale: Saved-script ids are account-linked operational metadata. The CLI may use them internally in a later mutation, but public docs and preview output should not normalize users into copying them around.
  Date/Author: 2026-04-28 / Codex.

## Outcomes & Retrospective

Implemented a dry-run-only preview command for Pine `alertcondition()` alerts. It verifies the local source candidate and the existence of a unique saved-script display-name match, while refusing normal mutation and omitting saved-script ids from the public payload. Normal alert creation for indicator alertconditions remains deferred until a future plan specifies endpoint payload construction, readback matching, and cleanup smoke.

## Context and Orientation

The `tv` CLI declares command arguments in `src/cli.rs` and dispatches them in `src/main.rs`. Alert operations live in `src/ops/alert.rs`. Pine static analysis lives in `src/ops/pine/analysis.rs`. Tests for command-line validation live in `tests/cli_contract.rs`.

`alertcondition()` is a Pine function that declares an alertable condition. TradingView alert payloads refer to such conditions through plot-like ids such as `plot_1`. The existing `tv pine alertconditions` command estimates these ids from local source order and marks them as `best_effort`. A "dry-run preview" in this plan means a command that resolves inputs and shows what would be used later, without sending any create request.

## Plan of Work

First, refactor `src/ops/pine/analysis.rs` so alertcondition candidates are available as typed data as well as JSON. Keep `tv pine alertconditions` output unchanged.

Second, add a new `AlertCommand::CreateIndicator` variant in `src/cli.rs`. The command should accept `--script <NAME>`, `--file <PATH>`, exactly one of `--condition-title <TEXT>` or `--alert-cond-id <ID>`, optional `--symbol`, optional `--resolution`, optional `--message`, and required `--dry-run`. In `src/main.rs`, read Pine source before connecting, validate `--dry-run`, then call a new operation in `src/ops/alert.rs`.

Third, implement `alert_create_indicator_dry_run` in `src/ops/alert.rs`. It should validate the script selector and condition selector, call the typed Pine source scanner, select exactly one candidate, list saved Pine scripts through the Pine facade endpoint, match exactly one script by name or title, and return a sanitized preview. The output should include the requested script name, matched display name/title/version if available, `script_id_available: true/false`, selected candidate, requested symbol/resolution/message, `dry_run: true`, `would_create: true`, `source: "indicator_alert_dry_run"`, and `mutation_supported: false`. It must not include the saved script id.

Fourth, update docs to describe this as preview only. Raw indicator-alert mutation remains deferred.

## Concrete Steps

Work from the repository root.

1. Edit `src/ops/pine/analysis.rs`, `src/cli.rs`, `src/main.rs`, `src/ops.rs`, `src/ops/alert.rs`, and `tests/cli_contract.rs`.
2. Edit `README.md`, `CHANGELOG.md`, `docs/internal-tradingview-apis.md`, `docs/v0.3-roadmap.md`, `docs/notes/upstream-pr-recheck-2026-04-27.md`, `docs/notes/rust-cli-contract-migration-2026-04-24.md`, `docs/notes/next-agent-handoff-prompt-2026-04-24.md`, and `docs/plans/README.md`.
3. Run:

    cargo test alert_indicator -- --nocapture
    cargo test pine_alertcondition -- --nocapture
    cargo test --test cli_contract alert -- --nocapture
    cargo test --test cli_contract pine -- --nocapture
    cargo fmt --check
    cargo clippy --all-targets --all-features -- -D warnings
    cargo test
    git diff --check
    git grep -nE '(/Users/|C:\\|USER;|sessionid|cookie|authorization|bearer|webhook|web_hook)' -- README.md CHANGELOG.md docs .agents/skills packaging scripts || true

4. If a TradingView session is available, run a dry-run only smoke against a harmless local Pine file and an existing saved-script display name. Do not create or delete alerts in this slice.

Validation results from this implementation:

    cargo test alert_indicator -- --nocapture
    result: ok. 3 passed.

    cargo test pine_alertcondition -- --nocapture
    result: ok. 6 tests passed across unit and CLI contract tests.

    cargo test --test cli_contract alert -- --nocapture
    result: ok. 13 passed.

    cargo test --test cli_contract pine -- --nocapture
    result: ok. 16 passed.

    cargo fmt --check
    result: ok.

    cargo clippy --all-targets --all-features -- -D warnings
    result: ok.

    cargo test
    result: ok. 342 unit tests and 87 CLI contract tests passed.

    live dry-run smoke
    result: ok. A redacted saved-script display name matched exactly once; the command returned `dry_run: true` and no alert was created.

## Validation and Acceptance

The change is accepted when:

- `tv alert create-indicator` refuses normal execution without `--dry-run` before connecting.
- dry-run selects an alertcondition candidate from local source by title or `plot_N` id.
- dry-run checks for exactly one saved script match by display name/title.
- dry-run output does not include saved script ids or raw alert payloads.
- all focused and full validation commands pass.

## Idempotence and Recovery

The command is read-only and dry-run-only. It can be repeated safely. If saved script matching is ambiguous, the command should return a validation error with sanitized candidate names. If Pine source candidate matching is ambiguous or absent, it should fail before any account mutation because there is no mutation path in this slice.

## Artifacts and Notes

The earlier discovery command produced this shape in local smoke:

    data.candidates[0].alert_cond_id = "plot_1"
    data.candidates[0].confidence = "best_effort"

The dry-run preview should consume the same candidate shape internally.

## Interfaces and Dependencies

At the end of this plan, these interfaces should exist:

    pub fn pine_alertcondition_candidates(source: &str) -> Vec<PineAlertconditionCandidate>;

    pub async fn alert_create_indicator_dry_run(
        runtime: &mut impl RuntimeEvaluator,
        request: IndicatorAlertDryRunRequest,
    ) -> Result<Value, AppError>;

The CLI surface should include:

    tv alert create-indicator --script <NAME> --file <PATH> --condition-title <TITLE> --dry-run
    tv alert create-indicator --script <NAME> --file <PATH> --alert-cond-id plot_1 --dry-run

No new external Rust dependencies are required.

## Open Questions

Normal indicator-alert mutation is intentionally out of scope. A future plan must still decide whether to expose it, how to construct the exact endpoint payload, how to verify created alerts, and how to clean up smoke alerts safely.
