# Library crate boundary

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This plan follows `.agents/PLANS.md` from the repository root. It is self-contained so a future contributor can continue from this file without relying on chat history.

## Purpose / Big Picture

The `tv` binary is now broad enough that keeping every module rooted in `src/main.rs` makes the codebase harder to reuse and harder to split later. After this change, the package will have both a library crate and the existing `tv` binary. Users should see no behavior change: `tv --help`, JSON envelopes, exit codes, direct HTTP reads, and CDP commands should work as before. The observable proof is that `cargo metadata` shows a library target and a `tv` binary target, while the existing CLI contract tests still pass.

This first slice is intentionally mechanical. It creates the library boundary without redesigning command dispatch, changing module names, or declaring a stable public Rust API.

## Progress

- [x] (2026-04-27 20:56Z) Created this ExecPlan after confirming the working tree was clean and `src/main.rs` still owned the module root.
- [x] (2026-04-27 21:00Z) Archived completed active ExecPlans and updated the plan index.
- [x] (2026-04-27 21:08Z) Added `src/lib.rs` and moved module declarations from `src/main.rs` to the library crate root.
- [x] (2026-04-27 21:08Z) Updated `src/main.rs` to import modules from `tradingview_cli`; `cargo check` passes.
- [x] (2026-04-27 21:14Z) Updated architecture, development, roadmap, and changelog docs.
- [x] (2026-04-27 21:20Z) Ran validation, read-only smoke, and hygiene checks.
- [ ] Commit the related tracked changes.

## Surprises & Discoveries

- Observation: `Cargo.toml` currently declares only the `tv` binary target, but Cargo can infer a library target automatically when `src/lib.rs` exists.
  Evidence: `Cargo.toml` has `[[bin]] name = "tv" path = "src/main.rs"` and no `[lib]` section.

- Observation: `src/main.rs` owns the module declarations for `cdp`, `cli`, `error`, `ops`, `output`, and `transport`.
  Evidence: the file starts with `mod cdp; mod cli; mod error; mod ops; mod output; mod transport;`.

- Observation: running several `git mv` commands in parallel can briefly contend on Git's index lock.
  Evidence: one parallel `git mv` invocation failed with `Unable to create '.git/index.lock'`; rerunning the remaining moves sequentially succeeded.

- Observation: making `cdp` public from the library root exposed Rust's `async_fn_in_trait` warning for the internal `Runtime` trait.
  Evidence: `cargo check` warned on the public async trait methods in `src/cdp.rs` after adding `src/lib.rs`.

## Decision Log

- Decision: Add `src/lib.rs` and make the existing modules public from that root.
  Rationale: The binary can then use `tradingview_cli::...`, and later slices can decide which modules become stable public API. This keeps the first split mechanical and low risk.
  Date/Author: 2026-04-27 / Codex.

- Decision: Keep command dispatch in `src/main.rs` for this slice.
  Rationale: Moving dispatch at the same time would make the behavior-preserving boundary harder to review. A later refactor can extract dispatch once the library root exists and tests prove the split is stable.
  Date/Author: 2026-04-27 / Codex.

- Decision: Do not add an explicit `[lib]` section unless Cargo metadata shows the inferred library target is missing or misnamed.
  Rationale: The package name `tradingview-cli` naturally maps to the Rust crate name `tradingview_cli`, and avoiding redundant manifest configuration keeps the slice smaller.
  Date/Author: 2026-04-27 / Codex.

- Decision: Add a crate-level `#![allow(async_fn_in_trait)]` in `src/lib.rs` for this first boundary slice.
  Rationale: The library surface is explicitly unstable/internal in this slice. Rewriting the runtime trait would be a behavior-preserving API cleanup worth doing later, but it would expand this mechanical split beyond its intended scope.
  Date/Author: 2026-04-27 / Codex.

## Outcomes & Retrospective

The package now has an inferred library target named `tradingview_cli` and the existing binary target named `tv`. `src/lib.rs` owns the top-level module declarations, while `src/main.rs` remains the binary entrypoint and still owns runtime setup, dispatch, JSON envelope output, and exit codes.

Completed plans for direct HTTP feasibility, indicator alertcondition mutation, chart data readiness, and symbol HTTP reads were moved under `docs/plans/archives/`, leaving the root plan index focused on future work plus this active split.

Validation passed: `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`, `cargo test --test cli_contract -- --nocapture`, `cargo build`, `git diff --check`, and tracked-doc hygiene grep. `cargo metadata --no-deps --format-version 1` showed both `lib` target `tradingview_cli` and `bin` target `tv`. Read-only smoke passed for `target/debug/tv --help` and `target/debug/tv info NYSE:IONQ`.

Commit is still pending.

## Context and Orientation

The repository is a Rust package named `tradingview-cli`. Its installed binary is `tv`, declared in `Cargo.toml`. In Rust, a package name with a hyphen is imported as a crate name with an underscore, so this package's library crate is referenced as `tradingview_cli`.

Today `src/main.rs` is both the binary entrypoint and the module root. It declares the project modules and then uses them directly. A module root is the file that tells Rust which source files belong to the crate. A library crate root is normally `src/lib.rs`; a binary crate root is normally `src/main.rs`.

