# Measure and bound multi-symbol concurrency

This ExecPlan is a living document. Keep `Progress`, `Surprises & Discoveries`,
`Decision Log`, and `Outcomes & Retrospective` current while working. Maintain
this plan according to `.agents/PLANS.md`.

## Purpose / Big Picture

`tv quotes`, Desktop-free `tv compare`, and `tv events compare` currently read
symbols one after another. This is deterministic but makes bounded candidate
sets wait for every earlier symbol. After this work, local measurements decide
whether each workflow should run at most four symbols concurrently while
preserving input order, per-symbol sequencing, partial results, deterministic
first errors, and all source metadata.

The change is retained only when a deterministic loopback fixture shows at
least 25 percent median improvement for both 10 and 25 symbols. `tv chart
compare` remains sequential because it mutates and restores selected-chart
state.

## Progress

- [x] (2026-07-11) Gate 5 was independently reviewed, committed as `b2c6f05`, and archived.
- [x] (2026-07-11) Re-read the three sequential loops, validation limits, payload item types, dependencies, roadmap, and ordered inventory.
- [x] (2026-07-11) Created this Gate 6 ExecPlan and made it current.
- [x] (2026-07-11) Added the private ordered bounded runner and deterministic measurement fixture.
- [x] (2026-07-11) Recorded sequential and bounded medians for 1, 2, 5, 10, and 25 symbols.
- [x] (2026-07-11) Retained concurrency for all three workflows because each exceeded the 25 percent threshold at 10 and 25 symbols.
- [x] (2026-07-11) Added the 25-symbol validation boundary and additive quote item index.
- [x] (2026-07-11) Synchronized help, stable docs, packaged agent guidance, and narrow runtime-skill references.
- [x] (2026-07-11) Ran focused tests, runtime-skill validation, strict clippy, and the full workspace baseline; all non-ignored tests pass.
- [x] (2026-07-11) Recorded outcomes as `implemented; independent review pending`; did not commit or start release audit first.
- [x] (2026-07-11) Independent review approved the implementation after one stale `CONTINUITY.md` statement was corrected and focused re-review reported no remaining findings.

## Surprises & Discoveries

- Observation: `compare` performs quote, symbol-info search, and fundamentals
  sequentially inside each symbol.
  Evidence: `compare_one_symbol` awaits those three sections in order. Gate 6
  preserves that sequence and only overlaps different symbols.

- Observation: `events compare` and `watch compare` already limit inputs to 25,
  but `quotes` and Desktop-free `compare` do not.
  Evidence: the CLI and Market validators define 25-symbol constants for events
  and watch only.

- Observation: `tradingview-market` already depends on `futures-util`.
  Evidence: no new dependency is needed for `buffer_unordered`.

- Observation: all three deterministic workloads exceeded the adoption
  threshold by a wide margin at 10 and 25 symbols.
  Evidence: quotes improved 71.3 and 72.1 percent, events compare improved 69.5
  and 68.6 percent, and compare improved 68.1 and 70.7 percent respectively.

- Observation: injecting three alternate endpoint sets through all stable
  single-symbol APIs would add test-only plumbing unrelated to scheduling.
  Evidence: the retained measurement instead runs the production bounded
  runner and reqwest client against loopback HTTP, using one request per symbol
  for quotes/events and three sequential requests per symbol for compare. The
  quote workflow additionally has an end-to-end reversed-completion fixture.

## Decision Log

- Decision: use a fixed private concurrency limit of four and restore results
  by zero-based requested index before payload shaping.
  Rationale: the work inventory already selected four as a conservative source
  load, while sorting prevents completion order from changing public output.
  Date/Author: 2026-07-11 / Codex.

- Decision: retain concurrency per workflow only if 10- and 25-symbol medians
  are each at least 25 percent faster than the sequential fixture baseline.
  Rationale: concurrency is justified by measured user benefit rather than
  implementation preference.
  Date/Author: 2026-07-11 / Codex.

- Decision: keep strict elapsed-time assertions out of the ordinary suite.
  Rationale: CI scheduling can vary. Deterministic tests prove the four-request
  ceiling, overlap, completion-order reversal, and restored input order; the
  five-run medians are captured in this plan as implementation evidence.
  Date/Author: 2026-07-11 / Codex.

- Decision: measure the scheduling boundary with loopback HTTP workloads that
  match each workflow's request count rather than threading test endpoint
  configuration through every single-symbol API.
  Rationale: this exercises the actual runner, reqwest overlap, and sequential
  inner requests while keeping endpoint normalization contracts covered by
  their existing focused tests. It avoids production complexity created only
  for benchmarking.
  Date/Author: 2026-07-11 / Codex.

## Outcomes & Retrospective

Implementation and deterministic measurement are complete. All three
workflows met the adoption threshold, so they now use at most four concurrent
symbol operations and restore input order before payload shaping. Quotes and
Desktop-free compare now reject more than 25 symbols, and batch quote items
include additive zero-based `requested_index`. Contract versions and all
existing fields remain unchanged.

The five-run median measurements, in `sequential_ms / bounded_ms` form, were:

