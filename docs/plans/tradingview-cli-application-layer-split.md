# CLI application layer split

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This plan follows `.agents/PLANS.md` from the repository root. It is self-contained so a future contributor can continue from this file without relying on chat history.

## Purpose / Big Picture

The `tv` binary previously kept process startup, command-line parsing, JSON envelope printing, target connection, command dispatch, stream looping, and Pine source input handling in one large `src/main.rs`. After this change, the binary entrypoint is thin and the root library owns an application layer under `src/app/`.

Users should see no command behavior change. The observable proof is that help, validation errors, JSON output, JSONL stream output, Desktop-free symbol reads, Pine static reads, and scanner reads still work while `src/main.rs` becomes a small process wrapper.

## Progress

- [x] (2026-04-28T07:05Z) Confirmed the working tree was clean and inspected `src/main.rs`, current workspace crates, and architecture docs.
- [x] (2026-04-28T07:05Z) Created this ExecPlan and archived the completed workspace crate split phase 3 plan.
- [x] (2026-04-28T07:05Z) Moved CLI runner, dispatch, runtime connection, input, output, stream, and safety helpers into `src/app/`.
- [x] (2026-04-28T07:05Z) Reduced `src/main.rs` to the thin process entrypoint and Windows stack-size wrapper.
- [x] (2026-04-28T07:07Z) Updated architecture, development, roadmap, changelog, and continuity docs.
- [x] (2026-04-28T07:10Z) Ran validation and behavior smoke checks.
- [ ] Commit related changes.

## Surprises & Discoveries

- Observation: The first application-layer split compiled without changing the command dispatch match.
  Evidence: `cargo check --workspace` completed successfully after moving the application code into `src/app/`.

- Observation: `src/main.rs` dropped to 26 lines, while dispatch remains large at 1111 lines.
  Evidence: `wc -l src/main.rs src/app/*.rs` showed `src/main.rs` at 26 lines and `src/app/dispatch.rs` at 1111 lines. This confirms the binary/application boundary is now clean, but domain-oriented dispatch splitting remains a future task.

## Decision Log

- Decision: Keep `src/app/dispatch.rs` dependent on the existing `cli::Command` enum for this phase.
  Rationale: The goal is to split binary startup from application orchestration without changing the CLI contract. Introducing separate domain request types would be useful later but would make this behavior-preserving refactor much larger.
  Date/Author: 2026-04-28 / Codex.

- Decision: Keep domain operations in `src/ops/` for this phase.
  Rationale: The operation modules already encode TradingView-specific behavior and tests. Moving them while also splitting `main.rs` would mix two refactor axes and make regressions harder to localize.
  Date/Author: 2026-04-28 / Codex.

## Outcomes & Retrospective

Not completed yet.
The `tv` binary is now a thin process wrapper, and library-owned application modules handle CLI parsing, command dispatch, runtime connection, input conversion, output envelopes, stream looping, and unsafe command gating. CLI behavior is unchanged.

Validation passed: `cargo fmt --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo test --workspace`, `cargo test --test cli_contract -- --nocapture`, `cargo metadata --no-deps --format-version 1`, and `git diff --check`. Behavior smoke confirmed `tv --help`, Desktop-free `info` and `quote` reads including `TV_CDP_PORT=9`, Pine analyze, and scanner scan.

## Context and Orientation

The repository is a Cargo workspace. The root package is `tradingview-cli`, and the installed binary is `tv`. Internal support crates under `crates/` own shared contracts and Desktop-free read or analysis surfaces. The root crate still owns CDP connection, CLI command definitions, and operation modules.

`src/cli.rs` declares the clap command surface. `src/ops/` contains TradingView operation implementations. `src/cdp.rs` and `src/transport.rs` own CDP evaluation and target selection. This plan adds `src/app.rs` and `src/app/` as the application layer between the thin binary entrypoint and those lower layers.

## Plan of Work

Move process-independent application code out of `src/main.rs`. `src/main.rs` should only call `tradingview_cli::app::run_cli()` and keep the Windows larger-stack wrapper. `src/app/runner.rs` should parse clap, build `TransportConfig`, route stream commands, and wrap normal command results in success or error envelopes. `src/app/dispatch.rs` should contain the existing command match and keep behavior unchanged. `src/app/runtime.rs`, `src/app/input.rs`, `src/app/output.rs`, `src/app/stream.rs`, and `src/app/safety.rs` should hold the helpers that were previously private to `main.rs`.

Add `pub mod app;` to `src/lib.rs`. Keep `src/ops/` and workspace support crates in place.

Update docs to record the new `binary -> app -> cli/ops/crates` layering and to warn future contributors not to put dispatch or operation logic back into `src/main.rs`.

## Concrete Steps

Work from the repository root.

1. Move the completed phase 3 crate split plan into `docs/plans/archives/`.
2. Create `src/app.rs` and `src/app/` modules for runner, dispatch, runtime, input, output, stream, and safety.
3. Replace `src/main.rs` with a thin entrypoint that calls `tradingview_cli::app::run_cli()`.
4. Add `pub mod app;` to `src/lib.rs`.
5. Update docs and continuity.
6. Run validation:

       cargo fmt --check
       cargo clippy --workspace --all-targets --all-features -- -D warnings
       cargo test --workspace
       cargo test --test cli_contract -- --nocapture
       cargo metadata --no-deps --format-version 1
       git diff --check

7. Run behavior smoke:

       target/debug/tv --help
       target/debug/tv info NYSE:IONQ
       target/debug/tv quote NYSE:IONQ
       TV_CDP_PORT=9 target/debug/tv info NYSE:IONQ
       TV_CDP_PORT=9 target/debug/tv quote NYSE:IONQ
       target/debug/tv pine analyze --file <test pine file>
       target/debug/tv scanner scan --limit 3

8. Commit with:

       refactor(cli): Split application runner layer

## Validation and Acceptance

The change is accepted when the validation commands pass and behavior smoke shows that the same CLI commands still produce the expected output. Help and version output must remain plain clap output. Validation errors must still produce JSON error envelopes with the same command name and exit code. `stream` must still emit JSONL envelopes rather than pretty JSON.

## Idempotence and Recovery

This refactor is safe to repeat. If imports fail, search for the helper name and either make it visible within `src/app/` or import it from the correct app submodule. If any CLI contract test changes, stop and restore the old JSON shape before continuing.

Do not move operation implementations out of `src/ops/` in this phase.

## Artifacts and Notes

Do not paste machine-specific absolute paths, live target ids, cookies, tokens, account-local identifiers, or raw account payloads into repository docs. Terminal evidence should be short and scrubbed.

## Interfaces and Dependencies

At completion, `src/app.rs` must expose:

    pub fn run_cli() -> std::process::ExitCode
    pub fn startup_error(message: impl Into<String>) -> std::process::ExitCode

`src/main.rs` should call only those public app functions plus the standard Windows thread wrapper. Application submodules may use `crate::cli`, `crate::ops`, `crate::transport`, and `crate::cdp`, but `src/main.rs` should not.

## Open Questions

No critical open questions block this phase. After this split is stable, a later plan can reduce `src/app/dispatch.rs` by grouping command-family dispatch into smaller modules or by introducing domain request types.
