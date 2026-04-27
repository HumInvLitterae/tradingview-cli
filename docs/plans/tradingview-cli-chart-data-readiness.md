# Improve chart data readiness diagnostics

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This plan follows `.agents/PLANS.md` from the repository root. It is self-contained so a future contributor can continue from this file without relying on chat history.

## Purpose / Big Picture

Agents and users rely on `tv ohlcv` to confirm that the visible TradingView chart has fresh bar data before making chart-based decisions. A downstream incident showed that when OHLCV failed, the failure was too opaque: the agent retried with an obsolete target-selection environment variable, tried invalid commands such as `tv interval`, truncated JSON errors with `tail`, and eventually needed user intervention even though the chart later returned data. After this change, `tv ohlcv` failures should explain whether the chart API, bars collection, or bar index state is missing, and the bundled agent guidance should tell operators how to recover with `tv tab list`, `--target-id`, `state`, and `ohlcv --count 1`.

## Progress

- [x] (2026-04-27 20:03Z) Created this ExecPlan and recorded the user-visible incident without target ids or local paths.
- [x] (2026-04-27 20:03Z) Inspected the current OHLCV implementation and CLI help contracts.
- [x] (2026-04-27 20:03Z) Added structured chart-bars readiness diagnostics to `tv ohlcv` and verified the targeted market tests.
- [x] (2026-04-27 20:03Z) Updated CLI help, README, runtime skills, packaged agent guide, changelog, and internal API reference.
- [x] (2026-04-27 20:03Z) Ran targeted tests, skill validation, full Rust tests, read-only live smoke, and hygiene checks; commit remains.

## Surprises & Discoveries

- Observation: `ohlcv_bars` previously relied on JavaScript `throw new Error(...)` for expected chart-readiness failures, which lost structured details before Rust could build an actionable error payload.
  Evidence: `cargo test market -- --nocapture` passed after replacing those throws with a readiness object that Rust maps to `internal_api_unavailable`.

- Observation: Live read-only smoke against the current active chart target showed the new diagnostic path working in the exact mixed-success shape reported downstream: `status`, `state`, and `quote` succeeded, while `ohlcv --count 1` failed with `reason: "bars_index_unreadable"`, `chart_api_available: true`, `bars_available: true`, and `size: 0`.
  Evidence: A redacted local smoke summary printed `status=true`, `state=true`, `quote=true`, and `ohlcv=failure:bars_index_unreadable`.

## Decision Log

- Decision: Do not restore `TV_CDP_TARGET_ID` support.
  Rationale: The current public target-selection contract is `tv --target-id <CDP_TARGET_ID> <command>`. Reintroducing the old environment variable would preserve the confusing path that caused part of the downstream incident.
  Date/Author: 2026-04-27 / Codex.

- Decision: Do not add a new `chart diagnose` command in this slice.
  Rationale: The immediate failure was in `ohlcv`; enriching that existing error surface and guidance is smaller and directly addresses the observed operator confusion.
  Date/Author: 2026-04-27 / Codex.

- Decision: Treat Pine Editor involvement as `UNCONFIRMED`.
  Rationale: The incident occurred while the chart eventually returned fresh OHLCV data, and current evidence points more strongly to target selection, invalid command usage, and weak `ohlcv` diagnostics.
  Date/Author: 2026-04-27 / Codex.

## Outcomes & Retrospective

Implemented. `tv ohlcv` now reports structured chart-bars readiness details for missing, unreadable, or empty bars instead of collapsing those states into a generic loading message. CLI help, README, runtime skills, and the packaged agent guide now direct agents toward full JSON error inspection, `tv tab list`, `target_cli_args` / `--target-id`, `state`, and `ohlcv --count 1`, and away from the obsolete `TV_CDP_TARGET_ID` and invalid `tv interval` patterns. A later slice added `tv info <SYMBOL>` as a Desktop-free symbol metadata read, while `tv info` without a symbol remains the current-chart metadata read.

