# Development guide

This document records the stable coding and validation rules for this
repository.

## Design rules

Keep the CLI easy to extend without mixing command logic, TradingView
internals, downstream workflow helpers, and JSON contract decisions.

Before adding a command, record why it belongs in the Rust CLI:

- which user, downstream, or operator workflow it unblocks
- whether it is old CLI migration parity, Rust-specific cleanup surface, or a
  new Rust-native capability
- what safety constraints apply
- what practical old CLI information must remain available
- how automated tests and live smoke will verify it

Do not implement a command only because it existed in the old JavaScript CLI.
Newly discovered old commands are migration backlog unless a durable decision
excludes them.

## Rust style

This project uses Rust 2024.

- Do not introduce `mod.rs`.
- Prefer facade files with same-named submodule directories for large
  capabilities.
- Keep top-level CLI package module declarations in `crates/cli/src/lib.rs`.
- Keep `crates/cli/src/cli.rs` focused on command and argument shape.
- Keep operation adapter implementations under `crates/cli/src/ops/` by
  capability.
- Put shared I/O-free command model logic in `crates/model/`. The
  `tradingview-model` crate owns validation, request interpretation, selector
  and target resolution, payload normalization/shaping, and fallback policy
  decisions. It must not depend on clap command enums, CDP runtime objects,
  HTTP clients, page-session execution, or UI automation. Accepted examples are
  `tradingview_model::watchlist`, `alert`, `replay`, `drawing`, and
  `screener`.
- Let `crates/cli/src/app/dispatch.rs` call `tradingview_model::*` directly for pure
  validation, request interpretation, target resolution, and payload shaping.
  Use `ops::*` from dispatch only for executable TradingView operations or
  adapter-specific request types. Do not re-export model helpers through
  `ops.rs` solely for dispatch convenience.
- When an operation adapter grows too large, split it behind a facade file and
  a same-named directory before creating a new workspace crate. `screener` is
  the current model: stable public adapter exports at the facade, sub-surface
  implementation modules underneath, and shared runtime/page-session helpers in
  a narrow common module.
- Prefer moving CDP-free input boundaries before runtime/storage/UI code.
  Screener is the larger example: validation, target resolution, and storage
  payload shaping live in `tradingview_model::screener`, while page-session storage
  fetch/save and UI operations remain in `ops/screener`.
- Storage-backed sub-surfaces are the next-best split candidates once
  validation is isolated. Screener columns live in
  `crates/cli/src/ops/screener/columns.rs`; Screener filters and screens now
  also own their operation bodies while shared open-state, storage fetch, click
  dispatch, and JavaScript helper expansion remain in `engine.rs`.
- Keep mixed page-session adapters split by user-visible sub-surface before
  extracting crates. Alert is the current model: list, normal create,
  indicator-alert create, delete, and public-safe payload normalization live
  under `crates/cli/src/ops/alert/`, while `alert.rs` preserves the adapter
  exports used by dispatch.
- Keep historical adapter names as facades when that avoids churn. Layout is
  now a facade over `crates/cli/src/ops/layout/watchlist.rs` and
  `crates/cli/src/ops/layout/pane.rs`; do not mix new watchlist and pane
  implementation bodies back into the facade file.
- Keep CDP-dependent Pine Editor operations in the CLI package, but split them
  by Editor sub-surface. `crates/cli/src/ops/pine/editor.rs` is now a facade
  over `runtime`, `source`, `scripts`, and `compile` modules. Desktop-free
  Pine static analysis and facade checks still belong in `crates/pine/`.
- Keep medium adapters behind the same facade pattern once they mix validation,
  reads, mutation, and payload shaping. Drawing, Replay, and chart-dependent
  Market now use same-named implementation directories under
  `crates/cli/src/ops/`. Do not gather new Drawing/Replay/Market operation
  bodies back into the facade files.
- Once an adapter split exposes CDP-free request interpretation or validation,
  move that logic into `crates/model/` if it is reusable and not tied
  to clap or live page state. Drawing is the request-boundary example:
  `tradingview_model::drawing` owns the request structs and position validation, while
  `ops/drawing` owns shape creation, entity post-checks, reads, and cleanup.
- Keep generic UI automation safety-aware. `crates/cli/src/ops/ui.rs` is a
  facade over `dom`, `input`, `selectors`, and `eval`; do not move the
  `TV_ALLOW_UNSAFE_UI_EVAL` gate out of the application safety/dispatch layer
  or hide new unsafe behavior inside the adapter.
- Keep `crates/cli/src/main.rs` as a thin process entrypoint. Put CLI parsing,
  command dispatch, JSON envelope output, stream loops, input conversion, and
  target connection orchestration under `crates/cli/src/app/`.
- Put reusable command logic and transport helpers in root library modules
  rather than adding binary-only code to `crates/cli/src/main.rs`.
