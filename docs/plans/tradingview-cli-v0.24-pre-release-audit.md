# v0.24.0 pre-release completion and architecture audit

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This plan follows `.agents/PLANS.md`. It is self-contained and describes the audit work needed before `v0.24.0` release readiness. It does not add new commands, options, JSON payload semantics, dependencies, or a version bump.

## Purpose / Big Picture

`v0.24.0` improved day-to-day usability: `tv launch` now uses safer macOS app launching in the normal no-path case, `tv bars` can resolve bare symbols such as `AAPL` through Desktop-free symbol search, and `tv events <SYMBOL>` now exposes scanner-backed earnings and dividend readback as `events.v1`. Before release readiness, this audit checks that these features agree across implementation, public docs, runtime skills, tests, and source boundaries.

The observable outcome is a recorded audit result that says whether release readiness can proceed. If small documentation, help, test, naming, or metadata drift is found, it is fixed here. If a larger architecture problem is found, this plan records a dedicated follow-up refactor recommendation instead of mixing that refactor into the audit.

## Progress

- [x] (2026-06-03 17:35Z) Created this ExecPlan and archived the completed `tv events` implementation plan.
- [x] (2026-06-03 17:40Z) Ran source-boundary, hygiene, TODO / panic, and architecture inspection commands.
- [x] (2026-06-03 17:45Z) Reviewed launch, bars symbol resolution, and events implementation boundaries for release-blocking drift.
- [x] (2026-06-03 18:10Z) Ran focused contract tests, full Rust baseline, runtime skill validation, and optional public-safe smoke.
- [ ] Commit the audit and roadmap documentation changes.

## Surprises & Discoveries

- Observation: The TODO / panic audit still reports the known live-smoke assertion-style `panic!` calls, the Pine template TODO string, and archived validation-command examples.
  Evidence: `rg -n "TODO|FIXME|panic!|unimplemented!|todo!" crates docs README.md AGENTS.md CLAUDE.md packaging/agent/AGENTS.md` did not show a new v0.24-specific unfinished implementation marker.

- Observation: The broad hygiene scan reports existing policy text, archived validation examples, placeholder Windows paths in launch tests, placeholder `/Users/example` URLs in target tests, and redacted `USER;redacted;script` test fixtures.
  Evidence: `rg -n '(/Users/|C:\\|USER;|sessionid|cookie|authorization|bearer|raw live payload|raw WebSocket|raw JSONL|raw bars|account-local|target id|downstream-private)' ...` produced known historical and policy hits; this audit did not identify newly added private data in current v0.24 docs or payload descriptions.

- Observation: `crates/market/src/events.rs` is 460 lines. That is acceptable for the first narrow `events.v1` slice, but it should not silently grow into a full calendar implementation.
  Evidence: `find crates/cli/src crates/market/src -type f -name '*.rs' -print0 | xargs -0 wc -l | sort -nr | head -40` shows `events.rs` below larger existing operation modules and below the split bars payload / transport modules.

## Decision Log

- Decision: Treat launch process handling as complete for `v0.24.0`.
  Rationale: The normal macOS no-path launch now uses the system app launcher, explicit `--path` remains direct spawn, and `--kill-existing` remains opt-in. The docs and packaged agent guide describe this boundary.
  Date/Author: 2026-06-03 / Codex

- Decision: Treat `tv bars` bare symbol resolution as complete for `v0.24.0`.
  Rationale: Bare symbols resolve through Desktop-free symbol search, exchange-qualified input remains the explicit override, and payloads report `requested_symbol`, `resolved_symbol`, `symbol`, and `symbol_resolution`. The implementation does not use selected-chart state, scanner quote, quote-data, Replay, or chart export as fallback.
  Date/Author: 2026-06-03 / Codex

- Decision: Treat narrow `tv events` symbol-scoped readback as complete for `v0.24.0`.
  Rationale: `tv events` is scanner fundamentals backed, source-labeled as `scanner_fundamentals_rest`, and intentionally limited to earnings and dividend field readback. It does not implement a full calendar, event ranking, recommendation, trading judgment, or chart fallback.
  Date/Author: 2026-06-03 / Codex