Automated validation passed: `cargo test market -- --nocapture`, `cargo test transport -- --nocapture`, `cargo test tab -- --nocapture`, the targeted `cli_contract` filters for `ohlcv`, `info`, `symbol`, and `timeframe`, `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`, `git diff --check`, and the tracked-doc hygiene grep. Skill validation passed for `chart-analysis` and `multi-symbol-scan`; `bash -n scripts/stage-release-package-files.sh` passed. Live read-only smoke showed a remaining chart-bars readiness gap on the current active chart target, but the new diagnostic correctly exposed it as `bars_index_unreadable` while `status`, `state`, and `quote` succeeded.

## Context and Orientation

The Rust CLI is a `tv` binary. Successful commands return JSON shaped like `{ "success": true, "command": "...", "data": ... }`; failed commands return `{ "success": false, "command": "...", "error": ... }`. Chart-specific commands connect to TradingView Desktop through the Chrome DevTools Protocol, which is the local debugging protocol exposed by TradingView Desktop when launched with a remote debugging port.

Explicit target selection is done with the global option `--target-id`. `tv tab list` returns `target_cli_args`, such as `["--target-id", "<ID>"]`, that agents can reuse. The old `TV_CDP_TARGET_ID` environment variable is no longer part of the public contract.

The current OHLCV implementation is in `src/ops/market.rs`. The function `ohlcv_bars` evaluates JavaScript inside the chart page, reads the active chart API, then reads the main-series bars collection. If the bars collection is missing or empty, it currently throws a generic JavaScript error that becomes `internal_api_unavailable` without enough structured detail for an agent to know whether to reselect a target, wait, inspect `state`, or ask the user to foreground the chart.

CLI command definitions live in `src/cli.rs`. The runtime skills that ship with the release archive live in `.agents/skills/`, and the packaged user-facing agent guide is `packaging/agent/AGENTS.md`.

## Plan of Work

First, update `src/ops/market.rs` so `ohlcv_bars` returns a structured object from its JavaScript evaluation instead of throwing plain JavaScript errors for expected readiness failures. The Rust function should inspect that object. If it represents a readiness failure, return `AppError` with `ErrorKind::InternalApiUnavailable`, a concise message, and details that include `phase`, `chart_api_available`, `bars_available`, `chart_symbol`, `resolution`, `bar_index_state`, and `next_action_hint`. Preserve the current success payload fields for successful reads.

Second, add or update tests around the operation behavior. The new tests should verify missing bars and empty bars become structured failures, and that successful OHLCV output retains the existing practical fields.

Third, improve CLI help in `src/cli.rs`. `ohlcv --help` should state that it reads the current chart target's bars, that multiple chart targets require `--target-id`, and that failures should be debugged with `tab list` and `state`. At the time of this slice, `info --help` clarified current-chart symbol metadata; a later Desktop-free symbol-read slice extends it to `info [SYMBOL]`. `timeframe --help` should make clear that the command name is `timeframe`, not `interval`.

Fourth, update operator documentation. README should gain a short recovery flow near the Multiple Chart Targets section. The `chart-analysis` and `multi-symbol-scan` skills should tell agents not to use `TV_CDP_TARGET_ID`, not to truncate JSON failures when debugging, and to re-run `tab list` rather than retrying the same target endlessly when `ohlcv` fails while `quote` or `symbol` succeeds. `packaging/agent/AGENTS.md` should carry the same guidance in a shorter release-archive form.

Finally, update `CONTINUITY.md`, run validation, record outcomes in this ExecPlan, and commit the related changes as one coherent slice.

## Concrete Steps

Work from the repository root.

1. Inspect the implementation and tests:

       sed -n '520,680p' src/ops/market.rs
       rg -n "ohlcv|Timeframe|Info|TV_CDP_TARGET_ID|target_cli_args" src tests README.md .agents/skills packaging/agent docs

2. Edit `src/ops/market.rs` and `src/cli.rs`, then update tests.

