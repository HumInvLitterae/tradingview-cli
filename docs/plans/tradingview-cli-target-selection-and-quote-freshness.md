# Improve target selection and quote freshness

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document follows `.agents/PLANS.md` from the repository root. It is self-contained so a reader can understand and continue the work without chat history.

## Purpose / Big Picture

Agents and downstream wrappers sometimes run `tv` when multiple TradingView chart targets are open. Today, the CLI can report `target_ambiguous`, but the most discoverable remedy is hidden in documentation: set `TV_CDP_TARGET_ID`. This causes agents to make poor choices such as closing tabs. Also, `tv quote <SYMBOL>` temporarily switches the chart and restores it, which can return stale quote data if TradingView has not refreshed the chart-model quote before the read. After this change, users can select a target directly with `tv --target-id <ID> ...`, ambiguous errors tell them exactly how to proceed, and symbol-targeted quote reads fail safely when the returned quote does not match the requested symbol.

## Progress

- [x] (2026-04-28 00:20Z) Inspected `src/cli.rs`, `src/main.rs`, `src/transport.rs`, `src/ops/market.rs`, `src/ops/tab.rs`, README, and runtime skills to confirm the current target-selection and quote paths.
- [x] (2026-04-28 00:25Z) Created this ExecPlan and recorded the decision to keep `TV_CDP_TARGET_ID` only as a v0.2.x fallback while making `--target-id` the primary user-facing handoff.
- [x] (2026-04-28 00:45Z) Implemented global `--target-id`, enriched target ambiguity hints, and tab handoff payloads.
- [x] (2026-04-28 01:05Z) Hardened `tv quote <SYMBOL>` freshness checks and added a non-mutating scanner REST quote path before chart-switch fallback.
- [x] (2026-04-28 01:20Z) Updated help, README, stable docs, runtime skills, CHANGELOG, and `CONTINUITY.md`.
- [x] (2026-04-28 01:35Z) Ran focused tests, skill validation, live smoke, full validation baseline, and tracked-doc safety grep. Ready to commit.

## Surprises & Discoveries

- Observation: `tv quote --help` and `tv symbol --help` currently show only `[SYMBOL]` with no usage semantics.
  Evidence: `cargo run --quiet -- quote --help` and `cargo run --quiet -- symbol --help` both printed only the short about line, usage, and empty argument description.
- Observation: `README.md` and `chart-analysis` already mention `TV_CDP_TARGET_ID`, so the problem is not total absence of documentation. The problem is that the discoverable CLI help and error payload do not lead agents toward the safe target handoff.
  Evidence: `README.md` has a paragraph under Quick Start describing `tv tab list` plus `TV_CDP_TARGET_ID`; `.agents/skills/chart-analysis/SKILL.md` mentions it in readiness step 3.
- Observation: The existing scanner REST endpoint can provide a non-mutating quote-like payload for US symbols.
  Evidence: a read-only scanner request for `NASDAQ:AAPL` returned one row with symbol, description, close, open, high, low, volume, change, exchange, type, and subtype. `cargo run --quiet -- quote NASDAQ:AAPL` then returned `source: "scanner_scan_rest"`, `non_mutating: true`, and `freshness_check.passed: true` without requiring a CDP target.
- Observation: Live `--target-id` smoke worked against the currently open chart without changing tabs.
  Evidence: `cargo run --quiet -- tab list` returned one chart target with `target_cli_args`; `cargo run --quiet -- --target-id <target> status` returned `target_selected_by: "cli_option"`; `cargo run --quiet -- --target-id <target> quote` returned `source: "chart_api"` for the current chart; `cargo run --quiet -- --target-id <target> ohlcv --count 1` returned one bar.

## Decision Log

- Decision: Add `--target-id` as a global option rather than adding per-command target flags.
  Rationale: CDP target selection affects every command that connects to a TradingView target, not only `quote`. A global option avoids repeated command-specific flags and keeps the CLI model simple.
  Date/Author: 2026-04-28 / Codex.
