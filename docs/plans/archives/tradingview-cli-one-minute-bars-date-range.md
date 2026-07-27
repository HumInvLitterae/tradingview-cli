# Add bounded one-minute bars date-range reads

This ExecPlan is a living document maintained according to `.agents/PLANS.md`.
Keep Progress, Surprises & Discoveries, Decision Log, and Outcomes current.

## Purpose / Big Picture

Allow downstream workflows to request one-minute historical bars through the
same Desktop-free, source-labeled, bounded date-range contract already used for
5-minute and 15-minute preparation:

    tv bars <SYMBOL> --timeframe 1 --from YYYY-MM-DD --to YYYY-MM-DD --count <N>

The existing `1m` alias must normalize to the same timeframe. Success continues
to use `contract_version: "bars.v1"` and
`source: "tradingview_bars_ws"`. Callers retain
`requested_range`, `returned_range`, `observed_range`,
`range_coverage_status`, `range_alignment`, `range_fetch_summary`,
`source_availability`, `wait_summary`, and `bars`.

The slice preserves the 5,000 returned-bar safety cap and does not add a new
pagination option, retry, a second source, historical basket inference, or
private package logic.

## Progress

- [x] (2026-07-27) Confirmed `v0.30.2` is released at `e8e480d` and the
  worktree is clean.
- [x] (2026-07-27) Confirmed recent-count mode already accepts `1`/`1m`, while
  date-range validation rejects only because `DATE_RANGE_TIMEFRAMES` omits
  normalized `1`.
- [x] (2026-07-27) Confirmed the current date-range transport already uses one
  bounded WebSocket series, 500-bar `request_more_data` windows, no-progress
  detection, one 10-second request deadline, and explicit fetch/coverage
  summaries.
- [x] (2026-07-27) Owner directed implementation after reviewing and
  correcting the roadmap/task inventory; no separate external plan-review
  turn was required.
- [x] (2026-07-27) Implemented the initial validation, range-boundary test,
  help, docs, and runtime-skill changes.
- [x] (2026-07-27) Ran focused bars validation/transport/payload/CLI contract
  tests, strict Clippy, and the complete workspace test baseline successfully.
- [x] (2026-07-27) Focused implementation review found no production contract
  defect but identified missing deterministic coverage, a missing date-range
  live harness, and stale roadmap wording.
- [x] (2026-07-27) Added exact end-boundary, timeout, source-exhaustion,
  no-progress, closure-shaped, payload-success/error, and aggregate-only
  three-case ignored-harness coverage; synchronized the roadmap.
- [x] (2026-07-27) Focused correction review confirmed the earlier findings
  closed and identified two remaining harness blockers: permissive symbol
  qualification and incomplete closure-classification consistency checks.
- [x] (2026-07-27) Required exactly one symbol separator with non-empty sides
  and added fail-closed aggregate consistency checks across coverage,
  truncation, timeout, wait completion, series completion, and partial-result
  fields.
- [x] (2026-07-27) Re-ran formatting, strict workspace Clippy, the complete
  workspace test baseline, public hygiene, package-script syntax, guide
  parity, and diff hygiene successfully after the second harness correction.
- [x] (2026-07-27) Focused correction re-review found no remaining findings,
  confirmed both harness blockers closed, and approved requesting owner
  authorization for the exact bounded live smoke.
- [x] (2026-07-27) Ran the owner-authorized bounded Desktop-free smoke exactly
  once. The first child returned a structured `connection` failure after about
  two seconds; the harness stopped with one attempt, zero completed cases, and
  no retry, substitution, or remaining-case execution.
- [x] (2026-07-27) Ran a bounded production-binary comparison to distinguish a
  feature defect from the common WebSocket path. Existing five-minute recent
  and date-range reads succeeded, as did one-minute single-window,
  additional-window, and closure-boundary reads. The one-minute additional
  range returned 780 bars with one `request_more_data` window; the
  closure-boundary range returned 390 bars with explicit partial
  `source_exhausted` classification.
- [x] (2026-07-27) Focused evidence review found no implementation or evidence
  defect beyond a stale local Git pointer. Corrected the ledger, confirmed the
  feature through bounded production-binary comparison, and archived the plan.

## Surprises & Discoveries

- Observation: this is not a new transport or pagination design.
  Evidence: `crates/market/src/bars/transport.rs` passes the normalized
  timeframe directly to `create_series` and already pages older data with
  `request_more_data`.