3. Update `README.md`, `.agents/skills/chart-analysis/SKILL.md`, `.agents/skills/multi-symbol-scan/SKILL.md`, `packaging/agent/AGENTS.md`, this ExecPlan, and `CONTINUITY.md`.

4. Run validation:

       cargo test market -- --nocapture
       cargo test transport -- --nocapture
       cargo test tab -- --nocapture
       cargo test --test cli_contract ohlcv -- --nocapture
       cargo test --test cli_contract info -- --nocapture
       cargo test --test cli_contract symbol -- --nocapture
       cargo fmt --check
       cargo clippy --all-targets --all-features -- -D warnings
       cargo test
       git diff --check
       git grep -nE '(/Users/|C:\\|USER;|sessionid|cookie|authorization|bearer)' -- README.md CHANGELOG.md docs .agents/skills packaging scripts || true

5. Validate changed skills using the repo-local checklist in `.agents/skills/discovering-skills/references/validation-checklist.md`. If a validator script is available, use it; otherwise record static validation.

6. If TradingView Desktop is available, run read-only smoke:

       tv tab list
       tv --target-id <target> status
       tv --target-id <target> state
       tv --target-id <target> ohlcv --count 1
       tv --target-id <target> ohlcv --summary
       tv --target-id <target> quote

   Do not record the live target id in tracked docs. Use `<target>` as a placeholder.

7. Commit with a Conventional Commit message. The expected message is:

       fix(ohlcv): Add chart data readiness diagnostics

## Validation and Acceptance

The change is accepted when a missing or unreadable chart bars collection returns a structured failure with actionable details rather than a generic loading message, and when successful `ohlcv` reads keep the same practical payload fields as before. The help text and bundled skills must lead agents toward `tv tab list`, `target_cli_args`, `--target-id`, `state`, and `ohlcv --count 1`, and away from `TV_CDP_TARGET_ID` and `tv interval`. Symbol arguments to `tv info` were introduced later for Desktop-free symbol metadata and are no longer invalid.

Automated acceptance is the targeted test list plus the full Rust baseline passing. Documentation acceptance is `git diff --check`, the tracked-doc hygiene grep, and skill validation for each changed runtime skill. Live smoke is read-only and optional if TradingView Desktop is not reachable.

## Idempotence and Recovery

All automated validation commands are safe to repeat. The live smoke commands are read-only if run exactly as listed. Do not run `tv symbol`, `tv timeframe`, or any other chart mutation as part of this smoke unless the user explicitly approves mutation in a later turn. If live smoke fails because TradingView Desktop is closed or no chart target exists, record the blocker and rely on automated validation.

If the JavaScript readiness probe cannot collect a field safely, return `null` for that field rather than throwing. If the new details accidentally include target ids or local paths, remove them before committing and rerun the hygiene grep.

## Artifacts and Notes

Expected new failure details shape:

    {
      "phase": "ohlcv_bars_read",
      "chart_api_available": true,
      "bars_available": false,
      "chart_symbol": "NASDAQ:IONQ",
      "resolution": "D",
      "bar_index_state": {
        "has_first_index": false,
        "has_last_index": false,
        "first_index": null,
        "last_index": null,
        "size": null,
        "result_count": 0
      },
      "next_action_hint": "Run `tv tab list`, choose the active chart target's target_cli_args, then run `tv --target-id <ID> state` and `tv --target-id <ID> ohlcv --count 1`. Do not use TV_CDP_TARGET_ID."
    }

## Interfaces and Dependencies

Use existing dependencies only. Do not add a new crate. Keep the public command names unchanged. `tv ohlcv` continues to support `--summary` and `--count`; `tv info` remains a current-chart read; `tv timeframe` remains the command for setting or reading the chart timeframe.

The Rust operation should continue returning `serde_json::Value` on success and `AppError` on failure. Details should be JSON values built with `serde_json::json!`.

## Open Questions

No critical open questions. Pine Editor involvement remains `UNCONFIRMED` and intentionally outside this slice unless implementation reveals direct evidence.
