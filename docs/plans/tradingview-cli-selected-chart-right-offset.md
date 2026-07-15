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
- [ ] Obtain focused independent review of this plan and apply corrections.
- [ ] Run the bounded read-only current-build capability probe.
- [ ] If read-only evidence is provisional go, obtain separate owner approval
  for a reversible set/read/restore feasibility probe.
- [ ] Implement the command only if both feasibility gates are green.
- [ ] Add deterministic validation, expression, payload, dispatch, help, and
  CLI contract tests.
- [ ] Synchronize public docs, packaged guidance, and runtime skills only where
  the agent decision changes.
- [ ] Run focused and full validation, then obtain independent implementation
  review before archiving this plan.

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

## Outcomes & Retrospective

Planning has started. The previous launch-hardening item is complete and
archived. Current-build right-offset ownership, readback, setter acceptance,
and restoration remain `UNCONFIRMED`; no command, production behavior, or live
chart mutation has been added yet.

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
the finite integer `before` value. Choose a bounded probe value that differs
from `before`: prefer `before + 1` when it is at most 500, otherwise
`before - 1` when non-negative. If neither is valid, stop without mutation.

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
it never returns exception text. If the probe setter throws or the first
readback is wrong, perform at most one identity-preserving restoration attempt
through the retained `ts` reference when it remains usable. Report restoration
status separately and do not reinterpret a failed mutation as production go.
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
once, captures `before`, calls the reviewed setter exactly once, then performs
the first immediate getter readback. Success requires exact equality. Their
payload uses the same contract marker with `action: "set"` or `"reset"`,
`source_category: "desktop_backed_operation"`, `non_mutating: false`,
`requested_right_offset`, `before_right_offset`, `observed_right_offset`,
`changed`, and `verified: true`. Reset requests exactly zero. Do not claim that
zero is TradingView's product default; it is this command's explicit no-empty-
slots reset value.

If set/reset readback mismatches after mutation, the same expression performs
at most one restoration call to `before` and one immediate restoration
readback. It returns a failure with `restoration_attempted` and `restored`
instead of a success payload. Setter or getter exceptions are reduced to fixed
stage labels; raw exception values never cross CDP. An outer Runtime timeout
has unknown mutation outcome and triggers no automatic retry or second
expression. The error instructs the user to run read mode before deciding
whether to retry or restore manually.

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
values, evaluation error sanitization, mismatch failure, and preservation of
the same time-scale ownership path. Tests must prove no `tv range`, screenshot,
bars, scanner, DOM click, or fallback expression appears.

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

Run focused model, chart operation, help, and Desktop contract tests, then the
full Rust baseline, metadata, public hygiene, packaging syntax, guide parity,
and diff hygiene. Run the pinned JavaScript gates only if their production
helpers changed; this plan should not touch Pine or study-value JavaScript.

Obtain focused independent review of feasibility evidence, exact mutation and
restoration order, integer validation, error sanitization, source/mutation
metadata, hidden-fallback absence, docs, and module ownership. Archive this
plan only after implementation review is green.

## Concrete Steps

Run from the repository root. Begin with read-only inspection:

    rg -n "timeScale|zoomToBarsRange|rightOffset|setRightOffset" crates docs
    cargo test -p tradingview-cli ops::chart -- --nocapture

After the feasibility gates authorize implementation, run focused checks:

    cargo test -p tradingview-model right_offset -- --nocapture
    cargo test -p tradingview-cli right_offset -- --nocapture
    cargo test -p tradingview-cli --test cli_contract_desktop right_offset -- --nocapture

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
