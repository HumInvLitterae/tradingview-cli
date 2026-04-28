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
- When an operation adapter grows too large, split it behind a facade file and
  a same-named directory before creating a new workspace crate. `screener` is
  the current model: stable public adapter exports at the facade, sub-surface
  implementation modules underneath, and shared runtime/page-session helpers in
  a narrow common module.
- Prefer moving CDP-free input boundaries before runtime/storage/UI code. For
  example, Screener validation lives in
  `crates/cli/src/ops/screener/validation.rs` before columns or saved-screen
  storage logic are split out of the implementation engine.
- Storage-backed sub-surfaces are the next-best split candidates once
  validation is isolated. Screener columns live in
  `crates/cli/src/ops/screener/columns.rs`; Screener filters and screens now
  also own their operation bodies while shared open-state, storage fetch, click
  dispatch, and JavaScript helper expansion remain in `engine.rs`.
- Keep `crates/cli/src/main.rs` as a thin process entrypoint. Put CLI parsing,
  command dispatch, JSON envelope output, stream loops, input conversion, and
  target connection orchestration under `crates/cli/src/app/`.
- Put reusable command logic and transport helpers in root library modules
  rather than adding binary-only code to `crates/cli/src/main.rs`.
- Put cross-crate contract types in `crates/core/` only when they are small,
  low-dependency, and broadly shared. Current examples are typed errors, JSON
  envelopes, and exit-code mapping.
- Put credential-free, Desktop-free market reads in `crates/market/` when they
  do not depend on CDP, chart state, or UI automation.
- Put credential-free, Desktop-free scanner reads in `crates/scanner/` when
  they can be exercised without TradingView Desktop.
- Put Desktop-free Pine helpers in `crates/pine/` when they are local source
  analysis or Pine facade checks. Keep Pine Editor operations in the root crate.
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
