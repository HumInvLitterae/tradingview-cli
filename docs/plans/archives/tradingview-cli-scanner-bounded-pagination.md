# Add bounded scanner pagination

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds. Maintain this document in accordance with `.agents/PLANS.md`.

## Purpose / Big Picture

After this change, `tv scanner scan` can read more than the first 100 matching rows without removing the existing 100-row per-request safety bound. A caller can request a bounded aggregate scan, observe page and drift metadata, and receive no successful aggregate result when a page fails or the provider population exceeds the declared bound. This enables downstream independent-universe discovery without pretending that sequential pages form an atomic market snapshot.

## Progress

- [x] (2026-07-16) Verified that the current CLI clamps one request to 100 rows while the provider accepts an offset range such as `[100, 200]`.
- [x] (2026-07-16) Fixed provider paging ownership in `tradingview-scanner`; downstream callers must not implement a second page merger.
- [x] (2026-07-16) Promoted this owner-approved downstream requirement as the final v0.28 feature slice before the queued completion audit.
- [x] (2026-07-16) Added deterministic offset, aggregate completion, drift, deduplication, malformed-total, premature-empty, and population-bound tests.
- [x] (2026-07-16) Implemented typed low-level offset and aggregate scan surfaces with one shared configured HTTP client.
- [x] (2026-07-16) Exposed the bounded options through `tv scanner scan` and validated an 11-page read-only aggregate without Desktop.
- [x] (2026-07-16) Ran the full repository gates and synchronized public,
  packaged-agent, skill-reference, API, roadmap, inventory, and changelog
  guidance.
- [x] (2026-07-16) Applied independent-review corrections for non-empty short
  pages, existing result-type compatibility, aggregate boundary coverage, and
  public API documentation.
- [x] (2026-07-16) Preserved provider row count before compatibility
  truncation so aggregate reads also reject overfull raw pages.
- [x] (2026-07-16) Focused independent implementation re-review reported no
  remaining findings; closeout and archive are complete.

## Milestones

### Milestone: define and test bounded page semantics

Fix the numeric limits, option conflicts, provider-total requirements,
deduplication, drift metadata, and no-partial-success policy. This milestone is
complete: deterministic tests prove offset construction, overflow rejection,
monotonic page progression, first-seen deduplication, total drift, malformed
total rejection, premature-empty rejection, and over-bound failure.

### Milestone: expose typed and CLI aggregate reads

Implement one-page offset and bounded aggregate APIs in `tradingview-scanner`,
then route `tv scanner scan` to single-page or aggregate mode without Desktop,
fallback, or a second CLI-owned merger. This milestone is complete: focused
Rust and CLI tests pass and the help exposes all three options and conflicts.

### Milestone: validate, document, and review

Synchronize public, Rust API, packaged-agent, skill-reference, roadmap, and
changelog guidance. Run the full repository baseline and obtain independent
implementation review. Documentation and the full local baseline are green;
the first review found two correctness/API blockers and two coverage/docs
findings. Corrections are applied, and this milestone remains open for focused
independent implementation re-review. That review is green, so this milestone
and the plan are complete. The v0.28 completion audit may now start.

## Surprises & Discoveries

- Observation: the 100-row maximum is a deliberate client safety cap rather than a provider-enforced first-page boundary.
  Evidence: `MAX_SCAN_LIMIT` clamps the current request, while a direct read-only provider request for range `[100, 200]` returned 100 rows.

- Observation: the worktree already contained the queued audit draft and its
  roadmap, work-item, and changelog transition when this feature was promoted.
  Evidence: the owner explicitly made scanner pagination the final v0.28
  feature exception, so those documents were updated in place to queue the
  audit behind this implementation rather than discarded or staged separately.

- Observation: a non-empty short page is not proof of provider completion.
  Evidence: advancing a fixed offset after a short page can skip rows while a
  later page still reaches the reported total boundary. Aggregate reads now
  require exactly `min(page_size, totalCount - offset)` rows from every page.

- Observation: compatibility truncation must not erase aggregate completeness
  evidence.
  Evidence: the shared single-page normalizer intentionally keeps at most the
  requested limit, so aggregate reads now retain the provider's raw `data`
  length separately and validate that count before consuming normalized rows.

## Decision Log

