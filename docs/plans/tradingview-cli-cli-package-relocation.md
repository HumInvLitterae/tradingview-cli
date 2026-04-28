# Relocate the CLI package under crates/cli

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This plan follows `.agents/PLANS.md`.

## Purpose / Big Picture

This refactor makes the workspace layout match the architecture that already emerged: reusable support crates live under `crates/`, and the `tv` command itself should live there too. After this change, the repository root is a virtual Cargo workspace with no package of its own. The `tradingview-cli` package, its `tv` binary, its application layer, and its operation adapter layer live under `crates/cli/`. Users should see no behavior change: `cargo build`, `cargo test`, release builds, and the `tv` binary continue to work.

## Progress

- [x] (2026-04-28T07:54Z) Moved the root package source and CLI contract tests under `crates/cli/`.
- [x] (2026-04-28T07:54Z) Converted the root `Cargo.toml` into a virtual workspace manifest and adjusted `crates/cli/Cargo.toml` dependency paths.
- [x] (2026-04-28T07:54Z) Archived the completed CDP crate extraction ExecPlan.
- [x] (2026-04-28T07:54Z) Verified the initial move with `cargo check --workspace`.
- [x] (2026-04-28T08:20Z) Updated architecture, development, release, roadmap, handoff, CI, hook, and skill docs for the new workspace layout.
- [x] (2026-04-28T08:55Z) Ran full validation, release package staging, skill validation, hygiene checks, and behavior smoke.
- [x] (2026-04-28T09:00Z) Prepared the behavior-preserving relocation for commit.

## Surprises & Discoveries

- Observation: The package move compiled after only changing dependency paths from `crates/<name>` to `../<name>`.
  Evidence: `cargo check --workspace` completed successfully with `tradingview-cli` loaded from `crates/cli`.

- Observation: Release package staging still consumes the built binary from `target/release/tv`; moving the package did not change the release artifact path.
  Evidence: `cargo build --release --locked` completed successfully, and `scripts/stage-release-package-files.sh target/release-package-smoke target/release/tv` staged the binary and runtime agent files.

## Decision Log

- Decision: Move the CLI package to `crates/cli/` and make the repository root a virtual workspace.
  Rationale: Once `core`, `market`, `scanner`, `pine`, and `cdp` are internal crates, keeping only the CLI package in root `src/` makes the workspace visually asymmetric. `crates/cli/` makes the package boundary explicit while preserving the `tradingview-cli` package name and `tv` binary name.
  Date/Author: 2026-04-28 / Codex.

- Decision: Keep `ops` inside the CLI package for now.
  Rationale: The current operation modules are not pure domain crates. They combine command-facing operation adapters, TradingView page-session APIs, DOM/page-object interaction, post-check logic, and JSON payload normalization. A single `tradingview-ops` crate would mostly recreate a large root crate under another name. Future extraction should be domain-specific and evidence-backed.
  Date/Author: 2026-04-28 / Codex.

## Outcomes & Retrospective

The workspace root now has no package of its own. `tradingview-cli` builds from `crates/cli/`, the `tv` binary remains available at `target/debug/tv` and `target/release/tv`, and CLI contract tests still pass from `crates/cli/tests/cli_contract.rs`.

Validation passed:

- `cargo check --workspace`
- `cargo fmt --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace`
- `cargo test -p tradingview-cli --test cli_contract -- --nocapture`
- `cargo metadata --no-deps --format-version 1`
- `cargo build --release --locked`
- `bash -n scripts/stage-release-package-files.sh`
- `scripts/stage-release-package-files.sh target/release-package-smoke target/release/tv`
- `python3 <skill-creator>/scripts/quick_validate.py .agents/skills/release-prep`
- `git diff --check`

Behavior smoke passed for `tv --help`, Desktop-free `info NYSE:IONQ`, Desktop-free `quote NYSE:IONQ`, and structured `TV_CDP_PORT=9 status` connection failure with exit code 2.

The tracked-doc hygiene grep returned only existing policy text and validation-command examples, including archived plans; no new live account identifiers, credentials, or machine-specific operational values were added.

## Context and Orientation

The repository contains a Rust CLI named `tv`. Before this plan, the root package was named `tradingview-cli` and stored its Rust source under `src/` and its CLI contract tests under `tests/`. The workspace already contained reusable crates under `crates/core`, `crates/market`, `crates/scanner`, `crates/pine`, and `crates/cdp`.