- Decision: Keep `TV_CDP_TARGET_ID` in this slice but demote it from the primary documented path.
  Rationale: Environment handoff is still useful for scripts and existing skills, but the v0.3.0 roadmap should decide whether to remove it or make it a hidden fallback. New docs and skills should prefer `--target-id`.
  Date/Author: 2026-04-28 / Codex.
- Decision: Do not add `tv symbol --set` in this slice.
  Rationale: The intended command shape is already `tv symbol [SYMBOL]`. The immediate issue is poor help text and error guidance, not a missing command surface.
  Date/Author: 2026-04-28 / Codex.
- Decision: Promote scanner REST to the first `tv quote <SYMBOL>` path.
  Rationale: It returns the practical quote fields needed for common symbol checks without touching the current TradingView chart. If it is unavailable before chart mutation, the existing chart-switch fallback remains available with a freshness check.
  Date/Author: 2026-04-28 / Codex.

## Outcomes & Retrospective

Implemented. Users can now run `tv --target-id <ID> ...` instead of setting `TV_CDP_TARGET_ID`, and `tv tab list` / `tv tab switch` provide `target_cli_args` for that workflow while retaining `target_env` as a v0.2.x fallback. `tv quote <SYMBOL>` now prefers a non-mutating scanner REST read and falls back to chart switching only when necessary. The chart-switch fallback fails if the observed quote symbol does not match the requested symbol, which prevents stale data from being reported as success. Focused tests, skill validation, live smoke, full Rust validation, and whitespace checks passed. The tracked-doc safety grep returned only existing policy text and validation-command examples; no new local paths, account-local ids, cookies, tokens, or authorization values were found in the changed docs.

## Context and Orientation

`src/cli.rs` defines the clap command-line interface. `src/main.rs` parses the CLI and dispatches commands. `src/transport.rs` reads `TV_CDP_HOST`, `TV_CDP_PORT`, and `TV_CDP_TARGET_ID`, fetches Chrome DevTools Protocol targets from `/json/list`, and chooses the chart target. A "target" is a debuggable page exposed by TradingView Desktop through Chrome DevTools Protocol. A "chart target" is the specific page that owns a chart.

`src/ops/tab.rs` implements `tv tab list` and `tv tab switch`, which already return `target_env.TV_CDP_TARGET_ID` as a handoff for follow-up commands. This plan keeps that field for now but adds `target_cli_args: ["--target-id", "<ID>"]` as the new primary handoff.

`src/ops/market.rs` implements `tv quote`, `tv symbol`, and `tv ohlcv`. `tv quote <SYMBOL>` currently reads the original chart symbol, optionally calls `chart.setSymbol(requested)`, reads quote data, then restores the original symbol. It reports `switch_performed` and `restored`, but those fields do not prove the quote data was fresh for the requested symbol.

## Plan of Work

First, update the CLI and transport boundary. Add `target_id: Option<String>` to the root `Cli` struct in `src/cli.rs` with `#[arg(long, global = true)]`. Add `TransportConfig::from_env_with_target_id` in `src/transport.rs` so `--target-id` overrides `TV_CDP_TARGET_ID`. Track whether the selected target id came from `cli_option` or `env`, and expose that in `status`.

Second, thread the parsed config through `src/main.rs`. Build one `TransportConfig` after parsing and pass it into `dispatch`, `run_stream_command`, and `connect_runtime`. This ensures every command uses the same target selection precedence.

Third, enrich target handoff. In `src/transport.rs`, update `target_ambiguous` details so each candidate includes `target_cli_args` and `target_env`, plus a `next_action_hint` that starts with `tv --target-id <ID> <command>`. In `src/ops/tab.rs`, add `target_cli_args` to chart tabs, Screener targets, and `tab switch` output while retaining `target_env`.

Fourth, harden quote freshness. In `src/ops/market.rs`, add a small helper that checks whether the requested symbol and observed quote symbol match by bare symbol. For `tv quote <SYMBOL>`, return `internal_api_unavailable` if the quote symbol does not match the request. Include `freshness_check` details. Keep current-chart `tv quote` behavior and add `source: "chart_api"` metadata.