- Put cross-crate contract types in `crates/core/` only when they are small,
  low-dependency, and broadly shared. Current examples are typed errors, JSON
  envelopes, and exit-code mapping.
- Put shared I/O-free request models, validation, normalization, target
  resolution, and public-safe payload shaping in `crates/model/`. The model
  crate may use `tradingview-core` and `serde_json`, but it must stay free of
  network, CDP, clap, and UI dependencies.
- Put credential-free, Desktop-free market reads in `crates/market/` when they
  do not depend on CDP, chart state, or UI automation.
- Put credential-free, Desktop-free scanner reads in `crates/scanner/` when
  they can be exercised without TradingView Desktop.
- Put Desktop-free Pine helpers in `crates/pine/` when they are local source
  analysis or Pine facade checks. Keep Pine Editor operations in the CLI
  package because they depend on CDP, Monaco, and visible TradingView UI state.
- Put shared TradingView Desktop CDP connection code in `crates/cdp/`. Do not
  duplicate target discovery, `RuntimeEvaluator`, screenshot/input event
  primitives, or target handoff helpers inside operation modules.
- Keep each library crate's `lib.rs` as a facade. When implementation grows,
  split into same-directory modules rather than gathering everything in
  `lib.rs`.
- Do not move chart-dependent market reads, Screener code, account mutation,
  or UI automation into another workspace crate merely because they are
  reusable in theory. Extract them only when a concrete follow-up plan proves
  the boundary and dependency set are useful.
- Before extracting more `ops` code, consult
  `docs/operation-adapter-boundaries.md`. Keep executable TradingView work in
  `ops` when it needs CDP/runtime access, page-session APIs, storage fetch/save,
  DOM/UI fallback, live chart state, or post-checks.
- Do not create a generic `ops` crate just to move files. Current `ops` modules
  are operation adapters inside the CLI package. Split large modules internally
  first, then extract domain-specific crates only when their dependency
  boundary is clear.
- Treat the workspace library crates as internal and unstable until a future
  plan explicitly defines a stable Rust API.
- Keep helpers as private as possible; use `pub(super)` for sibling operation
  modules when needed.
- Avoid unrelated cleanup while migrating commands or fixing behavior.

## JavaScript and TradingView safety

Many operations evaluate JavaScript through CDP. Treat user-provided strings as
data, not source code.

- Use JSON serialization helpers instead of hand-written quote escaping.
- Validate numeric inputs before embedding them in JavaScript or request
  payloads.
- Reject non-finite numeric input before connecting to CDP where possible.
- Centralize private TradingView API paths inside operation helpers.
- When TradingView internals change, report `internal_api_unavailable` rather
  than manufacturing a success payload.

Tracked docs must not contain live account-local identifiers or private
operational metadata. Scrub saved-script ids, saved-script names, alert ids,
layout ids, chart target ids, usernames, emails, account names, machine-local
paths, cookies, tokens, and raw live payloads unless they are intentionally
public example data.

## Testing

Operation unit tests should live next to the module they verify under
`#[cfg(test)]`. They must use fake runtime evaluators and must not require a
running TradingView Desktop.

CLI contract tests belong under `crates/cli/tests/cli_contract.rs`. They should
cover argument parsing, structured connection errors, validation errors, and
public command shape.

Live CDP smoke checks are useful but environment-dependent. Keep them separate
from automated tests and record meaningful results in the relevant ExecPlan or
note without account-local identifiers.

## Validation baseline

For code changes, run:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
git diff --check
```

For focused command work, also run the relevant module or contract tests. For
example:

```bash
cargo test screener -- --nocapture
cargo test -p tradingview-cli --test cli_contract screener -- --nocapture
```

For docs-only changes, at minimum run:

```bash
git diff --check
git grep -nE '(/Users/|C:\\|USER;|sessionid|cookie|authorization|bearer)' -- README.md CHANGELOG.md docs .agents/skills packaging scripts || true
```

If the grep finds only validation-command examples or public-safe policy
language, record that as acceptable. Remove any new local path, account id,
credential, or raw live payload before committing.

## Optional local hooks

Git 2.54 config-based hooks are available as optional local guardrails.

Install with `mise`:

```bash
mise run hooks:install
```

Or run the platform script directly:

```bash
scripts/install-config-hooks.sh
```

On Windows:

```powershell
./scripts/install-config-hooks.ps1
```

These hooks are convenience checks. They do not replace the validation baseline
or GitHub Actions.

## Commits

Use Conventional Commits with sentence-case subjects.

Keep command migration, refactors, documentation cleanup, release packaging,
and downstream workflow changes in separate commits unless they are inseparable
for one behavior.

Never push unless the user explicitly asks in the current turn.

## ExecPlans

Use an ExecPlan for complex features and significant refactors. Keep the plan
current while implementing, and record discoveries or changed decisions there
rather than leaving them only in chat history.
