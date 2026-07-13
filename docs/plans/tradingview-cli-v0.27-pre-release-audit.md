# v0.27.0 pre-release completion and architecture audit

This ExecPlan is a living document. Keep `Progress`, `Surprises & Discoveries`,
`Decision Log`, and `Outcomes & Retrospective` current while work proceeds.
Maintain it according to `.agents/PLANS.md`.

## Purpose / Big Picture

This slice freezes v0.27 feature work and audits the promoted release scope
before release readiness. The candidate includes current-build Strategy Tester
selection, bounded selected-chart history paging for `tv range`, opt-in
screenshot render readiness, and public-safe identity for one-shot and
streaming study values.

The audit must determine whether implementation, contracts, help, stable docs,
packaged guidance, runtime skills, tests, CI gates, and module responsibilities
agree. Small local drift may be corrected here. A public behavior change,
substantial module split, or ownership move must be reported as a separate
refactor plan rather than folded into release preparation.

No new command, option, payload field, dependency, source, fallback, or version
bump belongs in this slice. Indicator search and exact search-result add remain
deferred because their positive live readiness was not reproducible.

## Progress

- [x] (2026-07-13) Completed R10 screenshot render-readiness review and commit.
- [x] (2026-07-13) Completed R11 study-value identity implementation,
  validation, review corrections, final focused re-review, and commit.
- [x] (2026-07-13) Completed the R12 documentation and runtime-skill
  consolidation pass without expanding skill Core Workflow sections.
- [x] (2026-07-13) Archived the completed R11 ExecPlan.
- [x] (2026-07-13) Created this self-contained R13 audit ExecPlan.
- [x] (2026-07-13) Made this ExecPlan current in the plan index, roadmap, work
  inventory, changelog, and continuity ledger.
- [ ] Confirm the exact v0.27 candidate diff from tag `v0.26.0` and record any
  unexpected production or dependency changes.
- [ ] Audit Strategy Tester candidate selection, no-mutation behavior,
  ambiguity handling, and public-safe context across metrics, trades, and
  equity.
- [ ] Confirm indicator-search trial code remains absent and the named local
  prototype stash remains untouched.
- [ ] Audit `tv range` paging deadlines, stopping precedence, endpoint
  coverage, discrete-bar gaps, viewport application, and selected-chart-only
  source boundary.
- [ ] Audit screenshot readiness opt-in behavior, region-scoped observation,
  timeout no-write behavior, and immediate-capture compatibility.
- [ ] Audit one-shot and streaming study-value identity, same-instance
  association, compact-input filtering, optional-field fallback, ordering,
  and dedupe behavior.
- [ ] Confirm ordinary Cargo tests remain Rust-only and the executable
  JavaScript helper fixture remains a separately managed mandatory gate with
  Node.js `24.18.0` pinned in local tooling, CI, and release workflows.
- [ ] Inspect module sizes and production/test boundaries for Strategy Tester
  selection, visible-range paging, screenshot readiness, stream values, and
  shared study-value shaping.
- [ ] Decide whether any inspected module requires a release-blocking
  refactor, a deferred maintainability note, or no action.
- [ ] Audit help, README, stable docs, packaged guidance, and runtime skills
  for source, mutation, readiness, identity, and deferred-feature boundaries.
- [ ] Inspect changed skill size and routing, validate affected skills, and
  confirm no duplicate shipped skill copy has drifted.
- [ ] Scan tracked files and public diagnostics for private values, raw live
  evidence, stale current-state wording, and machine-specific paths.
- [ ] Run focused Strategy Tester, range, screenshot, study-value, stream, and
  Desktop contract tests.
- [ ] Run the pinned-Node JavaScript contract gate separately from ordinary
  Cargo tests.
- [ ] Run formatting, strict Clippy, the complete Rust-only workspace suite,
  metadata, hygiene, package-script, workflow, guide-parity, and diff checks.
- [ ] Record an explicit architecture verdict and all small corrections in
  this plan.
- [ ] Prepare a self-contained read-only reviewer prompt, obtain independent
  review, and correct any findings before R13 closeout.

## Surprises & Discoveries

- Observation: line count alone cannot decide the v0.27 architecture result.
  Evidence: `crates/cli/src/ops/chart/visible_range.rs` and
  `crates/cli/src/ops/screenshot/render_wait.rs` include substantial
  deterministic fixtures in the same file. The audit must identify production
  boundaries and responsibilities before recommending a split.

