# Add stable identity to study-value rows

This ExecPlan is a living document. Keep `Progress`, `Surprises & Discoveries`,
`Decision Log`, and `Outcomes & Retrospective` current while work proceeds.
Maintain it according to `.agents/PLANS.md`.

## Purpose / Big Picture

After this work, callers can distinguish two same-name indicators in
`tv values` and `tv stream values` without guessing from chart order or the
current numeric value. Every returned study row keeps its existing `name` and
`values` fields and additively reports the chart entity ID, a short display
name, a public-safe study kind, compact inputs, and visibility when the current
TradingView Desktop build exposes them.

This is R11 in `docs/v0.27-work-items.md`. It is a selected-chart evidence
identity improvement, not indicator search, mutation, ranking, or a change to
indicator calculations. It must not add, remove, hide, show, or reconfigure a
study. It must not expose Pine source text, oversized free-form input text,
raw metadata, account-local state, or target IDs.

The user-visible shape remains the existing command:

    tv values
    tv stream values --max-events 3

A representative row after implementation is:

    {
      "name": "Moving Average Exponential",
      "values": { "MA": "123.45" },
      "entity_id": "study-identity",
      "short_name": "EMA",
      "study_kind": "indicator",
      "inputs": { "length": 20 },
      "visible": true
    }

The example ID is synthetic. Never record a live entity ID in tracked files.

## Progress

- [x] (2026-07-13) Completed and independently reviewed R10 screenshot
  render-readiness.
- [x] (2026-07-13) Inspected current `tv values`, `tv stream values`, stream
  dedupe, `tv state`, `tv data indicator`, docs, tests, and upstream commit
  `55a5f5f`.
- [x] (2026-07-13) Ran a public-safe read-only current-build inventory on the
  dedicated layout without recording entity IDs or raw values.
- [x] (2026-07-13) Created this R11 ExecPlan and made it the active v0.27 plan.
- [ ] Inventory the exact identity/input method shapes used by the six
  value-bearing current-build sources and record aggregate availability only.
- [ ] Add one shared identity collector and public-safe compact input shaping.
- [ ] Add identity to one-shot and streaming values without changing existing
  value inclusion or formatting.
- [ ] Add deterministic same-name, sanitization, nullability, ordering, and
  stream-dedupe fixtures.
- [ ] Run a bounded public-safe live smoke on the dedicated layout without
  mutating studies.
- [ ] Synchronize stable docs, packaged guidance, and only affected skill
  references.
- [ ] Run focused and complete validation.
- [ ] Obtain independent implementation review and correct findings before
  closeout.

## Surprises & Discoveries

- Observation: one-shot and streaming values currently use different chart
  internals and different value representations.
  Evidence: `crates/cli/src/ops/data/indicator.rs::study_values` walks chart
  model data sources and reads formatted `dataWindowView()` items. The
  `values_expression` function in `crates/cli/src/ops/stream.rs` walks
  `chart.getAllStudies()`, resolves each wrapper with `getStudyById`, and reads
  numeric `_lastBarValues` or `_data`. R11 must preserve those established
  value paths while sharing identity semantics.

- Observation: current one-shot value rows contain only `name` and `values`.
  Evidence: a public-safe dedicated-layout smoke returned six value-bearing
  rows, all with exactly those two keys and no duplicate-name group in that
  particular chart state.

- Observation: current-build entity-scoped reads expose the inputs and
  visibility needed for identity enrichment.
  Evidence: `tv state` found 33 studies on the dedicated layout, and all 33
  synthetic-counted `tv data indicator` reads returned non-null inputs and
  known visibility. No entity ID, input value, or raw payload was recorded.

- Observation: upstream fixed the same user problem with a minimal id/inputs
  addition but did not establish this repository's safety or stream contract.
  Evidence: upstream commit `55a5f5f` adds `s.id()` and `s.inputs()` to
  `getStudyValues`. The commit has live anecdotal evidence for two EMAs, but
  its diff adds no focused test, returns the input object without compact
  public-safety filtering, and does not update the separate stream-values
  path.

## Decision Log

- Decision: preserve `name`, `values`, study inclusion, value formatting, and
  chart order exactly; add identity fields only.
  Rationale: R11 fixes evidence ambiguity. It must not silently change which
  studies or values callers observe.
  Date/Author: 2026-07-13 / Codex

