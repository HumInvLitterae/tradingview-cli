# Chart-analysis output contracts after v0.34.0

Status: direction and missing-value examples approved 2026-10-03.
OHLCV, Pine lines/boxes/labels/tables offline support and legacy alert-price
projection are implemented and locally validated. CI is owner-deferred.
The PM is the sole executor. The owner approved v0.35.0 preparation on
2026-10-04; publication remains separate. The [roadmap](../next-version-roadmap.md) and
[inventory](../next-version-work-items.md) own direction and priority.

## Outcome and consumers

Extend existing offline schema/validation to selected-chart OHLCV and Pine
lines/boxes/labels/tables without changing its Desktop/CDP source or fetching
data during discovery/validation.
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
readable studies. At that stage, labels/tables remained unsupported by these helpers; their
subsequent support is recorded below.

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

## Approved legacy alert price projection

The creator emits a simple condition with type cross/cross_up/cross_down and
series exactly `[ {type:barset}, {type:value,value:12.34} ]`. Before this change,
the public sanitizer removed series without copying its threshold into the
existing condition.value, leaving the exported condition without its known
threshold.

The owner approved a scalar projection only when condition.value is absent and
the condition matches this creator shape: supported condition type, exactly
those two entries in that order, no extra entry fields, and one finite JSON number.
Preserve zero; leave unsupported, study-based, ambiguous or malformed shapes without a projected
value. Existing top-level value behavior is unchanged. Never restore raw series.

Synthetic before: `{type:cross,series_count:2,has_study_series:false}`.
Approved after: `{type:cross,value:12.34,series_count:2,has_study_series:false}`.
It does not decode symbol markers or establish currency/adjustment/session
identity. A marker-based symbol may still prevent safe duplicate matching;
keep that identity decision separate rather than guess or change the public symbol.

Projection is implemented in the existing public-row sanitizer. Focused model
checks cover all three public row locations, supported condition types, real
zero, malformed/ambiguous shapes, private study removal, existing values
including null, and unchanged symbol markers. Model alert tests and the CLI
alert-spec test passed, as did scoped model Clippy, formatting, public/diff
hygiene and standalone/package skill parity. Package staging used the existing
installed binary solely for guidance validation, not candidate-binary acceptance.
No provider/account operation or dependency was added; CI remains owner-deferred.

## Pine labels and tables offline support

The owner approved proceeding on 2026-10-04. Before: labels/tables are unsupported
by schema/validate. After: `schema data labels` / `schema data tables` return
cli_schema.v1 documented-fields descriptions, and valid candidate argv returns
cli_validate.v1 status:valid with runtime:not_checked. Existing data output and
Desktop extraction remain unchanged. Reuse the embedded
standard schemas, clap parser and zero-I/O validator; no custom engine or
dependency is needed.

Labels preserve text, nullable price, extracted/readable/returned counts and
truncation state, plus opaque verbose identity/coordinates/styles. Default max
is 500 per study, zero is allowed and larger values retain the parser's usize
bound without an invented cap. Tables returns only string rows: grouping,
coordinate overwrite, dropped empty cells and unescaped delimiters remain
existing lossy behavior. Schema conformance does not establish count arithmetic,
chronology, primitive coverage, study identity or a rectangular typed table.

Cover production normalization with
fixtures for default/verbose labels, genuine zero/null, max zero/truncation,
empty/malformed collections, missing metadata and lossy table row formatting.
Reject wrong field types through the standard schema gate and exercise parser
acceptance/rejection, target arguments and no Desktop connections through the
existing offline CLI contract tests. Update the standalone chart-analysis skill
and check package parity. Local checks remain serial; CI is owner-deferred.

Labels/tables acceptance: validator unit tests (3), offline CLI contracts (5),
graphics adapter tests (6), schema catalog (1) and graphics spec (1) passed.
The standard schema gate passed, including production-normalizer fixtures and
wrong-type/missing-field rejection. Scoped CLI Clippy, formatting, skill metadata,
standalone references, package parity and public/diff hygiene passed. Staging used
the existing installed binary for guidance validation only. Existing acquisition,
consumer output and dependencies are unchanged; no live operation was performed.
CI remains owner-deferred.

