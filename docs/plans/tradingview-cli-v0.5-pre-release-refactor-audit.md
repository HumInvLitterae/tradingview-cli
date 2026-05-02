# v0.5 pre-release refactor audit

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document follows `.agents/PLANS.md` from the repository root. It is self-contained so a new contributor can understand the purpose, implementation boundary, and validation without prior chat context.

## Purpose / Big Picture

Before preparing the `v0.5.0` release, this slice pauses feature work and checks whether the newest market-data code is structurally safe to release. The expected user-visible behavior is deliberately unchanged: `tv fundamentals <SYMBOL>`, `--field`, and `--group` must keep returning the same JSON payloads and errors. The value of this work is release confidence: if there is a small low-risk cleanup that reduces maintenance risk, do it now; defer large refactors until after the release.

## Progress

- [x] (2026-05-02 06:05Z) Confirmed the working tree was clean and reviewed the largest Rust files plus `allow`, `TODO`, `FIXME`, `panic!`, `unimplemented!`, and `todo!` occurrences.
- [x] (2026-05-02 06:05Z) Decided there is no release-blocking structural issue, but `crates/market/src/fundamentals.rs` is a good low-risk pre-release cleanup because it mixes field lists, scanner request construction, and response normalization.
- [x] (2026-05-02 06:10Z) Split `tradingview-market` fundamentals internals into `fields`, `client`, and `normalize` modules while preserving the existing public API and CLI payload shape.
- [x] (2026-05-02 06:15Z) Kept `crates/market/src/fundamentals.rs` as the facade file instead of introducing `mod.rs`, matching the repository Rust style.
- [x] (2026-05-02 06:20Z) Ran focused fundamentals tests and CLI contract tests.
- [x] (2026-05-02 06:35Z) Ran full workspace validation, smoke checks, metadata, diff, and hygiene checks.
- [x] (2026-05-02 06:45Z) Committed the related changes as `refactor(market): Split fundamentals internals`.

## Surprises & Discoveries

- Observation: The release-blocker scan did not reveal a new must-fix code smell. Existing `allow` attributes are largely established compatibility or test-support allowances, and the current largest files are mostly operation adapters intentionally deferred from this slice.
  Evidence: `rg -n 'allow\\(|TODO|FIXME|panic!|unimplemented!|todo!' crates` did not show a new fundamentals-related suppression or unfinished implementation marker.

## Decision Log

- Decision: Do not split `crates/cli/src/app/dispatch.rs` or `crates/cli/src/cli.rs` before `v0.5.0`.
  Rationale: Those files are large, but splitting command dispatch or the clap command surface is a broader risk than this release-preparation slice needs. They are not showing a concrete contract bug.
  Date/Author: 2026-05-02 / Codex

- Decision: Split only `tradingview-market` fundamentals internals.
  Rationale: `fundamentals` recently gained field groups and is the freshest area of churn. Separating field selection, scanner request construction, and response normalization makes the module easier to review without changing public behavior.
  Date/Author: 2026-05-02 / Codex

- Decision: Keep `tradingview-market` public functions unchanged.
  Rationale: The release audit must not change CLI behavior or Rust caller compatibility. Existing callers continue to use `fundamentals_symbol`, `fundamentals_symbol_with_groups`, `fundamentals_symbol_typed`, and `fundamentals_symbol_with_groups_typed`.
  Date/Author: 2026-05-02 / Codex

## Outcomes & Retrospective

Completed. The release audit found no must-fix structural blocker, and the only code change was a behavior-preserving `tradingview-market` fundamentals split. `tv fundamentals` payloads and validation behavior remain compatible, so the next natural step is `v0.5.0` release readiness.

## Context and Orientation

The repository is a Rust workspace. The `tv` binary and command dispatch live in `crates/cli`. Desktop-free market reads live in `crates/market`, whose public exports are re-exported from `crates/market/src/lib.rs`.

`fundamentals` means the Desktop-free single-symbol scanner read used by `tv fundamentals <SYMBOL>`. It is not a financial-statement parser and it does not connect to TradingView Desktop. It asks the TradingView scanner REST endpoint for selected fields and returns those values under `field_values`.

Before this slice, `crates/market/src/fundamentals.rs` contained three kinds of code in one file:

- field constants, group definitions, and field selection validation;
- scanner HTTP request construction;
- scanner response normalization into the typed `Fundamentals` struct and CLI-compatible JSON.

This slice keeps the same module name from callers' perspective but changes its internal file layout to:

