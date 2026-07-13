# Add bounded screenshot render-readiness

This ExecPlan is a living document. Keep `Progress`, `Surprises & Discoveries`,
`Decision Log`, and `Outcomes & Retrospective` current while work proceeds.
Maintain it according to `.agents/PLANS.md`.

## Purpose / Big Picture

After this work, a caller taking a screenshot immediately after a symbol,
timeframe, visible-range, or panel change can explicitly ask `tv` to wait until
the selected TradingView Desktop chart presents a stable render signature.
The existing immediate screenshot behavior remains the default. When waiting
is requested, the command either captures after bounded readiness is observed
or returns a timeout diagnostic without writing a file; it never labels an
unstable timeout capture as ready.

The user-visible form is:

    tv screenshot --region chart --output target/chart.png --wait-for-render
    tv screenshot --region strategy --output target/strategy.png \
      --wait-for-render --wait-timeout-ms 10000

Successful opt-in output contains a nested `render_wait` object with the final
public-safe chart context, region bounds, sample count, stable sample count,
elapsed time, and `status: "ready"`. A timeout uses the existing structured
error envelope with `ErrorKind::Timeout`, includes the same final safe
observation, and leaves the requested output path unwritten.

This is R10 in `docs/v0.27-work-items.md`. It is a Desktop-backed visual
evidence improvement. It does not switch symbols or timeframes, open panels,
change chart state, infer strategy metrics, read another source, or promise
that visual pixels semantically match an earlier command beyond the reported
selected-chart context.

## Progress

- [x] (2026-07-13) Completed and independently reviewed R9 visible-range
  history paging.
- [x] (2026-07-13) Inspected the current Rust screenshot CLI, dispatch,
  adapter, tests, and output contracts.
- [x] (2026-07-13) Compared upstream screenshot render waiting at commits
  `e7932e7` and `01c09fb` against this repository's quality requirements.
- [x] (2026-07-13) Created this R10 implementation ExecPlan and made it the
  active v0.27 plan without changing screenshot behavior.
- [ ] Inventory current-build selected-chart, canvas, loading, and Strategy
  Tester panel signals on the dedicated test layout using public-safe counts
  and dimensions only.
- [ ] Add pre-connection option validation and deterministic readiness policy.
- [ ] Add bounded sequential observation and opt-in screenshot integration.
- [ ] Add paused-time adapter tests, CLI contract tests, and file-write tests.
- [ ] Run a bounded public-safe live smoke on the dedicated test layout and
  restore any test-only chart state that was changed.
- [ ] Synchronize stable docs, packaged guidance, and only affected runtime
  skill references.
- [ ] Run focused and complete validation.
- [ ] Obtain independent implementation review and correct findings before
  closeout.

## Surprises & Discoveries

- Observation: current Rust screenshots capture immediately after Desktop
  connection and perform no render-readiness observation.
  Evidence: `crates/cli/src/ops/screenshot.rs` calls `capture_screenshot` for
  `full`, or reads bounds and calls `capture_screenshot_clip` for `chart` and
  `strategy`, without a wait phase.

- Observation: upstream's opt-in feature preserves immediate capture by
  default, which is the correct compatibility direction, but its readiness and
  timeout evidence are too weak for this CLI.
  Evidence: upstream `src/wait.js::waitForChartRender` uses global
  `class*="loader"` and `class*="loading"` matches, falls back to any canvas,
  and declares readiness after repeated symbol/resolution/dimension strings.
  `src/core/capture.js` ignores the returned false timeout and still reports
  `waited_for_render: true` after capturing.

- Observation: the Rust CLI already has paused Tokio time available in test
  builds after R9.
  Evidence: `crates/cli/Cargo.toml` enables Tokio `test-util` only as a
  dev-dependency feature, and R9 uses it for deterministic deadline tests.

## Decision Log

- Decision: add `--wait-for-render` as an opt-in flag and keep no-wait capture
  byte and payload behavior unchanged.
  Rationale: existing automation may intentionally capture current visible
  state immediately. A readiness heuristic should not silently add latency or
  new failure modes to that workflow.
  Date/Author: 2026-07-13 / Codex

- Decision: add `--wait-timeout-ms <MS>` with a default of 5000, a valid range
  of 500 through 30000, and require `--wait-for-render` when it is explicitly
  provided.
  Rationale: the wait must be caller-visible and finite. Five seconds matches
  the upstream operational default, while the bounded override supports slower
  charts without allowing an unbounded Desktop operation.
  Date/Author: 2026-07-13 / Codex