- Observation: R11 introduced a test-tool boundary that ordinary Rust tests do
  not own.
  Evidence: the production JavaScript identity helper is executed by an ignored
  Rust fixture selected through `scripts/check-study-values-js-contract.py`.
  `mise.toml`, CI, and the release workflow pin Node.js `24.18.0`, while
  ordinary `cargo test --workspace` succeeds without Node on `PATH`.

## Decision Log

- Decision: audit the promoted v0.27 feature scope now rather than reopen R6b
  indicator search or R7 exact-add.
  Rationale: R6b was deliberately removed after positive result readiness could
  not be reproduced. Strategy selection, range paging, screenshot readiness,
  and study identity form a coherent current-build correctness release without
  claiming unsupported discovery behavior.
  Date/Author: 2026-07-13 / Codex

- Decision: treat the pinned-Node JavaScript contract as a required separate
  gate, not as part of the ordinary Cargo baseline.
  Rationale: execution-level coverage is necessary for the generated helper,
  but Rust-only development and release-target tests must not acquire an
  undeclared Node prerequisite.
  Date/Author: 2026-07-13 / Codex

- Decision: preserve existing reviewed live evidence unless deterministic
  inspection exposes a contradiction.
  Rationale: this audit should verify durable contracts and regression tests.
  It should not mutate a TradingView layout or repeat account-bearing probes
  merely to reproduce evidence already reviewed green.
  Date/Author: 2026-07-13 / Codex

## Outcomes & Retrospective

R13 planning is complete. Implementation, architecture inspection, complete
validation, and independent review remain. Release readiness must not begin
until this section records one explicit verdict: no release-blocking issue,
small corrections applied, or a larger refactor required before release.

## Context and Orientation

The latest release is `v0.26.0`, and the workspace package version remains
`0.26.0`. v0.27 has four promoted implementation areas.

Strategy Tester reads are owned by `crates/cli/src/ops/data/strategy.rs` and
`crates/cli/src/ops/data/strategy_selection.rs`. They select a report-bearing
strategy through current semantic metadata and return shared public-safe
context. They must not open the Strategy Tester panel, change study visibility,
or silently choose an ambiguous candidate.

Selected-chart history paging is split between I/O-free decisions in
`crates/model/src/visible_range.rs` and the Desktop adapter in
`crates/cli/src/ops/chart/visible_range.rs`. The existing `tv range --from
--to` operation requests more main-series history under finite controls and
then applies only a valid discrete-bar intersection. It must not call Desktop-
free bars, OHLCV export, Replay, or another source as fallback.

Screenshot readiness is owned by `crates/cli/src/ops/screenshot.rs` and
`crates/cli/src/ops/screenshot/render_wait.rs`. The new wait is opt-in. It
requires three stable region-relevant observations under one finite deadline
and writes no image after timeout. Immediate capture remains the default.

Study-value identity is shared by `crates/cli/src/ops/data/study_values.rs`,
the one-shot adapter in `crates/cli/src/ops/data/indicator.rs`, and the stream
adapter in `crates/cli/src/ops/stream.rs`. Both surfaces add `entity_id`,
`short_name`, `study_kind`, compact `inputs`, and `visible` while preserving
their intentionally different existing value readers. Optional metadata must
never erase a value row. The generated JavaScript helper has an executable
synthetic fixture outside the ordinary Cargo baseline.

R5 and R6 established an indicator-search contract and limited feasibility.
R6b trial implementation was removed when live positive-result readiness was
not reproducible. Its recovered local prototype remains in the named stash
`recovered-indicator-search-prototype-2026-07-12`; this audit must not apply,
drop, or inspect private values from that stash.

## Plan of Work

First, make this plan current without changing product behavior. Confirm the
candidate commit range and classify every v0.27 production, test, workflow,
documentation, and skill change by its owning release area. Verify that no
deferred search implementation survived removal.

Second, inspect each promoted area end to end. Trace selection or request input
through validation, JavaScript evaluation, normalization, payload shaping,
error details, help, docs, packaged guidance, and tests. Pay particular
attention to no-mutation claims, finite deadlines, terminal precedence,
partial or ambiguous state, discrete market gaps, optional metadata failure,
and stream dedupe.

Third, inspect architecture posture. Measure file sizes, locate production and
test boundaries, and evaluate whether shared helpers own one coherent concern.
Do not infer that a large file requires a split. Record a refactor only when a
module mixes independently changing responsibilities, duplicates a contract,
or makes reviewed behavior unsafe to maintain.

Fourth, verify the two test-tool layers separately. Run ordinary Cargo tests
with Node absent from `PATH` or otherwise prove the ignored fixture is not
selected. Run `scripts/check-study-values-js-contract.py` with the pinned Node
version and verify CI and release workflows install that exact version before
the dedicated job.

