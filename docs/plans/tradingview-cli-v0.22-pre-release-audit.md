# v0.22.0 pre-release completion and architecture audit

This ExecPlan is a living document. The sections `Progress`, `Surprises &
Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up
to date as work proceeds. Maintain this document according to
`.agents/PLANS.md`.

## Purpose / Big Picture

`v0.22.0` added workflow-oriented readback rather than a single data source:
bounded `tv watch compare` JSONL events, selected-chart export evidence
readback, Replay evidence readback, and clearer `snapshot.v1` / `compare.v1`
follow-up hints. Before release readiness, this audit checks that those pieces
fit together without hidden source mixing, private data leakage, or a new
module boundary problem.

This audit does not add user features. Small drift in docs, help, tests, or
metadata may be fixed here. If a larger refactor is needed, this plan records
the issue and leaves a dedicated follow-up plan as the next step.

## Progress

- [x] Created this pre-release architecture audit ExecPlan.
- [x] Archived the completed evidence follow-up workflow ExecPlan.
- [x] Updated the plan index, roadmap, and changelog to make this audit the
  current `v0.22.0` step.
- [x] Inspected implementation size and boundary placement for watch, compare,
  snapshot, selected-chart OHLCV readback, and Replay readback.
- [x] Ran audit grep commands for contracts, private-data hygiene, and
  unfinished-code markers.
- [x] Ran focused contract tests, workspace baseline, packaging syntax check,
  and runtime skill validation.
- [x] Recorded audit conclusions and next-step recommendation.

## Surprises & Discoveries

- Observation: `crates/cli/src/app/watch.rs` is not a release blocker, but it
  now contains validation, polling loop control, event construction, and tests
  in one file.
  Evidence: the file is about 620 lines, smaller than several existing
  operation adapters, and `rg watch_compare` shows the new contract remains
  localized to the watch app module and CLI help.
- Observation: `crates/market/src/compare.rs` is the largest touched
  Desktop-free market file and has some overlap with `snapshot.rs` in
  follow-up hint shaping.
  Evidence: the file is about 1229 lines while `snapshot.rs` is about 672
  lines. Both expose `follow_up_hints`, but the hint sets differ enough that
  immediate common abstraction would be premature.
- Observation: selected-chart export readback and Replay readback stayed on
  their intended sides of the repo boundary.
  Evidence: selected-chart context is in the Desktop-backed `ohlcv` operation,
  while Replay payload normalization lives in the I/O-free model module.
- Observation: unfinished-code and private-data greps did not reveal a new
  release blocker.
  Evidence: TODO / panic hits are existing live-smoke assertions, archived
  validation examples, and the Pine template placeholder. Private-data hygiene
  hits are existing policy text, validation commands, example paths, or
  redacted test fixtures.

## Decision Log

- Decision: Treat this as an architecture-aware release audit, not another
  implementation slice.
  Rationale: the `v0.22.0` feature lanes are implemented; the remaining risk is
  contract drift, source-boundary drift, or an emerging refactor need.
  Date/Author: 2026-05-27 / Codex.
- Decision: Do not split `watch.rs` or `compare.rs` before `v0.22.0`.
  Rationale: no release-blocking architecture issue was found. Both files have
  plausible future split points, but the current implementations are tested and
  localized.
  Date/Author: 2026-05-27 / Codex.
- Decision: Keep follow-up hints advisory and source-explicit.
  Rationale: `snapshot.v1` and `compare.v1` hints are possible next evidence
  checks. They must not be interpreted as automatic command dispatch, ranking,
  recommendation, or source mixing.
  Date/Author: 2026-05-27 / Codex.

## Outcomes & Retrospective

The audit found no release-blocking architecture issue. The biggest future
refactor candidates are `crates/cli/src/app/watch.rs`, if more watch commands
are added, and `crates/market/src/compare.rs`, if compare and snapshot
follow-up hint shaping grows further. Those are not blockers for `v0.22.0`.
Focused contract tests, workspace baseline, docs checks, packaging syntax, and
runtime skill validation passed.

The next step is `v0.22.0 release readiness`: version bump, changelog section,
release notes, README release asset examples, package staging, and final
release validation. Do not add new workflow features before that step unless a
new blocker is discovered.

## Context and Orientation

The `tv` CLI has Desktop-free data reads and Desktop-backed selected-chart
reads. Desktop-free means the command does not require TradingView Desktop to
be open, while Desktop-backed means the command reads or operates on the
selected TradingView Desktop chart through the Chrome DevTools Protocol.

For `v0.22.0`, the relevant surfaces are:

- `tv watch compare <SYMBOL>...`: a bounded Desktop-free JSONL workflow using
  scanner-backed quote evidence. It emits `watch_compare.v1` readiness,
  sample, heartbeat, and summary events.
- `tv ohlcv` and `tv range`: Desktop-backed selected-chart readback and
  visible-range operation diagnostics. These are not a replacement for
  browserless `tv bars --from/--to`.
- `tv replay ...`: Desktop-backed Replay status and Replay operations. Status
  is a read; start, step, stop, autoplay, and trade mutate Replay state.
- `tv snapshot` and `tv compare`: Desktop-free evidence packets with advisory
  `follow_up_hints[]` for possible next checks.

The audit focuses on `docs/plans/README.md`, `docs/v0.22-roadmap.md`,
`CHANGELOG.md`, public docs, runtime skills, `crates/cli/src/app/watch.rs`,
`crates/cli/src/ops/market/ohlcv.rs`, `crates/cli/src/ops/replay/*`,
`crates/model/src/replay.rs`, `crates/market/src/snapshot.rs`, and
`crates/market/src/compare.rs`.

## Plan of Work

First, archive the completed evidence follow-up workflow plan and create this
audit plan as the new current plan. Update the plan index and the `v0.22.0`
roadmap so all four lanes are marked complete for this release candidate and
this audit is the current step.

Second, inspect implementation boundaries. Use file-size scans and targeted
searches to find whether the new workflow logic has collected in the wrong
place. `watch.rs` may remain a single module for this release if it is still a
bounded watch adapter. `compare.rs` and `snapshot.rs` may keep separate
follow-up hint shaping unless the overlap is large enough to justify a shared
helper. Selected-chart and Replay readback should remain Desktop-backed and
must not become hidden fallbacks for `tv bars`.

Third, run contract, hygiene, focused test, workspace baseline, and runtime
skill validation. Small documentation or test drift may be fixed in this
audit. If a substantial module split or API-boundary move is needed, do not
perform it here; record it in this plan and recommend a dedicated next step.

## Concrete Steps

Run commands from the repository root.

Inspect the architecture shape:

    find crates/cli/src -type f -name '*.rs' -print0 | xargs -0 wc -l | sort -nr | head -30
    find crates/market/src crates/model/src crates/cdp/src -type f -name '*.rs' -print0 | xargs -0 wc -l | sort -nr | head -40
    rg -n "watch_compare|follow_up_hints|replay_context|chart_context|selected_chart_range_match" crates/cli/src crates/market/src crates/model/src crates/cdp/src

Run docs and hygiene checks:

    git diff --check
    bash -n scripts/stage-release-package-files.sh
    rg -n '(/Users/|C:\\|USER;|sessionid|cookie|authorization|bearer|raw live payload|raw WebSocket|raw JSONL|raw bars|account-local|target id|downstream-private)' README.md AGENTS.md CLAUDE.md CHANGELOG.md docs .agents/skills packaging scripts crates || true
    rg -n "TODO|FIXME|panic!|unimplemented!|todo!" crates docs README.md AGENTS.md CLAUDE.md packaging/agent/AGENTS.md
    rg -n "v0\\.22|watch_compare\\.v1|follow_up_hints|auto_execute|evidence_role|snapshot\\.v1|compare\\.v1|selected-chart|tv ohlcv|tv range|Replay|replay_context|source mixing|ranking|recommendation" README.md CHANGELOG.md docs .agents/skills packaging/agent/AGENTS.md

Run focused tests:

    cargo test -p tradingview-cli watch -- --nocapture
    cargo test -p tradingview-cli ops::market::ohlcv -- --nocapture
    cargo test -p tradingview-cli ops::chart -- --nocapture
    cargo test -p tradingview-cli ops::replay -- --nocapture
    cargo test -p tradingview-market snapshot -- --nocapture
    cargo test -p tradingview-market compare -- --nocapture
    cargo test -p tradingview-cli --test cli_contract_quote -- --nocapture
    cargo test -p tradingview-cli --test cli_contract_desktop -- --nocapture
    cargo test -p tradingview-cli --test live_snapshot
    cargo test -p tradingview-cli --test live_compare

Run the baseline:

    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1

Validate changed runtime skills:

    uvx --with pyyaml python <skill-validator>/quick_validate.py .agents/skills/market-data-interpretation
    uvx --with pyyaml python <skill-validator>/quick_validate.py .agents/skills/multi-symbol-scan
    uvx --with pyyaml python <skill-validator>/quick_validate.py .agents/skills/chart-analysis
    uvx --with pyyaml python <skill-validator>/quick_validate.py .agents/skills/replay-practice

Optional live smokes may be run only as public-safe summaries:

    target/debug/tv watch compare NASDAQ:AAPL NASDAQ:MSFT --duration-ms 10000 --interval 2000 --heartbeat-ms 3000
    target/debug/tv readiness
    target/debug/tv state
    target/debug/tv ohlcv --count 100
    target/debug/tv replay status

Do not paste raw JSONL, raw bars, target ids, local paths, account-local
metadata, credentials, or raw payloads into tracked docs.

## Validation and Acceptance

Acceptance requires the audit to state one of the following clearly:

- no release-blocking architecture issue was found;
- small fixes were applied and verified; or
- a larger refactor is recommended before release readiness.

For this audit, the expected outcome is that no release-blocking architecture
issue is found, while future refactor candidates are recorded as non-blocking.
All focused tests, workspace baseline, docs validation, packaging syntax check,
and runtime skill validation should pass.

`docs/v0.22-roadmap.md` should show all v0.22 lanes complete for this release
candidate and point to this audit as the current plan. `docs/plans/README.md`
should list this audit as current and the evidence follow-up workflow plan as
recently completed.

## Idempotence and Recovery

This audit is safe to rerun. If a command fails, fix only the smallest
release-blocking drift needed to make the current contracts true. Do not add a
new command, option, source, dependency, or version bump in this slice.

If a large refactor appears necessary, stop after recording the finding in this
plan and create a follow-up plan in the next step. Do not mix that refactor
with this audit.

## Artifacts and Notes

Architecture inspection summary:

    crates/cli/src/app/watch.rs is about 620 lines and remains localized to
    bounded watch request validation, polling, event construction, and tests.
    This is acceptable for v0.22.0, but it should be split if additional watch
    subcommands are added.

    crates/market/src/compare.rs is about 1229 lines and remains the largest
    touched market file. Its follow-up hint shaping overlaps with
    snapshot.rs, but immediate common abstraction would be premature because
    compare and snapshot hint sets differ.

    selected-chart readback is contained in Desktop-backed operation code, and
    Replay payload normalization is contained in model code. No hidden fallback
    from selected-chart or Replay surfaces to Desktop-free bars was found.

## Interfaces and Dependencies

This audit must not change public command behavior or add new stable payload
semantics. The existing relevant contract markers are `watch_compare.v1`,
`snapshot.v1`, and `compare.v1`. Replay and selected-chart readback remain
source diagnostics, not export contracts.

No new dependency is allowed. No release version bump is allowed in this audit.

## Open Questions

There are no critical open questions for this audit. The only follow-up
judgment is whether future growth should split `watch.rs` or factor shared
follow-up hint helpers. The current answer is: not before `v0.22.0` release
readiness unless new test or architecture evidence appears.

## Revision Note

Created on 2026-05-27 to turn the completed v0.22 workflow lanes into a
release-readiness gate that explicitly includes architecture and module-size
review.