In this plan, "virtual workspace" means a root `Cargo.toml` that has `[workspace]` but no `[package]`. The actual CLI package moves to `crates/cli/Cargo.toml`. The binary name remains `tv`, so normal build output should still be `target/debug/tv` or `target/release/tv`.

The operation adapter layer is the code currently under `crates/cli/src/ops/` after the move. It sits between the application dispatch layer and reusable lower-level crates. It is called an adapter layer because it adapts CLI command requests to TradingView Desktop operations and JSON payloads; it is not yet a clean domain crate boundary.

## Plan of Work

Move `src/` to `crates/cli/src/` and `tests/` to `crates/cli/tests/`. Move the old root package manifest to `crates/cli/Cargo.toml`. Create a new root `Cargo.toml` that contains only the workspace members. In `crates/cli/Cargo.toml`, keep package name `tradingview-cli`, binary name `tv`, and library crate name `tradingview_cli` implicit from the package name. Update internal path dependencies to use `../core`, `../market`, `../scanner`, `../pine`, and `../cdp`.

Update docs so stable architecture says the root is a virtual workspace and `crates/cli/` owns the CLI package. Replace root-package assumptions in validation examples with workspace-safe commands, especially `cargo test -p tradingview-cli --test cli_contract ...`.

Do not split `ops` into a new crate in this plan. Record that `ops` remains inside `crates/cli/src/ops/` and should be split internally by large domain module before any domain-specific crate extraction is attempted.

## Concrete Steps

Run commands from the repository root.

Initial move:

    mkdir -p crates/cli
    git mv src crates/cli/src
    git mv tests crates/cli/tests
    git mv Cargo.toml crates/cli/Cargo.toml

Create the root virtual manifest and adjust `crates/cli/Cargo.toml`. Then run:

    cargo check --workspace

After docs are updated, run:

    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo test -p tradingview-cli --test cli_contract -- --nocapture
    cargo metadata --no-deps --format-version 1
    cargo build --release --locked
    bash -n scripts/stage-release-package-files.sh
    rm -rf target/release-package-smoke
    scripts/stage-release-package-files.sh target/release-package-smoke target/release/tv
    find target/release-package-smoke -maxdepth 4 -print | sort
    git diff --check

Run behavior smoke:

    target/debug/tv --help
    target/debug/tv info NYSE:IONQ
    target/debug/tv quote NYSE:IONQ
    TV_CDP_PORT=9 target/debug/tv status

Do not pipe help output into `head`; redirect to a temporary file if only the first line is needed.

## Validation and Acceptance

Acceptance requires the `tradingview-cli` package to appear in `cargo metadata` with manifest path under `crates/cli/Cargo.toml`, while the binary target remains named `tv`. `cargo build --release --locked` must still produce `target/release/tv` on Unix-like hosts. The CLI contract tests must pass from their new package-local location. Release package staging must still accept `target/release/tv` and include the expected runtime files.

The behavior smoke must show `tv --help` exits 0, Desktop-free `info` and `quote` succeed, and `TV_CDP_PORT=9 tv status` still returns a structured connection error with exit code 2.

## Idempotence and Recovery

The move is mechanical. If a command fails because Cargo cannot find a package, inspect `cargo metadata --no-deps --format-version 1` and confirm the workspace member list includes `crates/cli`. If dependency resolution fails, check that `crates/cli/Cargo.toml` path dependencies use `../<crate>`, not `crates/<crate>`.

If release packaging fails, first verify that `cargo build --release --locked` produced `target/release/tv`; do not change package contents until the binary path is confirmed.

## Artifacts and Notes

Keep terminal evidence concise. Record only package names and smoke summaries, not local absolute paths from Cargo output.

## Interfaces and Dependencies

At completion:

    Cargo.toml
        [workspace]
        members = [
            "crates/cli",
            "crates/core",
            "crates/market",
            "crates/scanner",
            "crates/pine",
            "crates/cdp",
        ]
        resolver = "3"

    crates/cli/Cargo.toml
        [package]
        name = "tradingview-cli"
        ...
        [[bin]]
        name = "tv"
        path = "src/main.rs"

No public CLI commands, JSON envelope fields, or exit codes change.

## Open Questions

No critical open questions. The next refactor should choose one large operation module and split it internally, likely Screener first because it is currently the largest module.