- Decision: Do not split `crates/market/src/events.rs` before release readiness.
  Rationale: The file is a moderate first-slice module with validation, field shaping, availability readback, and tests in one place. A split would be useful only if future work adds calendar sources, date ranges, or more event families.
  Date/Author: 2026-06-03 / Codex

## Outcomes & Retrospective

Completed. No release-blocking architecture issue was found, and no larger
refactor is recommended before `v0.24.0` release readiness.

Small documentation updates were applied by making this audit the current plan
and moving the completed `tv events` implementation plan into the archive.
Implementation behavior was not changed in this audit.

Validation passed:

- `git diff --check`
- `bash -n scripts/stage-release-package-files.sh`
- focused contract tests for launch, bars, events, CLI bars, and CLI quote
  contracts
- `cargo fmt --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace`
- `cargo metadata --no-deps --format-version 1`
- runtime skill validation for market-data interpretation, multi-symbol scan,
  and chart analysis

Public-safe smoke confirmed:

- `tv launch` reused existing CDP, did not launch a new process, did not use a
  fallback, and reported `cdp_ready: true`.
- `tv bars AAPL --timeframe 1D --count 5` returned `bars.v1`, resolved `AAPL`
  to `NASDAQ:AAPL` through `symbol_search_rest`, returned 5 bars, and reported
  complete coverage.
- `tv events NASDAQ:AAPL` returned `events.v1`, source
  `scanner_fundamentals_rest`, source category `desktop_free_read`, event count
  4, and availability `events_present`.

The next step is `v0.24.0 release readiness`.

## Context and Orientation

The repository builds a single Rust CLI binary named `tv`. Desktop-free market reads live primarily in `crates/market`, while Desktop-backed operations live under `crates/cli/src/ops` and `crates/cdp`.

In this audit, "source boundary" means the user and downstream agent can tell where the data came from and which dependencies were used. `tv bars` uses the Desktop-free TradingView bars WebSocket source. `tv events` uses scanner-backed fundamentals fields. `tv launch` is an operation that starts or reuses TradingView Desktop and then checks CDP readiness. These should not be hidden fallbacks for one another.

The relevant v0.24 features are:

- `tv launch`: command-line entrypoint in `crates/cli/src/cli.rs`, dispatch in `crates/cli/src/app/dispatch.rs`, implementation in `crates/cli/src/ops/launch.rs`.
- `tv bars` symbol resolution: market facade in `crates/market/src/bars.rs`, payload shaping in `crates/market/src/bars/payload.rs`, validation in `crates/market/src/bars/validation.rs`, and public help in `crates/cli/src/cli.rs`.
- `tv events`: CLI command in `crates/cli/src/cli.rs`, dispatch through `crates/cli/src/app/dispatch.rs`, market API in `crates/market/src/events.rs`, and typed payloads in `crates/market/src/types.rs`.

## Plan of Work

First, update the durable plan index and roadmap so `docs/plans/tradingview-cli-v0.24-pre-release-audit.md` is the current plan and the `tv events` implementation plan is archived.

Second, run the source-boundary and architecture inspections. Confirm that `tv launch` remains bounded process handling and readiness readback, `tv bars` bare-symbol resolution remains Desktop-free and bars-specific, and `tv events` remains a thin event-shaped view over scanner fundamentals fields.

Third, run docs and hygiene validation. Treat archived validation commands, existing policy text, placeholder test paths, and assertion-style live-smoke `panic!` calls as known non-blockers. Treat any newly introduced private data, raw payload, source-mixing language, or unfinished implementation marker as a blocker.

Fourth, run focused tests, full Rust baseline, runtime skill validation, and optional public-safe smoke. If optional smoke is run, record only command, source marker, resolved symbol, event count, and availability summary. Do not paste raw JSON output.

Finally, record the audit outcome. If there is no release blocker or large refactor requirement, update `docs/v0.24-roadmap.md` to point to `v0.24.0 release readiness` as the next step.

## Concrete Steps