- Decision: retain a maximum page size of 100 and introduce explicit `offset`, `page_size`, and `max_results` semantics.
  Rationale: one unbounded request would remove the safety property; permanent first-page sampling would not satisfy downstream breadth discovery.
  Date/Author: 2026-07-16 / Codex.

- Decision: bound `max_results` to `1..=10_000`; aggregate `page_size` defaults to 100 and remains `1..=100`.
  Rationale: the aggregate must have a finite request-count ceiling of 100 pages, so the chosen `max_results` and `page_size` combination must also fit within 100 requests. Ten thousand rows covers the intended US-equity discovery use without creating an unbounded scanner crawler.
  Date/Author: 2026-07-16 / Codex.

- Decision: aggregate mode requires a non-negative integer `totalCount` on every page and reuses one configured HTTP client for the whole top-level operation.
  Rationale: without a provider total the client cannot prove bounded completion, and rebuilding transport state per page would discard the repository's connection-reuse policy.
  Date/Author: 2026-07-16 / Codex.

- Decision: expose a deterministic 64-bit FNV-1a hexadecimal fingerprint of the normalized market, columns, filters, and sort, excluding range.
  Rationale: callers need to correlate pages without returning a bulky query body or adding a hashing dependency. The fingerprint is diagnostic, not cryptographic or secret-bearing.
  Date/Author: 2026-07-16 / Codex.

- Decision: `tradingview-scanner` owns every provider request, offset, retry, termination, deduplication, drift signal, and incomplete-scan rejection. The CLI only renders the typed aggregate result.
  Rationale: downstream and CLI-layer page mergers would duplicate failure and completeness semantics.
  Date/Author: 2026-07-16 / Codex.

- Decision: a completed aggregate is a bounded sequential observation, not an atomic snapshot.
  Rationale: the provider supplies no snapshot token across page requests. Metadata must expose first, last, and maximum totals plus duplicate and timing evidence.
  Date/Author: 2026-07-16 / Codex.

## Outcomes & Retrospective

Planning, implementation, documentation, full local validation, and a bounded
read-only live proof are complete. The live smoke observed 11 pages and 1,070
rows with stable totals and no duplicates; only aggregate counts and source
markers are retained. Formatting, strict workspace Clippy, the full workspace
test suite, Cargo metadata, public hygiene, package-script syntax, guide parity,
diff hygiene, and the changed runtime-skill validation are green. Focused
independent implementation review found short-page completeness, existing
result-type compatibility, boundary coverage, and API-doc issues. The
first correction re-review found that compatibility truncation hid overfull
provider pages. Raw provider row count is now retained for aggregate validation,
and the production normalization path has deterministic regression coverage.
Focused/full local validation and focused independent re-review are green. The
feature is complete for v0.28, and the plan is archived. The key lesson is that
aggregate completeness must retain evidence from before compatibility
normalization; normalized row collections alone cannot prove an exact provider
page.

## Context and Orientation

`crates/scanner/src/scan.rs` validates scanner requests, sends Desktop-free REST calls, and normalizes provider rows. A default read builds `range: [0, limit]`; an explicit page builds `[offset, offset + limit]`. One request defaults to 20 rows and remains capped at 100. `crates/scanner/src/types.rs` owns the public typed results. `crates/cli/src/cli.rs` owns scanner arguments, while `crates/cli/src/ops/scanner.rs` re-exports the scanner API and the CLI dispatcher serializes its result. The scanner crate, not the CLI or downstream repository, is the provider boundary.

An offset is the zero-based first row requested from the provider. A page is one request of at most 100 rows. An aggregate scan is a sequence of pages with identical market, columns, filters, and sort. A query fingerprint is a deterministic representation of those fixed request fields. A drift signal records evidence that the sequential result changed while paging; it does not convert the result into an atomic snapshot.

## Plan of Work

Preserve `ScannerScanRequest` so existing Rust struct-literal callers do not break. Add `ScannerPageScanRequest`, which wraps the existing request with an offset used only for one-page reads. Preserve `limit` as the one-page size and keep its maximum at 100. Build the provider range as `[offset, offset + limit]`, rejecting overflow before network access.

Add a separate typed aggregate request and function in `crates/scanner/src/scan.rs`. It accepts the existing query fields plus `page_size` and `max_results`. It repeatedly calls the same internal one-page function, starting at zero. It rejects `page_size == 0`, `page_size > 100`, and `max_results == 0`. It fails before publishing a result when the maximum total observed across pages exceeds `max_results`, when a provider request remains failed after the existing bounded transport behavior, or when a page contains any count other than `min(page_size, totalCount.saturating_sub(offset))`. This rule applies independently to every page, including pages that report upward or downward total drift. It stops after the requested range reaches the total reported by the current complete page.

