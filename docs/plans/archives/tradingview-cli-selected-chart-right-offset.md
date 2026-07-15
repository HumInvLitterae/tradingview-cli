# Add verified selected-chart right-offset control

This ExecPlan is a living document. The sections `Progress`, `Surprises &
Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to
date as work proceeds.

Maintain this document in accordance with `.agents/PLANS.md` from the
repository root.

## Purpose / Big Picture

After this work, a user can inspect or set the number of empty bar slots shown
to the right of the latest bar on the selected TradingView Desktop chart. This
creates explicit future or projection space for extended trendlines, channels,
and screenshots without making `tv range` or `tv screenshot` change the chart
implicitly.

The intended surface is `tv right-offset`, which reads the current value when
called without a value, `tv right-offset <BARS>` to set a bounded non-negative
integer, and `tv right-offset --reset` to set the explicit reset value `0`.
Mutation success requires finite before/after readback from the same selected
chart time scale. The command is not implemented until a bounded current-build
feasibility probe proves the exact getter, setter, and restoration path.

## Progress

- [x] (2026-07-15) Closed and archived the independently reviewed launch
  environment hardening ExecPlan.
- [x] (2026-07-15) Inspected the current Rust chart operation, dispatch, model,
  help, source-taxonomy, and visible-range boundaries.
- [x] (2026-07-15) Inspected upstream pull request #225 as design evidence. It
  uses `model().timeScale().setRightOffset(n)` and reports live projection-space
  movement, but it does not verify a getter or restoration contract.
- [x] (2026-07-15) Created this self-contained feasibility-gated implementation
  plan and synchronized the current project state.
- [x] (2026-07-15) Applied initial review corrections: bounded the probe
  candidate from an in-range before value, required one-shot restoration for
  every responsive post-mutation failure, added an executable JavaScript
  contract gate, and corrected the continuity goal and authorization scope.
- [x] (2026-07-15) Applied focused re-review corrections: required production
  set/reset to validate the current value before mutation, fixed exact
  per-branch JavaScript call counts, and named the CI and release gate wiring.
- [x] (2026-07-15) Corrected restoration-setter throw handling so the retained
  time-scale getter is still called exactly once before the branch terminates.
- [x] (2026-07-15) Obtained focused independent review of the corrected plan;
  the reviewer reported no remaining findings and allowed the read-only probe.
- [x] (2026-07-15) Ran the bounded read-only current-build capability probe.
  The selected chart exposed the expected ownership path, callable setter and
  getter, and readable visible range, but the getter returned a finite
  non-integer number. The strict read-only gate therefore recorded no-go.
- [x] (2026-07-15) Stopped before owner approval or mutation feasibility because
  the read-only gate was no-go. No setter was invoked.
- [x] (2026-07-15) Stopped production implementation because both feasibility
  gates were not green. No stable command was added.
- [x] (2026-07-15) Left production expression, payload, dispatch, help, CLI
  contracts, workflow gates, and runtime skills unchanged because the
  implementation gate was no-go.
- [x] (2026-07-15) Synchronized the plan index, roadmap, work inventory,
  changelog, and continuity ledger with the read-only no-go evidence.
- [x] (2026-07-15) Ran focused probe compilation, the explicit read-only live
  probe, formatting, strict Clippy, full workspace tests, metadata, public
  hygiene, packaging syntax, guide parity, and diff checks successfully.
- [x] (2026-07-15) Corrected the probe's transport-configuration failure path
  to emit only a fixed public-safe message, limited the no-go conclusion to
  this plan's integer-before contract, and added a viable no-go archive path.
- [x] (2026-07-15) Obtained focused independent re-review of the corrected
  probe, limited no-go classification, float-aware future boundary, and no-go
  closeout route; the reviewer reported no findings.
- [x] (2026-07-15) Archived this plan as an integer-before contract no-go
  without promoting the untested finite-`f64` candidate or authorizing chart
  mutation.

## Surprises & Discoveries

- Observation: current Rust chart viewport code already owns selected-chart
  `timeScale()` access under `crates/cli/src/ops/chart.rs` and
  `crates/cli/src/ops/chart/visible_range.rs`, but no right-offset getter or
  setter appears in the repository.
  Evidence: repository search finds `zoomToBarsRange` but no `rightOffset` or
  `setRightOffset` production call.

- Observation: upstream PR #225 demonstrates a setter candidate and practical
  projection-space behavior, but its success contract catches setter errors,
  waits a fixed 300 ms, and reads only visible range. It does not prove the
  requested offset was accepted or restored.
  Evidence: the PR calls `timeScale().setRightOffset(n)` and returns
  `getVisibleRange()` plus the requested value.

- Observation: right offset changes the viewport composition and may move the
  left edge inward because the visible width is fixed.
  Evidence: upstream live notes explicitly report this effect. Therefore this
  command must not be a hidden follow-up inside range or screenshot workflows.

- Observation: the current-build selected-chart ownership path exposes
  callable `setRightOffset` and `rightOffset` methods, and `rightOffset()`
  returned a finite number, but the observed value was not an integer.
  Evidence: the bounded ignored live probe used one retained time-scale
  reference, called no setter, and reported only aggregate capability flags and
  a public-safe viewport count classification.

- Observation: a finite non-integer current value cannot satisfy this plan's
  bounded integer `before` contract or exact integer restoration requirement.
  Evidence: Milestone 1 requires an integer getter result, and Milestones 2 and
  3 prohibit mutation when `before` is non-integer.

- Observation: the public TradingView time-scale API describes both
  `rightOffset()` and `setRightOffset(offset)` with the JavaScript `number`
  type rather than an integer-only type.
  Evidence: the official Advanced Charts
  [`ITimeScaleApi`](https://www.tradingview.com/charting-library-docs/latest/api/interfaces/Charting_Library.ITimeScaleApi/)
  reference. This does not prove the private current-build setter's mutation or
  restoration semantics, but it means the observed fractional value is not
  evidence that right-offset control as a whole is impossible.

## Decision Log

- Decision: promote selected-chart right offset before native three-point
  drawings.
  Rationale: it is listed first in the ordered work inventory, has a narrow
  chart-local mutation boundary, and the roadmap permits at most one of these
  candidates before the completion audit.
  Date/Author: 2026-07-15 / Codex.

- Decision: use a top-level `tv right-offset` get/set surface rather than add a
  mutation beneath `tv chart compare`.
  Rationale: existing selected-chart controls such as `symbol`, `timeframe`,
  `type`, and `range` are top-level get/set commands. The `tv chart` group is
  currently a specialized multi-symbol comparison workflow.
  Date/Author: 2026-07-15 / Codex.

- Decision: accept only integer values from `0` through `500`; `--reset` means
  exactly `0` and conflicts with a positional value.
  Rationale: right offset is measured in discrete bar slots. Non-negative
  values avoid undocumented negative scrolling behavior, and 500 is a bounded
  projection range large enough for practical use without exposing an
  unbounded private API call. Zero has a clear meaning: no empty bar slots are
  requested to the right.
  Date/Author: 2026-07-15 / Codex.

- Decision: gate implementation behind read-only capability evidence and a
  separately approved reversible mutation probe.
  Rationale: upstream evidence proves a setter candidate but not the exact
  current-build getter, immediate readback, or restoration semantics required
  by this repository's fail-closed mutation standard.
  Date/Author: 2026-07-15 / Codex.

- Decision: never call `tv range`, `tv screenshot`, drawing operations, or an
  alternate UI/DOM path as fallback.
  Rationale: right offset is an explicit selected-chart mutation. Hidden source
  or mutation mixing would make viewport evidence ambiguous.
  Date/Author: 2026-07-15 / Codex.

- Decision: treat every responsive page-side failure after the probe or
  production setter may have run as a restoration branch, while treating an
  outer Runtime timeout as an unknown outcome that forbids automatic recovery.
  Rationale: setter application and JavaScript exception delivery are not an
  atomic boundary. A setter throw or a later malformed/throwing getter can
  leave changed state, but a caller-side timeout cannot safely establish
  whether another setter invocation would restore or compound that state.
  Date/Author: 2026-07-15 / Codex.

- Decision: stop before mutation when the observed `before` value is outside
  `0..=500`; otherwise probe `before + 1`, except probe `499` from `500`.
  Rationale: the reversible probe needs a distinct candidate inside the public
  contract and must not assume that an out-of-contract value can be restored
  through the proposed bounded setter contract.
  Date/Author: 2026-07-15 / Codex.

- Decision: require an executable production-expression contract under pinned
  Node.js in addition to Rust fake-runtime tests.
  Rationale: fake Runtime payloads cannot prove same-object identity, exact
  getter/setter ordering and count, or one-shot restoration after JavaScript
  exceptions. The dedicated gate must run in CI and release builds without
  making the ordinary Cargo baseline depend on Node.js.
  Date/Author: 2026-07-15 / Codex.

- Decision: production set/reset must validate that the captured `before`
  value is a finite integer in `0..=500` before invoking any setter.
  Rationale: restoration is safe only when the original value is inside the
  same bounded contract proved by feasibility. Getter throw, malformed,
  non-finite, non-integer, or out-of-range values stop with zero requested or
  restoration setter calls.
  Date/Author: 2026-07-15 / Codex.

- Decision: record current-build feasibility as no-go rather than round,
  truncate, coerce, or temporarily overwrite the finite non-integer value.
  Rationale: any such conversion would lose the exact pre-mutation viewport
  state and invalidate this plan's reviewed restoration contract.
  Date/Author: 2026-07-15 / Codex.

- Decision: limit this result to the integer-before contract; do not classify
  the broader right-offset capability as no-go.
  Rationale: a separate design could keep public requests as integers in
  `0..=500` while retaining the exact finite internal `before` value as `f64`
  for readback and restoration without rounding. That design is untested and
  must be promoted through a separate ExecPlan, focused review, and separate
  owner approval before any mutation probe. It is not an automatic
  continuation of this plan.
  Date/Author: 2026-07-15 / Codex.

## Outcomes & Retrospective

The corrected plan passed focused review and the bounded read-only probe is
complete. Current-build ownership, setter/getter presence, finite numeric
readback, and visible-range readability were confirmed, but the getter result
was non-integer. This violates the reviewed integer-before gate, so that
specific contract is no-go. It does not establish a capability-wide no-go: an
exact finite-`f64` restoration design remains an untested future candidate
that requires its own plan, review, and mutation approval. No setter, chart
mutation, stable command, production behavior, dependency, or workflow change
was added. Focused independent re-review reported no findings, so this plan is
complete and archived through its documented no-go path. The broader
right-offset capability remains undecided unless a future finite-`f64` design
is deliberately promoted through a separate plan and approval sequence.

## Context and Orientation

The CLI command enum is `crates/cli/src/cli.rs::Command`. Top-level dispatch is
in `crates/cli/src/app/dispatch.rs`. Selected-chart operations are adapters
under `crates/cli/src/ops/chart.rs`; the larger historical visible-range runner
is split into `crates/cli/src/ops/chart/visible_range.rs`. Runtime JavaScript is
evaluated through `tradingview_cdp::RuntimeEvaluator`. Pure request validation
and payload decisions belong in `crates/model/` when they do not need CDP.

The selected chart means the chart target resolved by the existing global
target-selection flow. This command must not pick another target, switch
symbols, load browserless bars, or query scanner data. A right offset is the
number of empty logical bar slots displayed after the latest loaded bar. It is
a viewport setting, not historical data and not a timestamp range.

Upstream PR #225 is evidence, not implementation authority. Its candidate path
is `chart._chartWidget.model().timeScale().setRightOffset(n)`. This repository
requires a current-build readback from that same time-scale object, exact
post-check, public-safe diagnostics, and restoration evidence before exposing
the setter as stable behavior.

## Plan of Work

### Milestone 1: Prove the current-build capability without mutation

Add an ignored, explicitly invoked probe or use a bounded one-off local probe
that connects through the normal selected-target runtime. It must evaluate one
public-safe object containing only these aggregate facts: whether
`model().timeScale()` resolves, whether `setRightOffset` is callable, whether a
candidate getter named `rightOffset` is callable, whether invoking the getter
returns a finite integer, and whether the visible range can be read. Do not
return function source, object keys, raw Runtime payload, target identifiers,
layout identifiers, chart contents, symbol, or account metadata.

The read-only gate is provisional go only when exactly one time-scale object is
used, setter and getter are callable, and the getter returns an integer in a
reasonable JavaScript-safe range. Method presence alone is not production go.
If the getter is absent, throws, returns a non-finite value, or requires an
alternate ownership path, record no-go and revise this plan before any
mutation. Do not probe alternate setter signatures automatically.

At the end of this milestone, the plan records the exact current-build getter
and setter ownership path or stops without changing chart state.

### Milestone 2: Prove set, immediate readback, and exact restoration

Proceed only after Milestone 1 is green, focused review confirms the exact
probe, and the owner separately authorizes a chart viewport mutation. Capture
the finite integer `before` value. Continue only when `before` itself is in
`0..=500`; an out-of-range current value is no-go before mutation because this
plan does not assume the setter can safely restore values outside the public
contract. Choose `before + 1` when `before < 500`; when `before == 500`, choose
`499`. The resulting candidate is always in `0..=500` and differs from
`before`. Any failure to construct that exact candidate stops before mutation.

Use one bounded page-side async expression that resolves `model` and `ts`
exactly once and retains that `ts` reference for the complete sequence. Call
`ts.setRightOffset(probe)` exactly once. The first immediate
`ts.rightOffset()` readback after the call must equal `probe` exactly.
Fixed-delay polling, coercion, visible-range movement alone, or a requested
value echoed by the probe is not acceptance. Then call
`ts.setRightOffset(before)` exactly once and require the first immediate
`ts.rightOffset()` readback to equal `before`. Record only `before`, requested
probe value, observed value, restored value, and boolean status. These are
viewport counts, not private identifiers.

The expression catches errors by stage and returns only a fixed stage label;
it never returns exception text. Once the probe setter has been invoked,
setter throw, post-set getter throw, non-finite or non-integer post-set value,
and exact-value mismatch all mean mutation may have occurred. For every such
responsive page-side failure, call `ts.setRightOffset(before)` exactly once.
Immediately after that call completes by return or throw, perform exactly one
`ts.rightOffset()` restoration readback on the retained object. This getter is
read-only evidence of whether an apply-then-throw setter restored the value; it
is not a retry. A restoration getter throw, malformed readback, non-finite or
non-integer value, or value other than `before` ends with no additional
setter/getter attempt. Report the fixed restoration outcome and do not
reinterpret a failed probe as production go.
If the outer Runtime evaluation times out and mutation outcome is unknown, do
not retry, poll, or restore automatically; obtain owner approval for a
read-only recovery observation.

At the end of this milestone, current-build evidence proves a reversible exact
integer setter contract or the plan records no-go and no stable command is
added.

### Milestone 3: Add the explicit get/set/reset command

Proceed only when both feasibility milestones and their focused review are
green. Add `Command::RightOffset { bars: Option<i64>, reset: bool }` in
`crates/cli/src/cli.rs`, map its command name to `right-offset`, and dispatch it
in `crates/cli/src/app/dispatch.rs`. Validate before CDP connection that the
positional value is in `0..=500`, that `--reset` is not combined with a value,
and that no-input mode is a read.

Put pure validation and action selection in a small
`crates/model/src/right_offset.rs` module if that avoids embedding the command
matrix in dispatch. Export only the minimum typed request needed by the CLI.
Keep CDP expression ownership in a focused chart operation module such as
`crates/cli/src/ops/chart/right_offset.rs`; do not enlarge visible-range paging
with unrelated state.

The read operation calls the reviewed getter and returns
`contract_version: "chart_right_offset.v1"`, `operation: "right_offset"`,
`action: "read"`, `source: "selected_chart_cdp"`,
`source_category: "desktop_backed_read"`, `requires_desktop: true`,
`non_mutating: true`, and `right_offset`.

Set and reset use one page-side expression that resolves the time-scale object
once and captures `before`. Before invoking any setter, require `before` to be
a finite integer in `0..=500`. A throwing getter, malformed value, non-finite
number, non-integer number, or out-of-range integer fails before mutation with
zero requested setter and zero restoration setter calls. Only after this check
does the expression call the reviewed setter exactly once and perform the first
immediate getter readback. Success requires exact equality. Their
payload uses the same contract marker with `action: "set"` or `"reset"`,
`source_category: "desktop_backed_operation"`, `non_mutating: false`,
`requested_right_offset`, `before_right_offset`, `observed_right_offset`,
`changed`, and `verified: true`. Reset requests exactly zero. Do not claim that
zero is TradingView's product default; it is this command's explicit no-empty-
slots reset value.

After the production setter is invoked, setter throw, post-set getter throw,
non-finite or non-integer post-set value, and exact-value mismatch all trigger
the same bounded recovery inside that expression: call
`ts.setRightOffset(before)` exactly once. Whether that restoration setter
returns or throws, call `ts.rightOffset()` exactly once immediately afterward
on the retained object. Return failure with
`restoration_attempted`, `restored`, and a fixed restoration stage; never
return success from a recovery branch. Restoration setter/getter throw,
malformed, non-finite, non-integer, or mismatched restoration readback ends the
expression without another setter/getter attempt. Raw
exception values never cross CDP. An outer Runtime timeout has unknown mutation
outcome and triggers no automatic retry, polling, restoration expression, or
second setter call. The error instructs the user to run read mode before
deciding whether to retry or restore manually.

Missing API, malformed getter values, setter exceptions, and readback mismatch
return fixed public-safe errors. Details may include contract marker, action,
requested value, finite before/observed values when safely available,
`requires_desktop`, `non_mutating`, `reason`, and a fixed next-action hint. Do
not include raw Runtime errors, stack/source, raw DOM, target ID, chart/layout
ID, account metadata, symbol, or executable paths.

At the end of this milestone, the new command is deterministic, bounded,
selected-chart-only, and fail-closed.

### Milestone 4: Add deterministic coverage and synchronize guidance

Add model tests for read/set/reset action selection, conflict rejection, and
the `0..=500` boundary. Add operation tests using the existing fake runtime for
read success, exact set verification, reset, missing getter/setter, malformed
values, evaluation error sanitization, mismatch failure, outer Runtime timeout,
public-safe timeout guidance, and each invalid `before` classification. Require
zero requested/restoration setter calls for pre-mutation getter throw,
malformed, non-finite, non-integer, and out-of-range `before` values. These Rust
tests verify typed payload and error boundaries; they are not sufficient
evidence for page-side call order.

Add an ignored Rust contract test that executes the production-generated
right-offset expression under pinned Node.js `24.18.0`, plus a repository
wrapper `scripts/check-right-offset-js-contract.py`. Add
`check:right-offset-js` to `mise.toml`, with both Unix and Windows commands.
Add a separately named right-offset JavaScript contract job to
`.github/workflows/ci.yml` and `.github/workflows/release.yml`, each installing
Node.js `24.18.0` and Rust before invoking the wrapper. Add the release job ID
to `.github/workflows/release.yml` under `build.needs` alongside the existing
JavaScript jobs so every release test, build, package, and publish path is
blocked when this contract fails. Document the dedicated command, pinned Node
version, CI/release responsibility, and Rust-only ordinary Cargo baseline in
`docs/development.md`.

The executable fixture must use synthetic model/time-scale objects with
identity assertions and call recording. It must prove `model()` and
`timeScale()` are each resolved exactly once; every getter/setter call uses the
same retained object. Pre-mutation getter throw, malformed, non-finite,
non-integer, and out-of-range `before` branches call the requested setter zero
times and the restoration setter zero times. Success calls the requested
setter once and the first immediate post-set getter once. A requested setter
that throws after applying calls the requested setter once, the post-set getter
zero times, the restoration setter exactly once, and the first immediate
restoration getter exactly once, including when the restoration setter throws.
Post-set getter throw, non-finite, non-integer, and mismatch branches call the
requested setter once, post-set getter once, restoration setter exactly once,
and first immediate restoration getter exactly once, including when the
restoration setter throws. If the restoration getter throws, is malformed,
non-finite, non-integer, or mismatches, its call count is one and all later
setter/getter call counts are zero. The fixture must
assert this exact order and these counts, with no delay, timer, polling,
alternate signature, or fallback. Private strings injected into thrown values
must not appear in the expression result or Rust error details. Outer Runtime
timeout is covered at the Rust evaluator boundary because a page-side fixture
cannot observe an evaluation result that the caller timed out waiting for.
Tests must also prove no `tv range`, screenshot, bars, scanner, DOM click, or
fallback expression appears.

Add CLI contracts showing `tv right-offset --help`, values below zero and above
500, and value-plus-reset conflicts fail before CDP connection. Preserve all
existing range and screenshot contracts.

Update `README.md`, `docs/command-source-taxonomy.md`,
`docs/observation-workflows.md`, `docs/development.md`, the English and Japanese
getting-started guides when appropriate, and `packaging/agent/AGENTS.md`.
Update `.agents/skills/chart-analysis` only if the workflow decision changes;
put command detail in a reference rather than expanding Core Workflow. Do not
add it to market-data or scanning skills because it changes viewport layout,
not evidence source data.

At the end of this milestone, users and agents understand that right offset is
an explicit chart-local viewport mutation and that range/screenshot commands
never apply it automatically.

### Milestone 5: Validate and obtain independent review

Run focused model, chart operation, help, Desktop contract, and the dedicated
pinned right-offset JavaScript contract tests, then the full Rust baseline,
metadata, public hygiene, packaging syntax, guide parity, workflow parsing, and
diff hygiene. Existing Pine and study-value JavaScript gates remain unchanged,
but the new right-offset gate is mandatory whenever its production expression
changes.

Obtain focused independent review of feasibility evidence, exact mutation and
restoration order, integer validation, error sanitization, source/mutation
metadata, hidden-fallback absence, docs, and module ownership. Archive this
plan through one of two explicit paths: implementation review is green after
both feasibility gates authorize and implementation completes, or focused
evidence/outcome review is green after a documented no-go stops implementation.

## Concrete Steps

Run from the repository root. Begin with read-only inspection:

    rg -n "timeScale|zoomToBarsRange|rightOffset|setRightOffset" crates docs
    cargo test -p tradingview-cli ops::chart -- --nocapture

After the feasibility gates authorize implementation, run focused checks:

    cargo test -p tradingview-model right_offset -- --nocapture
    cargo test -p tradingview-cli right_offset -- --nocapture
    cargo test -p tradingview-cli --test cli_contract_desktop right_offset -- --nocapture
    mise run check:right-offset-js

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

An optional live smoke after implementation requires explicit owner approval:

    target/debug/tv right-offset
    target/debug/tv right-offset 1
    target/debug/tv right-offset --reset

Do not use a user's preferred offset as disposable state. Capture and restore
the exact before value, and record only public-safe numeric readback and status.

## Validation and Acceptance

The work is accepted only when current-build evidence proves one exact getter
and setter on the same selected-chart time-scale object; the authorized probe
sets one bounded different value, verifies the first immediate readback, and
restores and verifies the exact prior value; read mode never mutates; invalid
or conflicting input fails before CDP; set/reset success requires exact
readback; errors are public-safe; range and screenshot commands remain
unchanged; no fallback or source mixing is added; and all focused/full checks
plus independent review are green.

If getter, setter, immediate readback, or restoration cannot be established,
the accepted outcome is documented no-go with no stable command. Upstream
method presence and visible-range movement alone are insufficient.

The current evidence satisfies only the no-go path for this plan's
integer-before contract. A future finite-`f64` internal restoration design is
outside this plan and is neither reviewed nor authorized.

## Idempotence and Recovery

Planning and read-only capability inspection are non-mutating and safe to
repeat. The feasibility mutation is not repeated automatically. It captures
one before value, performs at most one probe set, and performs at most one
restoration set. A timeout with unknown outcome stops the procedure; recovery
requires a separately approved read-only observation.

Production reads are idempotent. Repeating a successful set with the same value
should report `changed: false` after exact before/after readback. Reset is
idempotent at zero. Neither path may kill or restart TradingView, switch chart
targets, change symbol/timeframe, alter drawings, or save layout/account state
explicitly.

## Artifacts and Notes

Upstream pull request #225 is the originating evidence. It proposes
`model().timeScale().setRightOffset(n)` and reports live behavior where larger
values create future chart space while moving the left edge inward. Its tests
verify emitted JavaScript but not exact current-build readback or restoration.
This plan intentionally strengthens those boundaries rather than copying its
success contract.

Record probe evidence only as aggregate method availability, integer offsets,
exact-match booleans, and restoration status. Never store raw Runtime payload,
function source, object-key dumps, chart data, symbol, target/session/layout
identifiers, account metadata, or machine-specific paths.

## Interfaces and Dependencies

The intended public surface is:

    tv right-offset
    tv right-offset <BARS>
    tv right-offset --reset

No new dependency is expected. Reuse `tradingview_cdp::RuntimeEvaluator`,
`tradingview_core::AppError`, the selected-target connection flow, and existing
JSON envelope handling. If implementation is authorized, the internal model
should expose an I/O-free action equivalent to `Read`, `Set(i64)`, or `Reset`,
and the chart operation should expose one async adapter returning
`chart_right_offset.v1` data.

The command does not modify `tv range`, `tv scroll`, `tv screenshot`, drawing,
bars, OHLCV, export, Replay, scanner, ranking, recommendation, or trading
judgment behavior.

## Change Note

2026-07-15: Created after launch environment hardening passed independent
review and focused re-review. The plan promotes the first bounded chart-layout
candidate, incorporates upstream PR #225 as evidence, and adds current-build
getter plus reversible exact-readback gates before any stable implementation.

2026-07-15: Corrected the initial plan-review findings by requiring an in-range
distinct probe candidate, one bounded restore attempt for every responsive
post-mutation failure, no automatic recovery after an outer Runtime timeout,
and an executable same-object JavaScript contract gate. Focused re-review is
required before the read-only capability probe.

2026-07-15: Corrected the focused re-review findings by applying the bounded
`before` check to production before any setter, specifying exact call counts
for every executable fixture branch, and fixing the required `mise.toml`, CI,
release `build.needs`, and development-guide integration points. Another
focused re-review is required before the read-only capability probe.

2026-07-15: Corrected the remaining restoration-setter throw branch. A
responsive restoration setter throw now still requires exactly one immediate
read-only restoration getter on the retained time-scale object, followed by
zero additional setter/getter calls. Focused re-review remains required before
the read-only capability probe.

2026-07-15: Focused review reported no remaining findings. The subsequent
bounded read-only probe confirmed the expected time-scale ownership, callable
setter/getter, finite numeric getter result, and visible-range readability, but
the getter value was non-integer. Recorded no-go without invoking any setter;
mutation feasibility and production implementation did not start.

2026-07-15: Corrected the probe's remaining raw-error failure path and narrowed
the outcome to an integer-before contract no-go. Recorded a separate
finite-`f64` internal restoration design as an untested future ExecPlan
candidate, and allowed this plan to archive after green no-go evidence review
without requiring an implementation that the gate intentionally prohibited.

2026-07-15: Focused independent re-review reported no findings in the corrected
probe, limited no-go outcome, future-design boundary, or archive conditions.
Archived the plan through the documented no-go path. No setter, mutation
probe, production command, or automatic float-aware follow-up was authorized.
