# Add verified native three-point drawings

This ExecPlan is a living document. The sections `Progress`, `Surprises &
Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to
date as work proceeds.

Maintain this document in accordance with `.agents/PLANS.md` from the
repository root.

## Purpose / Big Picture

After this work, a user can create a TradingView native three-point drawing on
the selected Desktop chart through the existing `tv draw shape` command. The
first supported workflow is an explicit native parallel channel:

    tv draw shape --type parallel_channel \
      --time <T1> --price <P1> \
      --time2 <T2> --price2 <P2> \
      --time3 <T3> --price3 <P3>

The command creates one native TradingView object, returns its chart-local
`entity_id`, and remains compatible with `tv draw get` and `tv draw remove`.
It does not emulate a channel with two independent trend lines. It does not
derive the third point from a width value, because the sign, time anchor, and
price-unit semantics of that convenience have not been established as a
stable public contract.

Implementation is gated. A reviewed, owner-authorized disposable mutation
probe must first prove that the current TradingView Desktop build accepts the
exact three-point `createMultipointShape` call, exposes exactly one new entity,
provides a three-point readback, and lets the probe remove that exact entity.
Method presence or upstream evidence alone is not production go.

## Progress

- [x] (2026-07-15) Closed and archived the selected-chart right-offset plan as
  an independently reviewed integer-before contract no-go.
- [x] (2026-07-15) Inspected the current Rust drawing CLI, dispatch, model,
  create/read/remove operations, tests, public docs, and source taxonomy.
- [x] (2026-07-15) Inspected upstream pull request #223 at head
  `da77378f3f9920e1b238ec33f233882921bc6c49`, including its core operation,
  tool schema, deterministic tests, and reported live evidence.
- [x] (2026-07-15) Defined the initial Rust surface as explicit `point3`
  support on `tv draw shape`, with no width-derived convenience command.
- [x] (2026-07-15) Created this feasibility-gated ExecPlan and synchronized
  the current project state.
- [x] (2026-07-15) Applied the first focused-review correction wave: limited
  point3 to native `parallel_channel`, made ambiguity sticky, restricted probe
  cleanup to fully verified entities, defined Promise-independent observation,
  and fixed the no-go closeout path.
- [x] (2026-07-16) Focused independent re-review of the corrected plan reported
  no findings and authorized deterministic probe implementation.
- [x] (2026-07-16) Added the gated probe and deterministic probe-contract tests
  without running live or
  adding a stable CLI option.
- [x] (2026-07-16) Added the pinned production-expression Node.js gate to local
  tooling, CI, release, and development guidance while preserving Node-free
  ordinary Cargo tests.
- [x] (2026-07-16) Completed focused and full local validation without running
  the ignored live mutation test.
- [x] (2026-07-16) Obtained focused independent implementation review of the
  probe and its executable contract.
- [x] (2026-07-16) Obtained separate owner approval for one disposable native
  `parallel_channel` create/read/remove probe on an explicitly selected chart.
- [x] (2026-07-16) Ran the initially approved mutation probe once. It observed
  one new entity and three points, but exact point verification failed with
  `point_mismatch`. At owner direction, follow-up controlled runs then isolated
  native normalization and cleanup visibility behavior on the disposable
  target.
- [x] (2026-07-16) Investigated the initial mismatch on the disposable target.
  Readback showed bar-time normalization, native width-point canonicalization,
  and floating-point round-trip noise rather than a missing capability.
- [x] (2026-07-16) Confirmed a second probe false negative: exact geometry
  verification passed, removal was issued, and the entity disappeared shortly
  afterward, but the immediate cleanup readback was too early.
- [x] (2026-07-16) Validated the corrected width-point, numeric comparison, and
  cleanup-observation contract deterministically and on the disposable target.
  The final run verified and removed exactly one native entity.
- [x] (2026-07-16) Applied focused-review corrections for malformed inventory
  rows, executable fresh-cleanup coverage, immediate absence fields, exact-time
  mismatch, epsilon rejection, and stale durable state.
- [x] (2026-07-16) Distinguished a readable stale lookup handle from a failed
  lookup. Cleanup promotion now requires authoritative inventory absence and a
  successful lookup call, while lookup absence remains diagnostic.
- [x] (2026-07-16) Obtained focused review of the corrected evidence with no
  remaining findings; stable implementation was authorized.
- [x] (2026-07-16) Added paired third-point CLI/model validation and a bounded
  verified native `parallel_channel` production path while preserving the
  existing one/two-point path.
- [x] (2026-07-16) Completed model/operation/CLI contract tests, the pinned
  probe-and-production JavaScript gate, strict Clippy, workspace tests,
  metadata, hygiene, packaging syntax, guide parity, and diff checks.
- [x] (2026-07-16) Obtained focused independent implementation review; it
  reported Rust success-boundary, failure-whitelist, and durable-state
  findings, so public guidance, commit, and archive work remained blocked.
- [x] (2026-07-16) Applied implementation-review corrections: Rust now rejects
  contradictory success fields, normalizes Runtime enum strings and candidate
  handles through fixed public-safe rules, and covers each trust-boundary
  contradiction deterministically.
- [x] (2026-07-16) Closed the remaining success-boundary gap by requiring
  non-negative integer `before_count` / `after_count`, empty `text`, and a
  request-consistent integer `override_count`; missing and malformed variants
  now fail closed in focused regression tests.
- [x] (2026-07-16) Focused re-review reported no remaining findings after the
  preserved success-field correction.
- [x] (2026-07-16) Synchronized stable public, packaged, development, and skill
  guidance and completed the implementation closeout.

## Surprises & Discoveries

- Observation: native `parallel_channel` does not preserve an arbitrary third
  point as three independent coordinates. It canonicalizes the third point as
  a width point anchored to the first point's time.
  Evidence: the first disposable-target readback preserved the first two
  prices, normalized their times to loaded daily-bar anchors, and returned the
  third point at the first point's normalized time with a geometry-derived
  price.

- Observation: a canonical width point can round-trip with harmless binary
  floating-point noise.
  Evidence: with point3 time equal to point1 time, a requested decimal price
  was read back with only a machine-precision representation difference. An
  exactly representable price passed point verification.

- Observation: `removeEntity` can become observable after the immediate
  synchronous readback used by the initial probe.
  Evidence: the exactly representable run attempted cleanup and initially
  reported `cleanup_unverified`; the next read-only inventory contained zero
  drawings. Cleanup verification therefore needs bounded observation under the
  existing absolute deadline, not a second mutation or an unbounded wait.

- Observation: after verified removal, fresh inventory was authoritative while
  `getShapeById` still returned a stale handle.
  Evidence: the final corrected run reported `cleanup_inventory_absent: true`,
  `cleanup_lookup_absent: false`, and overall `verified_cleaned`. The chart
  inventory contained no candidate, and no second remove was issued.

- Observation: current Rust `tv draw shape` supports one point through
  `createShape` and two points through `createMultipointShape`, but the request
  model and CLI have no third point.
  Evidence: `crates/cli/src/ops/drawing/create.rs` constructs only `point` and
  optional `point2`; `crates/model/src/drawing.rs::DrawingShapeRequest` has the
  same two-point ceiling.

- Observation: current Rust creation waits a fixed 300 milliseconds and treats
  any non-null first new ID as success, even when more than one new ID appears.
  Evidence: `drawing_shape` computes `newIds`, returns `newIds[0]`, and checks
  only that `entity_id` is a string. Three-point support must not extend this
  ambiguous-success behavior.

- Observation: TradingView can normalize a supplied drawing time to its bar
  anchor.
  Evidence: the archived drawing-command live smoke supplied an intraday time
  and observed a normalized daily-bar time through `draw get`. Therefore the
  stable contract cannot assume arbitrary requested timestamps are returned
  exactly without first proving the normalization boundary.

- Observation: upstream pull request #223 reports that a three-point native
  `parallel_channel` can be created and removed, but the Promise returned by
  `createMultipointShape` may reject even when the shape is created.
  Evidence: the upstream operation deliberately does not use the Promise as
  its success boundary and polls `getAllShapes` instead.

- Observation: upstream pull request #223 returns `success: true` even if its
  bounded polling never finds a new entity ID.
  Evidence: `drawShape` returns `{ success: true, entity_id: newId }` after the
  loop without requiring `newId`. Rust must instead require exactly one new
  entity and verified three-point readback.

- Observation: upstream derives point3 from `point2.time` and
  `point2.price - width`, with positive width meaning a lower rail, but this is
  only one possible convention.
  Evidence: the wrapper fixes both sign and time anchor without a separate
  geometry contract. Explicit point3 avoids silently choosing those semantics.

## Decision Log

- Decision: extend `tv draw shape` with `--price3` and `--time3`; do not add a
  new top-level command or `draw parallel-channel` subcommand in this slice.
  Rationale: three-point creation is an arity extension of the existing generic
  drawing lifecycle. A second command would duplicate validation and ownership
  without adding a distinct capability.
  Date/Author: 2026-07-15 / Codex.

- Decision: require `--price3` and `--time3` together, and require a complete
  second point whenever a third point is supplied.
  Rationale: a third point without a second point has no coherent ordered point
  vector. Pair/ordering errors are I/O-free validation failures before CDP.
  Date/Author: 2026-07-15 / Codex.

- Decision: treat point3 for `parallel_channel` as a width point whose time must
  equal point1 time; do not claim support for an arbitrary third coordinate.
  Rationale: current-build live readback canonicalizes point3 to the first
  point's time. Requiring that anchor before mutation preserves caller intent
  and makes the native representation verifiable without reverse-engineering
  chart-coordinate projection.
  Date/Author: 2026-07-16 / Codex.

- Decision: compare finite prices with a scale-aware machine-epsilon boundary
  while keeping times exact after the probe's bar-anchor selection.
  Rationale: live readback changed only the binary representation of a decimal
  price. Exact JavaScript equality rejects an otherwise identical native value;
  a small relative epsilon does not permit tick, rounding, or material price
  normalization.
  Date/Author: 2026-07-16 / Codex.

- Decision: after the one permitted `removeEntity` call, first observe lookup
  and inventory under the page-side deadline. If that retained evaluation still
  reports the candidate, wait a fixed two seconds and perform exactly one
  separate read-only Runtime evaluation on a fresh CDP connection for the same
  candidate ID.
  Rationale: current-build removal was externally visible only after the
  mutation evaluation returned. A second read-only evaluation verifies the same
  mutation without retrying removal, broad cleanup, or another creation.
  Inventory absence is the completion boundary because inventory difference is
  also the authoritative creation-attribution boundary; lookup absence remains
  an additive diagnostic because a stale handle may outlive inventory removal.
  Every inventory row ID must be read successfully before absence can be true;
  malformed or throwing rows keep cleanup unverified. The lookup call must also
  complete successfully; a returned stale handle is diagnostic, but a thrown
  lookup keeps cleanup unverified.
  Date/Author: 2026-07-16 / Codex.

- Decision: in this initial slice, allow point3 only when the trimmed shape
  type is exactly `parallel_channel`.
  Rationale: the reviewed feasibility probe proves only native parallel
  channels. Allowing another three-point shape would mutate the chart before
  reaching a postcondition that the current evidence cannot satisfy. Other
  three-point shape types require their own feasibility evidence and plan
  revision before becoming stable.
  Date/Author: 2026-07-15 / Codex.

- Decision: reject non-empty `--text` when the trimmed shape type is exactly
  `parallel_channel`.
  Rationale: upstream current-build evidence reports that attaching text makes
  this native tool fail. Rejecting the known-invalid combination before CDP is
  safer than silently dropping user text.
  Date/Author: 2026-07-15 / Codex.

- Decision: treat the creation call's return or Promise settlement only as a
  sanitized signal, never as the success boundary.
  Rationale: upstream observed rejected Promises after successful creation.
  Success must come from chart state: exactly one new entity, expected native
  shape identity, and a ready three-point readback.
  Date/Author: 2026-07-15 / Codex.

- Decision: attach a non-blocking observer to a returned thenable and perform
  inventory polling independently of Promise settlement.
  Rationale: upstream evidence shows chart mutation and Promise outcome can
  diverge. Awaiting the Promise first would prevent bounded state verification
  and verified cleanup when the Promise never settles.
  Date/Author: 2026-07-15 / Codex.

- Decision: replace the fixed sleep and first-ID acceptance with one bounded
  absolute page-side observation deadline.
  Rationale: asynchronous registration needs bounded polling, but traffic must
  not extend the deadline. Zero IDs, multiple IDs, missing shape, malformed
  points, or deadline exhaustion cannot be success.
  Date/Author: 2026-07-15 / Codex.

- Decision: once any observation contains multiple new IDs, keep that run
  permanently ambiguous and never return it to success or automatic cleanup.
  Rationale: concurrent user activity could contribute to an ambiguous
  baseline difference. A later reduction to one ID does not establish that the
  remaining entity belongs to the probe.
  Date/Author: 2026-07-15 / Codex.

- Decision: during the owner-authorized probe, remove only an entity that has
  passed the complete native-identity and exact-point verification contract,
  then require post-remove absence.
  Rationale: uniqueness alone does not prove attribution. `draw clear`,
  display-name cleanup, unverified-candidate cleanup, broad polling cleanup,
  and layout reset are prohibited.
  Date/Author: 2026-07-15 / Codex.

- Decision: keep raw Runtime exceptions, Promise rejection text, raw DOM,
  function source, target IDs, and account/layout metadata out of diagnostics.
  Rationale: diagnostics need only fixed status enums, counts, validity flags,
  and chart-local entity IDs where exact manual cleanup requires them.
  Date/Author: 2026-07-15 / Codex.

## Outcomes & Retrospective

Planning review, deterministic probe implementation, stable implementation,
and focused re-review are complete. Follow-up investigation on the disposable target showed that the
initial no-go was not capability evidence: native width-point canonicalization,
machine-precision price round-trip, and delayed cleanup visibility each
conflicted with overly strict probe assumptions. The corrected probe contract
requires point3 to use the point1 time anchor, uses a narrow scale-aware price
comparison, and observes one cleanup mutation under the original absolute
deadline. The corrected live run then verified one native entity, three
canonical points, and inventory-confirmed cleanup. The stable CLI now exposes
verified native three-point `parallel_channel` creation while preserving the
existing one/two-point path. Its Rust success boundary validates both additive
verification metadata and preserved shape payload fields before success.

## Context and Orientation

The CLI surface is `crates/cli/src/cli.rs::DrawingCommand`. The `Shape` variant
accepts required `price` and `time`, optional paired `price2` and `time2`, text,
and JSON overrides. Dispatch in `crates/cli/src/app/dispatch.rs` validates the
pair, constructs `DrawingShapeRequest`, connects to Desktop, and calls
`ops::drawing_shape`.

I/O-free request types live in `crates/model/src/drawing.rs`. Creation is
`crates/cli/src/ops/drawing/create.rs::drawing_shape`. It records existing IDs,
invokes `createShape` or `createMultipointShape`, waits 300 milliseconds, and
returns the first new ID. Read and cleanup operations are
`crates/cli/src/ops/drawing/read.rs::drawing_get` and
`crates/cli/src/ops/drawing/lifecycle.rs::drawing_remove`.

A chart-local entity ID is the public handle returned by drawing commands and
accepted by `tv draw get/remove`. It is not a CDP target ID, layout ID, or
account-local identifier. A point is a finite Unix-seconds `time` and finite
chart `price`. Three-point creation passes exactly three point objects to
`createMultipointShape` in caller order.

Upstream pull request #223 is evidence, not implementation authority. Its head
is `da77378f3f9920e1b238ec33f233882921bc6c49`. It adds point3, native
`parallel_channel`, width convenience, polling, seven tests, and reported live
create/remove evidence. This plan adopts explicit point3 and native-object
evidence, rejects success without an ID, defers width, and strengthens
postconditions and failure handling.

## Plan of Work

### Milestone 1: Review and build a gated current-build mutation probe

After plan review is green, add an ignored integration test at
`crates/cli/tests/live_three_point_drawing_capability.rs`. It requires
`TV_LIVE_THREE_POINT_DRAWING_PROBE=1`, an explicit target ID, and six explicit
finite point values in `TV_LIVE_THREE_POINT_TIME1`,
`TV_LIVE_THREE_POINT_PRICE1`, `TV_LIVE_THREE_POINT_TIME2`,
`TV_LIVE_THREE_POINT_PRICE2`, `TV_LIVE_THREE_POINT_TIME3`, and
`TV_LIVE_THREE_POINT_PRICE3`. The target variable is
`TV_LIVE_THREE_POINT_TARGET_ID`. Missing, blank, malformed, or non-finite
values stop before CDP with fixed messages. The test never infers geometry from
quote, bars, screenshots, visible-range mutation, or another source.

The probe uses one page-side async mutation expression and retained chart API. It reads
baseline IDs and confirms `createMultipointShape`, `getAllShapes`,
`getShapeById`, and `removeEntity` are callable. It then calls exactly once:

    api.createMultipointShape([point1, point2, point3], {
        shape: "parallel_channel",
        overrides: {}
    })

It does not include text, try another shape, reorder points, derive width,
click UI, or call `createShape` as fallback.

Call the creation method exactly once. If its return is thenable, attach one
non-blocking settlement observer with `.then`/`.catch`; do not await settlement
before observing chart state. Inventory polling, readback, and verified cleanup
run independently of Promise settlement. Capture only
`returned_non_thenable`, `fulfilled`, `rejected`, `threw`, or
`pending_at_observation` as a creation-signal enum; never expose
rejection/exception text. `pending_at_observation` means the thenable had not
settled when the page-side operation reached its terminal observation result.
Settlement after that terminal observation does not revise the returned signal
or trigger more polling, readback, cleanup, or mutation.

Observe inventory and shape readiness under one three-second absolute
page-side deadline with a 100-millisecond interval. The deadline starts
immediately before the exactly-once creation call and is never reset by
polling, Promise activity, or readback. A five-second outer Runtime deadline
bounds evaluation.

Probe success requires that no observation has ever contained multiple new
IDs, followed by one observation before the deadline with exactly one ID in
`after - before`, the same ID resolving through `getShapeById`, native
identity exactly equal to `parallel_channel` in the inventory row, and
`getPoints()` returning exactly three finite time/price entries in caller order.
Point3 time must equal point1 time before CDP because native
`parallel_channel` represents the third point as a width point anchored there.
Observed times must equal the requested loaded-bar anchors exactly. Prices use
only a scale-aware `8 * Number.EPSILON` comparison; string coercion, tick
rounding, bar lookup, or broader normalization remains prohibited.

If one fully verified entity exists, call `removeEntity` exactly once for that
ID and observe absence from both lookup and inventory under the same absolute
deadline. If the entity is still visible when that expression returns, the Rust
runner may execute exactly one fixed read-only cleanup readback for the same
candidate ID on a fresh CDP connection after a fixed two-second delay. It
must not call remove again, create again, or broaden the candidate set. Probe
success requires inventory absence; lookup absence is reported separately but
does not override authoritative inventory removal. `cleanup_lookup_readable`
must be true so a lookup exception cannot masquerade as a stale handle.
Inventory absence requires every row to expose a readable string ID; an
unreadable or malformed row fails closed as `cleanup_unverified`.
Zero IDs, any observation of multiple IDs, identity mismatch,
point mismatch, or lookup/readback failure are no-go with no automatic
cleanup. Candidate chart-local IDs may be returned through the public-safe
manual-inspection handle field, but recovery or removal then requires a
separate owner-approved operation. Track only fixed statuses/counts in docs,
never target ID, raw Runtime payload, exception, or account/layout metadata.

Add deterministic tests for gate validation, fixed failures, field allowlist,
point order, deadline behavior, zero/one/multiple classification, sticky
ambiguity, never-settling and late-settling thenables,
rejection-with-verified-state, Promise-independent polling, and cleanup
ordering/call counts. The executable fixture must run the exact production
JavaScript expression generated by Rust; it must not duplicate the production
state machine in fixture-only logic. Do not run live mutation until focused
probe review and separate owner approval of target and points.

Add `scripts/check-three-point-drawing-js-contract.py` in this milestone. It
runs the ignored Rust test that emits and executes the exact production probe
expression under pinned Node.js `24.18.0`. Add
`check:three-point-drawing-js` to `mise.toml` and wire a named required
CI/release job while keeping normal `cargo test --workspace` Node-free. These
deterministic probe gates must be green and independently reviewed before
requesting live-mutation approval.

### Milestone 2: Add I/O-free three-point request validation

Only after mutation evidence and its review are green, extend
`DrawingShapeRequest` with `point3: Option<DrawingPoint>`. Add one model
validator used by dispatch and operation. It requires a non-empty trimmed shape
type, finite point values, and ordered arity: point1 always exists, point2 is
optional, and point3 is allowed only with point2.

Add `--price3` and `--time3` to `DrawingCommand::Shape`. Each point pair must be
complete before CDP. Reject point3 without point2. When point3 is present,
require the trimmed shape type to equal `parallel_channel` exactly. Reject
non-empty text with exact `parallel_channel`. Do not add `--width`, infer
point3, or change one/two-point defaults. Tests cover all
pair/finite/ordering/type combinations.

### Milestone 3: Implement verified bounded creation

Refactor `drawing_shape` around one point vector. One point calls
`createShape`; two/three call `createMultipointShape`. Serialize points/options
through `serde_json`, and invoke the selected creation method exactly once.

Reuse the probe-proven observation sequence. Success requires exactly one new
ID and readback consistent with requested arity. Three-point success requires
exact `parallel_channel` identity and exact point-array equality under this
contract. Promise signal never overrides a failed state postcondition.

Preserve existing fields and add `point3`, `requested_point_count`,
`observed_point_count`, fixed `creation_signal`,
`verification_status: "verified"`, `source: "chart_api"`,
`source_category: "desktop_backed_operation"`, `requires_desktop: true`, and
`non_mutating: false`.

Zero/multiple IDs, unresolved shape, wrong/malformed points, normalization
mismatch, and deadline expiration are `InternalApiUnavailable`. One or more
unverified candidates never trigger automatic production removal, because a
concurrent user drawing could be part of the inventory delta. Error details use
a whitelist of operation, requested shape/count, new-shape count, fixed
observation status, source metadata, and candidate chart-local IDs for exact
manual inspection or cleanup. Raw Promise values, shape objects, method source,
DOM, target/layout/account IDs, and exceptions are excluded.

### Milestone 4: Add contracts and synchronize guidance

Tests cover one/two/three-point serialization, validation before connection,
parallel-channel text rejection, exactly-once creation, rejected-Promise with
verified-state success, zero/multiple failure, probe cleanup failure,
production no-auto-cleanup, exact point equality, metadata, and existing
drawing regressions.

Extend the already reviewed executable JavaScript probe contract with the
stable production creation path. Fake Runtime payloads cannot prove
Promise/poll/readback/cleanup ordering. The contract continues to execute the
production expressions generated by Rust and covers never-settling, late
fulfill/reject, rejection-with-verified-state, sticky ambiguity, and
verified-only probe cleanup. Do not reconstruct the state machine in
fixture-only logic or add a second JavaScript gate for the same ownership
boundary.

Update README, source taxonomy, observation workflows, development/internal
API docs, packaged guidance, and chart-analysis drawing reference. Keep skill
core workflow concise; put options/cleanup details in the reference. Explain
explicit mutation, explicit point3, returned-ID cleanup, and prohibition on
`draw clear` for disposable cleanup.

### Milestone 5: Validate and obtain independent implementation review

Run focused/full validation. An optional final live smoke requires new owner
approval and explicit disposable points plus exact-ID removal. Independent
review covers validation ownership, point order/normalization, call counts,
absolute deadline, Promise precedence, attribution, cleanup, diagnostics,
metadata, docs, workflow gates, and existing drawing regressions.

Archive only after review is green. If the mutation probe cannot establish one
native entity, three ready points, and identity cleanup, record no-go and add no
CLI options. For this no-go path, update this plan's `Progress`, `Surprises &
Discoveries`, `Decision Log`, and `Outcomes & Retrospective`; obtain focused
review of the evidence and outcome; then move this plan to
`docs/plans/archives/` and synchronize `docs/plans/README.md`,
`docs/v0.28-roadmap.md`, `docs/v0.28-work-items.md`, `CHANGELOG.md`, and
`CONTINUITY.md` with the probe result, cleanup result, and absence of a stable
option. The probe executable gate remains in `mise.toml` and CI/release while
the ignored probe code remains, because it protects the reviewed safety and
cleanup state machine. Removing the probe later must remove that gate in the
same separately reviewed slice. Do not archive or claim closeout before the
no-go evidence/outcome review is green.

## Concrete Steps

Run from the repository root before code/live work:

    rg -n "DrawingShapeRequest|createMultipointShape|drawing_shape|point2" crates docs
    cargo test -p tradingview-model drawing -- --nocapture
    cargo test -p tradingview-cli drawing_shape -- --nocapture

Compile the reviewed ignored probe without running it:

    cargo test -p tradingview-cli --test live_three_point_drawing_capability -- --nocapture

The live command is specified only after owner approval. Never put target or
point environment values in tracked files.

After implementation is authorized:

    cargo test -p tradingview-model drawing -- --nocapture
    cargo test -p tradingview-cli drawing_shape -- --nocapture
    cargo test -p tradingview-cli --test cli_contract_desktop draw -- --nocapture
    mise run check:three-point-drawing-js

Run the full baseline:

    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1
    python3 scripts/check-public-hygiene.py --self-test
    python3 scripts/check-public-hygiene.py
    bash -n scripts/stage-release-package-files.sh
    cmp -s AGENTS.md CLAUDE.md
    git diff --check

## Validation and Acceptance

Probe acceptance requires exactly one native `parallel_channel`, exactly three
finite ordered points, and removal of that fully verified entity. Promise
fulfillment alone, method presence, visual appearance alone, zero/multiple
IDs, a unique but unverified candidate, or an unremoved entity is not go
evidence. Multiple new IDs observed at any point make the run permanently
ambiguous.

Implementation acceptance requires paired third-point help/options; pre-CDP
validation; preserved one/two-point contracts; one verified native object with
additive metadata; three-point `draw get`; exact-ID `draw remove`; no width,
fallback, or broad cleanup; all checks; and independent review.

## Idempotence and Recovery

Planning, compilation, and deterministic tests are repeatable. The live probe
is not repeated automatically. It performs at most one creation and one exact-ID
removal. It never calls `draw clear`, removes by name, reloads layout, changes
symbol/timeframe, or tries another signature.

An outer Runtime timeout is unknown outcome: no second mutation or automatic
cleanup expression. Recovery requires a separate owner-approved read-only
inventory check. A responsive `cleanup_unverified` result after the one
verified removal may use one fixed read-only evaluation for that exact ID.
Unique but unverified and multiple IDs remain untouched.

## Artifacts and Notes

    upstream PR: tradesdontlie/tradingview-mcp#223
    head: da77378f3f9920e1b238ec33f233882921bc6c49
    files: src/core/drawing.js, src/tools/drawing.js,
           tests/draw_parallel_channel.test.js
    reported live result: native parallel_channel create/remove succeeded
    upstream gap: success can contain entity_id null after polling

Tracked evidence contains only shape type, requested/observed point counts,
fixed creation signal, new-ID count, cleanup status, and go/no-go decision.

## Interfaces and Dependencies

If feasibility is green, extend:

    tv draw shape --type <TYPE> --price <P1> --time <T1> \
      [--price2 <P2> --time2 <T2>] \
      [--price3 <P3> --time3 <T3>] \
      [--text <TEXT>] [--overrides <JSON>]

In `crates/model/src/drawing.rs`:

    pub struct DrawingShapeRequest {
        pub shape_type: String,
        pub point: DrawingPoint,
        pub point2: Option<DrawingPoint>,
        pub point3: Option<DrawingPoint>,
        pub text: Option<String>,
        pub overrides: Option<serde_json::Value>,
    }

Define one model validator and re-export it through CLI drawing validation.
Keep the existing `drawing_shape` async operation signature. No new production
dependency, top-level command, JSONL contract, daemon, source fallback,
ranking, recommendation, or version bump is allowed.

## Open Questions

- UNCONFIRMED: whether the same width-point and cleanup timing semantics are
  stable across other TradingView Desktop builds and chart resolutions.
- UNCONFIRMED: whether width convenience merits a later separate slice.

2026-07-15: Created after the reviewed right-offset no-go closeout. The plan
adopts upstream #223 only as evidence, requires explicit point3, fixes
exactly-one postcondition and cleanup boundaries, defers width derivation, and
requires review plus separate owner approval before live mutation.

2026-07-15: Applied the first focused-review correction wave. Point3 is limited
to exact `parallel_channel`; multiple-ID ambiguity is sticky; only a fully
verified entity can be removed automatically; Promise settlement is observed
without blocking inventory/readback/cleanup; and the no-go durable closeout
path is explicit.

2026-07-16: Focused plan re-review reported no findings. Implemented the
ignored live probe, fixed environment/config validation, exact
production-expression Node.js fixtures, and required local/CI/release gate.
The live test was compiled but not run; no target or point values were selected
and no drawing mutation occurred. Focused/full validation is green, including
the pinned Node gate, strict Clippy, and the Node-free workspace test suite.

2026-07-16: Focused implementation re-review reported no findings. The first
live run exposed native bar/width-point normalization rather than a missing
creation capability. Follow-up read-only inspection and controlled runs on the
owner-designated disposable target isolated two probe defects: exact decimal
price equality and immediate-only cleanup readback. The plan now reflects the
native point1-anchored width point, scale-aware machine-epsilon comparison, and
bounded cleanup observation. Corrected evidence review, stable implementation,
and focused implementation re-review are green.

2026-07-16: The corrected deterministic fixture and final disposable-target
run are green. The run created one native candidate, verified three canonical
points with scale-aware price comparison, called remove once, and confirmed
fresh-inventory absence after the fixed delay. Lookup retained a stale handle,
so it remains diagnostic rather than authoritative. No drawing remained and no
stable CLI option was added at that probe stage. Focused evidence review then
passed, the stable CLI option was implemented and fully validated, and
implementation-review trust-boundary corrections were applied before the final
focused re-review.

2026-07-16: Focused re-review closed the final preserved-field success-boundary
finding. Stable public and packaged guidance was synchronized, final validation
passed, and this completed ExecPlan was archived.
