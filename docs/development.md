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
- Keep top-level module declarations in `src/lib.rs`.
- Keep `src/main.rs` focused on binary startup, dispatch, envelopes, and exit
  behavior.
- Keep `src/cli.rs` focused on command and argument shape.
- Keep operation implementations under `src/ops/` by capability.
- Put reusable command logic and transport helpers in root library modules
  rather than adding binary-only code to `src/main.rs`.
- Put cross-crate contract types in `crates/core/` only when they are small,
  low-dependency, and broadly shared. Current examples are typed errors, JSON
  envelopes, and exit-code mapping.
- Do not move operation logic, CDP clients, market reads, scanner code, or
  Screener code into another workspace crate merely because they are reusable
  in theory. Extract them only when a concrete follow-up plan proves the
  boundary and dependency set are useful.
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

CLI contract tests belong under `tests/cli_contract.rs`. They should cover
argument parsing, structured connection errors, validation errors, and public
command shape.

Live CDP smoke checks are useful but environment-dependent. Keep them separate
from automated tests and record meaningful results in the relevant ExecPlan or
note without account-local identifiers.

## Validation baseline

For code changes, run:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
git diff --check
```

For focused command work, also run the relevant module or contract tests. For
example:

```bash
cargo test screener -- --nocapture
cargo test --test cli_contract screener -- --nocapture
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
