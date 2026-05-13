# CLI contract tests split

This ExecPlan is a living document. Keep `Progress`,
`Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective`
current as work proceeds.

This document follows `.agents/PLANS.md` from the repository root. It records
the behavior-preserving test organization cleanup that splits the large CLI
contract integration test into command-family test targets.

## Purpose / Big Picture

`crates/cli/tests/cli_contract.rs` had grown to more than 2,800 lines and
mixed root help, Desktop-free market evidence, `bars`, `diagnose`, and
CDP-backed command validation in a single integration test target. That made
contract work harder to review and encouraged new coverage to keep landing in
one large file.

This slice keeps every assertion and public behavior intact while moving tests
into smaller command-family targets with shared integration-test helpers.

## Progress

- [x] Create this ExecPlan and archive the completed bars market internal split
  plan.
- [x] Add `crates/cli/tests/support/mod.rs` for shared `tv()` and stderr JSON
  parsing helpers.
- [x] Keep root-level tests in `cli_contract.rs`.
- [x] Move `tv bars` contract tests into `cli_contract_bars.rs`.
- [x] Move Desktop-free market evidence tests into `cli_contract_quote.rs`.
- [x] Move `diagnose quote-data` tests into `cli_contract_diagnose.rs`.
- [x] Move CDP-backed read/mutation command contract tests into
  `cli_contract_desktop.rs`.
- [x] Run focused tests, full baseline, and docs hygiene.
- [x] Commit the split.

## Surprises & Discoveries

- The existing contract tests were already grouped well enough by command
  family to move by function name without changing assertion intent.
- `cli_contract_desktop.rs` remains the largest target because it covers the
  broad CDP-backed surface. That is acceptable for this slice; further
  splitting can happen later by command family if it becomes painful.

## Decision Log

- Decision: Keep `cli_contract.rs` as the root-level contract test target.
  Rationale: Version, root help, unknown command, and generic connection
  envelope tests are cross-command contracts and do not belong to a command
  family.
  Date/Author: 2026-05-14 / Codex.

- Decision: Create `cli_contract_bars`, `cli_contract_quote`,
  `cli_contract_diagnose`, and `cli_contract_desktop` as separate integration
  test targets.
  Rationale: These are the current high-value boundaries: stable browserless
  bars, Desktop-free market evidence, quote-data diagnostics, and CDP-backed
  operations.
  Date/Author: 2026-05-14 / Codex.

- Decision: Do not change CLI behavior or assertion semantics while splitting.
  Rationale: This is a release-prep refactor. Any help wording, validation, or
  payload contract change belongs in a separate plan.
  Date/Author: 2026-05-14 / Codex.

## Outcomes & Retrospective

The CLI contract suite is now split by command family:

- `cli_contract.rs`: root CLI contracts;
- `cli_contract_bars.rs`: stable browserless historical bars contracts;
- `cli_contract_quote.rs`: quote, quotes, fundamentals, snapshot, compare,
  scanner, info, and search contracts;
- `cli_contract_diagnose.rs`: quote-data diagnostics contracts;
- `cli_contract_desktop.rs`: CDP-backed command help, validation, and
  connection-envelope contracts.

The split is test-only. No public command, option, help text, JSON contract,
validation behavior, dependency, or version changed.

## Plan of Work

1. Add shared integration-test support helpers.
2. Move existing test functions into command-family files.
3. Keep root-level tests in the original `cli_contract.rs`.
4. Update docs to record the test organization rule.
5. Run focused test targets and the full workspace baseline.

## Acceptance Criteria

- All test functions from the original `cli_contract.rs` still exist in one of
  the split targets.
- Focused split test targets pass.
- `cargo test --workspace` passes.
- Public behavior and JSON contract remain unchanged.
- No new dependency or product code change is introduced.

## Validation

Run:

    cargo test -p tradingview-cli --test cli_contract -- --nocapture
    cargo test -p tradingview-cli --test cli_contract_bars -- --nocapture
    cargo test -p tradingview-cli --test cli_contract_quote -- --nocapture
    cargo test -p tradingview-cli --test cli_contract_diagnose -- --nocapture
    cargo test -p tradingview-cli --test cli_contract_desktop -- --nocapture
    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1
    git diff --check
    bash -n scripts/stage-release-package-files.sh

## Interfaces and Dependencies

No public interface changes. No new command, option, source, payload field,
validation precedence, dependency, version bump, realtime feed, automatic
fallback, source mixing, ranking, scoring, recommendation, or trading action
is introduced.