The aggregate result records rows after deterministic first-seen symbol deduplication, raw row count, duplicate count, pages fetched, first/last/maximum provider totals, query fingerprint, scan start/end epoch seconds, and drift flags for changing totals or duplicates. Acquisition sort is caller-supplied; downstream Discovery Sweep initially probes `name asc` and ranks locally.

Expose `--offset` for diagnostics and `--max-results` plus `--page-size` for aggregate mode. `--offset` cannot be combined with aggregate mode. Aggregate mode is activated only by `--max-results`; `--page-size` without it is invalid. `--limit` is single-page-only and conflicts with aggregate mode. The CLI validates offset-plus-limit overflow before network access. Do not add `--all` in this slice.

Default single-page output remains the existing `ScannerScanResult` without a new required field. Explicit-offset reads use a distinct flattened `ScannerPageScanResult` wrapper that adds `offset` without breaking existing Rust struct literals or the default JSON payload. Aggregate output is a distinct `ScannerAggregateScanResult` with the existing source metadata, market, columns, sort, and filters plus `page_size`, `max_results`, `count`, `raw_count`, `duplicate_count`, `pages_fetched`, first/last/maximum totals, query fingerprint, start/end epoch seconds, total-change and duplicate drift flags, and deduplicated `symbols`. Every aggregate page must contain an integer `totalCount` and the exact row count implied by its current offset, page size, and total; missing or malformed totals and incomplete pages fail closed as `InternalApiUnavailable`. If any observed total exceeds `max_results`, return a validation-style bounded-result error and publish no partial result.

## Concrete Steps

Work from the `tradingview-cli` repository root.

1. Add focused RED tests and run:

       cargo test -p tradingview-scanner scanner_scan

2. Implement typed paging and CLI arguments, then run:

       cargo test -p tradingview-scanner
       cargo test -p tradingview-cli --test cli_contract_quote scanner_scan -- --nocapture

3. Run a bounded read-only live smoke with a small maximum and verify the aggregate metadata without recording symbols in tracked files.

4. Run:

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

Tests must prove `[offset, offset + page_size]` construction, offset overflow rejection, fixed query fields across pages, monotonically increasing offsets, maximum page size 100, maximum-observed-total bound rejection, premature empty-page rejection, first-seen deduplication, drift metadata, and no successful aggregate result after a page failure. CLI tests must prove conflicting single-page and aggregate arguments fail before network access.

A read-only live smoke is accepted when more than one page is fetched, no Desktop connection is required, the returned count and page metadata agree, and the output explicitly identifies itself as a sequential bounded observation rather than an atomic snapshot. No live symbols or raw provider payloads enter tracked files.

## Idempotence and Recovery

All scanner operations are read-only. Retrying after a network or provider failure is safe. The typed function constructs the result only after every accepted page completes; callers receive an error rather than a partial successful aggregate. Existing single-page behavior remains the default when aggregate arguments are absent.

## Artifacts and Notes

The downstream requirement is recorded in `t-tools-and-memo` under `docs/plans/backtest-execplan-midterm10-discovery-sweep.md`. That repository adds provenance after receiving this crate's aggregated typed result and must not page independently.

## Interfaces and Dependencies

Keep `scanner_scan_typed` as the one-page typed read and add an aggregate API centered on:

    pub struct ScannerPageScanRequest {
        pub scan: ScannerScanRequest,
        pub offset: usize,
    }

    pub struct ScannerAggregateScanRequest { ... }

    pub async fn scanner_scan_aggregate_typed(
        request: ScannerAggregateScanRequest,
    ) -> Result<ScannerAggregateScanResult, AppError>;

Use existing `reqwest`, `serde`, `serde_json`, and error contracts. Add no dependency.

## Open Questions

None. Roadmap, work inventory, plan index, audit sequencing, changelog, and continuity state now identify this as the final v0.28 feature slice.

2026-07-16: Created the bounded-pagination implementation plan from the approved downstream Discovery Sweep contract while preserving the pre-existing dirty documentation worktree.

2026-07-16: Promoted the plan into v0.28 by owner direction and synchronized it ahead of the queued completion audit.