- Decision: require three consecutive ready observations at a 200 ms interval.
  Rationale: one observation can coincide with a transient chart update. Three
  matching signatures provide a short stability window while keeping the
  default opt-in wait responsive. The first ready observation starts the count;
  readiness therefore requires two subsequent matching observations.
  Date/Author: 2026-07-13 / Codex

- Decision: timeout is a structured `ErrorKind::Timeout`; do not capture or
  write the output file after timeout.
  Rationale: the roadmap explicitly forbids presenting an unstable frame as
  ready. Capturing anyway would make the opt-in flag misleading. Callers that
  want best-effort immediate evidence can omit the flag and retain existing
  behavior.
  Date/Author: 2026-07-13 / Codex

- Decision: keep observation and capture sequential on one
  `&mut RuntimeEvaluator` and do not introduce a background task.
  Rationale: the selected target and its chart state have one ownership order.
  Sequential evaluation makes the observed state immediately precede capture
  and keeps errors attributable.
  Date/Author: 2026-07-13 / Codex

## Outcomes & Retrospective

R10 is planned but not implemented. The outcome will be recorded after the
current-build signal inventory, focused tests, live evidence, complete
validation, and independent review. If current Desktop cannot expose a
bounded, public-safe stability signature that distinguishes loading from ready
state, record no-go rather than copying broad upstream selectors or weakening
timeout semantics.

## Context and Orientation

`crates/cli/src/cli.rs` defines `Command::Screenshot` with `--region` and
`--output`. `crates/cli/src/app/dispatch.rs` validates the region and path,
connects to the selected Desktop runtime, and dispatches to one of three
functions exported by `crates/cli/src/ops/screenshot.rs`.

`screenshot_full` captures the selected target viewport. `screenshot_chart`
and `screenshot_strategy` first evaluate a DOM-bounds expression, request a CDP
clipped screenshot, and fall back to a full-page capture plus local PNG crop if
clipped capture fails. All three return source metadata and write the requested
file only after successful capture. `strategy` is visual evidence for the
currently visible Strategy Tester panel; it does not open the panel or replace
structured `tv data strategy`, `tv data trades`, or `tv data equity` reads.

In this plan, a “render observation” is one public-safe read of the selected
chart context and requested region geometry. A “render signature” is the
normalized tuple used to compare consecutive observations: selected symbol,
resolution, last loaded main-series bar timestamp when available, canvas width
and height, and requested-region width and height. Raw DOM, class names, target
IDs, chart entity IDs, bar values, account-local metadata, and screenshot bytes
are not part of the signature or JSON output.

A “known loading indicator” means a visible loading element found inside the
selected chart or requested panel using a current-build signal confirmed by
the initial inventory. Absence of such a signal is reported as
`known_loading_visible: false`; it is not a universal claim that every
TradingView subsystem has finished. Stable chart context and nonzero bounds
remain independently required.

## Required Contract

Extend the existing command without renaming current options:

    tv screenshot --region <full|chart|strategy> --output <PATH> \
      [--wait-for-render] [--wait-timeout-ms <MS>]

`--wait-for-render` defaults to false. `--wait-timeout-ms` defaults internally
to 5000 when waiting is enabled. An explicitly supplied timeout without the
flag is a validation error. Values below 500 or above 30000 are validation
errors. All option errors must occur before `connect_runtime`; preserve the
existing validation exit code 1 and JSON error envelope.

Without `--wait-for-render`, call the existing screenshot paths directly and
do not add a `render_wait` field. Existing capture mode, clip, output path,
source metadata, error mapping, and file behavior remain unchanged.

With `--wait-for-render`, begin one absolute deadline immediately before the
first render observation. The deadline covers every observation evaluation and
every 200 ms interval. Each evaluation must itself use that same absolute
deadline through `tokio::time::timeout_at`; repeated traffic must not extend
the operation.

A ready observation requires all of the following:

1. selected chart symbol and resolution are present and nonempty;
2. the chart canvas has finite positive width and height;
3. the requested region has finite positive width and height (`full` uses the
   finite positive viewport, `chart` uses chart bounds, and `strategy` uses the
   visible Strategy Tester panel bounds);
4. no known scoped loading indicator is visible; and
5. the complete normalized signature matches the preceding ready signature
   for three consecutive ready observations.

Any unavailable field, visible known loader, changed signature, malformed
observation, or region disappearance resets the consecutive stable count to
zero. It does not switch targets, open panels, retry through another source, or
capture early. A runtime evaluation error remains its original `AppError` kind
with sanitized render-wait context. A guard expiration becomes
`ErrorKind::Timeout`, even if the last observation was otherwise parseable.

