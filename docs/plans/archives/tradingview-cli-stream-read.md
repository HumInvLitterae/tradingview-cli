# Add read-only stream commands

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document follows `.agents/PLANS.md`.

## Purpose / Big Picture

After this change, a user can run `tv stream quote`, `tv stream bars`, `tv stream values`, `tv stream lines`, `tv stream labels`, `tv stream tables`, or `tv stream all` to monitor the current TradingView Desktop session as newline-delimited JSON. Newline-delimited JSON means each stdout line is a complete JSON object, which lets shell tools and downstream monitor processes read updates one at a time.

This is the next old CLI migration slice because the remaining deferred commands mostly involve persistent saves, bulk deletion, launch process control, or generic UI automation. The stream commands are read-only polling wrappers over data that Rust already knows how to read, so they advance migration while keeping the core CLI narrower than the old bridge.

## Progress

- [x] (2026-04-24 17:51Z) Read `.agents/PLANS.md`, current Rust command/output modules, current migration notes, and the old JavaScript stream implementation.
- [x] (2026-04-24 17:51Z) Created this ExecPlan.
- [x] (2026-04-24 18:05Z) Add stream command-line surface and special dispatch path.
- [x] (2026-04-24 18:05Z) Implement read-only stream polling, dedupe, JSONL output, and validation.
- [x] (2026-04-24 18:10Z) Add unit and CLI contract tests.
- [x] (2026-04-24 18:20Z) Update README, AGENTS, migration inventory, contract notes, handoff notes, and relevant repo-local skills.
- [x] (2026-04-24 18:30Z) Run automated validation, skill validation, and live smoke against TradingView Desktop.
- [x] (2026-04-24 18:35Z) Commit the completed slice.

## Surprises & Discoveries

- Observation: The current Tokio dependency does not enable the `signal` feature, so `tokio::signal::ctrl_c()` is unavailable without changing dependency features.
  Evidence: `cargo test ops::stream -- --nocapture` failed with `could not find signal in tokio`.

- Observation: Live smoke returned one JSONL line each for `stream quote`, `stream bars`, and `stream all` against the current chart.
  Evidence: The smoke output included `quote {"_stream": "quote", "symbol": "BATS:LWLG"}`, `bars {"_stream": "bars", "resolution": "1D", "symbol": "BATS:LWLG"}`, and `all {"_stream": "all", "pane_count": 1}`.

## Decision Log

- Decision: Implement stream as a special top-level command that writes JSONL directly instead of returning one final success envelope through the normal `dispatch` path.
  Rationale: The existing CLI prints one JSON object per invocation, while stream commands are intentionally long-running and must emit many JSON objects over time. Routing stream through the normal envelope would add an extra final object and would not match the old CLI's pipe-friendly behavior.
  Date/Author: 2026-04-24 / Codex.

- Decision: Keep this slice read-only and exclude `pine save`, `pine raw-compile`, `draw clear`, `alert delete --all`, `launch`, and generic UI automation.
  Rationale: Those remaining old CLI surfaces have larger side effects or process/session ownership questions. Streaming is safer because it only polls the current chart/session and writes stdout lines.
  Date/Author: 2026-04-24 / Codex.

## Outcomes & Retrospective

Implemented read-only JSONL stream commands for quote, bars, values, line primitives, label primitives, table primitives, and all panes. The implementation validates interval values before connecting, emits compact one-line JSON envelopes for changed samples only, and leaves high-risk deferred surfaces such as Pine save, raw compile, draw clear, alert bulk deletion, launch, and generic UI automation untouched.

## Context and Orientation

The Rust CLI is a single binary named `tv`. Command-line shape lives in `src/cli.rs`. Normal command dispatch lives in `src/main.rs`: it parses a `Command`, calls `dispatch`, wraps the returned payload in `output::SuccessEnvelope`, and prints one JSON object. Errors are printed as `output::ErrorEnvelope` to stderr.

Streaming needs one intentional exception to that normal path. A stream command is a polling loop: it repeatedly reads data from the running TradingView Desktop page through Chrome DevTools Protocol, abbreviated CDP, and writes one JSON object per update. CDP is the local debugging protocol exposed when TradingView Desktop is launched with a remote debugging port.

Existing read functions already provide most of the data. `src/ops/market.rs` has `quote` and `ohlcv_bars`. `src/ops/data/drawings.rs` has line, label, and table reads. `src/ops/layout.rs` has pane and watchlist support. The stream implementation should reuse existing JavaScript patterns where practical, but it should not create or modify TradingView state.

## Plan of Work

First update `src/cli.rs`. Add `Command::Stream { command: StreamCommand }` and a `StreamCommand` enum with `Quote`, `Bars`, `Values`, `Lines`, `Labels`, `Tables`, and `All`. Each subcommand should accept `--interval <MS>` as `Option<u64>`. `Lines`, `Labels`, and `Tables` should also accept `--filter <TEXT>`.

