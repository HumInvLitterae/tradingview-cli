# Extract CDP and transport support crate

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This plan follows `.agents/PLANS.md`.

## Purpose / Big Picture

This refactor makes the TradingView Desktop connection layer reusable without changing what the `tv` CLI does. After the change, the root CLI crate still owns command parsing, application orchestration, and operation modules, but the shared Chrome DevTools Protocol client and target discovery code live in an internal workspace crate named `tradingview-cdp`. A user can see the change working by running the same CLI commands and tests as before and observing identical JSON envelopes and exit codes.

## Progress

- [x] (2026-04-28T07:32Z) Created the `tradingview-cdp` crate, moved CDP client and target discovery code into it, and updated root crate imports.
- [x] (2026-04-28T07:32Z) Archived the completed application-layer split ExecPlan.
- [x] (2026-04-28T07:45Z) Ran full workspace validation, focused CDP/transport/tab checks, metadata inspection, and behavior smoke.
- [x] (2026-04-28T07:45Z) Updated architecture, development, roadmap, changelog, upstream notes, and continuity notes.
- [x] (2026-04-28T07:45Z) Committed the behavior-preserving refactor.

## Surprises & Discoveries

- Observation: The root crate still needs `reqwest` after extracting target discovery because tab activation and launch readiness checks call CDP HTTP endpoints directly.
  Evidence: `rg -n "reqwest::" src` shows uses in `src/ops/tab.rs` and `src/ops/launch.rs`.

- Observation: Piping `target/debug/tv --help` into `head` can cause a broken-pipe panic from stdout printing, so the behavior smoke should redirect help output to a file or run it without truncating the pipe.
  Evidence: `target/debug/tv --help >/tmp/tv-help-smoke.txt` exited 0 and printed the expected first line.

## Decision Log

- Decision: Put both the WebSocket CDP client and CDP target discovery in one crate, `crates/cdp/`, rather than splitting transport into a second crate.
  Rationale: Operation modules use `CdpClient`, `RuntimeEvaluator`, input event types, `TransportConfig`, and `Target` together. A single internal crate gives a clean reusable Desktop connection boundary without creating artificial package seams.
  Date/Author: 2026-04-28 / Codex.

- Decision: Keep `tradingview-cdp` internal and unstable.
  Rationale: The crate exposes TradingView Desktop implementation details that are useful for this workspace but not yet a supported public Rust API.
  Date/Author: 2026-04-28 / Codex.

## Outcomes & Retrospective

The CDP and transport code now live in `tradingview_cdp`, root operation modules compile through that facade, and `src/lib.rs` no longer declares local `cdp` or `transport` modules. Validation preserved the CLI contract: workspace tests, focused CDP/transport/tab tests, CLI contract tests, metadata inspection, and read-only smoke all passed. The only live smoke gap was optional `--target-id status`, skipped because `tv tab list` reported no chart target in the current session.

## Context and Orientation

The repository is a Cargo workspace for a Rust-native TradingView CLI. The binary target remains `tv` in the root package. Existing internal crates already own shared contracts (`tradingview-core`), Desktop-free market reads (`tradingview-market`), scanner reads (`tradingview-scanner`), and Pine static/check helpers (`tradingview-pine`).

Before this plan, the root crate still owned two shared Desktop connection modules:

- `src/cdp.rs`: the WebSocket Chrome DevTools Protocol client, the `RuntimeEvaluator` trait, screenshot clip type, keyboard event type, mouse event type, and CDP response/error mapping.
- `src/transport.rs`: CDP target list fetching, chart target selection, `--target-id` handling, target handoff fields, and `TV_CDP_HOST` / `TV_CDP_PORT` endpoint configuration.

In this plan, "CDP" means Chrome DevTools Protocol, the local protocol TradingView Desktop exposes when started with remote debugging. "Target discovery" means reading the local `/json/list` endpoint and choosing the intended TradingView chart target.

## Plan of Work

Add `crates/cdp/` as package `tradingview-cdp` and crate `tradingview_cdp`. Move `src/cdp.rs` to `crates/cdp/src/client.rs` and `src/transport.rs` to `crates/cdp/src/transport.rs`. Add `crates/cdp/src/lib.rs` as a facade that re-exports the public internal types currently used by operation modules.

Update the root `Cargo.toml` workspace members and dependencies. Remove root dependencies that only the new CDP crate needs, but keep root dependencies still used by command modules.

Update root imports from `crate::cdp` and `crate::transport` to `tradingview_cdp`. Remove `pub mod cdp;` and `pub mod transport;` from `src/lib.rs`. Preserve all function names and payload fields used by root modules so command behavior does not change.

Update stable docs to describe `tradingview_cdp` as the internal Desktop connection crate. Archive the completed application-layer split plan and make this plan the active crate-split plan.

## Concrete Steps

Run commands from the repository root.

First verify the working tree and inspect dependencies:

    git status --short
    rg -n "crate::(cdp|transport)|tradingview_cdp|TransportConfig|RuntimeEvaluator" src crates -g '*.rs'

After edits, run:

    cargo check --workspace
    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo test cdp -- --nocapture
    cargo test transport -- --nocapture
    cargo test tab -- --nocapture
    cargo test --test cli_contract status -- --nocapture
    cargo test --test cli_contract tab -- --nocapture
    cargo metadata --no-deps --format-version 1
    git diff --check

Run behavior smoke after building:

    target/debug/tv --help
    target/debug/tv info NYSE:IONQ
    target/debug/tv quote NYSE:IONQ
    TV_CDP_PORT=9 target/debug/tv status

If a live TradingView Desktop session is available, also run:

    target/debug/tv tab list
    target/debug/tv --target-id <ID> status

Do not record live target ids in tracked docs.

## Validation and Acceptance

Acceptance requires all workspace tests to pass and the smoke commands above to keep the same public behavior. The `cargo metadata` output must include package `tradingview-cdp`, and the root package must still expose binary target `tv`.

The JSON error envelope for connection errors and target ambiguity must remain unchanged except for implementation location. `target_ambiguous` details must still include `next_action_hint` and `target_cli_args`. `TV_CDP_HOST`, `TV_CDP_PORT`, and `--target-id` behavior must remain intact.

## Idempotence and Recovery

This refactor is mechanical and safe to retry. If imports become inconsistent, search for `crate::cdp`, `crate::transport`, `src/cdp.rs`, and `src/transport.rs` and replace them with the `tradingview_cdp` facade. If validation fails in command behavior, revert the import or facade change that altered the public type or helper name instead of changing the CLI contract.

## Artifacts and Notes

Keep evidence concise. A successful `cargo metadata --no-deps --format-version 1` run should list `tradingview-cdp` among workspace packages. A successful smoke run should show Desktop-free `info` and `quote` still succeed with `source` values from the existing direct-read paths.

## Interfaces and Dependencies

At completion, `crates/cdp/src/lib.rs` must publicly re-export:

    pub use client::{
        CdpClient, KeyEvent, KeyEventType, MouseEvent, MouseEventType, RuntimeEvaluator,
        ScreenshotClip,
    };
    pub use transport::{
        Target, TargetSelection, TransportConfig, discover_target, fetch_targets,
        is_app_window_target, select_target, target_cli_args, target_title_for_handoff,
        target_url_for_handoff,
    };

The crate depends on `tradingview-core` for `AppError` and `ErrorKind`. Root operation modules depend on the facade and should not import CDP implementation submodules directly.

## Open Questions

No critical open questions. The next likely refactor after this one is domain module cleanup for large root operation modules such as Screener, alert, or layout.
