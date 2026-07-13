# Define bounded visible-range history paging

This ExecPlan is a living document. Keep `Progress`, `Surprises & Discoveries`,
`Decision Log`, and `Outcomes & Retrospective` current while work proceeds.
Maintain it according to `.agents/PLANS.md`.

## Purpose / Big Picture

`tv range --from <UNIX_SECONDS> --to <UNIX_SECONDS>` currently searches only
the bars already loaded in the selected TradingView Desktop chart. When the
requested start is older than that in-memory window, the command can move the
viewport to the earliest loaded bar and still return success without explaining
that older chart history was never requested.

This R8 contract slice defines how a later R9 implementation will load older
history from the selected chart itself before moving the viewport. The loading
loop will be bounded, report why it stopped, and preserve the existing
`requested` and `actual` fields. This slice adds no command, option, payload, or
runtime behavior. Its observable result is a reviewed, self-contained contract
that lets R9 implement paging without deciding termination, diagnostics, or
source boundaries along the way.

## Progress

- [x] (2026-07-13) Confirmed the current Rust `set_visible_range` scans only
  `mainSeries().bars()` and calls `zoomToBarsRange` without requesting history.
- [x] (2026-07-13) Inspected the current local upstream implementation and its
  four unit tests for `requestMoreData(1000)` paging.
- [x] (2026-07-13) Defined the provisional controls, stopping conditions,
  additive payload, source boundary, and error behavior for R9.
- [x] (2026-07-13) Synchronized the plan index, roadmap, work inventory,
  changelog, and continuity ledger with R8 as the active contract slice.
- [x] (2026-07-13) Applied first-review corrections for right-edge partial
  coverage, pre-connection validation placement, terminal-condition
  precedence, and getter/setter metadata separation.
- [x] (2026-07-13) Applied second-review corrections for deterministic
  viewport clamping/no-overlap behavior and exact paging deadline measurement.
- [x] (2026-07-13) Applied third-review correction for timestamp intervals
  that overlap but contain no matching discrete bar.
- [x] (2026-07-13) Completed focused independent re-review after three
  correction rounds; the reviewer reported no remaining findings.
- [x] (2026-07-13) Closed R8 and prepared archival before creating the separate
  R9 implementation ExecPlan. No runtime behavior changed in R8.

## Surprises & Discoveries

- Observation: current Rust behavior can silently clamp an old request to the
  loaded bar window.
  Evidence: `crates/cli/src/ops/chart.rs::set_visible_range` reads
  `firstIndex()` and `lastIndex()`, searches that interval, and zooms directly;
  it never invokes `requestMoreData` or reports loaded-history coverage.

- Observation: upstream proves that the selected chart main series exposes a
  usable history request path, but its quality boundary is weaker than this
  repository requires.
  Evidence: the local upstream `src/core/chart.js` calls
  `mainSeries().requestMoreData(1000)` up to 25 times with a fixed 1800 ms wait,
  defaults `requestMoreDataAvailable()` exceptions to `true`, and does not
  report no-progress, timeout, request count, or coverage diagnostics. Its
  tests cover already-covered, progress, exhaustion, and final zoom, but use a
  mock that advances immediately on each request.

- Observation: history paging changes selected-chart in-memory state even
  though it does not change symbol, timeframe, drawings, studies, or account
  state.
  Evidence: requesting older main-series data makes additional bars available
  to the selected chart and is a prerequisite to the requested viewport
  movement. The existing command already moves the viewport.

## Decision Log

- Decision: keep paging inside `tv range`; do not add a separate public
  history-load command in this slice.
  Rationale: users are asking `tv range` to display a requested range. Loading
  enough selected-chart history to attempt that operation is part of making the
  existing operation truthful, provided the work is bounded and observable.
  Date/Author: 2026-07-13 / Codex

- Decision: classify only bounded `tv range --from/--to` success and error
  metadata as `desktop_backed_operation` and `non_mutating: false` after R9.
  Keep no-argument `tv range` as `desktop_backed_read` and
  `non_mutating: true`.
  Rationale: the setter moves the viewport and may expand the chart's loaded
  history, while the getter only observes current range state. Correcting the
  setter must not alter the separate read contract.
  Date/Author: 2026-07-13 / Codex