## Pre-release dependency refresh

On 2026-10-04 the owner required all dependencies to be updated before formal
release. Review current stable versions for every existing direct/dev dependency,
refresh Cargo.lock transitively and check changed behavior before adopting
compatibility overrides. Repeat the freshness check immediately before release;
any later dependency changes need affected verification and candidate CI. Keep
this work before the release-version commit. Version selection remains separate.

All 24 direct/dev requirements were checked against crates.io metadata. Tokio
1.53.1 -> 1.53.2 is the only direct update; its upstream changes include Windows
child-process cleanup and runtime/sync/timer fixes. These upstream fixes are
adopted without compatibility overrides. The lock refresh also updates
async-recursion, cc, lazy_static, libc, quinn-proto, quinn-udp, uuid and yoke-derive.
cc changes build-environment caching/diagnostics; libc's time64 change is opt-in
and no new cfg is enabled. async-recursion now uses the existing syn 3 dependency.
No public command, output format, credential policy or dependency feature was
changed. The same macOS/Windows/Linux distribution scope remains intended.

The refreshed resolver leaves generic-array 0.14.7 because crypto-common 0.1.7
requires exactly that version in the oauth2/sha2 chain used by rmcp. Newer 0.14.x
versions exist, but the current stable rmcp release resolves this pinned OAuth
chain. No override was introduced for this upstream constraint. The update
therefore uses the latest resolvable stable graph, not each transitive package
independently at its latest release.

Check affected offline CLI/schema fixtures and MCP deadline/protocol behavior
with one Cargo job/test thread. Do not repeat the full local suite; normal CI
owns broad/platform coverage and remains owner-deferred.

Proposed release scope: truthful selected-chart OHLCV nulls across raw/summary,
export summaries and Replay attachments; offline OHLCV and four Pine graphics
schemas/invocation checks; exact legacy simple-price threshold projection; and
the dependency refresh. No additional data backend, default routing change or
new live-qualification claim is included.

Dependency acceptance on 2026-10-04: all 24 direct/dev stable versions checked;
locked metadata and the resolver dry-run verified. Offline CLI contracts (5),
loopback MCP protocol cases (2), whole-operation deadline fixture (1) and the
standard output-schema gate passed on the refreshed graph. Formatting and
public/diff hygiene passed. No new native/platform or live-provider acceptance
is claimed; normal CI remains owner-deferred. Clippy was not rerun locally for
this manifest/lock refresh.

Release preparation refresh on 2026-10-04 found Mio 1.2.4 after the earlier
refresh. All 24 direct/dev requirements remain current. Adopt Mio's Windows
named-pipe lifetime fix and FreeBSD/libc compatibility fix; loopback protocol
cases (2) and the whole-operation deadline fixture (1) passed. A full Cargo
update retained the existing Windows dependency edges after selective update
had temporarily resolved broad windows-sys ranges to 0.52. No override was
added; the final lock diff changes only Mio's version/checksum. Platform CI
remains pending.

## v0.35.0 preparation

The owner approved the proposed scope/version on 2026-10-04. Curated notes are
in [v0.35.0.md](../releases/v0.35.0.md). README, CLI specification and both
packaged getting-started guides describe the released command paths and null
migration. Documentation precedes the version commit; no feature is added.

Package review found README links to repository-only documents and the source
skills tree, neither included at those paths in archives. Those links now point
to the online repository while bundled getting-started links stay local. The
existing transitive package checker now starts from README/CHANGELOG as well as
agent guides and skills. Its missing-reference fixture and all 13 self-tests
passed; disposable staging passed with seven standalone skills per root.
Staging used an existing binary only for documentation/resource validation.
Full optimized/native archives and broad CI remain pending; do not call the
release published or fully qualified from these checks.