On success, add this nested object to the existing screenshot payload:

    {
      "render_wait": {
        "contract_version": "screenshot_render_wait.v1",
        "requested": true,
        "status": "ready",
        "timeout_ms": 5000,
        "poll_interval_ms": 200,
        "required_stable_samples": 3,
        "sample_count": 4,
        "stable_sample_count": 3,
        "elapsed_ms": 620,
        "final_observation": {
          "symbol": "NASDAQ:AAPL",
          "resolution": "1D",
          "last_bar_time": 1783814400,
          "known_loading_visible": false,
          "canvas_width": 1200,
          "canvas_height": 640,
          "region_width": 1200,
          "region_height": 640
        }
      }
    }

`last_bar_time` is nullable because an otherwise visible chart may not expose
a loaded bar yet; a null last bar cannot form a ready signature. The example
values are illustrative, not live evidence to copy into tests or docs.

On timeout, do not call any capture method and do not create or overwrite the
output path. Return exit code 4 with details containing `phase:
"wait_for_render"`, region, timeout, elapsed time, sample count, stable sample
count, and the last public-safe observation or null. Include
`output_written: false`, existing screenshot source metadata, and a short next
action hint that distinguishes retrying with a longer bounded timeout from
omitting the opt-in flag for an intentional immediate capture.

## Plan of Work

First, perform a bounded read-only inventory on the owner-approved dedicated
test layout. Inspect selected-chart symbol and resolution from the current
chart API, main-series last-bar timestamp, chart canvas bounds, viewport
bounds, Strategy Tester bounds when visible, and current loading indicators.
Record only signal availability, booleans, and dimensions in this plan. Do not
record raw DOM, selectors containing generated class names, target IDs, chart
contents, or account-local values. If no scoped loading signal is credible,
retain `known_loading_visible` as a narrowly named best-effort field rather
than broadening global class-fragment selectors.

Then extend `Command::Screenshot` in `crates/cli/src/cli.rs` and validate the
new controls in `crates/cli/src/app/dispatch.rs` before Desktop connection.
Keep option interpretation in a small request/control type so direct callers
receive the same bounds defensively.

Split render waiting from image encoding and file I/O. Add
`crates/cli/src/ops/screenshot/render_wait.rs` behind the existing screenshot
facade. It should own the observation expression, observation normalization,
signature comparison, absolute-deadline loop, success payload, and sanitized
failure details. Keep `crates/cli/src/ops/screenshot.rs` responsible for region
dispatch, bounds parsing, CDP capture, PNG crop, file writing, and existing
metadata. Do not move screenshot bytes through the readiness module.

Pass an optional successful render-wait payload into the existing full and
clipped capture functions. Capture only after the wait returns ready. For
`chart` and `strategy`, read final capture bounds normally after readiness;
do not assume the observation's prior rectangle is still exact. If those final
bounds fail, preserve the existing bounds error instead of claiming the wait
guaranteed capture readiness.

Finally synchronize screenshot help, README or getting-started text only where
the workflow is already described, source taxonomy, development docs,
observation workflows, packaged agent guidance, and chart-analysis references.
Explain that readiness is opt-in bounded evidence, not a pixel-semantic
guarantee and not an automatic follow-up after chart mutations.

## Milestones

Milestone 1 establishes that the current Desktop build exposes usable
read-only signals. Run a bounded inventory on the dedicated layout in stable,
immediately-after-symbol-change, immediately-after-timeframe-change, and
visible Strategy Tester states. At the end, this plan records a go/no-go
decision and only public-safe aggregate evidence. No Rust behavior changes in
this milestone.

Milestone 2 adds control validation and the deterministic readiness state
machine. Paused-time tests prove stable readiness, every reset condition,
absolute timeout, evaluation failure separation, and elapsed-time accounting.
At the end, no screenshot is captured yet unless the existing no-wait path is
used.

Milestone 3 integrates successful waits with full, chart, and strategy capture
while preserving no-wait behavior. Tests prove timeout and readiness failures
write no file, ready waits capture exactly once, clipped-capture fallback still
works, and bounds failures after readiness retain existing error semantics.

Milestone 4 synchronizes docs and runs a public-safe live smoke plus complete
validation. At the end, an independent reviewer can compare implementation,
contract, evidence, and current project state without reconstructing prior
discussion.

## Concrete Steps

Run all commands from the repository root. Establish the current focused
baseline before edits:

    cargo test -p tradingview-cli screenshot -- --nocapture
    cargo test -p tradingview-cli --test cli_contract_desktop screenshot -- --nocapture

During implementation, run the new render-wait tests directly and expect
paused-time tests to complete without real multi-second sleeps:

    cargo test -p tradingview-cli screenshot::render_wait -- --nocapture
    cargo test -p tradingview-cli screenshot -- --nocapture
    cargo test -p tradingview-cli --test cli_contract_desktop screenshot -- --nocapture

