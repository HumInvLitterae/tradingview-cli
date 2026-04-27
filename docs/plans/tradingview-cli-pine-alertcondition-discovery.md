# Pine alertcondition discovery

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document follows `.agents/PLANS.md` from the repository root. It is intentionally self-contained so a future contributor can restart from this file alone.

## Purpose / Big Picture

Upstream PR #112 showed a way to create TradingView alerts for Pine `alertcondition()` signals, but the raw mutation depends on saved-script identifiers, exact alert-condition ids, Pine inputs, and account-local alert payload details. Exposing that directly would be powerful but easy to misuse.

This change adds a safe first building block: a local, read-only Pine source scanner that reports `alertcondition()` candidates and their best-effort TradingView alert-condition ids. A user can run `tv pine alertconditions --file script.pine` or pipe source through stdin and see which alertcondition entries could later be used by a guarded dry-run or normal indicator-alert command. The command does not connect to TradingView, does not read saved account metadata, and does not create alerts.

## Progress

- [x] (2026-04-28 00:00Z) Read `.agents/PLANS.md`, current Pine CLI dispatch, `src/ops/pine/analysis.rs`, README and current upstream PR #112 notes.
- [x] (2026-04-28 00:00Z) Chose a safe initial command shape: `tv pine alertconditions [--file <PATH>]`.
- [x] (2026-04-28 00:00Z) Implemented static `alertcondition()` discovery and wired it into the CLI as `tv pine alertconditions [--file <PATH>]`.
- [x] (2026-04-28 00:00Z) Updated README, CHANGELOG, contract notes, handoff notes, roadmap, upstream recheck note, plan index, and internal API reference.
- [x] (2026-04-28 00:00Z) Ran focused tests, full Rust baseline, hygiene grep, and a local smoke with a temporary Pine file.
- [ ] Update `CONTINUITY.md`, record final outcomes in this plan, and commit the related changes.

## Surprises & Discoveries

- Observation: Existing `pine analyze` already reads Pine source from stdin or `--file` before any CDP connection.
  Evidence: `src/main.rs` dispatch calls `read_pine_source(file.as_deref())?` before `ops::pine_analyze(&source, input_source)`.

- Observation: The new static scanner can run through the CLI without a TradingView Desktop session.
  Evidence: `cargo run --quiet -- pine alertconditions --file target/pine-alertconditions-smoke.pine` returned `success: true`, `candidate_count: 1`, and `alert_cond_id: "plot_1"`.

## Decision Log

- Decision: Put the new discovery command under `pine` as `tv pine alertconditions`, not under `alert`.
  Rationale: The command analyzes Pine source locally and does not create or preview an account alert. The later account mutation can use this output, but the source of truth for this slice is Pine source analysis.
  Date/Author: 2026-04-28 / Codex.

- Decision: Use a best-effort static scanner rather than attempting to open saved scripts or create a dry-run alert from account metadata.
  Rationale: `pine list` can expose saved-script metadata, but saved script identifiers and account-local names must not be copied into docs, and `pine get` can depend on the local editor. Static local source scanning avoids account mutation and gives users a safe first step.
  Date/Author: 2026-04-28 / Codex.

- Decision: Report candidate ids as `plot_<N>` with a `best_effort` confidence marker.
  Rationale: TradingView alertcondition alert payloads refer to plot-like outputs by index. Static source scanning can estimate this from source order, but TradingView compiler behavior is the final authority, so the CLI must avoid presenting the result as guaranteed.
  Date/Author: 2026-04-28 / Codex.

## Outcomes & Retrospective

The command was implemented as local static discovery. It gives a safe next building block for PR #112 without exposing raw saved-script ids, webhook payloads, or account mutation. What remains is a future account-safe dry-run design that can combine these local source candidates with user-selected saved-script metadata and alert endpoint readback.

## Context and Orientation

The CLI is a Rust binary named `tv`. Command-line arguments are declared in `src/cli.rs`, command dispatch lives in `src/main.rs`, and operation logic is grouped under `src/ops/`. Pine-specific operations are exported through `src/ops/pine.rs` and currently implemented in `src/ops/pine/analysis.rs`, `src/ops/pine/check.rs`, and `src/ops/pine/editor.rs`.

`alertcondition()` is a Pine function that declares an alertable condition in an indicator script. TradingView internally indexes alertable outputs together with other plot-like functions. For this first slice, "candidate id" means a best-effort string such as `plot_2` derived from source order. It is only a discovery aid. The command must not create an alert, must not require TradingView Desktop, and must not store raw account metadata.

## Plan of Work