- Decision: use stable field names `entity_id`, `short_name`, `study_kind`,
  `inputs`, and `visible` in both one-shot and stream rows.
  Rationale: snake_case matches current Rust payloads and existing
  `tv data indicator` terminology. Keeping both surfaces identical prevents
  agents from learning two identity contracts.
  Date/Author: 2026-07-13 / Codex

- Decision: permit unavailable identity fields to be `null`, but require
  deterministic fixtures with non-null, distinct `entity_id` and differing
  compact inputs for same-name studies.
  Rationale: failure to read optional metadata must not erase otherwise useful
  existing values. The actual ambiguity fix is accepted only when the known
  positive case is distinguishable without chart-order inference.
  Date/Author: 2026-07-13 / Codex

- Decision: restrict `study_kind` to `indicator`, `strategy`, or `unknown` and
  derive it only from explicit current metadata flags or type markers.
  Rationale: free-form metadata types and study names can contain unstable or
  misleading text. A small vocabulary is easier to interpret and does not
  pretend to classify protected, built-in, or custom scripts beyond observed
  evidence.
  Date/Author: 2026-07-13 / Codex

- Decision: expose `inputs` as a compact object keyed by public input ID, with
  deterministic key order and strict value filtering.
  Rationale: `{ "length": 20 }` distinguishes common same-name studies more
  clearly than a full internal descriptor array. Sorting prevents stream
  dedupe from emitting because an internal object changed insertion order.
  Date/Author: 2026-07-13 / Codex

- Decision: centralize identity collection and compact-input shaping rather
  than duplicating it in `study_values` and `values_expression`.
  Rationale: the two commands may retain different value readers, but entity
  ID, short name, kind, inputs, visibility, nullability, and sanitization must
  not drift.
  Date/Author: 2026-07-13 / Codex

- Decision: identity and input changes are meaningful stream changes.
  Rationale: `StreamDedupe` already removes only `_ts` and `_event` before
  comparison. A changed length or visibility should emit a new values sample;
  unchanged identity with only timestamp metadata should remain deduped.
  Date/Author: 2026-07-13 / Codex

## Outcomes & Retrospective

R11 is planned but not implemented. Current code and read-only live evidence
show that the required identity signals exist, while upstream confirms the
user value of distinguishing same-name studies. Implementation still requires
an exact current-build method-shape inventory, one shared safe collector,
deterministic same-name fixtures, stream parity, full validation, and
independent review.

## Context and Orientation

The `tv values` command is dispatched from
`crates/cli/src/app/dispatch.rs` to
`crates/cli/src/ops/data/indicator.rs::study_values`. Its JavaScript expression
walks selected-chart data sources, reads `metaInfo()` for a display name, and
reads formatted data-window items. It returns an object with `study_count` and
`studies`; each study currently has only `name` and `values`.

The `tv stream values` command is built by
`crates/cli/src/ops/stream.rs::values_expression`. It currently includes only
visible studies, reads numeric last-bar values, and returns `symbol`,
`study_count`, and the same minimal `studies` rows. The application runner adds
`stream.v1` metadata. `StreamDedupe` compares the full sample after removing
only `_ts` and `_event`.

The `tv state` command already returns selected-chart study IDs and names.
`tv data indicator <ENTITY_ID>` returns visibility and input descriptors for
one study and filters especially large string values. These commands prove
that identity exists, but R11 must not implement `tv values` by issuing one CDP
round trip per study. One selected-runtime evaluation per sample remains the
operation shape.

In this plan, “compact inputs” means a public-safe JSON object containing at
most 32 input entries. Keys must be nonempty strings of at most 100 characters.
Allowed values are null, booleans, finite numbers, strings of at most 200
characters, or arrays of at most 16 allowed scalar values. Omit nested objects,
non-finite numbers, source/script text fields, keys named `text`
case-insensitively, keys whose lowercase ID contains `source` or `script`,
oversized values, and entries beyond the limit. Return `null` when no safe
input can be observed. Never include raw input metadata, option catalogs, Pine
source, runtime exception text, or an unbounded object.

An “entity ID” is the selected chart's identifier for one study instance. It
is useful in command output and can be passed to explicit lifecycle commands,
but a live ID is local selected-chart state and must not be copied into tracked
documentation or tests. Tests use synthetic IDs only.