The key files for this slice are:

- `Cargo.toml`, which names the package and binary.
- `src/main.rs`, which should remain the `tv` runtime entrypoint.
- `src/lib.rs`, which will be added as the library module root.
- `docs/architecture.md`, `docs/development.md`, and `docs/v0.3-roadmap.md`, which describe the stable project structure and near-term roadmap.

## Plan of Work

First, archive completed active ExecPlans. Move the direct HTTP feasibility, indicator alertcondition mutation, chart data readiness, and symbol HTTP reads plans from `docs/plans/` into `docs/plans/archives/`. Update `docs/plans/README.md` so root active plans only describe future or still-open work.

Second, add `src/lib.rs`. It should declare the existing modules as `pub mod cdp;`, `pub mod cli;`, `pub mod error;`, `pub mod ops;`, `pub mod output;`, and `pub mod transport;`. It should not add new behavior or re-export a curated API yet.

Third, remove the same `mod ...` declarations from `src/main.rs` and import from the library crate instead. The binary should use paths such as `tradingview_cli::cdp::CdpClient`, `tradingview_cli::cli::Cli`, `tradingview_cli::error::AppError`, `tradingview_cli::output::SuccessEnvelope`, and `tradingview_cli::transport::TransportConfig`. The dispatch function can remain in `src/main.rs` and continue calling `ops::...`.

Fourth, update durable docs. The architecture guide should explain that the package now has a library crate root plus a thin binary entrypoint, while warning that the Rust library surface is not yet a stable public API. The development guide should tell future contributors to put reusable command logic in library modules, not in `src/main.rs`. The roadmap should mark the first library-boundary slice as started or complete, and CHANGELOG should record the internal refactor.

Finally, run validation and commit the related tracked changes.

## Concrete Steps

Work from the repository root.

1. Move completed plans. This has already been done in this slice; if repeating
   the work in a fresh checkout where the files are still at the root, use:

       git mv docs/plans/tradingview-cli-direct-http-feasibility.md docs/plans/archives/
       git mv docs/plans/tradingview-cli-indicator-alertcondition-mutation.md docs/plans/archives/
       git mv docs/plans/tradingview-cli-chart-data-readiness.md docs/plans/archives/
       git mv docs/plans/tradingview-cli-symbol-http-reads.md docs/plans/archives/

2. Edit `docs/plans/README.md`.

3. Add `src/lib.rs` with the module declarations listed above.

4. Edit `src/main.rs` so it imports modules from `tradingview_cli` and no longer declares modules directly.

5. Update docs and continuity.

6. Run validation:

       cargo fmt --check
       cargo clippy --all-targets --all-features -- -D warnings
       cargo test
       cargo test --test cli_contract -- --nocapture
       cargo metadata --no-deps --format-version 1
       cargo build
       target/debug/tv --help
       target/debug/tv info NYSE:IONQ
       git diff --check
       git grep -nE '(/Users/|C:\\|USER;|sessionid|cookie|authorization|bearer)' -- README.md CHANGELOG.md docs .agents/skills packaging scripts || true

7. Commit with:

       refactor(core): Add library crate boundary

## Validation and Acceptance

The change is accepted when `cargo metadata --no-deps --format-version 1` shows both a `lib` target and the existing `bin` target named `tv`, `target/debug/tv --help` still prints the normal CLI help, `target/debug/tv info NYSE:IONQ` still succeeds as a Desktop-free read, and the full Rust validation baseline passes.

There should be no user-visible command behavior changes. If any CLI contract test changes beyond import fallout, stop and inspect whether the split accidentally changed behavior.

## Idempotence and Recovery

The plan moves files and changes module roots only. If a `git mv` was already performed, do not duplicate files; continue by checking `git status --short` and editing the moved files in place. If compilation fails because a module is private, make the module public in `src/lib.rs` rather than reintroducing `mod` declarations into `src/main.rs`.

Do not use `cargo fmt` unless formatting fails and a formatter rewrite is needed. If formatting is required, run it once and re-run `cargo fmt --check`.

## Artifacts and Notes

Expected `cargo metadata --no-deps --format-version 1` should include target entries similar to:

    "kind":["lib"],"crate_types":["lib"],"name":"tradingview_cli"
    "kind":["bin"],"crate_types":["bin"],"name":"tv"

Expected `target/debug/tv info NYSE:IONQ` should return a success envelope with `command: "info"` and `data.source: "symbol_search_rest"`.

## Interfaces and Dependencies

No new dependency is required. `src/lib.rs` must expose these modules:

    pub mod cdp;
    pub mod cli;
    pub mod error;
    pub mod ops;
    pub mod output;
    pub mod transport;

The binary continues to own process setup, tracing initialization, CLI parse error formatting, async runtime startup, dispatch, and stdout/stderr JSON envelope printing.

## Open Questions

No critical open questions block this slice. A later slice should decide whether to move dispatch out of `src/main.rs` and which library types, if any, should be documented as stable for downstream Rust callers.