Fifth, update user-facing guidance. Improve clap long help for `quote` and `symbol`, add README guidance for multiple target handoff, update `docs/v0.3-roadmap.md` and `docs/internal-tradingview-apis.md`, and refresh `chart-analysis` plus `multi-symbol-scan` skills to prefer `--target-id` and OHLCV freshness confirmation after chart symbol mutation.

## Concrete Steps

Run all commands from the repository root.

1. Implement the code changes in `src/cli.rs`, `src/transport.rs`, `src/main.rs`, `src/ops/tab.rs`, and `src/ops/market.rs`.
2. Update tests in `src/transport.rs`, `src/ops/tab.rs`, `src/ops/market.rs`, and `tests/cli_contract.rs`.
3. Update README, CHANGELOG, docs, skills, and `CONTINUITY.md`.
4. Run focused tests:

       cargo test transport -- --nocapture
       cargo test market -- --nocapture
       cargo test tab -- --nocapture
       cargo test --test cli_contract quote -- --nocapture
       cargo test --test cli_contract symbol -- --nocapture
       cargo test --test cli_contract tab -- --nocapture

5. Run the baseline:

       cargo fmt --check
       cargo clippy --all-targets --all-features -- -D warnings
       cargo test
       git diff --check
       git grep -nE '(/Users/|C:\\|USER;|sessionid|cookie|authorization|bearer)' -- README.md CHANGELOG.md docs .agents/skills packaging scripts || true

6. If a live TradingView Desktop session is available, run bounded smoke:

       tv tab list
       tv --target-id <target-id> status
       tv --target-id <target-id> quote
       tv --target-id <target-id> symbol NASDAQ:MU
       tv --target-id <target-id> ohlcv --count 1

## Validation and Acceptance

The change is accepted when `tv --help` shows `--target-id`, `tv quote --help` explains current-chart versus symbol-targeted quote reads, and `tv symbol --help` explains read versus set behavior. When multiple chart targets are open, an unqualified chart command should still fail with `target_ambiguous`, but the JSON details should include `next_action_hint` and `target_cli_args` for each candidate so an agent can retry without closing tabs.

`tv tab list` and `tv tab switch <INDEX>` should include both the old `target_env.TV_CDP_TARGET_ID` and the new `target_cli_args`. `tv quote <SYMBOL>` should not return success if the observed quote symbol does not match the requested symbol after chart switching. The automated tests listed above must pass, followed by the full baseline.

## Idempotence and Recovery

All code changes are additive or safety-hardening. Running the test commands repeatedly is safe. Live smoke that sets a chart symbol changes only the selected test chart; if the symbol should be restored manually, run `tv --target-id <target-id> symbol <original-symbol>`. Do not run destructive tab commands as part of this plan.

## Artifacts and Notes

The expected new ambiguous-target hint shape is:

    {
      "next_action_hint": "tv --target-id <target-id> <command>",
      "targets": [
        {
          "id": "...",
          "target_cli_args": ["--target-id", "..."],
          "target_env": {"TV_CDP_TARGET_ID": "..."}
        }
      ]
    }

Do not copy live target ids, account ids, cookies, or raw quote endpoint payloads into tracked docs.

## Interfaces and Dependencies

`TransportConfig` must support:

    pub fn from_env_with_target_id(target_id: Option<&str>) -> Result<Self, AppError>

The existing `TransportConfig::from_env()` should remain available for tests and call sites that do not have CLI args. `Cli` must expose:

    #[arg(long, global = true)]
    pub target_id: Option<String>

`tab` payloads should contain:

    "target_cli_args": ["--target-id", "<target-id>"]
    "target_env": {"TV_CDP_TARGET_ID": "<target-id>"}

## Open Questions

The direct or page-session non-mutating quote path is not yet confirmed. If no safe endpoint is found during implementation, record it as `research_only` and keep the freshness-hardened chart-switch path.