Run these commands from the repository root.

    git diff --check
    bash -n scripts/stage-release-package-files.sh
    rg -n '(/Users/|C:\\|USER;|sessionid|cookie|authorization|bearer|raw live payload|raw WebSocket|raw JSONL|raw bars|account-local|target id|downstream-private)' README.md AGENTS.md CLAUDE.md CHANGELOG.md docs .agents/skills packaging scripts crates || true
    rg -n "TODO|FIXME|panic!|unimplemented!|todo!" crates docs README.md AGENTS.md CLAUDE.md packaging/agent/AGENTS.md
    rg -n "v0\\.24|tv launch|symbol_resolution|resolved_symbol|events\\.v1|tv events|scanner_fundamentals_rest|earnings|dividends|source mixing|ranking|recommendation" README.md CHANGELOG.md docs .agents/skills packaging/agent/AGENTS.md
    find crates/cli/src crates/market/src -type f -name '*.rs' -print0 | xargs -0 wc -l | sort -nr | head -40
    rg -n "launch|symbol_resolution|resolved_symbol|events_symbol|events\\.v1|scanner_fundamentals_rest|fundamentals_symbol" crates/cli/src crates/market/src crates/model/src crates/cdp/src

Run focused contract tests:

    cargo test -p tradingview-cli launch -- --nocapture
    cargo test -p tradingview-market bars -- --nocapture
    cargo test -p tradingview-market events -- --nocapture
    cargo test -p tradingview-cli market::bars -- --nocapture
    cargo test -p tradingview-cli events -- --nocapture
    cargo test -p tradingview-cli --test cli_contract_bars -- --nocapture
    cargo test -p tradingview-cli --test cli_contract_quote -- --nocapture

Run the full baseline:

    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1

Validate changed runtime skills:

    uvx --with pyyaml python "${CODEX_HOME:-$HOME/.codex}/skills/.system/skill-creator/scripts/quick_validate.py" .agents/skills/market-data-interpretation
    uvx --with pyyaml python "${CODEX_HOME:-$HOME/.codex}/skills/.system/skill-creator/scripts/quick_validate.py" .agents/skills/multi-symbol-scan
    uvx --with pyyaml python "${CODEX_HOME:-$HOME/.codex}/skills/.system/skill-creator/scripts/quick_validate.py" .agents/skills/chart-analysis

Optional smoke commands may be run if the local environment is suitable:

    target/debug/tv launch
    target/debug/tv bars AAPL --timeframe 1D --count 5
    target/debug/tv events NASDAQ:AAPL

## Validation and Acceptance

The audit is accepted when the focused tests and full Rust baseline pass, the runtime skills validate, no new public-doc private-data leak is found, and the audit conclusion clearly states either:

- no release-blocking architecture issue found;
- small fixes were applied; or
- a larger refactor is recommended before release readiness.

Release readiness may proceed only if there is no release blocker and no large refactor requirement.

## Idempotence and Recovery

This audit is safe to rerun. Grep commands are read-only. Tests and builds may update normal build artifacts under `target/`, but they do not change tracked source files. Optional live smoke for `tv launch` may start or reuse TradingView Desktop; do not run destructive launch variants such as `--kill-existing` in this audit.

If a validation command fails, record the failure in `Surprises & Discoveries`, fix only small drift in this plan, and rerun the relevant command. If the failure implies a larger design issue, stop and record a follow-up refactor plan recommendation.

## Artifacts and Notes

Initial architecture inspection produced these notable summaries:

    crates/cli/src/ops/launch.rs: 732 lines
    crates/market/src/types.rs: 723 lines
    crates/market/src/events.rs: 460 lines
    crates/market/src/bars/validation.rs: 403 lines

The largest files remain existing screener, alert, and CLI dispatch modules. The new `events.rs` module is not a release-blocking size concern for this first slice.

## Interfaces and Dependencies

No interfaces or dependencies are added in this audit. The relevant v0.24 public contracts remain:

    tv launch
    tv bars <SYMBOL>
    tv events <SYMBOL> --event-type <all|earnings|dividends>

The audit must not change their JSON contract except for small public-safe wording or documentation corrections.

## Open Questions

No critical open question remains. If validation passes, the next step is `v0.24.0 release readiness`.
