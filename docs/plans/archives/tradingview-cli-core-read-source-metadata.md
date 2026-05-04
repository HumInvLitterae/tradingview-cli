# Core read command source metadata

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document follows `.agents/PLANS.md` from the repository root. It is self-contained and describes how to add source taxonomy metadata to core Desktop-backed read commands without changing command behavior.

## Purpose / Big Picture

The `tv` CLI now classifies commands by source rather than splitting into separate binaries. Stream, readiness, and screenshot payloads already report this classification, but core reads such as `status`, `tab list`, `state`, `ohlcv`, and chart-source quote still expose the same boundary less consistently. After this change, agents can inspect JSON payloads and tell whether a result came from TradingView Desktop, whether Desktop was required, and whether the command was intended to mutate TradingView state.

This work is additive. It does not add new commands, change exit codes, change fallback behavior, or alter existing practical fields.

## Progress

- [x] (2026-05-04T18:38Z) Read the current roadmap, taxonomy, plan index, and core read implementation shape.
- [x] (2026-05-04T18:45Z) Added source metadata helpers and wired them into `status`, `tab list`, `state`, `ohlcv`, and chart-source quote payloads.
- [x] (2026-05-04T18:50Z) Updated docs and runtime skills to describe the now-consistent core read metadata.
- [x] (2026-05-04T19:02Z) Ran focused tests, full Rust validation, skill validation, packaging script syntax check, metadata check, and hygiene grep.
- [x] (2026-05-05T00:10Z) Commit the related changes.

## Surprises & Discoveries

- Observation: `tv quote <SYMBOL> --source chart` is not a purely non-mutating read because it can switch and restore the visible chart symbol.
  Evidence: `crates/cli/src/ops/market/quote.rs` already reports `switch_performed` and `restored`; this plan sets `non_mutating` from `!switch_performed` for chart-source quote success payloads.

## Decision Log

- Decision: Add metadata directly to the existing payloads rather than wrapping them in a new object.
  Rationale: Existing downstream users consume practical fields from the top-level data object. Additive top-level fields preserve compatibility.
  Date/Author: 2026-05-04 / Codex.

- Decision: Keep Desktop-free scanner quote and batch quote out of this slice.
  Rationale: The current gap is core Desktop-backed read consistency. Desktop-free read crates can be reviewed separately if needed.
  Date/Author: 2026-05-04 / Codex.

## Outcomes & Retrospective

Implemented. Core Desktop-backed read payloads now expose source taxonomy metadata without changing command surface, exit codes, or existing practical fields. Focused tests, full workspace tests, formatting, clippy, metadata, skill validation, packaging script syntax, and diff checks passed. The hygiene grep reported only existing policy text, archived validation commands, and secret-safety wording.

## Context and Orientation

The command source taxonomy lives in `docs/command-source-taxonomy.md`. It defines `Desktop-backed read` as a command that reads TradingView Desktop, a selected CDP target, or visible chart state. Such commands should expose `source_category: "desktop_backed_read"`, `requires_desktop: true`, and `non_mutating` when that statement is meaningful.

The core implementation files are:

- `crates/cli/src/ops/status.rs` for `tv status`.
- `crates/cli/src/ops/tab.rs` for `tv tab list`.
- `crates/cli/src/ops/chart.rs` for `tv state`.
- `crates/cli/src/ops/market/ohlcv.rs` for `tv ohlcv` and `tv ohlcv --summary`.
- `crates/cli/src/ops/market/quote.rs` for chart-source quote.
- `crates/cli/src/ops/common.rs` for shared operation helpers.

## Plan of Work

Add a small shared helper in `crates/cli/src/ops/common.rs` that produces Desktop-backed read metadata. Use it from Rust-built payloads. For JavaScript-built payloads such as `state` and raw OHLCV reads, add the metadata fields inside the returned object so the existing runtime path stays simple.

For chart-source quote, keep `source: "chart_api"` and add Desktop-backed metadata. Set `non_mutating` to `false` when a symbol switch happened and `true` when the requested symbol is already the current chart. Preserve `switch_performed`, `restored`, and `freshness_check` so downstream agents can see the chart mutation boundary.

Update documentation and skills only enough to make the taxonomy guidance accurate. Do not add machine-specific validator commands or local paths to tracked docs.

## Concrete Steps

From the repository root, edit the files named above. Then run:

    cargo test -p tradingview-cli status -- --nocapture
    cargo test -p tradingview-cli tab -- --nocapture
    cargo test -p tradingview-cli chart -- --nocapture
    cargo test -p tradingview-cli market::ohlcv -- --nocapture
    cargo test -p tradingview-cli market::quote -- --nocapture
    cargo test -p tradingview-cli --test cli_contract status -- --nocapture
    cargo test -p tradingview-cli --test cli_contract tab -- --nocapture
    cargo test -p tradingview-cli --test cli_contract quote -- --nocapture
    cargo test -p tradingview-cli --test cli_contract ohlcv -- --nocapture
    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1
    git diff --check

Also validate changed runtime skills with the repository's normal skill validation tool and run:

    bash -n scripts/stage-release-package-files.sh

Finally run a hygiene grep over public docs and skills to ensure no local machine paths, secrets, or environment-specific validation notes were added.

## Validation and Acceptance

Acceptance is reached when tests pass and representative payloads include the new metadata without losing old fields. In unit tests, assert that `status`, `tab list`, `state`, `ohlcv`, `ohlcv --summary`, and chart-source quote success payloads expose the expected source metadata. Existing scanner quote and `tv quotes` tests should continue passing without payload changes.

Optional live smoke may run these commands against a local TradingView Desktop session:

    target/debug/tv readiness
    target/debug/tv status
    target/debug/tv tab list
    target/debug/tv state
    target/debug/tv ohlcv --count 1
    target/debug/tv quote PLUG --source chart

Do not record live target ids or local output paths in tracked docs.

## Idempotence and Recovery

The changes are additive and can be applied repeatedly. If validation fails because formatting changed, run the formatter and re-run checks. If a live smoke fails due to TradingView Desktop state, do not change code based only on that failure; use structured error details to decide whether the failure belongs to this metadata slice.

## Artifacts and Notes

Validation evidence:

    cargo test -p tradingview-cli status -- --nocapture
    cargo test -p tradingview-cli tab -- --nocapture
    cargo test -p tradingview-cli chart -- --nocapture
    cargo test -p tradingview-cli market::ohlcv -- --nocapture
    cargo test -p tradingview-cli market::quote -- --nocapture
    cargo test -p tradingview-cli --test cli_contract status -- --nocapture
    cargo test -p tradingview-cli --test cli_contract tab -- --nocapture
    cargo test -p tradingview-cli --test cli_contract quote -- --nocapture
    cargo test -p tradingview-cli --test cli_contract ohlcv -- --nocapture
    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1
    git diff --check
    bash -n scripts/stage-release-package-files.sh

All completed successfully. Changed runtime skills also passed validation.

## Interfaces and Dependencies

No new external dependencies are introduced. The new shared helper is internal to `crates/cli/src/ops/common.rs` and is not a public Rust API. JSON payload additions are additive fields only.

At completion, these fields must be present on the covered Desktop-backed read success payloads:

    source_category: "desktop_backed_read"
    requires_desktop: true
    non_mutating: true or false depending on command behavior

For `status`, `tab list`, `state`, and `ohlcv`, `non_mutating` is true. For chart-source quote, `non_mutating` is false when `switch_performed` is true.

## Open Questions

No open questions. This plan intentionally excludes Desktop-free scanner/market payload metadata polish.
