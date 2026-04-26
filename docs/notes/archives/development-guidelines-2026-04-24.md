# Development guidelines 2026-04-24

This note records repo-local development and design guidelines for the Rust-native `tv` CLI. Use it before adding old CLI surface area or doing broad refactors.

The goal is simple: keep the CLI easy to extend without letting command logic, TradingView internals, downstream workflow helpers, and JSON contract decisions blur together.

## Architecture boundaries

The project is a CLI-first Rust binary named `tv`. It is not an MCP server, and downstream integration should use ordinary process invocation plus structured JSON output.

Keep responsibilities separated:

- `src/main.rs` owns program startup, CLI dispatch, runtime connection setup, success/error envelope printing, and exit codes. Do not grow command implementation logic there.
- `src/cli.rs` owns the `clap` command surface, argument definitions, and command names. Avoid placing validation or TradingView behavior there unless it is pure argument shape.
- `src/ops.rs` is a thin facade that declares operation modules and re-exports operation functions used by `src/main.rs`.
- `src/ops/` contains operation implementations grouped by capability, such as chart, market, diagnostics, data, layout, screenshot, and status.
- `src/cdp.rs` owns Chrome DevTools Protocol evaluation and screenshot primitives.
- `src/transport.rs` owns TradingView CDP target discovery and connection setup.
- `src/output.rs` owns JSON success and error envelopes.
- `src/error.rs` owns typed application errors and exit-code mapping.

If new code does not clearly belong to one of those areas, stop and record the placement decision before implementing it.

## Module layout rules

This project uses Rust 2024. Do not introduce `mod.rs`. Prefer a facade file plus a same-named directory for submodules, as with `src/ops.rs` and `src/ops/`.

Add a new operation to the capability module that matches the user-visible surface, not to a generic catch-all. For example, chart state and chart mutation belong in `src/ops/chart.rs`; quotes, OHLCV, and symbol search belong in `src/ops/market.rs`; diagnostic reads belong in `src/ops/diagnostics.rs`.

Do not reintroduce a monolithic operation file. If a capability module becomes hard to scan, split it by sub-surface before adding another command. `src/ops/data.rs` is already the main watch point; the next substantial data-related change should consider smaller submodules such as strategy data, drawing-derived data, and indicator reads.

Shared helpers should stay as private as possible. Use `pub(super)` for sibling operation modules and avoid making helpers `pub(crate)` unless another top-level module truly needs them. Keep the public operation facade limited to functions that `src/main.rs` dispatches or intentionally exposes for tests.

## CLI and JSON contract rules

The Rust CLI intentionally uses the structured envelope `{ success, command, data }` for successful output and `{ success, command, error }` for failures. Do not move command payload fields back to the top level to mimic the old JavaScript CLI.

For migrated commands, preserve practical information compatibility with the old CLI. Field names may change when documented, and the envelope is allowed to differ, but useful old information must remain available under `data` unless a durable project decision accepts the loss.

Any change to public CLI behavior, arguments, exit codes, payload fields, or error shape must update the relevant note under `docs/notes/`, especially `docs/notes/rust-cli-contract-migration-2026-04-24.md` when JSON contract changes are involved.

Do not describe unimplemented old commands as non-goals unless the repository has an explicit decision excluding them. Ordinary missing old commands remain migration backlog or deferred migration surface.

## TradingView and JavaScript safety

TradingView operations often evaluate JavaScript through CDP. Treat every user-provided string as data, not source code. Use JSON serialization helpers such as the existing JavaScript string helper instead of hand-written quote escaping at call sites.

Validate numeric inputs before embedding them in JavaScript. Use existing finite-number validation helpers where available, and reject non-finite values before evaluating JavaScript.

Keep internal TradingView API paths centralized in operation helpers. When an operation depends on DOM selectors or private TradingView internals, document that dependency in the plan or note for the slice and add tests for failure handling where possible.

## Testing and validation style

Operation unit tests should live next to the module they verify under `#[cfg(test)]`. They must use fake runtime evaluators and must not require a running TradingView Desktop.

CLI contract tests belong under `tests/cli_contract.rs`. They should cover argument parsing, structured connection errors, validation errors, and public command shape.

Live CDP smoke checks are useful but environment-dependent. Keep them separate from automated tests, and record meaningful smoke results in the relevant ExecPlan or note.

When recording live smoke results, preserve the behavior evidence but scrub operator-specific metadata. Do not write real TradingView saved-script ids, saved-script names, alert ids, layout ids, chart target ids, usernames, account names, emails, machine-local paths, or other account-local identifiers into tracked files unless they are intentionally public example data. Prefer placeholders such as `<saved script name>`, `<account-local-script-id>`, `<alert-id>`, `<layout-id>`, and `<target-id>`.

Before committing public-release docs or smoke notes, run targeted scans for the concrete values observed during smoke in addition to the usual generic secret scan. If a sensitive or account-local value lands in git history before the repository is public, scrub it from current files and rewrite local history before pushing.

The default validation baseline before committing code changes is:

    cargo fmt --check
    cargo clippy --all-targets --all-features
    cargo test
    git diff --check

Git 2.54 config-based hooks are available as optional local guardrails. Install
them with `mise run hooks:install`, `scripts/install-config-hooks.sh`, or
`scripts/install-config-hooks.ps1`. These hooks are convenience checks, not a
replacement for the validation baseline above or GitHub Actions.

For docs-only changes, at minimum run:

    git diff --check
    git grep with the repository's standard local-absolute-path pattern

Do not write machine-specific absolute filesystem paths into tracked docs.

## Change and commit discipline

Keep command migrations, refactors, documentation cleanup, and downstream workflow work in separate commits unless they are inseparable for a single behavior. Use Conventional Commits with sentence-case subjects.

Do not implement a command just because it existed in the old JavaScript CLI. First record why it belongs in this Rust CLI, which downstream or operator workflow it unblocks, what safety constraints apply, what information compatibility requires, and how it will be verified.

Keep downstream workflow helpers out of the core CLI unless investigation shows they are required CLI migration surface. Repo-local skills and downstream adapters may guide design, but they should not become hidden requirements inside the binary.

Use an ExecPlan for complex features and significant refactors. Keep the plan current while implementing, and record discoveries or changed decisions in the plan rather than leaving them in chat history.
