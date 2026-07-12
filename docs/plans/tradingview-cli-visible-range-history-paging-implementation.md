# Implement bounded visible-range history paging

This ExecPlan is a living document. Keep `Progress`, `Surprises & Discoveries`,
`Decision Log`, and `Outcomes & Retrospective` current while work proceeds.
Maintain it according to `.agents/PLANS.md`.

## Purpose / Big Picture

After this work, `tv range --from <UNIX_SECONDS> --to <UNIX_SECONDS>` can ask
the selected TradingView Desktop chart for older main-series history before it
moves the viewport. A request older than the bars currently loaded will no
longer clamp silently. The JSON response will state how many history requests
were made, why paging stopped, whether loaded endpoints cover the request,
whether matching bars exist, and whether the viewport was applied, clamped, or
left unchanged.

This is the R9 implementation of the reviewed R8 contract archived at
`docs/plans/archives/tradingview-cli-visible-range-history-paging-contract.md`.
It changes only the bounded `tv range --from/--to` setter and the enriched
range-operation object embedded by `tv export chart-bars`. The no-argument
`tv range` getter remains a non-mutating read. No new command, option, source,
dependency, or version bump is added.

## Progress

- [x] (2026-07-13) Completed and independently reviewed the R8 contract.
- [x] (2026-07-13) Created this separate R9 implementation ExecPlan and made it
  the active v0.27 plan without changing Rust behavior.
- [ ] Add shared I/O-free range validation and paging policy models.
- [ ] Add deterministic model tests for validation, state transitions,
  precedence, coverage, matching bars, and viewport application.
- [ ] Refactor the selected-chart adapter into inspect, request, observe, apply,
  and readback operations under one absolute paging deadline.
- [ ] Move setter validation before Desktop connection while retaining direct
  caller defense and export validation compatibility.
- [ ] Add CLI operation and contract tests, including proof that invalid ranges
  do not connect to Desktop.
- [ ] Run a bounded public-safe live smoke on a dedicated chart and restore the
  original visible range.
- [ ] Update help, stable docs, packaged guidance, and only affected runtime
  skill references.
- [ ] Run focused and complete validation.
- [ ] Obtain independent implementation review and correct findings before
  closeout.

## Surprises & Discoveries

- Observation: current Rust range selection uses loaded index values as both
  data and “unset” sentinels.
  Evidence: `crates/cli/src/ops/chart.rs::set_visible_range` initializes
  `fromIdx = startIdx` and updates it while `fromIdx === startIdx`, so an exact
  earliest-bar match can be overwritten. Requests wholly outside loaded
  history can leave both defaults and zoom to the whole loaded range.

- Observation: the local upstream implementation confirms the current-build
  `mainSeries().requestMoreData(1000)` path but is not production quality for
  this repository.
  Evidence: it uses fixed 1800 ms sleeps, catches and ignores internal API
  exceptions, and reports no request count, timeout, no-progress, clamp, or
  coverage diagnostics.

- Observation: timestamp endpoint coverage does not imply a selectable bar.
  Evidence: weekend, market-closure, halt, and intraday session gaps can fall
  within loaded earliest/latest timestamps while containing zero bar
  timestamps. R9 must keep coverage and matching-bar evidence separate.

## Decision Log

- Decision: implement the reviewed R8 contract without reopening public
  vocabulary or timeout policy during coding.
  Rationale: three review rounds resolved right-edge partial coverage,
  terminal precedence, validation placement, index clamping, deadline scope,
  and discrete-bar gaps. Scope changes require updating this living plan before
  code changes continue.
  Date/Author: 2026-07-13 / Codex

- Decision: keep all history requests sequential on one
  `&mut RuntimeEvaluator`.
  Rationale: selected-chart state has one ownership order. Concurrent CDP
  evaluation or background paging would make request/result attribution and
  restoration harder without user benefit.
  Date/Author: 2026-07-13 / Codex

- Decision: use an I/O-free model state machine and keep JavaScript limited to
  current chart observations and one explicit operation.
  Rationale: policy, precedence, and payload shaping need deterministic Rust
  fixtures rather than large JavaScript expressions or timing-dependent tests.
  Date/Author: 2026-07-13 / Codex

- Decision: preserve the recovered indicator-search prototype stash untouched.
  Rationale: it is unrelated deferred work and was retained at the owner's
  request. R9 must not apply, drop, rewrite, or include it.
  Date/Author: 2026-07-13 / Codex

## Outcomes & Retrospective