- quotes: 1 `23/23`, 2 `47/23`, 5 `118/49`, 10 `241/69`, 25 `577/161`;
- events compare: 1 `23/22`, 2 `45/22`, 5 `112/45`, 10 `225/68`, 25 `583/183`;
- compare: 1 `67/76`, 2 `154/75`, 5 `378/150`, 10 `696/221`, 25 `1758/514`.

The single-symbol overhead is irrelevant to compare/events because their
public minimum is two, and quotes retains the same one-symbol operation with
no overlapping work. The ordered runner and quote end-to-end fixture prove a
maximum of four active operations and input-order restoration. Focused tests,
runtime-skill validation, strict clippy, and the full workspace suite are
green. Independent review approved the implementation after one stale
`CONTINUITY.md` statement was corrected, and focused re-review reported no
remaining findings. Gate 6 is complete and ready to commit.

## Context and Orientation

`crates/market/src/quote.rs`, `compare.rs`, and `events.rs` own the three
sequential loops. Their typed payloads live in `crates/market/src/types.rs`.
Quotes and events perform one scanner request per symbol. Compare performs
three HTTP reads per symbol in a fixed internal sequence. The same configured
reqwest client is shared across each top-level operation.

Gate 6 changes only Desktop-free multi-symbol scheduling. Chart-backed compare,
bars, Replay, screenshots, HTTP taxonomy, retry, timeout, and source selection
are outside scope.

## Plan of Work

Add a private Market module with an async ordered bounded collector. It accepts
indexed inputs and an async operation, executes at most four operations using
`futures_util::StreamExt::buffer_unordered`, collects `(requested_index,
result)`, and sorts by index before returning. Add deterministic tests with
reversed completion delays and atomic active/max-active counters.

Use a loopback HTTP measurement workload around the production bounded runner
and configured reqwest client. The server delays each response by 20
milliseconds and tracks active requests. Model quotes/events with one request
per symbol and compare with three sequential requests per symbol. Measure
sequential limit one and bounded limit four for 1, 2, 5, 10, and 25 symbols,
five runs each, and record medians here. Apply the fixed four limit separately
to each workflow only when both required medians improve by at least 25
percent.

For retained workflows, enumerate normalized symbols before scheduling. Keep
each compare symbol's quote, info, and fundamentals awaits sequential. Sort
completed work before `finalize_quote_items`, `finalize_compare_items`, or
`events_compare_from_items`. Fold ordered quote results only after sorting so
the all-failed kind continues to come from the earliest requested symbol.

Add `requested_index` to `BatchQuoteItem`. Add a 25-symbol maximum to Market
validation for quotes and compare and matching CLI pre-network contract tests.
Validation details report minimum, maximum, and existing source boundaries.
Events and watch limits remain unchanged; watch inherits quote scheduling
without changing JSONL events.

Update CLI help and stable architecture/development guidance. Runtime skills
may mention the 25-symbol command constraint only in the existing detailed
reference that owns command limits; do not add scheduling internals to Core
Workflow sections.

## Concrete Steps

Run from the repository root:

    cargo test -p tradingview-market bounded -- --nocapture
    cargo test -p tradingview-market quote -- --nocapture
    cargo test -p tradingview-market compare -- --nocapture
    cargo test -p tradingview-market events -- --nocapture
    cargo test -p tradingview-cli watch -- --nocapture
    cargo test -p tradingview-cli --test cli_contract_quote -- --nocapture
    cargo test -p tradingview-cli --test cli_contract_desktop chart -- --nocapture
    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1
    git diff --check
    bash -n scripts/stage-release-package-files.sh
    cmp -s AGENTS.md CLAUDE.md

## Validation and Acceptance

The fixture must observe at least one and at most four active symbol operations,
and zero after completion. Reversed completion order must still produce input-
ordered items and requested indexes. Compare must retain per-symbol section
order. Partial success, all-failed details, first-error kind, source metadata,
freshness fields, follow-up hints, and events/watch contracts must remain
unchanged.

Quotes accept 1 through 25 symbols; compare and events compare accept 2 through
25. A 26th symbol fails before network I/O with `Validation` and public-safe
minimum/maximum details. Chart compare remains unchanged and sequential.

## Idempotence and Recovery

Measurements use ephemeral loopback ports and are safe to repeat. Record all
five-run medians rather than selecting one favorable run. If either the 10- or
25-symbol median for a workflow improves by less than 25 percent, restore that
workflow's sequential scheduling, retain the measurements and validation
boundary, and explain the stop decision in this plan.

## Interfaces and Dependencies

Keep all public Rust function signatures and existing contract versions.
`BatchQuoteItem.requested_index: usize` is the only additive wire field. Use
the existing `futures-util` dependency. Add no command option, dependency,
retry, timeout, cache, source fallback, or package-version change.

## Open Questions

None. The measurement matrix, adoption threshold, fixed limits, ordering rule,
failure determinism, payload addition, and stop condition are fixed.

Revision note (2026-07-11): created the Gate 6 measurement and implementation
plan after Gate 5 completed independent review and commit.

Revision note (2026-07-11): recorded the measurement design, adoption result,
implementation, documentation sync, full validation, and independent-review
handoff.