Then update `src/main.rs`. After parsing and before normal `dispatch`, detect `Command::Stream` and call a dedicated stream runner. This runner should validate the interval before connecting. On startup validation or connection failure, print the existing error envelope and return the existing exit code. During the polling loop, successful samples should print `SuccessEnvelope::new("stream", data)` to stdout, one line per update. Runtime polling errors after startup should be printed to stderr as an error envelope and the loop should continue after a short delay.

Add `src/ops/stream.rs` and expose it through `src/ops.rs`. Define a small `StreamRequest` type that captures the stream kind, interval, and optional filter. Define validation helpers for default intervals and the minimum interval. Implement a polling loop that evaluates one JavaScript expression per sample, hashes or compares the returned `serde_json::Value`, and emits only when the value changes. Each emitted `data` object should include `_stream` and `_ts` plus the practical fields from the old CLI. `_ts` should be milliseconds since Unix epoch.

For `quote`, return old practical fields such as `symbol`, `time`, `open`, `high`, `low`, `close`, and `volume`. For `bars`, return `symbol`, `resolution`, `bar_time`, `open`, `high`, `low`, `close`, `volume`, and `bar_index`. For `values`, return `symbol`, `study_count`, and `studies`. For `lines`, `labels`, and `tables`, preserve `symbol`, `study_count`, and `studies`, with optional filter matching study names case-insensitively. For `all`, return `layout`, `pane_count`, and `panes`.

Finally update tests and docs. Tests should verify validation and CLI help without requiring TradingView Desktop. Operation tests should use fake runtime evaluators where possible. Docs should move stream commands from deferred backlog to implemented surface and explicitly state that stream is read-only, JSONL, and intended for external monitoring rather than request-response adapters.

## Concrete Steps

Run all commands from the repository root.

Targeted validation while implementing:

    cargo test ops::stream -- --nocapture
    cargo test --test cli_contract stream -- --nocapture

Full validation before commit:

    cargo fmt --check
    cargo clippy --all-targets --all-features -- -D warnings
    cargo test
    git diff --check
    rg -n "(/[U]sers/|[C]:\\\\)" README.md AGENTS.md docs .agents/skills || true

## Validation and Acceptance

Automated acceptance is that the full Rust baseline passes and tests prove stream help, interval validation, CDP connection failure shape, and operation-level dedupe/sample behavior. The command `TV_CDP_PORT=9 cargo run --quiet -- stream quote` should fail with a structured `connection` error before any JSONL success output.

Live smoke should run only against a running TradingView Desktop session:

    cargo run --quiet -- stream quote --interval 300

The smoke should read one JSONL line, verify it parses as JSON with `success: true`, `command: "stream"`, `data._stream: "quote"`, and a current chart symbol, then terminate the process. If stable, also smoke `stream bars` and `stream all` for one line each. Live smoke must not run any mutating commands.

## Idempotence and Recovery

The implementation is read-only. Re-running tests or live smoke should not change TradingView account, chart, Pine, drawing, alert, replay, tab, or watchlist state. If a live stream process is left running, stop it with Ctrl-C or by terminating the local process; no TradingView cleanup is required. The implementation does not install a custom Ctrl-C handler because the current Tokio dependency does not enable the `signal` feature.

## Artifacts and Notes

- `cargo test ops::stream -- --nocapture` passed with 6 stream unit tests.
- `cargo test --test cli_contract stream -- --nocapture` passed with 3 stream CLI contract tests.
- `cargo fmt --check && cargo clippy --all-targets --all-features -- -D warnings && cargo test` passed.
- `git diff --check` passed.
- `rg -n '(/[U]sers/|[C]:\\\\)' README.md AGENTS.md docs .agents/skills || true` returned no tracked-doc absolute local paths.
- The skill validator passed for `.agents/skills/chart-analysis` and `.agents/skills/multi-symbol-scan`.
- Live smoke passed for `stream quote`, `stream bars`, and `stream all`, each reading one JSONL line and terminating the process.

## Interfaces and Dependencies

At completion, `src/cli.rs` exposes:

    tv stream quote [--interval <MS>]
    tv stream bars [--interval <MS>]
    tv stream values [--interval <MS>]
    tv stream lines [--filter <TEXT>] [--interval <MS>]
    tv stream labels [--filter <TEXT>] [--interval <MS>]
    tv stream tables [--filter <TEXT>] [--interval <MS>]
    tv stream all [--interval <MS>]

At completion, `src/ops/stream.rs` exposes validation helpers and a stream runner used by `src/main.rs`. No new crates are required.

## Open Questions

No unresolved critical questions remain for this slice.