R9 is planned but not implemented. The outcome will be recorded after focused
tests, live evidence, full validation, and independent review. If current
TradingView Desktop cannot provide bounded observable progress through the
reviewed internal API boundary, stop implementation, preserve any trial in a
named stash after owner confirmation, and record no-go rather than weakening
the contract.

## Context and Orientation

`crates/cli/src/app/dispatch.rs` routes `Command::Range`. With no bounds it
connects and calls `crates/cli/src/ops/chart.rs::visible_range`. With both
bounds it currently connects first and then calls `set_visible_range`. R9 must
validate the bounded form before `connect_runtime`.

`crates/cli/src/ops/chart.rs::set_visible_range` currently validates finite
numbers, reads `mainSeries().bars()`, searches the loaded bar indices, calls
`timeScale().zoomToBarsRange`, waits 500 ms inside the evaluated promise, and
returns `requested` and `actual`. It neither requests history nor states that a
request was clamped.

`crates/cli/src/ops/market/ohlcv.rs::export_chart_bars` validates its request,
calls `set_visible_range`, then reads selected-chart OHLCV. R9 enriches the
embedded `range_operation`; export remains `export_chart_bars.v1` and does not
gain another source or a completeness guarantee.

The operation's source is the selected chart main series. Do not call
Desktop-free `tv bars`, selected-chart `tv ohlcv` as a paging source, Replay,
scanner endpoints, screenshots, or export recursively. “Coverage” means the
loaded earliest/latest timestamp endpoints cover both requested endpoints.
“Matching bars” means discrete loaded bar timestamps actually lie inside the
requested/loaded intersection. These are independent facts.

## Required Contract

The command form remains:

    tv range --from <UNIX_SECONDS> --to <UNIX_SECONDS>

Both values must be finite and `from < to`. Invalid input returns the existing
validation envelope with exit code 2 before Desktop connection. The bounded
setter's success and error metadata use `source: "chart_api"`,
`source_category: "desktop_backed_operation"`, `requires_desktop: true`, and
`non_mutating: false`. The no-argument getter remains
`desktop_backed_read` and `non_mutating: true` with no paging object.

Preserve `operation`, `source`, `requires_desktop`, `requested`, and `actual`.
Add `viewport_application` and `history_paging`:

    {
      "operation": "visible_range",
      "source": "chart_api",
      "source_category": "desktop_backed_operation",
      "requires_desktop": true,
      "non_mutating": false,
      "requested": { "from": 1700000000, "to": 1701000000 },
      "actual": { "from": 1700000000, "to": 1701000000 },
      "viewport_application": {
        "status": "applied",
        "applied": true,
        "clamped": false,
        "matching_bar_count": 12,
        "applied_range": { "from": 1700000000, "to": 1701000000 }
      },
      "history_paging": {
        "request_size": 1000,
        "request_limit": 25,
        "request_count": 2,
        "deadline_ms": 30000,
        "elapsed_ms": 1840,
        "earliest_loaded_before": 1700500000,
        "earliest_loaded_after": 1699000000,
        "latest_loaded_after": 1702000000,
        "coverage_status": "complete",
        "stop_reason": "coverage_reached",
        "history_exhausted": false,
        "limit_reached": false,
        "timed_out": false
      }
    }

Stable `coverage_status` values are `complete`, `partial`, and `unavailable`.
Stable `stop_reason` values are `paging_not_needed`, `coverage_reached`,
`history_exhausted`, `no_progress`, `request_limit_reached`, and
`deadline_elapsed`. Stable viewport statuses are `applied`,
`applied_clamped`, `unchanged_no_overlap`, and
`unchanged_no_matching_bars`.

`limit_reached` is true only when `request_limit_reached` stopped paging.
`timed_out` is true only when `deadline_elapsed` stopped paging. Endpoint
coverage may be complete while `matching_bar_count` is zero. Do not infer bar
presence from coverage.

## Paging State Machine

Use a request size of 1000 bars, a request limit of 25, a 200 ms observation
interval, a 3-second per-request progress window, and one absolute 30-second
paging deadline. These are internal constants, not CLI options.

Create the deadline and `elapsed_ms` start instant immediately before initial
loaded-range inspection. Include initial inspection, requests, progress
observations, and the last safe loaded-range inspection. Stop elapsed time at
the terminal paging decision. Exclude viewport index selection, zoom, the
post-zoom wait, and actual-range readback; those retain existing per-evaluation
CDP deadlines.

Backward paging begins only when `earliest_loaded > requested.from`. At initial
inspection, stop as `paging_not_needed` if the left edge is already covered;
otherwise stop as `history_exhausted` if availability is explicitly false;
otherwise request one page if deadline and request guards permit it. If the
deadline expires before that first request, return `deadline_elapsed`, zero
requests, and `timed_out: true` using the safe initial observation.