- Decision: use an absolute 30-second paging deadline, a maximum of 25 history
  requests, a request size of 1000 bars, a 200 ms observation interval, and a
  3-second per-request progress window as first-slice constants.
  Rationale: the upstream request size and request-count guard are established
  current-build evidence, while finite observation and one absolute deadline
  avoid fixed multi-second sleeps and timeout extension. These are internal
  constants, not new CLI options in R9.
  Date/Author: 2026-07-13 / Codex

- Decision: require strict backward progress after every accepted request.
  Rationale: a request is useful only when the earliest loaded timestamp moves
  earlier, coverage becomes complete, or the feed explicitly reports no more
  data. A request that produces none of those outcomes within its progress
  window stops as `no_progress`; repeated blind requests are unsafe and slow.
  Date/Author: 2026-07-13 / Codex

- Decision: partial coverage is a successful viewport operation with explicit
  diagnostics, not a terminal command error.
  Rationale: exhaustion, no progress, iteration limit, and total timeout can
  still leave a useful best-effort visible range. Callers must be able to
  distinguish that result from complete coverage. Failures to inspect or
  request chart history remain structured operation errors because coverage
  cannot be assessed safely.
  Date/Author: 2026-07-13 / Codex

- Decision: do not copy upstream exception swallowing or fixed waits.
  Rationale: unavailable internal methods and inspection failures are source
  diagnostics, not evidence that more history exists. Progress should be
  observed through chart state under bounded deadlines.
  Date/Author: 2026-07-13 / Codex

- Decision: enter backward paging only when
  `earliest_loaded > requested.from`.
  Rationale: loading older bars cannot repair a request whose left edge is
  already covered but whose right edge is newer than the latest loaded bar.
  That zero-request case stops as `paging_not_needed`; its independently
  computed coverage may still be `partial`.
  Date/Author: 2026-07-13 / Codex

- Decision: resolve competing terminal observations in a fixed order and make
  guard booleans describe the actual stopping cause.
  Rationale: coverage, exhaustion, deadline, request limit, and no-progress can
  coincide. Deterministic precedence prevents payload differences caused by
  implementation timing.
  Date/Author: 2026-07-13 / Codex

- Decision: apply the viewport only when the requested and loaded timestamp
  intervals overlap; otherwise leave the viewport unchanged and report partial
  success.
  Rationale: zooming to the whole loaded range when the requested interval is
  entirely older or newer is not a truthful approximation. Explicitly not
  moving is safer and lets the caller decide whether the available evidence is
  useful.
  Date/Author: 2026-07-13 / Codex

- Decision: start one absolute paging clock immediately before initial history
  inspection and stop it at the terminal paging decision after the final safe
  loaded-range observation. Exclude viewport application and actual-range
  readback from `elapsed_ms`.
  Rationale: initial/final inspection is part of paging evidence, while zoom
  and readback already use the normal CDP evaluation deadline and should not
  change why history loading stopped.
  Date/Author: 2026-07-13 / Codex

- Decision: distinguish geometric interval overlap from the presence of a
  selectable loaded bar.
  Rationale: weekends, market closures, trading halts, and intraday session
  gaps can leave an overlapping timestamp interval with zero bar timestamps.
  Such a request must not produce reversed indices or a misleading zoom.
  Date/Author: 2026-07-13 / Codex

## Outcomes & Retrospective

R8 is complete and independently reviewed. It establishes a narrow R9
implementation boundary around the selected chart main series, with no hidden
fallback and no public controls. The reviewed contract preserves existing
range fields while defining bounded paging, endpoint coverage, discrete-bar
matching, viewport application, deadline measurement, validation placement,
and public-safe failure behavior. Runtime implementation and live acceptance
belong to R9.

## Context and Orientation

The `tv` binary dispatches `tv range` in
`crates/cli/src/app/dispatch.rs::dispatch`. With no arguments it calls
`crates/cli/src/ops/chart.rs::visible_range`, a selected-chart read. With both
`--from` and `--to` it calls
`crates/cli/src/ops/chart.rs::set_visible_range`. Dispatch currently connects
to Desktop before this function validates finite Unix seconds. The function
then inspects the selected chart's main-series bar collection, finds bar
indices inside the requested timestamps, and invokes `zoomToBarsRange`.

The main-series bar collection is only the history currently loaded in the
TradingView Desktop chart. “History paging” in this plan means asking that same
main series to load an older batch through `requestMoreData(1000)`, then
observing whether the earliest loaded bar timestamp moved backward. It does not
mean calling the Desktop-free `tv bars` WebSocket source, selected-chart
`tv ohlcv`, `tv export chart-bars`, Replay, or any scanner endpoint.

