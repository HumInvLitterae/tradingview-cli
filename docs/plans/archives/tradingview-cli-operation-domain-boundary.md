# Operation adapter / domain boundary

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This plan follows `.agents/PLANS.md` in this repository. It is self-contained and describes a behavior-preserving refactor that introduces a small domain/service boundary inside the CLI package.

## Purpose / Big Picture

The `tv` CLI already has a thin binary, an application runner, and several internal support crates. The remaining `ops` modules are operation adapters: they translate CLI commands into TradingView operations and preserve the JSON payloads users already depend on. After the recent facade splits, the next useful improvement is not another surface-level file split. It is to move reusable command logic out of `ops` and into a `domain` layer that does not depend on clap command enums or CDP runtime objects.

This first domain-boundary slice uses watchlist logic as the pilot. After completion, users should see no command behavior change, but maintainers can see the new boundary by running focused watchlist tests. The observable CLI checks are still `tv watchlist ...` validation and structured connection errors with a bad CDP port.

## Progress

- [x] (2026-04-29) Inspected `layout/watchlist.rs` and confirmed it mixes validation, bulk aggregation, API payload normalization, DOM fallback, and CDP runtime work.
- [x] (2026-04-29) Added a CLI-package `domain` facade and a first `domain::watchlist` module.
- [x] (2026-04-29) Moved CDP-free watchlist validation, symbol normalization, bulk aggregation helpers, API payload normalization, fallback policy, and remove payload normalization into `domain::watchlist`.
- [x] (2026-04-29) Kept CDP RuntimeEvaluator usage, JavaScript expressions, DOM fallback, panel opening, key input, and post-check execution in `ops/layout/watchlist.rs`.
- [x] (2026-04-29) Archived the completed UI adapter split plan and created this plan.
- [x] (2026-04-29) Updated stable docs and continuity ledger.
- [x] (2026-04-29) Ran validation, behavior smoke, and hygiene checks.
- [ ] Commit the related changes as one refactor.

## Surprises & Discoveries

- Observation: The first practical boundary is smaller than an entire watchlist service crate.
  Evidence: `watchlist_add_via_api` still builds and evaluates page-session JavaScript through `RuntimeEvaluator`, so moving the JavaScript execution now would blur the new domain layer with CDP/page-session infrastructure. This slice only moves pure validation, aggregation, and payload normalization.
- Observation: The watchlist bulk-add CLI takes symbols as positional arguments, not through a `--symbols` option.
  Evidence: `target/debug/tv watchlist add-bulk --symbols ""` returns a clap usage error. The intended validation smoke is `target/debug/tv watchlist add-bulk ""`, which returns `Symbol must not be empty` before CDP connection.

## Decision Log

- Decision: Add `crates/cli/src/domain.rs` and `crates/cli/src/domain/watchlist.rs` inside the existing CLI package, not a new workspace crate.
  Rationale: The watchlist boundary is still being proven. Keeping it inside the CLI package avoids prematurely promising a stable Rust API while still separating reusable logic from operation adapters.
  Date/Author: 2026-04-29 / Codex.
- Decision: Keep the `layout` operation facade and watchlist public exports unchanged.
  Rationale: Dispatch and CLI contract tests already depend on the current adapter surface. This is an internal refactor and must preserve command behavior.
  Date/Author: 2026-04-29 / Codex.
- Decision: Leave API-backed mutation JavaScript execution in `ops/layout/watchlist.rs`.
  Rationale: That code depends on `RuntimeEvaluator` and logged-in page-session fetch behavior. The new domain module should not depend on CDP runtime objects in this slice.
  Date/Author: 2026-04-29 / Codex.

## Outcomes & Retrospective

At completion, `domain::watchlist` should own the watchlist input and payload boundary that can be tested without a fake CDP runtime. `ops/layout/watchlist.rs` should remain the command adapter that performs CDP evaluation, DOM fallback, key input, and post-checks. No public CLI behavior should change.

## Context and Orientation

The repository is a Rust workspace. The `tv` binary and command adapters live in the `tradingview-cli` package under `crates/cli/`.

The term "operation adapter" means a module called by application dispatch to perform a user-visible CLI command. Operation adapters preserve the command's JSON payload shape and decide how to call TradingView. The term "domain layer" means a module containing reusable command logic that does not depend on clap command enums or a live CDP runtime. In this slice, the domain layer may still use `serde_json::Value` and `tradingview_core::AppError` because the existing CLI payload contract is JSON-shaped.

The relevant files are:

- `crates/cli/src/lib.rs`, which exposes top-level library modules.
- `crates/cli/src/domain.rs`, the new domain facade.
- `crates/cli/src/domain/watchlist.rs`, the new watchlist domain module.
- `crates/cli/src/ops/layout/watchlist.rs`, the existing watchlist operation adapter.
- `docs/plans/README.md`, `docs/architecture.md`, `docs/development.md`, `docs/v0.3-roadmap.md`, and `CHANGELOG.md`, which record durable project structure.

Rust 2024 is used in this repository. Do not introduce `mod.rs`.

## Plan of Work

Add `pub mod domain;` to `crates/cli/src/lib.rs`. Create `crates/cli/src/domain.rs` as a facade that exposes `watchlist`.

Move CDP-free watchlist helpers from `crates/cli/src/ops/layout/watchlist.rs` into `crates/cli/src/domain/watchlist.rs`. The moved functions are symbol normalization, add-bulk validation, unique symbol counting, add-bulk result aggregation, watchlist API payload normalization, API fallback policy, and remove payload normalization. Keep these helpers independent from `RuntimeEvaluator`.