After each request, apply this exact priority:

1. If the left edge is covered, stop as `coverage_reached`.
2. Otherwise, if `requestMoreDataAvailable() === false`, stop as
   `history_exhausted`.
3. Otherwise, if the earliest timestamp moved backward, accept progress. Then
   stop for the absolute deadline if elapsed, then the 25-request limit if
   reached, otherwise request another page.
4. If no progress was observed and the absolute deadline elapsed, stop as
   `deadline_elapsed`.
5. If the 3-second progress window elapsed first, stop as `no_progress`.

A twenty-fifth request that reaches coverage reports `coverage_reached`. A
twenty-fifth request that progresses but remains uncovered reports
`request_limit_reached`. An absolute timeout with no new progress reports
`deadline_elapsed`, not `no_progress`.

If initial inspection fails or times out before any safe loaded range exists,
return a structured error. A non-timeout CDP evaluation failure is always a
structured error. If the absolute deadline expires after a safe observation,
use the most recent safe range for partial success. Do not disguise an
evaluation failure as a paging guard.

## Viewport Application

After the terminal paging decision, compute the intersection of requested and
loaded endpoint intervals. Use separate optional indices; never use a loaded
index value as the unset sentinel.

When the intervals overlap, select loaded bars whose timestamps fall inside
the intersection. The first selected index is the first timestamp greater than
or equal to the intersection start; the last is the last timestamp less than
or equal to the intersection end. Require at least one match and
`from_index <= to_index`. A one-bar match may use the same index twice.

If matching bars exist, call `zoomToBarsRange`. Report `applied` when requested
bounds required no endpoint clamp and `applied_clamped` when either endpoint
was clamped to loaded history. `applied_range` is the first/last selected bar
timestamp range and `matching_bar_count` is the number of selected bars.

If endpoint intervals do not overlap, do not zoom. Preserve the pre-operation
visible range as `actual` and return `unchanged_no_overlap`, false `applied`,
false `clamped`, zero matches, and null `applied_range`.

If endpoint intervals overlap but contain no loaded bar, also do not zoom.
Return the same false/null fields with `unchanged_no_matching_bars`. This covers
weekends, closed sessions, trading halts, and other gaps. An empty or unreadable
loaded collection is not this state; it is a structured unavailable error.

## Failure Contract

Stable `paging_phase` values are `inspect_initial`, `request_history`,
`observe_progress`, `inspect_final`, and `apply_visible_range`. Runtime errors
should include the bounded setter metadata, requested bounds, phase, request
count, safe earliest timestamps where available, and a public-safe next action.

Missing `requestMoreData` or `requestMoreDataAvailable` is
`internal_api_unavailable` when paging is needed. If the left edge is already
covered, those methods do not need to be called. Do not include raw DOM,
evaluated JavaScript, raw payloads, target IDs, account-local metadata,
credentials, session IDs, or local filesystem paths in payloads, errors, tests,
or tracked evidence.

## Plan of Work

First add `crates/model/src/visible_range.rs` and export it from
`crates/model/src/lib.rs`. Define validated bounds, typed validation failure,
loaded-range observations, paging decisions, stop reasons, viewport decisions,
and payload shaping. Keep time and CDP outside this module. Add exhaustive table
fixtures before touching the Desktop adapter.

Then refactor `crates/cli/src/ops/chart.rs`. Add small private evaluator calls
for initial/final state inspection, one `requestMoreData(1000)`, and viewport
application. Orchestrate them in Rust with `tokio::time::Instant`, `timeout_at`,
and `sleep`. Use one absolute deadline and one sequential runtime connection.
Keep JavaScript class-free and return only normalized public-safe primitives.

Update `crates/cli/src/app/dispatch.rs` so bounded range validation runs before
`connect_runtime`. Retain validation inside `set_visible_range` for direct Rust
callers. Refactor `validate_export_chart_bars_request` to use the same typed
finite/order predicate while preserving its existing command-specific error
message, details, exit code, and count validation.

Finally update CLI help, README if needed, `docs/command-source-taxonomy.md`,
`docs/observation-workflows.md`, `docs/development.md`, packaged agent guidance,
and only runtime skill references that route range/export evidence. Keep core
skill workflows short.

## Concrete Steps

Run from the repository root. Begin with model and focused operation tests:

    cargo test -p tradingview-model visible_range -- --nocapture
    cargo test -p tradingview-cli ops::chart -- --nocapture
    cargo test -p tradingview-cli --test cli_contract_desktop range -- --nocapture
    cargo test -p tradingview-cli export -- --nocapture