`tv export chart-bars` in `crates/cli/src/ops/market/ohlcv.rs` calls
`set_visible_range` before reading selected-chart bars. Therefore R9 improves
its range operation transitively, but does not turn export into a different
source or guarantee that returned OHLCV covers the requested range. Export
keeps its own `export_chart_bars.v1` diagnostics and must embed the enriched
range-operation result without renaming existing fields.

The local upstream checkout has a paging loop in `src/core/chart.js` and tests
in `tests/chart_history.test.js`. It is evidence that the internal main-series
method exists, not a contract to copy. This repository requires public-safe
failure details, absolute deadlines, explicit progress and stop reasons, and
deterministic I/O-free tests.

## Provisional Contract

R9 keeps the current command form:

    tv range --from <UNIX_SECONDS> --to <UNIX_SECONDS>

No new option is added. R9 must make validation genuinely pre-connection:
both values are required together, both must be finite, and `from >= to` is a
validation error with the existing validation exit code 1 before connecting to
Desktop. This is a
behavioral correction; finite validation currently occurs only after
`connect_runtime`.

Add one shared I/O-free visible-range validator in `tradingview-model`.
`Command::Range` dispatch must call it before `connect_runtime`, and
`set_visible_range` must call the same validator defensively for direct Rust
callers. Refactor `validate_export_chart_bars_request` to delegate finite and
ordering checks to that validator before applying its count rule. Preserve the
existing export command's error kind, exit code, public-safe `from` / `to`
details, and command-specific message; `tv range` receives the equivalent
range-specific message rather than a second ordering rule.

The success payload preserves `operation`, `source`, `requires_desktop`,
`requested`, and `actual`. It corrects the operation metadata and adds one
`history_paging` object:

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

Timestamps are Unix seconds or null when no main-series bar can be observed.
Counts and elapsed values are non-negative integers. The stable
`coverage_status` vocabulary is `complete`, `partial`, or `unavailable`.
`complete` means `earliest_loaded_after <= requested.from` and the loaded
latest timestamp is not earlier than `requested.to`. `partial` means the
loaded range is readable but does not cover both requested boundaries.
`unavailable` means no safe loaded range could be established; R9 normally
returns that state through a structured error rather than a success payload.

`viewport_application.status` has the stable vocabulary `applied`,
`applied_clamped`, `unchanged_no_overlap`, or
`unchanged_no_matching_bars`. `matching_bar_count` is the non-negative count of
loaded bar timestamps inside the requested/loaded interval intersection and is
zero for both unchanged statuses. `applied_range` is the timestamp range
represented by the selected loaded bar indices, or null when the viewport is
unchanged. `applied` is false for both unchanged statuses. `clamped` is true
only for `applied_clamped`; it does not describe history coverage by itself.

Backward paging begins only when the initial earliest timestamp is later than
`requested.from`. The stable `stop_reason` vocabulary is
`paging_not_needed`, `coverage_reached`, `history_exhausted`, `no_progress`,
`request_limit_reached`, or `deadline_elapsed`. `paging_not_needed` uses zero
requests and means only that the requested left edge was already loaded. It
does not imply complete coverage: if `latest_loaded_after < requested.to`, the
same result has `coverage_status: partial`.

`history_exhausted` requires an explicit
`requestMoreDataAvailable() === false` observation while the left edge remains
uncovered. Exceptions or absent methods are not exhaustion. `limit_reached`
and `timed_out` mean that the named guard actually stopped paging, not merely
that `request_count == request_limit` or the clock reached the deadline after a
higher-priority terminal observation.

R9 must use this deterministic state-machine order. At initial inspection,
first return `paging_not_needed` when the left edge is covered; otherwise stop
as `history_exhausted` if unavailability is explicit; otherwise issue one
request if both guards permit it. After each request, inspect observations in
this order:

1. If the left edge is covered, stop as `coverage_reached`, even if the same
   observation also reports exhaustion or is the twenty-fifth request.
2. Otherwise, if availability is explicitly false, stop as
   `history_exhausted`.
3. Otherwise, if the earliest timestamp moved backward, accept progress. Then
   stop as `deadline_elapsed` if the absolute deadline has elapsed; otherwise
   stop as `request_limit_reached` if 25 requests have completed; otherwise
   issue the next request.