Keep `ops/layout/watchlist.rs` as the adapter. It imports the domain helpers, re-exports `validate_watchlist_add_bulk_request` for existing dispatch imports, and still owns `watchlist_get`, `watchlist_add`, `watchlist_add_bulk`, `watchlist_add_via_api`, `watchlist_remove_via_api`, `watchlist_remove`, `ensure_watchlist_panel_open`, `wait_after_panel_open`, `dispatch_key`, and all page-session JavaScript.

Update docs to describe the new layer as a pilot boundary, not a stable public Rust API. Update the local `CONTINUITY.md` ledger but do not commit it.

## Concrete Steps

Run all commands from the repository root.

After implementation, run focused tests:

    cargo test -p tradingview-cli domain::watchlist -- --nocapture
    cargo test -p tradingview-cli layout::watchlist -- --nocapture
    cargo test -p tradingview-cli --test cli_contract watchlist -- --nocapture

Run full validation:

    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1
    git diff --check

Run behavior smoke:

    target/debug/tv watchlist --help
    target/debug/tv watchlist add ""
    target/debug/tv watchlist add-bulk ""
    target/debug/tv watchlist remove ""
    TV_CDP_PORT=9 target/debug/tv watchlist get
    TV_CDP_PORT=9 target/debug/tv watchlist add NASDAQ:AAPL

Run hygiene:

    rg -n '(/Users/|C:\\|USER;|sessionid|cookie|authorization|bearer)' README.md CHANGELOG.md docs .agents/skills packaging scripts || true

Commit:

    git add crates/cli/src/lib.rs crates/cli/src/domain.rs crates/cli/src/domain crates/cli/src/ops/layout/watchlist.rs docs CHANGELOG.md
    git commit -m "refactor(domain): Introduce watchlist service boundary"

## Validation and Acceptance

The change is accepted when all tests pass and the smoke checks preserve existing behavior:

- invalid watchlist symbols fail validation before CDP connection;
- bad CDP port watchlist reads and mutations return structured connection errors;
- `watchlist add-bulk` still reports duplicate, added, already-present, failed, and partial results in the same JSON shape;
- `watchlist add` and `watchlist remove` still prefer API-backed mutation and fall back to DOM only when the API error explicitly allows fallback;
- `ops/layout/watchlist.rs` no longer owns CDP-free validation and payload normalization tests.

## Idempotence and Recovery

This is a behavior-preserving refactor. If a test fails, compare the JSON field names and counts against the pre-refactor tests before changing behavior. Keep page-session JavaScript in the adapter unless a helper can be tested without `RuntimeEvaluator`. If a moved helper needs CLI command types or CDP runtime objects, move it back to the adapter and record the reason in this plan.

## Artifacts and Notes

Initial structural evidence:

    find crates/cli/src/ops -maxdepth 2 -type f -name '*.rs' -print | xargs wc -l | sort -n | tail
    result: layout/watchlist.rs is still one of the largest operation files and mixes API/DOM behavior with validation and aggregation.

Validation evidence:

    cargo test -p tradingview-cli domain::watchlist -- --nocapture
    result: 7 passed; 0 failed

    cargo test -p tradingview-cli layout::watchlist -- --nocapture
    result: 19 passed; 0 failed

    cargo test -p tradingview-cli --test cli_contract watchlist -- --nocapture
    result: 4 passed; 0 failed

    cargo fmt --check
    result: passed

    cargo clippy --workspace --all-targets --all-features -- -D warnings
    result: passed

    cargo test --workspace
    result: passed

    cargo metadata --no-deps --format-version 1
    result: passed

Behavior smoke evidence:

    target/debug/tv watchlist --help
    result: passed

    target/debug/tv watchlist add ""
    result: validation error before CDP connection, exit 1

    target/debug/tv watchlist add-bulk ""
    result: validation error before CDP connection, exit 1

    target/debug/tv watchlist remove ""
    result: validation error before CDP connection, exit 1

    TV_CDP_PORT=9 target/debug/tv watchlist get
    result: structured connection error, exit 2

    TV_CDP_PORT=9 target/debug/tv watchlist add NASDAQ:AAPL
    result: structured connection error, exit 2

Final hygiene evidence:

    git diff --check
    result: passed

    rg -n '(/Users/|C:\\|USER;|sessionid|cookie|authorization|bearer)' README.md CHANGELOG.md docs .agents/skills packaging scripts || true
    result: only existing policy text and validation-command examples, including this plan's hygiene command

## Interfaces and Dependencies

At the end of this slice, these interfaces must exist:

    pub mod domain;

    // in crates/cli/src/domain.rs
    pub mod watchlist;

    // in crates/cli/src/domain/watchlist.rs
    pub const MAX_WATCHLIST_BULK_SYMBOLS: usize;
    pub const MAX_WATCHLIST_BULK_DELAY_MS: u64;
    pub struct WatchlistBulkAccumulator;
    pub fn normalize_watchlist_symbol(symbol: &str) -> Result<String, AppError>;
    pub fn unique_watchlist_symbol_count(symbols: &[String]) -> Result<usize, AppError>;
    pub fn validate_watchlist_add_bulk_request(symbols: &[String], delay_ms: u64) -> Result<(), AppError>;
    pub fn normalize_watchlist_api_payload(data: Value) -> Result<Value, AppError>;
    pub fn watchlist_api_error_allows_fallback(error: &AppError) -> bool;
    pub fn normalize_watchlist_remove_payload(data: Value) -> Result<Value, AppError>;

No new external dependencies should be added.

## Open Questions

No critical question blocks this slice. Whether watchlist should later become a separate workspace crate is intentionally deferred until this in-package domain boundary has proved useful.
