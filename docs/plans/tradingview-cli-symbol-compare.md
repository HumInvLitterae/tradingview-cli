# `tv compare <SYMBOL>...` Initial Implementation Plan

This ExecPlan defines the first implementation slice for the `v0.9.0`
Desktop-free comparison lane.

## Purpose

Add `tv compare <SYMBOL>...` as a Desktop-free comparison packet for several
symbols. The command should help agents compare candidates without switching
the visible TradingView Desktop chart or pretending scanner-backed data is a
realtime selected-chart feed.

## Surface

Initial command:

```bash
tv compare <SYMBOL>...
```

Validation:

- require at least two non-empty symbols before network access;
- preserve input order;
- allow bare and exchange-qualified symbols using the same resolution behavior
  as the existing Desktop-free market reads;
- do not add `--source`, chart include, bars include, screenshot, watch/JSONL,
  or ranking-option flags in this slice.

## Payload Direction

Return a normal JSON envelope with `command: "compare"`.

The `data` payload should be a Desktop-free comparison packet:

- `source: "compare_desktop_free"`;
- `source_category: "desktop_free_read"`;
- `requires_desktop: false`;
- `non_mutating: true`;
- requested/resolved/error counts;
- ordered per-symbol items;
- per-item evidence sections for quote, info, and fundamentals;
- public-safe per-item errors and missing fields;
- `next_action_hints` for chart follow-up, such as `tv snapshot`,
  `tv observe chart`, or `tv quote --source chart` for finalists.

Do not compute buy/sell recommendations. If summary fields are added, keep
them descriptive and evidence-based, such as available quote/fundamentals
values and missing-data summaries.

## Implementation Notes

Prefer the existing Desktop-free typed APIs in `tradingview-market`. Add a
typed compare API there if it reduces duplication and keeps the CLI wrapper
behavior-preserving.

Do not use TradingView Desktop, CDP, chart-source quote, `tv bars`,
screenshots, or `observe chart` in compare. Chart evidence remains a follow-up
workflow after comparison narrows the symbol set.

Existing commands must keep their payloads and behavior:

- `tv quote <SYMBOL>`;
- `tv quotes <SYMBOL>...`;
- `tv snapshot <SYMBOL>`;
- `tv fundamentals <SYMBOL>`;
- `tv observe chart`;
- `tv quote <SYMBOL> --source chart|auto`.

## Docs

Update README and `docs/observation-workflows.md` with a short compare example.
Update `docs/command-source-taxonomy.md` to classify compare as a
Desktop-free read. Update runtime skills only enough to explain when to use
`compare` versus `quotes`, `snapshot`, and `observe chart`.

## Validation

Focused tests:

```bash
cargo test -p tradingview-market compare -- --nocapture
cargo test -p tradingview-cli --test cli_contract compare -- --nocapture
```

Baseline:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
cargo metadata --no-deps --format-version 1
git diff --check
bash -n scripts/stage-release-package-files.sh
```

Read-only smoke after building:

```bash
target/debug/tv compare NASDAQ:AAPL NYSE:IONQ
target/debug/tv compare AAPL MSFT NVDA
TV_CDP_PORT=9 target/debug/tv compare NASDAQ:AAPL NYSE:IONQ
```

Expected: compare runs without Desktop/CDP and returns ordered per-symbol
evidence with source metadata and public-safe section errors.

## Assumptions

- `compare` is not a trading recommendation command.
- Scanner-backed quote freshness remains screening evidence, not a realtime
  entitlement guarantee.
- Missing fields are unknown, not zero.
- Chart-source quote mismatch, if later reproduced, is handled in a separate
  patch lane.
- Related implementation should be committed in one sensible batch.
