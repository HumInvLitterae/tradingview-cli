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
- [ ] Obtain focused independent review before adding or running the
  disposable three-point mutation probe.
- [ ] Add the gated probe and deterministic probe-contract tests without
  adding a stable CLI option.
- [ ] Obtain separate owner approval for one disposable native
  `parallel_channel` create/read/remove probe on an explicitly selected chart.
- [ ] Run the mutation probe, record only public-safe aggregate evidence, and
  stop on any ambiguous or unverified state.
- [ ] If and only if mutation evidence and its focused review are green,
  implement the stable explicit-three-point CLI contract.
- [ ] Run focused/full validation and obtain independent implementation review
  before archiving this plan.

## Surprises & Discoveries

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

- Decision: support only explicit third-point geometry initially; defer
  width-derived channels.
  Rationale: explicit points preserve caller intent. Width derivation requires
  a separate contract for sign, units, time anchor, zero/negative width, and
  price normalization. Upstream behavior is evidence, not sufficient authority
  to choose those semantics here.
  Date/Author: 2026-07-15 / Codex.

- Decision: use native `parallel_channel` as the disposable feasibility shape,
  while keeping generic point3 support compatible with other TradingView
  three-point shape names such as `pitchfork`.
  Rationale: native parallel channels are the concrete roadmap workflow and
  have upstream live evidence. The CLI already accepts generic shape names; a
  broad allowlist would overstate support for private identifiers.
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
  shape identity when readable, and a ready three-point readback.
  Date/Author: 2026-07-15 / Codex.

- Decision: replace the fixed sleep and first-ID acceptance with one bounded
  absolute page-side observation deadline.
  Rationale: asynchronous registration needs bounded polling, but traffic must
  not extend the deadline. Zero IDs, multiple IDs, missing shape, malformed
  points, or deadline exhaustion cannot be success.
  Date/Author: 2026-07-15 / Codex.

- Decision: do not automatically remove multiple newly observed IDs.
  Rationale: concurrent user activity could contribute to an ambiguous
  baseline difference. Removing every delta could delete an unrelated drawing.
  Date/Author: 2026-07-15 / Codex.

- Decision: during the owner-authorized probe, remove only the one uniquely
  verified entity ID and require post-remove absence.
  Rationale: disposable cleanup must be identity-scoped. `draw clear`,
  display-name cleanup, broad polling cleanup, and layout reset are prohibited.
  Date/Author: 2026-07-15 / Codex.

- Decision: keep raw Runtime exceptions, Promise rejection text, raw DOM,
  function source, target IDs, and account/layout metadata out of diagnostics.
  Rationale: diagnostics need only fixed status enums, counts, validity flags,
  and chart-local entity IDs where exact manual cleanup requires them.
  Date/Author: 2026-07-15 / Codex.

## Outcomes & Retrospective

Planning is complete. The first slice is explicit third-point support in the
existing drawing command, with a concrete current-build proof obligation before
stable implementation. No Rust behavior, CLI option, dependency, workflow,
chart drawing, or TradingView state has changed. Independent plan review is the
next gate.

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

The probe uses one page-side async expression and retained chart API. It reads
baseline IDs and confirms `createMultipointShape`, `getAllShapes`,
`getShapeById`, and `removeEntity` are callable. It then calls exactly once:

    api.createMultipointShape([point1, point2, point3], {
        shape: "parallel_channel",
        overrides: {}
    })

It does not include text, try another shape, reorder points, derive width,
click UI, or call `createShape` as fallback.

Capture only `returned_non_thenable`, `fulfilled`, `rejected`, or `threw` as a
creation-signal enum; never expose rejection/exception text. Observe inventory
and shape readiness under one three-second absolute page-side deadline with a
100-millisecond interval. The deadline begins immediately before creation and
is never reset. A five-second outer Runtime deadline bounds evaluation.

Probe success requires one observation before the deadline with exactly one
ID in `after - before`, the same ID resolving through `getShapeById`, native
identity exactly equal to `parallel_channel` in the inventory row, and
`getPoints()` returning exactly three finite time/price entries in caller order.
Every observed time and price must equal its requested value without rounding,
tolerance, string coercion, bar lookup, or tick normalization. The owner must
therefore approve points already chosen from suitable loaded bar anchors and
tick-aligned prices. Any normalization or mismatch is no-go for this contract;
a different normalization contract requires plan revision and focused review.

If one verified entity exists, call `removeEntity` exactly once for that ID and
require absence from both lookup and inventory. Probe success requires cleanup
success. Zero/multiple IDs are no-go with no broad cleanup. One unique but
unverified candidate may receive one identity-scoped cleanup attempt before
no-go. Track only fixed statuses/counts in docs, never target ID, raw Runtime
payload, exception, or account/layout metadata.

Add deterministic tests for gate validation, fixed failures, field allowlist,
point order, deadline behavior, zero/one/multiple classification,
rejection-with-verified-state, and cleanup call counts. Do not run live mutation
until focused probe review and separate owner approval of target and points.

### Milestone 2: Add I/O-free three-point request validation

Only after mutation evidence and its review are green, extend
`DrawingShapeRequest` with `point3: Option<DrawingPoint>`. Add one model
validator used by dispatch and operation. It requires a non-empty trimmed shape
type, finite point values, and ordered arity: point1 always exists, point2 is
optional, and point3 is allowed only with point2.

Add `--price3` and `--time3` to `DrawingCommand::Shape`. Each point pair must be
complete before CDP. Reject point3 without point2. Reject non-empty text with
exact `parallel_channel`. Do not add `--width`, infer point3, or change
one/two-point defaults. Tests cover all pair/finite/ordering combinations.

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

Add an ignored executable JavaScript contract and
`scripts/check-three-point-drawing-js-contract.py` because fake Runtime payloads
cannot prove Promise/poll/readback/cleanup ordering. Run pinned Node.js
`24.18.0`, add `check:three-point-drawing-js` to `mise.toml`, and wire a named
required CI/release job. Normal `cargo test --workspace` remains Node-free.

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
CLI options. That no-go can close after evidence/outcome review without an
implementation.

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
finite ordered points, and removal of that entity. Promise fulfillment alone,
method presence, visual appearance alone, zero/multiple IDs, or an unremoved
entity is not go evidence.

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
inventory check. A uniquely observed ID before responsive failure may receive
the one in-expression removal described above. Multiple IDs remain untouched.

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

- UNCONFIRMED: whether requested times are preserved or bar-normalized.
- UNCONFIRMED: whether prices are exact or tick-normalized.
- UNCONFIRMED: whether creation return fulfills, rejects, or varies while chart
  state still verifies.
- UNCONFIRMED: the exact inventory name for native `parallel_channel`.
- UNCONFIRMED: whether width convenience merits a later separate slice.

2026-07-15: Created after the reviewed right-offset no-go closeout. The plan
adopts upstream #223 only as evidence, requires explicit point3, fixes
exactly-one postcondition and cleanup boundaries, defers width derivation, and
requires review plus separate owner approval before live mutation.