First, extend `src/ops/pine/analysis.rs` with a public function `pine_alertconditions(source: &str, input_source: &str) -> serde_json::Value`. The function scans source text after stripping comments and string literal contents for call discovery. It counts plot-like calls in source order and emits one candidate per `alertcondition(...)` call. Each candidate includes line number, column, title if a literal title can be extracted, message if a literal message can be extracted, `alert_cond_id`, `plot_index`, counted preceding outputs, and `confidence: "best_effort"`. The top-level payload includes `input_source`, `candidate_count`, `counted_output_count`, `candidates`, and a note that TradingView compile/runtime validation remains required.

Second, add `Alertconditions { file: Option<PathBuf> }` to `PineCommand` in `src/cli.rs`, dispatch it in `src/main.rs` through the existing `read_pine_source` helper, and export `pine_alertconditions` from `src/ops/pine.rs` and `src/ops.rs`.

Third, add unit tests in `src/ops/pine/analysis.rs` and CLI contract tests in `tests/cli_contract.rs`. The new tests should prove the command runs without CDP, rejects missing source before connecting, emits expected `plot_<N>` ids, ignores commented-out calls, and rejects no source in the same way as `pine analyze`.

Fourth, update stable documentation: `README.md`, `CHANGELOG.md`, `docs/internal-tradingview-apis.md`, `docs/notes/upstream-pr-recheck-2026-04-27.md`, `docs/notes/rust-cli-contract-migration-2026-04-24.md`, `docs/notes/next-agent-handoff-prompt-2026-04-24.md`, and `docs/v0.3-roadmap.md`. The docs must explain that `tv pine alertconditions` is local static discovery, not alert creation, and that raw indicator-alert mutation remains deferred until a later dry-run design is ready.

## Concrete Steps

Work from the repository root.

1. Edit `src/ops/pine/analysis.rs`, `src/ops/pine.rs`, `src/ops.rs`, `src/cli.rs`, `src/main.rs`, and `tests/cli_contract.rs`.
2. Edit the docs named in the plan.
3. Run:

    cargo test pine_alertcondition -- --nocapture
    cargo test --test cli_contract pine -- --nocapture
    cargo fmt --check
    cargo clippy --all-targets --all-features -- -D warnings
    cargo test
    git diff --check
    git grep -nE '(/Users/|C:\\|USER;|sessionid|cookie|authorization|bearer|webhook|web_hook)' -- README.md CHANGELOG.md docs .agents/skills packaging scripts || true

4. Run a local smoke with a temporary file under `target/`, for example:

    cargo run --quiet -- pine alertconditions --file target/pine-alertconditions-smoke.pine

The smoke output should be a success envelope with `command: "pine"` and a `data.candidates` array. It should not require a running TradingView Desktop session.

Validation results from this implementation:

    cargo test pine_alertcondition -- --nocapture
    result: ok. 6 filtered Pine alertcondition tests passed across unit and CLI contract tests.

    cargo test --test cli_contract pine -- --nocapture
    result: ok. 16 passed.

    cargo fmt --check
    result: ok.

    cargo clippy --all-targets --all-features -- -D warnings
    result: ok.

    cargo test
    result: ok. 339 unit tests and 83 CLI contract tests passed.

    git diff --check
    result: ok.

    credential/local-path grep
    result: existing policy text and historical validation-command examples only; no new live ids, credentials, raw payloads, or local paths were added.

## Validation and Acceptance

The change is accepted when `tv pine alertconditions` can analyze a Pine source file or stdin without CDP, returns at least one candidate for a script containing `alertcondition()`, and exposes enough information for a later dry-run command to identify a candidate without requiring raw saved-script metadata.

The command must not create an alert or call TradingView. Missing source must fail with a validation error before any CDP connection attempt. The full Rust baseline must pass.

## Idempotence and Recovery

All changes are additive and safe to rerun. The local smoke uses files under `target/`, which are ignored build artifacts. If static candidate indexing proves too optimistic later, keep this command read-only and adjust `confidence` or notes rather than treating it as an account mutation contract.

## Artifacts and Notes

Important evidence from the initial read:

    src/main.rs dispatch for `PineCommand::Analyze` reads source before invoking `ops::pine_analyze`.
    docs/plans/tradingview-cli-indicator-alertcondition-feasibility.md records that raw PR #112 mutation remains deferred until discovery/dry-run.

## Interfaces and Dependencies

At the end of the plan, these interfaces should exist:

    pub fn pine_alertconditions(source: &str, input_source: &str) -> serde_json::Value;

The CLI surface should include:

    tv pine alertconditions [--file <PATH>]

No new external Rust dependencies are required.

## Open Questions

There are no unresolved critical questions for this safe discovery slice. A later indicator-alert dry-run slice must still decide how to match local source candidates to saved TradingView script metadata without exposing account-local identifiers.