After implementation and documentation synchronization, run:

    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1
    git diff --check
    python3 scripts/check-public-hygiene.py
    bash -n scripts/stage-release-package-files.sh
    cmp -s AGENTS.md CLAUDE.md

Validate each changed runtime skill with the repository's configured skill
validator. Do not place a machine-specific validator path in tracked docs.

The optional live smoke uses only the dedicated test layout. Capture into
ignored `target/` paths, verify file existence and nonzero size after ready
success, and remove smoke files afterward. Record only command form, region,
status, elapsed time, sample counts, public symbol/resolution, dimensions, and
whether the file was written. Do not commit images or raw command output.

## Validation and Acceptance

Deterministic option tests must prove the timeout bounds and dependency on
`--wait-for-render` before connection. Existing no-wait help and capture tests
must remain green and must prove no readiness evaluations were added.

Render-wait fixtures must cover immediate malformed/unavailable state,
visible known loading state, zero canvas or region bounds, symbol change,
resolution change, last-bar change, bounds change, three consecutive stable
samples, exact-deadline collision, evaluation error before timeout, and timeout
after partial stability. The tests must verify that only complete matching
signatures increment stability and all non-ready observations reset it.

Adapter tests must prove ready success captures once, timeout captures zero
times, timeout creates no file, a preexisting output file is not overwritten on
timeout, and runtime evaluation errors preserve their kind without raw
details. Full, chart, and strategy regions must each attach
`screenshot_render_wait.v1` only when requested. Existing chart clip fallback,
Strategy Tester evidence role, parent-directory creation, and output metadata
must remain unchanged.

Acceptance requires one dedicated-layout smoke where a wait immediately after
a test-only chart mutation observes at least one unstable or loading sample
before ready success, and one stable-state smoke that reaches ready without
mutation. If the former cannot be reproduced, do not invent evidence; record
the limitation and rely on deterministic fixtures plus the stable-state smoke.

## Idempotence and Recovery

Option validation, deterministic tests, and stable-state read-only inventory
are safe to repeat. Live symbol or timeframe changes affect only the
owner-approved dedicated test layout and must be restored from a captured
public-safe before-state. Screenshot smoke files live under `target/` and may
be removed after existence and size checks.

The unrelated stash named
`recovered-indicator-search-prototype-2026-07-12` must remain untouched. Never
apply, drop, overwrite, or include it without explicit owner confirmation. If
R10 trial changes must be withdrawn, create a separately named stash first and
ask before deleting it.

## Artifacts and Notes

The intended flow is:

    validate controls before connect
      -> connect selected Desktop target
      -> if requested, observe one absolute bounded readiness loop
      -> on timeout/error, write no file
      -> on ready, perform the existing region capture path
      -> attach render-wait evidence only to opt-in success

The upstream implementation is evidence for product value and opt-in
compatibility only. Do not copy its global generated-class matching, arbitrary
canvas fallback, ignored timeout result, or unconditional
`waited_for_render: true` claim.

## Interfaces and Dependencies

In `crates/cli/src/cli.rs`, extend `Command::Screenshot` with fields equivalent
to:

    wait_for_render: bool
    wait_timeout_ms: Option<u64>

In `crates/cli/src/ops/screenshot/render_wait.rs`, define private or
crate-private types equivalent to:

    struct RenderWaitControls {
        timeout: Duration,
        poll_interval: Duration,
        required_stable_samples: usize,
    }

    struct RenderObservation {
        symbol: String,
        resolution: String,
        last_bar_time: Option<f64>,
        known_loading_visible: bool,
        canvas_width: f64,
        canvas_height: f64,
        region_width: f64,
        region_height: f64,
    }

    async fn wait_for_render(
        runtime: &mut impl RuntimeEvaluator,
        region: ScreenshotRegion,
        controls: RenderWaitControls,
    ) -> Result<Value, AppError>

The exact visibility of these types may remain private. Use existing
`tradingview_cdp::RuntimeEvaluator`, `tradingview_core::AppError`, Tokio time,
Serde JSON, and current screenshot capture helpers. Add no dependency, source,
background task, retry/backoff policy, or version bump.

## Open Questions

No unresolved contract question blocks implementation. The initial current-
build inventory must still confirm the narrow scoped loading signal and exact
main-series bar-time read path. If either is unavailable, update this living
plan with the observed limitation and a reviewed no-go or reduced readiness
contract before writing production behavior.

Revision note (2026-07-13): Created R10 after R9 implementation and focused
re-review completed green. The plan incorporates direct upstream quality
comparison, preserves immediate screenshots by default, and makes timeout a
no-capture structured failure rather than a misleading ready result.
