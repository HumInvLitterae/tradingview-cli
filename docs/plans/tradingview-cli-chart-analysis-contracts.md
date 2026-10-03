# Chart-analysis output contracts after v0.34.0

Status: direction and missing-value examples approved 2026-10-03.
OHLCV is implemented and locally validated. The owner deferred CI and approved
continuing other work. Pine lines/boxes offline support is implemented and
locally validated; alert-price projection remains a proposal.
The PM is the sole executor. Version selection and release preparation are
separate. The [roadmap](../next-version-roadmap.md) and
[inventory](../next-version-work-items.md) own direction and priority.

## Outcome and consumers

Extend existing offline schema/validation to selected-chart OHLCV and Pine
lines/boxes without
changing its Desktop/CDP source or fetching data during discovery/validation.
This supports an existing command; it is not a new data backend or analysis service.

The [producer](../../crates/cli/src/ops/market/ohlcv.rs) reads recent loaded
chart bars, not just visible bars. Its summary helper is shared by `export
chart-bars --summary` and Replay OHLCV attachments.

In v0.34.0, raw extraction used `volume: v[5] || 0`; aggregation substituted zero
for missing endpoint prices, dropped missing extrema/volume from aggregation and
used "0%" when opening price is zero. These are source-inspection findings,
not observed live missing-data cases. Empty/unavailable chart errors stay errors.

## Approved missing-value contract

Keep field names, real zeros and complete numeric results, including rounding.
Missing/non-finite numeric cells become null in raw bars and last_5_bars.
Malformed bar structure or missing/non-finite timestamp remains an error.

| Case (synthetic fragments) | Before | Proposed after |
| --- | --- | --- |
| Volumes 10 and missing | volume:10, avg_volume:5 after raw zero substitution | volume:null, avg_volume:null; missing raw volume:null |
| Volumes 10 and genuine zero | volume:10, avg_volume:5 | Unchanged |
| First open missing, last close 12 | open:0, close:12, change:12, change_pct:"0%" | open:null, close:12, change:null, change_pct:null |
| Genuine open 0, close 12 | open:0, change:12, change_pct:"0%" | open:0, change:12, change_pct:null |
| Highs 11 and missing | high:11 from the known row only | high:null, range:null; complete low retained |

open/close use the first open/last close respectively. high/low require the
corresponding value in every returned bar; range requires both. volume and
avg_volume require volume in every returned bar. change requires both endpoints;
change_pct also requires nonzero open. Retain unrelated complete fields rather
than discard the entire observation. No synthetic replacement, completeness
score or implicit reread is added.

Failing the whole observation on any missing numeric field would preserve a
numeric-only success shape, but discard valid prices when volume is unavailable.
Nulls are recommended. Consumers must support optional numeric/string fields
and handle unavailable values explicitly; do not default null to
zero. Producer changes also affect export summaries and Replay attachments.
The owner approved these concrete examples before implementation.
No dependency or new persisted format is proposed here.

## Approved offline direction and work

The owner approved `tv schema ohlcv` and
`tv validate -- ohlcv --summary --count 100`. Reuse the
[schema exporter](../../crates/cli/src/app/schema.rs) and
[validator](../../crates/cli/src/app/validate.rs); no custom engine or duplicate
parser is needed. Command-path schema discovery covers raw and summary outputs.
Runtime quality, target readiness, finality and arithmetic remain unchecked.

1. Settle the missing-value contract and consumer compatibility requirements.
2. Correct extraction/aggregation with shared export and Replay fixtures.
3. Add the standard embedded schema and offline validation for both modes.
   Reuse execution count normalization: default 100, clamped 1..500. Do not
   reject count 0 or 501 in validation while execution clamps it. Runtime stays
   not_checked even with an explicit target ID.
4. Update CLI docs/spec and standalone chart-analysis guidance; skill references
   must stay inside each distributed skill directory.

## Acceptance