## Required Contract

Do not add a command or option. Extend each existing row additively:

    {
      "name": string,
      "values": object,
      "entity_id": string | null,
      "short_name": string | null,
      "study_kind": "indicator" | "strategy" | "unknown",
      "inputs": object | null,
      "visible": boolean | null
    }

`name` and `values` must remain JSON-equivalent for a given runtime fixture.
Preserve row ordering and `study_count`. Do not exclude
a one-shot row merely because identity is partial. Preserve the stream's
existing visible-only inclusion rule and numeric value representation.

Normalize a candidate entity ID and short name by trimming strings and mapping
empty or non-string values to null. Do not echo a requested identifier because
there is no identifier input to these commands. `study_kind` must use explicit
metadata only and fall back to `unknown`.

The exact current-build collector may use the model data source's `id()` /
`inputs()` / `isVisible()` methods, public wrapper methods returned by
`getStudyById`, or observed properties. Milestone 1 must establish their shapes
for value-bearing sources before production code chooses precedence. Do not
catch a broad exception and replace a known false visibility with null; handle
each optional field independently.

## Plan of Work

First, run a bounded read-only inventory against the dedicated test layout.
For each value-bearing source, inspect only aggregate availability of entity
ID, short-name metadata, explicit strategy/type markers, input method shape,
and visibility. Record counts and method-shape labels in this plan, not IDs,
names, input values, raw metadata, or runtime payloads. If no single source can
associate an ID with the existing one-shot formatted values, stop and revise
the plan rather than joining rows by name or chart order.

Next, extract study-value identity work from
`crates/cli/src/ops/data/indicator.rs` into a focused child module such as
`crates/cli/src/ops/data/study_values.rs` if that keeps the shared collector
readable. Keep the existing public `study_values` export and public Rust helper
signature unchanged. Define one shared JavaScript identity helper or one
shared Rust normalizer that both one-shot and stream paths use. Do not move
unrelated `data_indicator` behavior.

The runtime expression should collect only bounded candidate metadata needed
for the contract. Apply compact-input filtering in one testable location. If
raw candidate input data crosses the CDP boundary before Rust normalization,
normalization errors must not attach that raw value to an `AppError`. Prefer
collecting the compact form inside the selected-runtime expression and then
validating its shape in Rust.

Update `study_values` so its established formatted `values` object and row
inclusion remain unchanged while identity fields are merged from that same
source instance. Update `values_expression` to call the same identity helper
for the already selected wrapper/source and preserve its visible-only numeric
value path. Avoid name-based or index-based joins between independently
enumerated collections.

Add deterministic tests with two rows whose `name` is identical but whose
synthetic `entity_id` and `inputs.length` differ. Tests must also cover missing
metadata, hidden one-shot rows, stream visible-only behavior, input filtering
and limits, deterministic input key ordering, stable row order, unchanged
legacy fields, and malformed optional metadata. Add stream tests proving
identity fields are present in value samples, timestamp-only samples dedupe,
and changed identity/input/visibility causes emission.

Finally, update only the stable docs and runtime-skill references that explain
study values or selected-chart evidence. Keep Core Workflow sections short;
put field interpretation and same-name examples in existing references. Do not
turn identity fields into ranking, recommendation, or permission to mutate a
study automatically.

## Milestones

Milestone 1 proves current-build identity feasibility without changing public
behavior. It ends when aggregate inventory shows how every value-bearing
source can obtain identity from the same source instance, or records a no-go
if only unsafe name/order joining is possible. Re-run `tv values` afterward and
confirm the chart and study state are unchanged.

Milestone 2 implements shared safe identity shaping and one-shot enrichment.
It ends when two same-name fixture rows preserve legacy values and expose
distinct synthetic IDs and compact lengths, while unsafe inputs are omitted.

Milestone 3 adds stream parity and dedupe coverage. It ends when stream value
samples contain the same identity fields, unchanged samples dedupe, and a
changed input or visibility emits one new sample.

Milestone 4 synchronizes docs, runs public-safe live readback, executes the
complete baseline, and stops for independent review. Do not begin R12 or commit
the implementation before that review is green.

## Concrete Steps

Run all commands from the repository root.

