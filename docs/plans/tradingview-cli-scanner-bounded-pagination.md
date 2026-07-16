# Add bounded scanner pagination

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds. Maintain this document in accordance with `.agents/PLANS.md`.

## Purpose / Big Picture

After this change, `tv scanner scan` can read more than the first 100 matching rows without removing the existing 100-row per-request safety bound. A caller can request a bounded aggregate scan, observe page and drift metadata, and receive no successful aggregate result when a page fails or the provider population exceeds the declared bound. This enables downstream independent-universe discovery without pretending that sequential pages form an atomic market snapshot.

## Progress

- [x] (2026-07-16) Verified that the current CLI clamps one request to 100 rows while the provider accepts an offset range such as `[100, 200]`.
- [x] (2026-07-16) Fixed provider paging ownership in `tradingview-scanner`; downstream callers must not implement a second page merger.
- [ ] Add RED tests for offset request construction and bounded aggregate completion.
- [ ] Implement the typed low-level offset and aggregate scan surfaces.
- [ ] Expose the bounded options through `tv scanner scan` and validate live read-only behavior.
- [ ] Run full repository gates and close the plan.

## Surprises & Discoveries

- Observation: the 100-row maximum is a deliberate client safety cap rather than a provider-enforced first-page boundary.
  Evidence: `MAX_SCAN_LIMIT` clamps the current request, while a direct read-only provider request for range `[100, 200]` returned 100 rows.

- Observation: the worktree already contains unrelated uncommitted roadmap, work-item, changelog, and pre-release-audit changes.
  Evidence: `git status --short` listed those files before this plan began. This slice does not edit or stage them.

## Decision Log

- Decision: retain a maximum page size of 100 and introduce explicit `offset`, `page_size`, and `max_results` semantics.
  Rationale: one unbounded request would remove the safety property; permanent first-page sampling would not satisfy downstream breadth discovery.
  Date/Author: 2026-07-16 / Codex.

- Decision: `tradingview-scanner` owns every provider request, offset, retry, termination, deduplication, drift signal, and incomplete-scan rejection. The CLI only renders the typed aggregate result.
  Rationale: downstream and CLI-layer page mergers would duplicate failure and completeness semantics.
  Date/Author: 2026-07-16 / Codex.

- Decision: a completed aggregate is a bounded sequential observation, not an atomic snapshot.
  Rationale: the provider supplies no snapshot token across page requests. Metadata must expose first, last, and maximum totals plus duplicate and timing evidence.
  Date/Author: 2026-07-16 / Codex.

## Outcomes & Retrospective

Planning and provider feasibility are complete. No implementation or aggregate live proof is claimed yet.

## Context and Orientation

`crates/scanner/src/scan.rs` validates scanner requests, sends Desktop-free REST calls, and normalizes provider rows. It currently builds `range: [0, limit]`, defaults to 20 rows, and clamps one request to 100. `crates/scanner/src/types.rs` owns the public typed result. `crates/cli/src/cli.rs` owns scanner arguments, while `crates/cli/src/ops/scanner.rs` re-exports the scanner API and the CLI dispatcher serializes its result. The scanner crate, not the CLI or downstream repository, is the provider boundary.

An offset is the zero-based first row requested from the provider. A page is one request of at most 100 rows. An aggregate scan is a sequence of pages with identical market, columns, filters, and sort. A query fingerprint is a deterministic representation of those fixed request fields. A drift signal records evidence that the sequential result changed while paging; it does not convert the result into an atomic snapshot.

## Plan of Work

Extend `ScannerScanRequest` with an offset used only for one-page reads. Preserve `limit` as the one-page size and keep its maximum at 100. Build the provider range as `[offset, offset + limit]`, rejecting overflow before network access.

Add a separate typed aggregate request and function in `crates/scanner/src/scan.rs`. It accepts the existing query fields plus `page_size` and `max_results`. It repeatedly calls the same internal one-page function, starting at zero. It rejects `page_size == 0`, `page_size > 100`, and `max_results == 0`. It fails before publishing a result when the maximum total observed across pages exceeds `max_results`, when a provider request remains failed after the existing bounded transport behavior, or when an empty page appears before the applicable observed total. It stops after reaching the applicable total or receiving an empty page at or beyond that boundary.

The aggregate result records rows after deterministic first-seen symbol deduplication, raw row count, duplicate count, pages fetched, first/last/maximum provider totals, query fingerprint, scan start/end epoch seconds, and drift flags for changing totals or duplicates. Acquisition sort is caller-supplied; downstream Discovery Sweep initially probes `name asc` and ranks locally.

Expose `--offset` for diagnostics and `--max-results` plus `--page-size` for aggregate mode. `--offset` cannot be combined with aggregate mode. Aggregate mode is activated only by `--max-results`; `--page-size` without it is invalid. Do not add `--all` in this slice.

## Concrete Steps

Work from the `tradingview-cli` repository root.

1. Add focused RED tests and run:

       cargo test -p tradingview-scanner scanner_scan

2. Implement typed paging and CLI arguments, then run:

       cargo test -p tradingview-scanner
       cargo test -p tradingview-cli scanner

3. Run a bounded read-only live smoke with a small maximum and verify the aggregate metadata without recording symbols in tracked files.

4. Run:

       cargo fmt --check
       cargo clippy --all-targets --all-features -- -D warnings
       cargo test
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

    pub struct ScannerAggregateScanRequest { ... }

    pub async fn scanner_scan_aggregate_typed(
        request: ScannerAggregateScanRequest,
    ) -> Result<ScannerAggregateScanResult, AppError>;

Use existing `reqwest`, `serde`, `serde_json`, and error contracts. Add no dependency.

## Open Questions

None. Roadmap and work-item index synchronization is intentionally deferred because those tracked files had unrelated uncommitted changes before this slice; the implementation commit must not absorb them.

2026-07-16: Created the bounded-pagination implementation plan from the approved downstream Discovery Sweep contract while preserving the pre-existing dirty documentation worktree.
