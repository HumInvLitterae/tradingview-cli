# Add watchlist bulk add

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document follows `.agents/PLANS.md`.

## Purpose / Big Picture

After this change, `tv watchlist add-bulk` can add a bounded list of symbols to the active TradingView watchlist in one operator command. This unblocks a common scanner-to-watchlist workflow while preserving the existing single-symbol post-add verification path.

This is an explicit watchlist mutation. It does not change Screener filters, save Screener screens, change columns, or add workflow scanner packs.

## Progress

- [x] (2026-04-25 19:15Z) Checked working tree, current watchlist implementation, upstream PR #65, and mutation boundaries.
- [x] (2026-04-25 19:45Z) Added `watchlist add-bulk` CLI surface and dispatch.
- [x] (2026-04-25 19:45Z) Implemented bulk add aggregation using existing verified `watchlist_add`.
- [x] (2026-04-25 19:45Z) Added unit and CLI contract tests.
- [x] (2026-04-25 19:45Z) Updated README, changelog, contract notes, and upstream triage note.
- [x] (2026-04-25 19:45Z) Ran focused tests, live smoke, and full validation baseline.
- [x] (2026-04-25 19:45Z) Recorded outcomes, committed tracked changes, and updated local continuity ledger.

## Surprises & Discoveries

- Observation: Existing Rust `watchlist_add` already handles exact already-present symbols and verifies newly added symbols after input.
  Evidence: `src/ops/layout.rs` returns `already_present` without text input and returns an error if `matched_after` is not true.

- Observation: Upstream PR #65's remaining unaddressed watchlist value is bulk add; watchlist remove and click hardening are already addressed in Rust.
  Evidence: `docs/notes/upstream-pr-triage-2026-04-25.md` marks PR #65 as partially addressed and leaves bulk add deferred.

- Observation: The live smoke target had multiple open TradingView targets, so commands needed an explicit `TV_CDP_TARGET_ID`.
  Evidence: the initial `target/debug/tv watchlist get` returned `target_ambiguous`; `target/debug/tv tab list` identified chart target `A80F6F4622DF34104163A605015B059C`.

- Observation: Bulk add smoke added all requested test symbols, and cleanup left none of those symbols in the watchlist.
  Evidence: `tv watchlist add-bulk NYSE:IBM NASDAQ:INTC NASDAQ:CSCO --delay-ms 500 --allow-partial` returned `added_count: 3`; subsequent removes cleared `NYSE:IBM` and `NASDAQ:CSCO`, `NASDAQ:INTC` was already absent by remove time, and final `tv watchlist get` found none of the three symbols.

## Decision Log

- Decision: Implement bulk add by calling existing `watchlist_add` sequentially.
  Rationale: Reusing the verified single-symbol path keeps the mutation logic small and avoids adding a second DOM automation path.
  Date/Author: 2026-04-25 / Codex.

- Decision: Keep duplicate inputs in the result as `skipped_duplicate`.
  Rationale: Operators should be able to understand why input and processed counts differ without hidden normalization.
  Date/Author: 2026-04-25 / Codex.

- Decision: Fail by default when any unique symbol fails, while `--allow-partial` returns an aggregate success payload.
  Rationale: Default behavior should be strict for account mutations, but operators may intentionally accept partial progress.
  Date/Author: 2026-04-25 / Codex.

## Outcomes & Retrospective

`tv watchlist add-bulk` is implemented as a bounded operator mutation. It accepts one or more symbols, rejects blank symbols, caps delay at 10000 milliseconds, caps the request at 50 unique symbols, skips exact duplicate inputs with per-input result entries, and uses the existing verified single-symbol `watchlist_add` path for each unique symbol.

The command is strict by default: if any unique add fails, it attempts all unique symbols and then returns `internal_api_unavailable` with the aggregate payload in `error.details`. Operators can opt into success-with-failures by passing `--allow-partial`.

Automated validation covered aggregation, duplicates, strict failure, partial failure, input validation before CDP connection, and CLI help visibility. Live smoke against a test watchlist added `NYSE:IBM`, `NASDAQ:INTC`, and `NASDAQ:CSCO`, then confirmed none of those smoke symbols remained in the final watchlist. The only notable operational wrinkle was target ambiguity, which was resolved by setting `TV_CDP_TARGET_ID` to the selected chart target.