4. If no backward progress was observed and the absolute deadline elapsed,
   stop as `deadline_elapsed`.
5. If the per-request progress window elapsed first, stop as `no_progress`.

This ordering means a twenty-fifth request that reaches coverage reports
`coverage_reached`; a twenty-fifth request that makes progress but remains
uncovered reports `request_limit_reached`; and a request with no progress when
the absolute deadline expires reports `deadline_elapsed`. Set
`limit_reached: true` only for `request_limit_reached`, and `timed_out: true`
only for `deadline_elapsed`. Both are false for every other stop reason.

After paging stops, R9 derives viewport indices from loaded bars without using
an index value as an “unset” sentinel. Compute the timestamp intersection of
the requested and loaded ranges. If it is non-empty, choose the first loaded
bar whose timestamp is greater than or equal to the intersection start and the
last loaded bar whose timestamp is less than or equal to the intersection end.
An exact earliest-bar match must retain the earliest index. A one-bar
intersection may use the same index for both bounds.

Interval overlap alone is insufficient. After selecting candidates, verify
that at least one loaded bar timestamp lies inside the intersection and that
`from_index <= to_index`. If no bar matches, do not call `zoomToBarsRange`;
preserve the pre-operation viewport, return it as `actual`, and report
`status: "unchanged_no_matching_bars"`, `applied: false`, `clamped: false`,
`matching_bar_count: 0`, and `applied_range: null`. This covers weekend-only,
closed-session, trading-halt, and other gap-only requests. It is distinct from
`unchanged_no_overlap`, where the requested and loaded endpoint intervals do
not intersect at all.

When requested bounds extend beyond loaded history but the intervals still
overlap, clamp the relevant side to the earliest or latest loaded bar and
report `viewport_application.status: "applied_clamped"`, `applied: true`, and
`clamped: true`. When the requested interval lies wholly before or wholly after
the loaded interval, do not call `zoomToBarsRange`; preserve the pre-operation
viewport, return its observed value as `actual`, and report
`status: "unchanged_no_overlap"`, `applied: false`, `clamped: false`, and
`applied_range: null`. This is partial success, not a structured error, because
the loaded range and reason for non-application are known. An empty or
unreadable loaded bar collection remains a structured unavailable error.

The command must never label partial coverage complete merely because `actual`
is non-null. If the requested end is newer than the latest loaded bar, coverage
is partial; this first slice does not request future or realtime data.
`coverage_status` describes whether loaded-history endpoint timestamps cover
the requested endpoint interval. It does not guarantee that the requested
interval contains a bar. Therefore a weekend/session-gap request may have
`coverage_status: "complete"` together with
`viewport_application.status: "unchanged_no_matching_bars"`.

Create the absolute deadline and `elapsed_ms` start instant once, immediately
before initial loaded-range inspection. Include initial inspection, all history
requests, all progress observations, and the last safe loaded-range inspection
in that deadline. Stop `elapsed_ms` when the paging state machine chooses its
terminal `stop_reason`, before viewport index selection, `zoomToBarsRange`, the
post-zoom wait, and `actual` range readback. Those later operations remain
bounded by the existing per-evaluation CDP deadlines but do not alter
`history_paging.elapsed_ms` or `stop_reason`.

If initial inspection succeeds but the absolute deadline expires before the
first request, return a zero-request partial result with
`stop_reason: "deadline_elapsed"` and `timed_out: true`, using that safe initial
range for clamping. If the deadline expires during request/progress/final
inspection after at least one safe observation, stop paging and use the most
recent safe loaded range. If initial inspection itself fails or times out
before any safe loaded range exists, return a structured error rather than a
partial success. Any non-timeout CDP evaluation error is also a structured
error; do not relabel it `deadline_elapsed` or silently use stale state.

## Failure Contract

Pre-connection input errors use the existing validation envelope and exit code
1. Runtime inspection or request failures use the existing Desktop operation
error mapping and exit code; R9 adds public-safe details where available:
`operation`, `source`, `source_category`, `requires_desktop`, `non_mutating`,
`requested`, `paging_phase`, `request_count`, `earliest_loaded_before`,
`earliest_loaded_after`, and `next_action_hint`.

The stable `paging_phase` vocabulary is `inspect_initial`, `request_history`,
`observe_progress`, `inspect_final`, or `apply_visible_range`. Details must not
include raw DOM, raw evaluated JavaScript, raw payloads, target IDs,
account-local metadata, credentials, session IDs, or local filesystem paths.