- Observation: v0.30.2 already supports narrow intraday date ranges for `5`,
  `15`, `30`, and `60`.
  Evidence: `DATE_RANGE_TIMEFRAMES` and CLI contract tests list those four
  values alongside `1D`, `1W`, and `1M`; only `1` is required by the reported
  downstream blocker.

- Observation: the 5,000 value is a returned-bar safety cap, while transport
  work is bounded by 500-bar windows, source progress, and the absolute
  request deadline.
  Evidence: `finalize_result` filters the observed range and truncates returned
  bars to `request.count`; `should_request_more` is driven by the oldest
  observed timestamp.

- Observation: the CLI does not own an exchange calendar.
  Evidence: date ranges are UTC calendar bounds and coverage is derived from
  observed source timestamps. Weekend and holiday handling must remain
  conservative rather than guessed.

## Decision Log

- Decision: widen only normalized timeframe `1`.
  Rationale: it closes the concrete downstream requirement with the smallest
  coherent contract change. Other guarded intraday intervals have no current
  workload evidence.
  Date/Author: 2026-07-27 / Codex

- Decision: preserve `bars.v1` and all existing date-range fields.
  Rationale: downstream already consumes the 5-minute and 15-minute form. A
  new version or parallel payload would create unnecessary integration work.
  Date/Author: 2026-07-27 / Codex

- Decision: keep 5,000 as a per-request returned-bar cap.
  Rationale: the owner explicitly requested the existing bound. Larger private
  corpora can use multiple explicit date windows without changing this public
  safety contract.
  Date/Author: 2026-07-27 / Codex

- Decision: do not reinterpret closure-only ranges as complete using a guessed
  calendar.
  Rationale: the source does not provide a reviewed exchange-calendar
  authority. No bars, entitlement gaps, closures, and source exhaustion must
  stay distinguishable through conservative existing diagnostics.
  Date/Author: 2026-07-27 / Codex

## Outcomes & Retrospective

Implementation corrections and focused non-live validation are complete.
Date-range validation accepts normalized `1` and the existing `1m` alias while
preserving every existing transport and `bars.v1` payload boundary. A
dedicated ignored harness now fixes the three bounded live cases and emits
aggregate-only evidence. Its symbol gate and classification checks now fail
closed on the contradictions found by focused review. Narrow re-review is
green. The first authorized harness run stopped on a structured `connection`
failure before range classification. A subsequent bounded production-binary
comparison showed the common five-minute path and all three intended
one-minute scenarios succeeding. The evidence supports current-build live
feasibility: single-window retrieval, one additional 500-bar fetch window, and
conservative closure-boundary classification all worked. The initial failure
is therefore classified as a transient common WebSocket connection failure,
not a one-minute date-range defect. The implementation, deterministic
validation, bounded live verification, and closeout are complete. No
dependency, stash, version, tag, push, workflow, or GitHub Release changed.

## Context and Orientation

`crates/market/src/bars/validation.rs` normalizes `1m` to `1` and accepts `1`
in count-only mode. `validate_bars_range_request_with_resolution` then checks
the normalized value against `DATE_RANGE_TIMEFRAMES`, now:

    ["1", "5", "15", "30", "60", "1D", "1W", "1M"]

The implementation widens only that allowlist and its error/help contract.

`crates/market/src/bars/types.rs` defines:

- `MAX_RECENT_BAR_COUNT = 500`;
- `MAX_DATE_RANGE_BAR_COUNT = 5_000`;
- `DATE_RANGE_FETCH_CHUNK = 500`;
- inclusive calendar-date requests represented as
  `[from_time, to_time + 86_400)`;
- `range_alignment.bar_timestamp_semantics = "period_start"`;
- truncation reasons `count_cap`, `timeout`, `source_exhausted`, and `none`.

`crates/market/src/bars/transport.rs` creates one WebSocket chart series,
merges bars by timestamp, requests older 500-bar windows while the oldest
observed timestamp is newer than `from`, stops when source progress stalls,
and uses one absolute request deadline. `finalize_result` filters timestamps
to the requested range, truncates returned bars to the requested count cap,
and builds `BarsFetchSummary`.

`crates/market/src/bars/payload.rs` owns `bars.v1` shaping and
`range_coverage_status`. `crates/cli/src/cli.rs`,
`crates/cli/tests/cli_contract_bars.rs`, README, stable source docs, packaged
guidance, and runtime skills enumerate the currently supported range
timeframes.