Before editing, inspect the current boundaries:

    cargo test -p tradingview-cli ops::data::indicator -- --nocapture
    cargo test -p tradingview-cli stream -- --nocapture
    cargo test -p tradingview-cli --test cli_contract_desktop -- --nocapture

During implementation, run focused checks after each milestone:

    cargo fmt --check
    cargo test -p tradingview-cli study_values -- --nocapture
    cargo test -p tradingview-cli stream -- --nocapture
    cargo test -p tradingview-cli --test cli_contract_desktop -- --nocapture

At the final local gate, run:

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

The optional live smoke is read-only and uses the dedicated layout. Record
only study count, rows with non-null identity, rows with compact inputs, rows
with known visibility, duplicate-name group count, and whether duplicate
groups are distinguishable. Do not record names, entity IDs, inputs, values,
target IDs, raw output, or screenshots.

## Validation and Acceptance

Acceptance requires a deterministic fixture containing two same-name studies
with distinct synthetic entity IDs and compact inputs. Both `tv values` and
the stream-values sample must preserve the original `name` and `values` while
making the two rows distinguishable through identity fields alone.

Tests must prove all seven row fields and their nullability/vocabulary,
32-entry and scalar/array limits, source/text and oversized-value omission,
finite numeric filtering, deterministic input key order, stable row order, and
unchanged counts. Malformed optional identity must degrade to null/unknown and
must not remove an existing values row.

Stream tests must prove that `_ts` and `_event` remain excluded from dedupe,
that unchanged identity does not produce another sample, and that changed
entity ID, compact input, or visibility does. Existing quote, bars, lines,
labels, tables, and all-panes stream contracts must remain unchanged.

The live smoke is supporting evidence, not a substitute for the same-name
fixture. If the dedicated layout has no duplicate-name studies, record zero
duplicate groups without adding or modifying indicators. If identity is
missing for current-build value rows, do not guess or weaken the test; record
the blocker and stop for plan revision.

## Idempotence and Recovery

Inventory, tests, and live smoke are read-only and safe to repeat. They must
not add, remove, toggle, or configure studies. No cleanup action should be
needed. If a probe unexpectedly changes chart state, stop, report the observed
public-safe difference, and restore only from the previously captured
dedicated-layout state.

The unrelated stash named
`recovered-indicator-search-prototype-2026-07-12` must remain untouched. Never
apply, drop, overwrite, or include it without explicit owner confirmation. If
R11 trial changes must be withdrawn, create a separately named stash first and
ask before deleting it.

## Artifacts and Notes

The intended data flow is:

    selected value-bearing study instance
      -> existing value reader remains unchanged
      -> shared bounded identity collector
      -> compact public-safe identity fields
      -> existing one-shot or stream envelope
      -> stream dedupe compares identity as evidence

Reject these shortcuts:

    join rows by display name
    join independently enumerated arrays by index
    return raw s.inputs() or full input descriptors
    infer strategy kind from the displayed name
    hide a row because optional identity is unavailable
    mutate visibility to make identity easier to read

## Interfaces and Dependencies

Keep these public interfaces unchanged:

    pub async fn study_values(
        runtime: &mut impl RuntimeEvaluator,
    ) -> Result<Value, AppError>

    pub async fn stream_sample(
        runtime: &mut impl RuntimeEvaluator,
        request: &StreamRequest,
    ) -> Result<Value, AppError>

If a new child module is added, keep its identity collector, compact-input
normalizer, and expression builders private or crate-private. Re-export only
the existing `study_values` helper through the current facade. Use existing
Serde JSON, `RuntimeEvaluator`, and `AppError`; add no dependency, source,
background task, retry, command option, or package-version change.

## Open Questions

No product-contract question blocks Milestone 1. The exact current-build
method precedence for entity ID, short name, kind, inputs, and visibility is
intentionally confirmed by aggregate probe before production implementation.
If identity cannot be read from the same instance that supplies values, R11 is
a no-go until a safe association exists; name/order matching is not an option.

Revision note (2026-07-13): Created R11 after R10 focused re-review completed
green. The plan incorporates current Rust one-shot/stream differences, direct
upstream quality comparison, public-safe compact inputs, same-instance
association, deterministic same-name acceptance, and stream dedupe semantics.