Fifth, audit documentation and skills. Stable docs must distinguish Desktop-
backed operations from Desktop-free reads, describe screenshot readiness and
study identity conservatively, and keep search/add deferred. Skill Core
Workflow sections must remain concise; detailed interpretation belongs in
references. Apply only small corrections with clear evidence.

Finally, run focused and complete validation, write the architecture verdict,
and prepare a temporary self-contained reviewer prompt. Stop with audit changes
uncommitted until independent review and any focused re-review are green.

## Concrete Steps

Run commands from the repository root. Inspect the candidate and architecture:

    git diff --stat v0.26.0..HEAD
    git diff --name-status v0.26.0..HEAD
    find crates/cli/src crates/model/src -type f -name '*.rs' -print0 | xargs -0 wc -l | sort -nr | head -40
    rg -n "strategy_context|visible_range|render_wait|study_values|tvStudyValueIdentity" crates/cli/src crates/model/src
    git stash list

Run focused behavior and contract tests:

    cargo test -p tradingview-cli strategy -- --nocapture
    cargo test -p tradingview-cli visible_range -- --nocapture
    cargo test -p tradingview-cli screenshot -- --nocapture
    cargo test -p tradingview-cli study_values -- --nocapture
    cargo test -p tradingview-cli stream -- --nocapture
    cargo test -p tradingview-cli --test cli_contract_desktop -- --nocapture
    python3 scripts/check-study-values-js-contract.py

Run the complete baseline:

    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1
    python3 scripts/check-public-hygiene.py --self-test
    python3 scripts/check-public-hygiene.py
    bash -n scripts/stage-release-package-files.sh
    cmp -s AGENTS.md CLAUDE.md
    git diff --check

Validate the affected runtime skills with the portable validator under
`${CODEX_HOME:-$HOME/.codex}/skills/.system/skill-creator/scripts/quick_validate.py`.
At minimum validate `chart-analysis`, `market-data-interpretation`, and
`strategy-report`. Determine any additional changed skill from the actual
`v0.26.0..HEAD` diff rather than guessing.

## Validation and Acceptance

The audit is acceptable only when each promoted feature has the same meaning
in implementation, tests, help, stable docs, packaged guidance, and runtime
skills. Strategy reads must remain non-mutating and ambiguity-safe. Range
paging must be finite and selected-chart-only. Screenshot timeout must write no
file. Optional study identity failure must preserve existing rows, and
same-name instances must be distinguishable when identity is available.

Ordinary `cargo test --workspace` must remain Rust-only. The separately
managed JavaScript contract gate must execute the production helper with exact
Node.js `24.18.0`, and CI plus release workflows must require that gate.

The audit must state whether architecture is release-ready. A small correction
is allowed only when it does not add or change a public contract. Any larger
refactor, source-boundary change, or payload change blocks release readiness
and requires its own ExecPlan. Public docs and diagnostics must contain no raw
live payload, raw report values, credentials, session IDs, account-local IDs,
raw target IDs, or machine-specific absolute paths.

## Idempotence and Recovery

All inspections and tests are non-destructive and repeatable. Do not run a live
mutation or apply/drop the preserved stash. Documentation corrections should
be small and made with `apply_patch`. Build artifacts may be regenerated safely.
Do not commit audit changes until independent review and focused corrections
are complete.

## Artifacts and Notes

The reviewed R11 plan is archived at
`docs/plans/archives/tradingview-cli-study-value-identity.md`. The preserved
indicator-search prototype remains outside the tracked tree in the named stash
`recovered-indicator-search-prototype-2026-07-12`.

Do not retain a point-in-time reviewer prompt after review. Record durable
review criteria, findings, corrections, and outcomes in this ExecPlan.

## Interfaces and Dependencies

This audit adds no public interface and no production or development
dependency. Node.js `24.18.0` remains a pinned test tool for one separately
managed JavaScript contract gate; it is not a runtime dependency and not a
prerequisite for ordinary Cargo tests.

The audit may edit tests or documentation only to correct clear drift without
changing semantics. If production code must change, record the blocker and
create a dedicated plan before implementation.

## Open Questions

The audit verdict is open. It must conclude one of the following: no release-
blocking architecture issue, small corrections applied with no public behavior
change, or a larger refactor required before release readiness.

Revision note (2026-07-13): Created after R11 final focused review reported no
findings and the R12 documentation/runtime-skill consolidation pass completed.
R6b indicator search and R7 exact-add remain explicitly deferred.
