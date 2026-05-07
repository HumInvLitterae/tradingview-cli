# `tv compare` Opt-in Live Contract Smoke Plan

This ExecPlan adds an ignored Rust integration test for the live
`tv compare <SYMBOL>...` JSON contract. It does not change command behavior.

Status as of 2026-05-07: implemented. The ignored test is present and normal
workspace tests skip it unless `--ignored` and `TV_LIVE_COMPARE_SMOKE=1` are
used.

## Purpose

Confirm that the Desktop-free compare packet works against live TradingView
scanner, symbol search, and fundamentals reads without making live network
availability a normal CI requirement.

## Surface

Add `crates/cli/tests/live_compare.rs`.

The test is ignored and gated:

```bash
TV_LIVE_COMPARE_SMOKE=1 cargo test -p tradingview-cli --test live_compare -- --ignored --nocapture
```

Optional environment variables:

- `TV_LIVE_COMPARE_SYMBOLS`: comma-separated public symbols, defaulting to
  `NASDAQ:AAPL,NYSE:IONQ`;
- `TV_LIVE_COMPARE_RUNS`: positive repeat count, defaulting to `1`.

## Contract Checks

Each run invokes the test-built `tv` binary with `compare <SYMBOL>...`.

The smoke validates only public contract fields:

- success envelope with `command: "compare"`;
- `source: "compare_desktop_free"`;
- `source_category: "desktop_free_read"`;
- `requires_desktop: false`;
- `non_mutating: true`;
- requested count and input item order;
- per-item `ok`, `sections`, `errors`, and `missing_summary`;
- quote, info, and fundamentals section success/error shape;
- at least one successful item;
- top-level `errors` and `next_action_hints` arrays.

Failure messages must summarize only symbol list, exit status, command,
resolved/error counts, item section states, and public error kind/message. Do
not print raw JSON, live response bodies, account-local metadata, target ids,
or local absolute paths.

## Docs

Update `docs/development.md` with the opt-in command and environment
variables. Update `docs/v0.9-roadmap.md`, `docs/plans/README.md`, and
`CHANGELOG.md` to record this as a test/tooling slice after the first compare
implementation.

## Validation

```bash
cargo test -p tradingview-cli --test live_compare
cargo test -p tradingview-cli --test cli_contract compare -- --nocapture
cargo test -p tradingview-market compare -- --nocapture
cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
cargo metadata --no-deps --format-version 1
git diff --check
bash -n scripts/stage-release-package-files.sh
```

Optional live smoke:

```bash
TV_LIVE_COMPARE_SMOKE=1 cargo test -p tradingview-cli --test live_compare -- --ignored --nocapture
```

## Assumptions

- This slice is test/tooling/docs only.
- `tv compare` CLI surface, payload, validation, and section behavior remain
  unchanged.
- Compare stays Desktop-free and does not include chart reads, screenshots,
  lab bars, watch/JSONL, or scoring/ranking options.
- Related changes should be committed in one sensible batch.

## Outcomes

- Added `crates/cli/tests/live_compare.rs` as an ignored live smoke.
- Documented `TV_LIVE_COMPARE_SMOKE`, `TV_LIVE_COMPARE_SYMBOLS`, and
  `TV_LIVE_COMPARE_RUNS` in development docs.
- Archived the initial compare implementation plan and left v0.9 ready for the
  next slice selection after smoke evidence or downstream feedback.