- `crates/market/src/fundamentals.rs`: public facade and symbol-search candidate enrichment;
- `crates/market/src/fundamentals/fields.rs`: default fields, supported fields, group expansion, and selection validation;
- `crates/market/src/fundamentals/client.rs`: scanner request construction and HTTP response reading;
- `crates/market/src/fundamentals/normalize.rs`: scanner response normalization and normalization tests.

## Plan of Work

First archive the completed fundamentals field-group ExecPlan under `docs/plans/archives/` and create this pre-release refactor audit plan as the new current plan.

Next keep `crates/market/src/fundamentals.rs` as the facade file and create `fields.rs`, `client.rs`, and `normalize.rs` under `crates/market/src/fundamentals/`. This follows the repository rule to avoid `mod.rs` while still allowing `mod fundamentals;` in `crates/market/src/lib.rs` to work. Move code without changing the signatures of the public functions re-exported by `crates/market/src/lib.rs`.

Then update stable docs only where they help future contributors understand the boundary. `docs/v0.5-roadmap.md` should say this release-preparation slice is a pre-release refactor audit and that release readiness is next. `docs/development.md` should record the lightweight rule that Desktop-free read crates should split field selection, request construction, and normalization when a read surface grows. `CHANGELOG.md` should include an internal refactor note only because this slice changes code.

Finally run focused tests, full validation, and read-only smoke commands. If any validation shows a behavior change in `tv fundamentals`, revert or adjust the refactor so payloads remain compatible.

## Concrete Steps

Work from the repository root.

Inspect the largest files and unfinished-code markers:

    find crates -path '*/src/*.rs' -o -path '*/src/**/*.rs' | xargs wc -l | sort -nr | head -40
    rg -n 'allow\\(|TODO|FIXME|panic!|unimplemented!|todo!' crates

Split the fundamentals module as described above. After editing, run:

    cargo test -p tradingview-market fundamental -- --nocapture
    cargo test -p tradingview-cli --test cli_contract fundamentals -- --nocapture

Run the smoke checks:

    target/debug/tv fundamentals NYSE:IONQ
    target/debug/tv fundamentals NYSE:IONQ --group earnings
    target/debug/tv fundamentals AAPL --group valuation --group dividends
    target/debug/tv fundamentals NYSE:IONQ --group banana

The first three commands should preserve the existing successful scanner-backed fundamentals payload shape. The invalid group command should fail with a structured validation error before any Desktop/CDP connection is needed.

Then run the full validation:

    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1
    git diff --check
    rg -n '(/Users/|C:\\|USER;|sessionid|cookie|authorization|bearer)' README.md CHANGELOG.md docs .agents/skills packaging scripts || true

## Validation and Acceptance

Acceptance is met when the following are true:

- The public `tradingview-market` fundamentals API names and signatures are unchanged.
- `tv fundamentals <SYMBOL>`, `tv fundamentals <SYMBOL> --group earnings`, and explicit `--field` / `--group` combinations keep their JSON payload shape.
- Invalid groups and fields still return validation errors with supported values.
- Full workspace tests and clippy pass.
- No raw live payloads, local absolute paths, credentials, cookies, tokens, or authorization values are added to tracked docs.

## Idempotence and Recovery

This refactor is file movement plus behavior-preserving code extraction. Re-running tests and smoke commands is safe. If a module split causes an import cycle or visibility issue, keep the public facade in `crates/market/src/fundamentals.rs`, move only private helper code into child modules, and rerun focused tests before continuing. If a CLI payload changes, prefer moving serialization-sensitive code back into `normalize.rs` unchanged rather than adjusting tests to match a new shape.

## Artifacts and Notes

The initial audit found `crates/market/src/fundamentals.rs` at roughly 650 lines after field groups were added. Other larger files remain known operation adapters or command-surface files and are deferred because they do not present a concrete release blocker for `v0.5.0`.

## Interfaces and Dependencies

The following public functions must remain available from `tradingview-market`:

    pub async fn fundamentals_symbol(symbol: &str, fields: Vec<String>) -> Result<Value, AppError>
    pub async fn fundamentals_symbol_with_groups(symbol: &str, groups: Vec<String>, fields: Vec<String>) -> Result<Value, AppError>
    pub async fn fundamentals_symbol_typed(symbol: &str, fields: Vec<String>) -> Result<Fundamentals, AppError>
    pub async fn fundamentals_symbol_with_groups_typed(symbol: &str, groups: Vec<String>, fields: Vec<String>) -> Result<Fundamentals, AppError>

The child modules should remain private implementation details. Do not expose `fields`, `client`, or `normalize` from `crates/market/src/lib.rs`.

## Open Questions

None. If validation reveals a behavior change, treat it as a refactor bug and preserve the existing behavior.