Ambiguous or missing TradingView targets continue through existing runtime
connection errors. A missing `requestMoreData` or
`requestMoreDataAvailable` method is `internal_api_unavailable`; R9 does not
silently fall back to current clamped behavior. If the initial loaded range is
already covered on the left, those methods do not need to be invoked. The
no-argument getter does not use paging and retains
`source_category: "desktop_backed_read"` and `non_mutating: true`.

## Plan of Work

R8 itself changes documentation only. Obtain focused review of the contract,
especially the mutation classification, timeout composition, coverage rule,
and partial-success boundary. Resolve findings in this living plan before
creating R9.

R9 should first add shared I/O-free validation, state interpretation,
stop-reason selection, and payload shaping in `tradingview-model` rather than
embedding policy in JavaScript. A small model module should validate requested
bounds and accept observed loaded ranges, request availability, request count,
and elapsed state, then return the next action or final outcome
deterministically. Dispatch calls validation before Desktop connection;
`set_visible_range` and export validation reuse it defensively.

Next, split `crates/cli/src/ops/chart.rs::set_visible_range` into small private
Desktop adapter operations: inspect main-series history state, request one
older batch, wait for observable progress under the shared absolute deadline,
and apply the visible bar range. Keep one `RuntimeEvaluator` connection and
sequential ownership. Do not start a background task and do not issue parallel
history requests.

Finally, update CLI help, stable source taxonomy, observation workflows,
development guidance, packaged agent guidance, and only the runtime skills
that actually route `tv range` or `tv export chart-bars`. Detailed stop-reason
matrices belong in references, not core skill workflows.

## Concrete Steps

All commands run from the repository root.

During R8 review, run:

    git diff --check
    python3 scripts/check-public-hygiene.py
    bash -n scripts/stage-release-package-files.sh
    cmp -s AGENTS.md CLAUDE.md

After review is green, archive this plan and create R9 without implementing it
in the same commit. The R9 implementation should use focused checks such as:

    cargo test -p tradingview-model visible_range -- --nocapture
    cargo test -p tradingview-cli ops::chart -- --nocapture
    cargo test -p tradingview-cli --test cli_contract_desktop range -- --nocapture

Then run the normal baseline:

    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1
    git diff --check
    python3 scripts/check-public-hygiene.py
    bash -n scripts/stage-release-package-files.sh

An optional R9 live smoke may use a dedicated chart and a request whose start
is older than its initially loaded history. Tracked documents record only the
requested bounds, earliest timestamps before and after, request count,
coverage, stop reason, elapsed time, and actual range. They never record raw
bars, evaluated JavaScript, target IDs, or account-local state.

## Validation and Acceptance

R8 is complete when independent review can answer all of the following without
inventing policy: when paging starts, how much it requests, how long it may
run, what counts as progress, every terminal stop reason, when partial success
is allowed, which existing fields survive, and which sources are forbidden.

R9 will be accepted only when deterministic tests prove paging-not-needed,
coverage reached after multiple requests, explicit exhaustion, no progress,
request limit, absolute deadline, unavailable internal API, invalid range, and
partial best-effort zoom. The complete workspace baseline must remain green.
A right-edge-only gap must prove zero requests, `paging_not_needed`, and
`coverage_status: partial`. Pre-connection contract tests must prove invalid
finite/order input never attempts Desktop connection. Collision fixtures must
prove the documented precedence and cause-only booleans.
Index fixtures must prove exact earliest/latest boundaries, left-side clamp,
right-side clamp, one-bar overlap, wholly older no-overlap, and wholly newer
no-overlap. No-overlap fixtures must prove `zoomToBarsRange` is not called and
the prior visible range is returned as `actual`. Deadline fixtures must prove
the clock starts before initial inspection, includes the final safe
inspection, can stop before the first request with count zero, excludes
viewport/readback time from `elapsed_ms`, and distinguishes timeout guards from
CDP evaluation errors.
A weekend or session-gap fixture must prove that overlapping endpoint
intervals with zero matching bars return
`unchanged_no_matching_bars`, preserve `actual`, and never pass reversed or
missing indices to `zoomToBarsRange`.
A bounded live smoke must additionally show at least one real request moving
`earliest_loaded_after` earlier than `earliest_loaded_before` without changing
symbol, timeframe, studies, drawings, or account state. If current Desktop no
longer exposes a trustworthy request/progress boundary, R9 records no-go and
does not publish misleading coverage.