## Context and Orientation

Watchlist operations live in `src/ops/layout.rs` because they share the right-panel layout surface with pane operations. `watchlist_add` opens the watchlist panel if needed, clicks the add-symbol control, types the symbol, presses Enter, dismisses the overlay, and verifies the symbol appears afterward.

## Plan of Work

Add `tv watchlist add-bulk <SYMBOL>... [--delay-ms <MS>] [--allow-partial]`. Validate inputs before connecting when possible: at least one symbol, no blank symbols, delay at most 10000 milliseconds, and at most 50 unique symbols.

Implement `watchlist_add_bulk` so it iterates over the original symbol list, skips duplicate normalized symbols after the first occurrence, calls `watchlist_add` for each unique symbol, sleeps between unique additions when `delay_ms` is non-zero, and returns aggregate counts and per-input results.

If one or more unique additions fail and `allow_partial` is false, return `internal_api_unavailable` with the aggregate payload in `error.details` after all unique symbols have been attempted. If `allow_partial` is true, return the aggregate payload with `failed_count`.

## Concrete Steps

Run commands from the repository root.

1. Edit `src/cli.rs`, `src/main.rs`, and `src/ops/layout.rs`.
2. Add focused tests in `src/ops/layout.rs` and `tests/cli_contract.rs`.
3. Update `README.md`, `CHANGELOG.md`, `docs/notes/rust-cli-contract-migration-2026-04-24.md`, and `docs/notes/upstream-pr-triage-2026-04-25.md`.
4. Run focused tests:

        cargo test ops::layout::tests::watchlist_add_bulk -- --nocapture
        cargo test --test cli_contract watchlist -- --nocapture

5. Run live smoke against the current test watchlist:

        target/debug/tv watchlist get
        target/debug/tv watchlist add-bulk NYSE:IBM NASDAQ:INTC NASDAQ:CSCO --delay-ms 500 --allow-partial
        target/debug/tv watchlist remove <ADDED_SYMBOL>

6. Run full validation:

        cargo fmt --check
        cargo clippy --all-targets --all-features -- -D warnings
        cargo test
        git diff --check
        git grep -nE '(/Users/|C:\\|USER;)' -- README.md AGENTS.md docs .agents/skills || true

## Validation and Acceptance

The change is accepted when help output lists `add-bulk`, automated tests prove aggregate counts and partial-failure behavior, live smoke can add several test symbols and clean up those newly added symbols, and the full validation baseline passes.

## Idempotence and Recovery

The command is idempotent for symbols already in the visible watchlist because existing `watchlist_add` returns `already_present` without typing. If live smoke leaves test symbols behind, record the exact symbols and remove them with `tv watchlist remove <SYMBOL>`.

## Artifacts and Notes

Record only command summaries, counts, added symbols, and cleanup outcomes. Do not paste full watchlist contents into tracked docs.

Focused and full validation passed:

    cargo fmt --check
    cargo test ops::layout::tests::watchlist_add_bulk -- --nocapture
    cargo test --test cli_contract watchlist -- --nocapture
    cargo clippy --all-targets --all-features -- -D warnings
    cargo test
    git diff --check
    git grep -nE '(/Users/|C:\\|USER;)' -- README.md AGENTS.md docs .agents/skills || true

The grep command returned only existing validation-command examples in plan documents, not live account identifiers or newly introduced machine-specific paths.

Live smoke summary:

    TV_CDP_TARGET_ID=A80F6F4622DF34104163A605015B059C target/debug/tv watchlist add-bulk NYSE:IBM NASDAQ:INTC NASDAQ:CSCO --delay-ms 500 --allow-partial

The command returned `added_count: 3`, `failed_count: 0`, and per-symbol `status: "added"` results for the three test symbols. Cleanup removed `NYSE:IBM` and `NASDAQ:CSCO`; `NASDAQ:INTC` was already absent when remove was attempted. A final watchlist read confirmed none of the three test symbols remained.

## Interfaces and Dependencies

Add `watchlist_add_bulk` to the public operation facade through the existing `ops` re-export path. No new crate dependencies are required.

## Open Questions

None.

Revision note: updated after implementation to record the completed CLI surface, validation evidence, live smoke outcome, and cleanup result.
