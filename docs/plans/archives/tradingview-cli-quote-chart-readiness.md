# Quote chart source readiness plan

This ExecPlan is a living document. Keep `Progress`, `Discoveries`,
`Decisions`, and `Validation` current while implementing the fix.

## Purpose

Prevent `tv quote <SYMBOL> --source chart` from reporting success when the
visible chart symbol has changed but the chart bars / quote payload still comes
from the previous symbol.

This is a post-`v0.4.0` hardening fix. The scanner-backed default quote path
and `tv quotes` batch command remain unchanged.

## Progress

- [x] Created this ExecPlan and archived the completed v0.4.0 release
  readiness plan.
- [x] Add chart quote readiness polling after symbol switching.
- [x] Add a single retry only when requested-symbol readiness times out.
- [x] Preserve restore behavior and fail if restore cannot be verified.
- [x] Add focused unit and CLI contract coverage.
- [x] Update README, API docs, skills, and changelog.
- [x] Run validation.
- [x] Commit the related changes.

## Decisions

- `--source chart` should prefer correctness over speed. A symbol-targeted
  chart quote may take a few seconds.
- The command should not require downstream agents to sleep or double-call.
- Retry is limited to one extra requested-symbol switch after readiness timeout.
- `--source auto` keeps scanner fallback only for pre-mutation chart
  unavailability. It must not fall back after chart mutation/readiness failure.

## Implementation

Replace fixed-delay symbol switching with:

1. Read the current quote before switching and derive a small bar signature from
   symbol/time/OHLCV fields.
2. If the requested symbol differs from the current chart symbol, call
   `setSymbol`.
3. Poll current chart quote until:
   - observed symbol matches the requested symbol by bare ticker;
   - bar-derived quote values are present;
   - the bar signature differs from the pre-switch signature.
4. If requested readiness times out, call `setSymbol` for the requested symbol
   once more and repeat the readiness poll.
5. Restore the original symbol as before and fail if restore cannot be
   verified.

Success payloads keep existing practical quote fields and metadata, and add a
`freshness_check` object with `passed`, `kind`, `attempts`, and `elapsed_ms`.

Failure details include requested/original/observed symbols, restore state,
attempts, elapsed time, `freshness_check`, and a `next_action_hint`.

## Validation

Planned commands:

```bash
cargo test -p tradingview-cli market::quote -- --nocapture
cargo test -p tradingview-cli --test cli_contract quote -- --nocapture
cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
cargo metadata --no-deps --format-version 1
git diff --check
```

If live TradingView Desktop is available, run:

```bash
target/debug/tv quote PLUG --source chart
target/debug/tv quote AAPL --source chart
target/debug/tv quote PLUG --source auto
target/debug/tv symbol
```

Do not write live target ids, account-local values, raw payloads, cookies,
tokens, or local absolute paths into tracked docs.

Completed validation:

- `cargo test -p tradingview-cli market::quote -- --nocapture`
- `cargo test -p tradingview-cli --test cli_contract quote -- --nocapture`
- `python3 .../quick_validate.py .agents/skills/market-data-interpretation`
- `python3 .../quick_validate.py .agents/skills/chart-analysis`
- `python3 .../quick_validate.py .agents/skills/multi-symbol-scan`
- `cargo fmt --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace`
- `cargo metadata --no-deps --format-version 1`
- `git diff --check`
- tracked-doc hygiene grep for local paths and secret-like strings
- live smoke with `quote PLUG --source chart`, `quote AAPL --source chart`,
  `quote PLUG --source auto`, and `symbol`

## Rollback

Revert this plan, the quote readiness code changes, tests, and documentation
updates. The default scanner quote path and batch quote path are not expected
to change in this slice.
