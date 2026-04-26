# Split scanner operations by sub-surface

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document follows `.agents/PLANS.md`.

## Purpose / Big Picture

After this change, scanner operations stay easier to extend because the Hotlist preset REST command and the generic scanner scan REST command live in separate modules. Users should observe no CLI behavior change: `tv scanner hotlist ...` and `tv scanner scan ...` must keep the same arguments, JSON payloads, validation behavior, and read-only REST semantics.

This is a refactor-only slice. It prepares the codebase for later scanner filter expansion without mixing that feature work into the structural change.

## Progress

- [x] (2026-04-25 17:22Z) Read current `src/ops/scanner.rs`, module layout examples, and development guideline module-split rules.
- [x] (2026-04-25 17:24Z) Split scanner operations into facade plus `hotlist`, `scan`, and `common` submodules.
- [x] (2026-04-25 17:25Z) Ran focused scanner tests and read-only REST smoke.
- [x] (2026-04-25 17:27Z) Ran full validation baseline.
- [x] (2026-04-25 17:27Z) Recorded outcomes and prepared the completed refactor for commit.

## Surprises & Discoveries

- Observation: `src/ops/scanner.rs` has grown to 809 lines after adding `tv scanner scan`.
  Evidence: `wc -l src/ops/scanner.rs` reported `809`.

- Observation: After the split, the scanner facade is small and the implementation files are separated by sub-surface.
  Evidence: `wc -l src/ops/scanner.rs src/ops/scanner/*.rs` reported 6 lines for the facade, 258 lines for `hotlist.rs`, 556 lines for `scan.rs`, and 9 lines for `common.rs`.

## Decision Log

- Decision: Split before adding more scanner filters.
  Rationale: `docs/development.md` says not to reintroduce monolithic operation files and to split a capability module before adding another command when it becomes hard to scan.
  Date/Author: 2026-04-25 / Codex.

- Decision: Keep this refactor behavior-preserving.
  Rationale: Public CLI behavior and JSON contracts were just updated for `tv scanner scan`; changing behavior in the same slice would make regressions harder to spot.
  Date/Author: 2026-04-25 / Codex.

## Outcomes & Retrospective

Completed. `src/ops/scanner.rs` is now a thin facade, `tv scanner hotlist`
implementation and tests live in `src/ops/scanner/hotlist.rs`, `tv scanner
scan` implementation and tests live in `src/ops/scanner/scan.rs`, and the small
shared compact-row helper lives in `src/ops/scanner/common.rs`.

Behavior was intentionally unchanged. Focused scanner tests passed, full
baseline passed, and read-only REST smoke still returned the expected source
values:

- `scanner_preset_rest` for `tv scanner hotlist volume_gainers --limit 1`
- `scanner_scan_rest` for `tv scanner scan --exchange NASDAQ --exchange NYSE --sort market_cap_basic --desc --limit 3`

Validation passed:

- `cargo fmt --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test`
- `git diff --check`
- `git grep -nE '(/Users/|C:\\|USER;)' -- README.md AGENTS.md docs .agents/skills || true`

The grep command reported only tracked command examples that intentionally show
the grep pattern itself; no live account identifiers or machine-specific paths
were added.

## Context and Orientation

The scanner command group is defined in `src/cli.rs` as `ScannerCommand`. Dispatch in `src/main.rs` calls operation functions exported from `src/ops.rs`. Before this refactor, both scanner operations lived in one large `src/ops/scanner.rs` file.

This repository uses Rust 2024 module layout and does not use `mod.rs`. A thin facade file plus a same-named directory is the established pattern, as shown by `src/ops/data.rs` with submodules under `src/ops/data/`, and by `src/ops/pine.rs` with submodules under `src/ops/pine/`.

## Plan of Work

Create `src/ops/scanner/` and move Hotlist-specific code into `src/ops/scanner/hotlist.rs`. Move generic scan code and `ScannerScanRequest` into `src/ops/scanner/scan.rs`. Move only the shared compact-row helper `field_values_object` into `src/ops/scanner/common.rs`.

Replace `src/ops/scanner.rs` with a thin facade:

    mod common;
    mod hotlist;
    mod scan;

    pub use hotlist::scanner_hotlist;
    pub use scan::{ScannerScanRequest, scanner_scan};

Do not change `src/cli.rs`, `src/main.rs`, or the public exports in `src/ops.rs` unless the split reveals a compile-only import issue. Do not change README or contract notes because public behavior must remain identical.

## Concrete Steps

Run commands from the repository root.

1. Add `src/ops/scanner/common.rs`, `src/ops/scanner/hotlist.rs`, and `src/ops/scanner/scan.rs`.
2. Replace `src/ops/scanner.rs` with the facade.
3. Run:

        cargo test scanner -- --nocapture
        cargo test --test cli_contract scanner -- --nocapture

4. Run read-only REST smoke:

        target/debug/tv scanner hotlist volume_gainers --limit 1
        target/debug/tv scanner scan --exchange NASDAQ --exchange NYSE --sort market_cap_basic --desc --limit 3

5. Run:

        cargo fmt --check
        cargo clippy --all-targets --all-features -- -D warnings
        cargo test
        git diff --check
        git grep -nE '(/Users/|C:\\|USER;)' -- README.md AGENTS.md docs .agents/skills || true

## Validation and Acceptance

The refactor is accepted when scanner focused tests and full baseline pass, and the two read-only REST smoke commands still return success envelopes with unchanged `source` values: `scanner_preset_rest` for Hotlist and `scanner_scan_rest` for scan.

No TradingView Desktop session is required, and no chart, watchlist, alert, layout, drawing, Pine, replay, or tab state should change.

## Idempotence and Recovery

This refactor is safe to retry. If a module import fails, keep the facade exports stable and move only private helpers between submodules until tests pass. Since no public behavior changes are intended, any payload drift observed in smoke should be treated as a regression and fixed before commit.

## Artifacts and Notes

Do not paste raw scanner payloads or long symbol lists into tracked docs. Record only source values, counts, and validation command results.

## Interfaces and Dependencies

Public operation exports must remain:

    pub async fn scanner_hotlist(slug: &str, limit: Option<usize>) -> Result<Value, AppError>;
    pub struct ScannerScanRequest;
    pub async fn scanner_scan(request: ScannerScanRequest) -> Result<Value, AppError>;

No new crate dependencies are required.

## Open Questions

None.