## Idempotence and Recovery

R8 is documentation-only and safe to repeat. The indicator-search prototype
remains in its named Git stash and is outside this plan; do not apply, drop, or
rewrite it while working on range paging.

R9 history requests are additive to the selected chart's in-memory history but
cannot unload already fetched bars. Retrying the command may request less or no
additional history because the chart retains prior pages. Tests must therefore
assert outcomes from controlled state rather than assume every invocation
starts with the same loaded range. If a live smoke changes the viewport, record
the original visible range and restore it explicitly after evidence capture.

## Artifacts and Notes

Current Rust behavior:

    inspect loaded bars -> choose loaded indices -> zoom -> wait 500 ms -> report actual

Provisional R9 behavior:

    validate -> inspect loaded range -> bounded request/observe loop
             -> intersect requested/loaded range
             -> zoom only on overlap -> report paging, application, and actual

The upstream comparison is intentionally directional evidence only. Its
`requestMoreData(1000)` call and 25-request cap inform the initial constants;
its fixed 1800 ms sleep, exception swallowing, and sparse payload do not.

## Interfaces and Dependencies

R8 adds no Rust interface or dependency. R9 should introduce an I/O-free model
under `crates/model/src/` for request validation and paging decisions, then
reuse the existing `RuntimeEvaluator` abstraction in
`crates/cli/src/ops/chart.rs` for Desktop calls.

In a new model module such as `crates/model/src/visible_range.rs`, define a
validated bounds type and a typed validation failure independent of command
wording:

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

The CLI adapter converts that typed failure to `AppError`. This lets
`Command::Range` validate before connection and lets `set_visible_range`
repeat the same check for direct callers, while
`validate_export_chart_bars_request` preserves its existing command-specific
message and public details when mapping the same failure. Do not implement two
finite/order predicates in CLI modules.

No production dependency is needed. Use `tokio::time::Instant`,
`timeout_at`, and `sleep` already available in the workspace for one absolute
deadline and bounded observation. Keep `&mut impl RuntimeEvaluator` sequential;
do not add concurrent CDP evaluations.

The existing public functions remain:

    pub async fn visible_range(
        runtime: &mut impl RuntimeEvaluator,
    ) -> Result<serde_json::Value, AppError>;

    pub async fn set_visible_range(
        runtime: &mut impl RuntimeEvaluator,
        from: f64,
        to: f64,
    ) -> Result<serde_json::Value, AppError>;

R9 may add private typed helpers but must not change these signatures or the
normal JSON envelope. `tv export chart-bars` continues to call
`set_visible_range` and receives the enriched additive result.

## Open Questions

- UNCONFIRMED until R9 live evidence: whether current Desktop reliably changes
  the earliest bar timestamp after `requestMoreData(1000)` within the proposed
  3-second progress window.
- UNCONFIRMED until R9 live evidence: whether
  `requestMoreDataAvailable() === false` is stable exhaustion evidence across
  symbols and timeframes.
- UNCONFIRMED until R9 implementation inspection: whether the internal method
  returns a promise or event that is stronger than polling earliest loaded
  time. Prefer that signal if it is bounded and deterministic, but do not alter
  the public contract.

Revision note (2026-07-13): Created R8 after indicator search implementation
was deferred. The plan incorporates current Rust behavior and upstream paging
evidence while defining stricter deadlines, progress, diagnostics, and source
boundaries for a separate R9 implementation.

Revision note (2026-07-13): Applied independent-review corrections by defining
the right-edge partial case, exact terminal precedence and cause booleans,
shared pre-connection validation placement, and setter-only metadata changes.

Revision note (2026-07-13): Applied focused re-review corrections by defining
exact bar-index intersection/clamping, unchanged no-overlap behavior, and the
start/stop boundary for the paging deadline and `elapsed_ms`.

Revision note (2026-07-13): Applied the next focused re-review correction by
separating endpoint coverage from discrete-bar presence and defining
`unchanged_no_matching_bars` for weekend, session-gap, and equivalent ranges.

Revision note (2026-07-13): Recorded focused re-review green and closed R8 for
archive. The separate R9 plan owns implementation and live validation.

Revision note (2026-07-13): Corrected the validation exit-code text during R9
implementation after verifying `tradingview-core::AppError::exit_code`; the
reviewed pre-connection behavior is unchanged.
