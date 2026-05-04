# Desktop readiness integrated read command

## Summary

This plan adds `tv readiness` as a narrow Desktop-backed read command for
agent workflows. It is not a broad troubleshooting command and does not replace
`status`, `tab list`, `state`, or `ohlcv`; it aggregates their most useful
readiness signals into one public-safe payload that an agent can read before
running chart-dependent commands.

The command is non-mutating. It does not switch symbols, activate tabs, write
screenshot files, or change account/page state. If CDP is unreachable, it uses
the existing structured connection error. If CDP is reachable but target
selection, chart API state, or bars are not ready, it returns `success: true`
with `ready: false` and next-action hints.

## Implementation

- Add `tv readiness` to the CLI command surface.
- Add `crates/cli/src/ops/readiness.rs`.
- Reuse CDP target discovery and existing chart/OHLCV reads.
- Return source taxonomy metadata:
  - `source: "desktop_readiness"`
  - `source_category: "desktop_backed_read"`
  - `requires_desktop: true`
  - `non_mutating: true`
- Include CDP endpoint/target counts, target handoff arrays, selected target,
  chart readiness, bars readiness, `next_action_hint`, and a screenshot hint.
- Keep `tv diagnose` deferred. A broader troubleshooting command should wait
  until `readiness` usage shows what is still missing.

## Safety

- No TradingView account mutation.
- No tab activation or screenshot capture.
- No live target ids, account-local ids, raw page payloads, or local paths are
  written to tracked docs.
- `readiness` observes the currently selected or explicitly requested target
  only.

## Validation

- `cargo test -p tradingview-cli readiness -- --nocapture`
- `cargo test -p tradingview-cli status -- --nocapture`
- `cargo test -p tradingview-cli tab -- --nocapture`
- `cargo test -p tradingview-cli market::ohlcv -- --nocapture`
- `cargo test -p tradingview-cli --test cli_contract readiness -- --nocapture`
- `cargo test -p tradingview-cli --test cli_contract status -- --nocapture`
- `cargo test -p tradingview-cli --test cli_contract tab -- --nocapture`
- `cargo test -p tradingview-cli --test cli_contract ohlcv -- --nocapture`
- `cargo fmt --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace`
- `cargo metadata --no-deps --format-version 1`
- `git diff --check`

## Outcome

`tv readiness` becomes the first structured check for Desktop-backed chart
workflows. It gives agents a single payload that answers whether chart target
selection, chart API state, and one recent bar are usable, while keeping
existing low-level commands available for follow-up.