After implementation and docs synchronization, run:

    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1
    git diff --check
    python3 scripts/check-public-hygiene.py
    bash -n scripts/stage-release-package-files.sh
    cmp -s AGENTS.md CLAUDE.md

The optional public-safe live smoke uses a dedicated chart. Record the original
visible range, request a start older than the initially loaded history, verify
at least one request moved the earliest timestamp backward, and restore the
original visible range. Store only bounds, counts, coverage, stop/application
status, elapsed time, and restoration outcome in this plan.

## Validation and Acceptance

Deterministic tests must cover finite/order validation, pre-connection failure,
direct-caller defense, export mapping, no-argument getter preservation,
paging-not-needed with complete and right-edge-partial coverage, multi-request
coverage, exhaustion, no-progress, request limit, zero-request deadline,
mid-loop deadline, unavailable methods, and CDP failure separation.

Collision fixtures must prove the exact terminal precedence and cause-only
booleans. Deadline fixtures must prove the clock includes initial/final paging
inspection and excludes viewport/readback time.

Viewport fixtures must cover exact earliest/latest boundaries, left/right
clamp, one-bar match, wholly older/newer no-overlap, and overlapping endpoints
with zero matching bars. The latter three must prove no zoom occurs, the prior
visible range remains `actual`, and no reversed/missing index reaches
`zoomToBarsRange`.

Live acceptance requires a bounded real request that moves the earliest loaded
timestamp backward on a dedicated chart without changing symbol, timeframe,
studies, drawings, or account state. If this cannot be reproduced, do not
weaken the payload or publish misleading paging; record no-go and preserve
trial work safely after owner confirmation.

## Idempotence and Recovery

Model and deterministic tests are safe to repeat. Live paging may retain older
bars in Desktop memory, so repeated smokes may need a fresh dedicated chart to
exercise initial loading. Always capture and restore the visible range around a
live smoke. Paging cannot unload history and does not require account cleanup.

The unrelated stash named
`recovered-indicator-search-prototype-2026-07-12` must remain untouched. Never
drop or overwrite a stash or discard an uncommitted prototype without explicit
owner confirmation. If R9 trial changes must be withdrawn, create a separately
named stash first and ask before deleting it.

## Artifacts and Notes

The intended flow is:

    validate before connect
      -> inspect selected-chart loaded range
      -> bounded sequential request/observe state machine
      -> derive endpoint coverage and discrete matching bars
      -> zoom only when at least one ordered bar index exists
      -> report paging, viewport application, and actual range

The upstream call and guard are evidence only:

    mainSeries().requestMoreData(1000)
    maximum requests: 25

Do not copy its fixed wait, exception swallowing, or sparse response.

## Interfaces and Dependencies

In `crates/model/src/visible_range.rs`, define at minimum:

    pub struct VisibleRangeBounds {
        pub from: f64,
        pub to: f64,
    }

    pub enum VisibleRangeValidationFailure {
        NonFinite { field: &'static str },
        InvalidOrder { from: f64, to: f64 },
    }

    pub fn validate_visible_range_bounds(
        from: f64,
        to: f64,
    ) -> Result<VisibleRangeBounds, VisibleRangeValidationFailure>;

Add model types for `LoadedRangeObservation`, `PagingStopReason`,
`PagingDecision`, `ViewportApplicationStatus`, and the final public-safe
readback. Derive only traits supported by contained floating-point fields; do
not force `Eq` where it is invalid.

Keep the existing public CLI operation signatures:

    pub async fn visible_range(
        runtime: &mut impl RuntimeEvaluator,
    ) -> Result<serde_json::Value, AppError>;

    pub async fn set_visible_range(
        runtime: &mut impl RuntimeEvaluator,
        from: f64,
        to: f64,
    ) -> Result<serde_json::Value, AppError>;

No new production dependency is required. Reuse Tokio time primitives already
in the workspace and the existing `RuntimeEvaluator` abstraction.

## Open Questions

- UNCONFIRMED until live implementation evidence: whether current Desktop
  moves earliest loaded time within the 3-second per-request progress window.
- UNCONFIRMED until live implementation evidence: whether
  `requestMoreDataAvailable() === false` is stable exhaustion evidence across
  symbols and timeframes.
- UNCONFIRMED until adapter inspection: whether a current internal promise or
  event provides stronger bounded progress evidence than timestamp polling.
  Prefer it only if it preserves the reviewed public contract.

Revision note (2026-07-13): Created R9 after R8 contract review completed with
no remaining findings. This plan repeats the reviewed state machine, viewport,
validation, deadline, and source boundaries so implementation can proceed from
this file alone.
