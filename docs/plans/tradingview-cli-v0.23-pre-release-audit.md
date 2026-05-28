# v0.23.0 pre-release architecture audit

This ExecPlan is a living document. The sections `Progress`, `Surprises &
Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up
to date as work proceeds. This document follows `.agents/PLANS.md`.

## Purpose / Big Picture

`v0.23.0` added explicit export and chart workflow maturity:

- `tv export chart-bars` for explicit Desktop-backed selected-chart bars
  export;
- `tv replay log --steps <N>` for bounded Replay JSONL step logs;
- chart-backed compare contract planning;
- standalone `tv events` feasibility planning.

This audit checks that those slices fit together without hidden source mixing,
private data leakage, a new ranking/recommendation surface, or a release-time
architecture problem. It adds no feature, option, source, dependency, payload
semantics, or version bump.

## Progress

- [x] (2026-05-28) Create this ExecPlan.
- [x] (2026-05-28) Archive the completed `tv events` feasibility plan.
- [x] (2026-05-28) Update plan index, v0.23 roadmap, and changelog for the
  pre-release architecture audit.
- [x] (2026-05-28) Inspect v0.23 source boundaries, docs, runtime skills, and
  module sizes.
- [x] (2026-05-28) Run docs validation, hygiene checks, Rust baseline,
  focused contract tests, runtime skill validation, and commit the audit
  slice.

## Surprises & Discoveries

- Observation: the largest CLI modules remain Screener, layout/watchlist,
  dispatch, alert, quote, quote-data, and selected-chart OHLCV/export code.
  Evidence: `find crates/cli/src -type f -name '*.rs' -print0 | xargs -0 wc -l
  | sort -nr | head -30` reported `ops/market/ohlcv.rs` at 697 lines,
  `app/watch.rs` at 620 lines, and larger pre-existing Screener / alert
  modules above them.

- Observation: v0.23 implementation did not create a new oversized events
  module or hidden event surface.
  Evidence: `rg` over `crates/*/src` found fundamentals still under
  `crates/market/src/fundamentals*`; `tv events` appears only in docs /
  planning context, not as a runtime command.

- Observation: `tv replay log` has a separate JSONL runner while existing
  Replay one-shot commands remain in the existing Replay operation modules.
  Evidence: `crates/cli/src/app/replay_log.rs` contains
  `replay_step_log.v1` event construction, while `ops::replay` tests still
  cover status/start/step/stop/autoplay/trade.

- Observation: `tv export chart-bars` currently lives in
  `crates/cli/src/ops/market/ohlcv.rs` with existing selected-chart OHLCV
  readback.
  Evidence: the module is under 700 lines and focused tests cover export
  metadata, summary output, and validation. No release-blocking split is
  required before `v0.23.0`.

## Decision Log

- Decision: Treat `v0.23.0` feature work as complete and move to release
  readiness next.
  Rationale: the roadmap lanes are complete for selected-chart export, Replay
  step logging, chart-backed compare planning, and events feasibility.
  Date/Author: 2026-05-28 / Codex.

- Decision: Do not do a pre-release module split.
  Rationale: no v0.23 module shows a release-blocking architecture issue.
  `ohlcv.rs`, `app/watch.rs`, `compare.rs`, and `snapshot.rs` are moderately
  large but still organized around their command families and covered by
  focused tests.
  Date/Author: 2026-05-28 / Codex.

- Decision: Keep chart-backed compare and standalone `tv events` as planning
  outcomes, not stable commands, for `v0.23.0`.
  Rationale: adding those commands now would introduce new source contracts
  after the audit and blur release readiness.
  Date/Author: 2026-05-28 / Codex.

## Outcomes & Retrospective

No release-blocking architecture issue was found. No larger refactor is
recommended before `v0.23.0` release readiness.

The audit confirmed:

- `tv export chart-bars` remains explicit Desktop-backed selected-chart export
  and is not a hidden fallback for Desktop-free `tv bars --from/--to`;
- `tv replay log --steps <N>` remains bounded Replay workflow evidence and
  does not start/stop Replay, export bars, attach screenshots, or replace
  `tv bars`;
- chart-backed compare is not a stable command; `tv compare` and
  `tv watch compare` remain Desktop-free scanner-backed workflows;
- standalone `tv events` is feasibility only; current earnings and dividends
  evidence remains scanner-backed fundamentals fields;
- docs and runtime skills continue to reject source mixing, ranking,
  recommendation, and trading judgment.

Small release-time findings were documentation/audit observations only. The
TODO / panic grep reported existing live-smoke assertions, archived validation
commands, and a Pine template TODO string; none are new release blockers.

## Context and Orientation

The release candidate now has four v0.23 lanes:

- selected-chart export command: implemented as `tv export chart-bars`;
- Replay workflow: implemented as `tv replay log --steps <N>`;
- chart-backed compare: contract planned, no stable command added;
- standalone events: feasibility planned, no stable command added.

The common theme is explicit workflow evidence. Desktop-free `tv bars`,
scanner-backed compare, selected-chart export, Replay state, and
fundamentals/event-like scanner fields must remain visibly separate.

## Plan of Work

Archive the completed events feasibility plan, create this audit plan, update
the active plan pointers, then validate the completed v0.23 surface. If a
small docs or metadata drift appears, fix it here. If a larger architecture
issue appears, record it as a follow-up refactor plan instead of mixing it
into release readiness.

## Concrete Steps

Run all commands from the repository root.

Archive the completed plan and update docs:

    git mv docs/plans/tradingview-cli-events-feasibility.md docs/plans/archives/tradingview-cli-events-feasibility.md

Inspect architecture:

    find crates/cli/src -type f -name '*.rs' -print0 | xargs -0 wc -l | sort -nr | head -30
    find crates/market/src crates/model/src crates/cdp/src -type f -name '*.rs' -print0 | xargs -0 wc -l | sort -nr | head -40
    rg -n "export_chart_bars|replay_step_log|chart-backed compare|follow_up_hints|tv events|fundamentals" crates/cli/src crates/market/src crates/model/src crates/cdp/src

Validate:

    git diff --check
    bash -n scripts/stage-release-package-files.sh
    rg -n '(/Users/|C:\\|USER;|sessionid|cookie|authorization|bearer|raw live payload|raw WebSocket|raw JSONL|raw bars|account-local|target id|downstream-private)' README.md AGENTS.md CLAUDE.md CHANGELOG.md docs .agents/skills packaging scripts crates || true
    rg -n "TODO|FIXME|panic!|unimplemented!|todo!" crates docs README.md AGENTS.md CLAUDE.md packaging/agent/AGENTS.md
    rg -n "v0\\.23|export_chart_bars\\.v1|replay_step_log\\.v1|tv export chart-bars|tv replay log|chart-backed compare|tv events|fundamentals|source mixing|ranking|recommendation" README.md CHANGELOG.md docs .agents/skills packaging/agent/AGENTS.md

Run Rust checks:

    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1

Run focused contract checks:

    cargo test -p tradingview-cli export -- --nocapture
    cargo test -p tradingview-cli ops::market::ohlcv -- --nocapture
    cargo test -p tradingview-cli ops::chart -- --nocapture
    cargo test -p tradingview-cli ops::replay -- --nocapture
    cargo test -p tradingview-cli replay -- --nocapture
    cargo test -p tradingview-cli --test cli_contract_desktop -- --nocapture
    cargo test -p tradingview-market compare -- --nocapture
    cargo test -p tradingview-market snapshot -- --nocapture

Validate runtime skills:

    uvx --with pyyaml python <skill-validator>/quick_validate.py .agents/skills/chart-analysis
    uvx --with pyyaml python <skill-validator>/quick_validate.py .agents/skills/market-data-interpretation
    uvx --with pyyaml python <skill-validator>/quick_validate.py .agents/skills/multi-symbol-scan
    uvx --with pyyaml python <skill-validator>/quick_validate.py .agents/skills/replay-practice

## Validation and Acceptance

This slice is accepted when:

- v0.23 docs, runtime skills, and tests describe the same source boundaries;
- no new hidden fallback, source mixing, ranking, recommendation, or trading
  judgment is introduced;
- public docs and packaged assets do not contain raw target ids, raw DOM, raw
  payloads, raw JSONL, raw bars, credentials, account-local metadata, or local
  validation paths;
- focused and workspace Rust checks pass;
- architecture inspection states that no release-blocking architecture issue
  exists, or records a dedicated follow-up refactor plan if one does.

## Idempotence and Recovery

This audit is safe to rerun. If the events feasibility plan is already
archived, keep it archived and update the plan index only. If a validation
command fails because of environment availability, record the command, failure
class, and whether it blocks release readiness; do not hide it in release
notes.

## Artifacts and Notes

Do not paste raw live command output, raw JSONL, raw bars, target ids, raw DOM,
raw payloads, account-local metadata, credentials, or local absolute paths into
tracked docs. Audit evidence should use high-level pass/fail summaries,
module names, line counts, source markers, and public-safe command names.

## Interfaces and Dependencies

No interface is added in this slice.

## Open Questions

None. If validation remains green, the next step is `v0.23.0 release
readiness`.

## Change Note

No runtime behavior changes in this slice.