Cover complete observations, genuine zero versus missing/null, incomplete
endpoints/extrema/volume, zero-open percentage, malformed/non-finite cells,
empty/unavailable bars and raw/summary agreement. Exercise generated extraction
JavaScript with synthetic bars so FakeRuntime cannot conceal coercion. Include
export-summary and Replay-attachment outcomes.

Run affected OHLCV/export/Replay tests serially, the standard schema gate with
production fixtures and invalid field types, cli_contract_offline and validator
unit tests. Verify no target discovery, Desktop/provider access or input-file
read during validation, for both modes, malformed argv and clamped counts.
Finish with scoped Clippy, formatting and public/diff hygiene; validate standalone
skills and package staging if they change. Use one Cargo job/test thread and
normal CI for broad coverage, with no repeated full local suite/release build.
Live Desktop/provider acceptance is separate; no new live action is needed for
these deterministic contracts.

## Progress

- Producer, export/Replay callers, offline facilities inspected on 2026-10-03.
- v0.34.0 publication/workflow/asset metadata verified and record archived.
  Existing implementation evidence was not rerun.
- Owner approved missing-value examples. Extraction, aggregation, schema and
  offline validation are implemented. OHLCV/export/Replay fixtures: 15 passed;
  validator unit tests: 3 passed; offline CLI contracts: 5 passed; capture-spec
  execution-bounds fixture: 1 passed. The standard
  schema gate passed, executing the production JavaScript with Node 24.18.0
  and validating raw/summary/partial/error outputs with jsonschema 4.26.0.
- Standalone skill metadata/references and disposable package parity passed
  (seven runtime skills per root). Scoped CLI Clippy (`--lib --tests`, warnings
  denied), formatting, public hygiene and staged-diff checks passed.
  No dependency or live operation was added. Normal upstream CI remains pending.

## Pine lines and boxes offline support

The owner approved continuing after the two-path proposal. Add `schema data
lines`, `schema data boxes` and matching `validate -- data ...` support with
existing cli_schema.v1 / cli_validate.v1 envelopes. Keep acquisition, filtering,
rounding, deduplication and output unchanged. Before: these paths are unsupported
by the offline helpers. After: schema returns a structural description; valid
argv returns status:valid with runtime:not_checked. Lines/boxes have only the
real parser's string filter and verbose options; empty filter still selects all
readable studies. Labels/tables remain unsupported by these helpers for now.

Cover default/verbose, real zero, null coordinates, opaque primitive
IDs/styles, empty observations and existing malformed-collection-to-empty
behavior with production adapter fixtures. Schemas do not validate study identity,
primitive coverage, coordinate/scale meaning, rounding arithmetic or style types.
Empty output is not proof of absent graphics. Use the existing standard schema
gate, validator/offline CLI tests and focused Clippy. No live Desktop operation,
new dependency or full local suite is needed.

Pine acceptance: validator 3 tests, offline CLI 5 tests, graphics adapter 6 tests,
schema catalog 1 test and the standard production-fixture schema gate passed.
Scoped CLI Clippy, formatting and standalone/package parity checks passed.
Acquisition/output and dependencies are unchanged; CI remains owner-deferred.

## Legacy alert price proposal — pending owner review

The creator emits a simple condition with type cross/cross_up/cross_down and
series exactly `[ {type:barset}, {type:value,value:12.34} ]`. The public sanitizer
removes series and does not copy its threshold into the existing condition.value.
The exported condition therefore omits a known threshold.

Propose a scalar projection only when condition.value is absent and the condition
matches this creator shape: supported condition type, exactly those two entries
in that order, no extra entry fields, and one finite JSON number. Preserve zero;
leave unsupported, study-based, ambiguous or malformed shapes without a projected
value. Existing top-level value behavior is unchanged. Never restore raw series.

Synthetic before: `{type:cross,series_count:2,has_study_series:false}`.
Proposed after: `{type:cross,value:12.34,series_count:2,has_study_series:false}`.
This is an additive public-contract change requiring agreement before code edits.
It does not decode symbol markers or establish currency/adjustment/session
identity. A marker-based symbol may still prevent safe duplicate matching;
keep that identity decision separate rather than guess or change the public symbol.