The reported downstream work successfully exercised the existing 5-minute and
15-minute contract and identified one-minute date ranges as the CLI blocker.
Its other private preparation work is not part of this plan.

## Plan of Work

### Milestone 1: Widen the exact validation boundary

Add normalized `"1"` to `DATE_RANGE_TIMEFRAMES`. Update the fixed validation
message and supported-timeframe details. Keep all other normalized timeframe
rules unchanged.

Tests must prove:

- `1` and `1m` produce a date-range `BarsRequest` with timeframe `"1"`;
- count 5,000 is accepted and 5,001 is rejected;
- invalid and reversed dates still fail;
- `3`, `45`, `120`, `180`, and `240` remain date-range validation errors;
- validation succeeds before any network attempt through the CLI contract
  boundary.

### Milestone 2: Fix the one-minute coverage contract with deterministic data

Use synthetic `Bar` values and the existing request/result helpers. Do not
create a second one-minute code path.

Tests must cover:

- timestamps exactly at `from` are included;
- timestamps immediately before `to + 1 day` are included and timestamps at
  `to + 1 day` are excluded;
- returned bars remain ascending and period-start semantics are unchanged;
- more than the requested cap yields `range_coverage_status: "partial"`,
  `range_truncated: true`, and `range_truncation_reason: "count_cap"`;
- an incomplete read yields explicit timeout truncation;
- a completed read that cannot establish both observed boundaries remains
  partial/source-exhausted;
- a no-progress fetch does not issue unbounded `request_more_data`;
- closure-shaped fixtures contain no synthetic bars and do not become complete
  through a hard-coded weekend or holiday rule;
- success and structured failures retain the existing allowlisted
  `range_fetch_summary`, availability, and alignment fields.

Do not change the 10-second request timeout, 500-bar fetch chunk, or truncation
vocabulary in this milestone. If deterministic tests expose an existing
timeframe-independent defect, stop and revise this plan rather than hiding the
fix inside the one-minute allowlist change.

### Milestone 3: Synchronize public guidance

Update the CLI help, README, `docs/command-source-taxonomy.md`,
`docs/internal-tradingview-apis.md`, packaged agent guide, chart-analysis,
market-data-interpretation, and multi-symbol-scan skills and references.

Guidance must state:

- date-range mode supports `1`, `5`, `15`, `30`, `60`, `1D`, `1W`, `1M`;
- `1m` aliases to `1`;
- the count cap is per request and remains 5,000;
- a larger corpus may require multiple explicit downstream windows;
- callers must inspect coverage and truncation fields rather than infer
  completeness from exit status or row count;
- `tv bars` remains Desktop-free and does not fall back to selected-chart
  data.

### Milestone 4: Validate and collect bounded live evidence

Add or extend an ignored integration test so date-range assertions exercise
the test-built production binary. Keep it disabled unless an explicit
environment gate is set. The run must be finite and must not compensate for a
failed trial with extra requests.

The live evidence should cover:

- one one-minute range that fits within one returned-bar cap;
- one range expected to need at least one additional 500-bar fetch window;
- one boundary containing a known non-trading calendar day, interpreted only
  through returned aggregate fields rather than a built-in calendar claim.

The ignored test is `one_minute_date_range_live_smoke`. It requires
`TV_LIVE_BARS_RANGE_SMOKE=1`, one exchange-qualified
`TV_LIVE_BARS_RANGE_SYMBOL`, and explicit `*_FROM` / `*_TO` values for
`SINGLE`, `PAGED`, and `CLOSURE`. It performs exactly those three subprocess
invocations, gives each child a 15-second outer deadline, and never retries or
adds a replacement case.

Print only aggregate fields: requested/completed counts, bar counts,
coverage status, fetch-window/request-more counts, truncation boolean/reason,
and elapsed milliseconds. Do not retain symbol, dates, bars, prices, raw
envelopes, endpoint values, or credentials in tracked evidence.

The live run is evidence for current provider behavior, not a guarantee for
all symbols, exchanges, entitlements, or historical periods. A partial or
truncated result is acceptable evidence when its classification is explicit
and internally consistent.

## Concrete Steps

Run from the repository root:

    rg -n "DATE_RANGE_TIMEFRAMES|validate_bars_range_request|should_request_more|range_coverage_status|range_truncated|range_alignment|5000" crates/market/src/bars crates/cli/src crates/cli/tests
    cargo test -p tradingview-market bars::validation -- --nocapture
    cargo test -p tradingview-market bars::transport -- --nocapture
    cargo test -p tradingview-market bars::payload -- --nocapture
    cargo test -p tradingview-cli --test cli_contract_bars -- --nocapture
    cargo test -p tradingview-cli --test live_bars -- --nocapture
    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1
    python3 scripts/check-public-hygiene.py --self-test
    python3 scripts/check-public-hygiene.py
    bash -n scripts/stage-release-package-files.sh
    cmp -s AGENTS.md CLAUDE.md
    git diff --check

Run the ignored live command only after implementation review and with its
documented explicit gate. Do not run unrelated ignored tests.

    TV_LIVE_BARS_RANGE_SMOKE=1 \
    TV_LIVE_BARS_RANGE_SYMBOL=<EXCHANGE:SYMBOL> \
    TV_LIVE_BARS_RANGE_SINGLE_FROM=<YYYY-MM-DD> \
    TV_LIVE_BARS_RANGE_SINGLE_TO=<YYYY-MM-DD> \
    TV_LIVE_BARS_RANGE_PAGED_FROM=<YYYY-MM-DD> \
    TV_LIVE_BARS_RANGE_PAGED_TO=<YYYY-MM-DD> \
    TV_LIVE_BARS_RANGE_CLOSURE_FROM=<YYYY-MM-DD> \
    TV_LIVE_BARS_RANGE_CLOSURE_TO=<YYYY-MM-DD> \
    cargo test -p tradingview-cli --test live_bars \
      one_minute_date_range_live_smoke -- --ignored --exact --nocapture

## Validation and Acceptance

Acceptance requires:

- exact timeframe `1` and alias `1m` work in `--from/--to` mode;
- `bars.v1`, source labels, JSON envelope, and existing fields remain stable;
- `range_alignment.bar_timestamp_semantics` remains `period_start`;
- `requested_range`, `returned_range`, `observed_range`,
  `range_coverage_status`, and `range_fetch_summary` behave identically to
  other date-range timeframes;
- the returned-bar cap remains 5,000 and any count-cap/timeout/source boundary
  is explicit rather than silent;
- boundary, cap, closure-shaped, partial, and no-progress tests are
  deterministic;
- other guarded intraday timeframes remain guarded;
- recent-count mode and unrelated sources have no behavior change;
- public and tracked evidence contains no downstream-private values;
- focused implementation and evidence review are green.

## Idempotence and Recovery

Deterministic tests and docs checks are safe to rerun. The ignored live test
must remain single-pass and bounded; do not automatically retry, widen dates,
increase count, change symbol, or add another source after a failure.

If a live run returns an unknown outcome, stop and record only its public-safe
classification. If the provider rejects one-minute date ranges despite the
shared protocol path, revert no evidence and do not try alternate signatures;
record a current-build no-go and return the plan for review.

Never apply or drop either local stash as part of this plan.

## Artifacts and Notes

Tracked artifacts may contain only fixed contract vocabulary, deterministic
fixture values, aggregate test counts, and public-safe live summaries. Private
symbols, prices, dates, basket membership, case labels, raw bars, authority
hashes, target IDs, endpoints, credentials, and machine paths are forbidden.

## Interfaces and Dependencies

The public command remains:

    tv bars <SYMBOL> --timeframe <TIMEFRAME> [--count <N>]
      [--from YYYY-MM-DD --to YYYY-MM-DD]

No option or payload field is added. The accepted date-range timeframe set adds
normalized `"1"` only. No dependency, feature flag, workflow, environment
default, or package layout change is planned.

The production owners remain:

- `crates/market/src/bars/validation.rs`: request validation and aliases;
- `crates/market/src/bars/types.rs`: bounds and public readback vocabulary;
- `crates/market/src/bars/transport.rs`: WebSocket fetching and pagination;
- `crates/market/src/bars/payload.rs`: `bars.v1` payload shaping;
- `crates/cli/src/cli.rs`: CLI help;
- `crates/cli/tests/cli_contract_bars.rs`: pre-network public contract;
- an ignored bars live test: bounded current-provider evidence.

## Open Questions

- UNCONFIRMED: whether the current provider accepts one-minute
  `request_more_data` over every public-safe live fixture under the existing
  10-second deadline.
- UNCONFIRMED: whether a single request is sufficient for the downstream
  corpus. The public cap remains 5,000 regardless; downstream windowing is the
  intended fallback when more rows are needed.
- UNCONFIRMED: whether closure-only ranges need a future calendar source. This
  slice makes no such source or completeness claim.
