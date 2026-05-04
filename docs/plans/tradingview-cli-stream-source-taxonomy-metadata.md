# Stream source taxonomy metadata

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document follows `.agents/PLANS.md` from the repository root. It is self-contained so a new contributor can finish the stream source taxonomy metadata polish without prior chat context.

## Purpose / Big Picture

Make `tv stream ...` JSONL events self-describing in the `v0.6.0` command source taxonomy. Stream commands are Desktop-backed read commands: they poll the selected TradingView Desktop target through CDP, emit read-only observations, and should not be confused with Desktop-free scanner or browserless WebSocket reads.

After this change, every stream sample and heartbeat event includes additive metadata that says it came from `desktop_chart_stream`, belongs to `desktop_backed_read`, requires Desktop, and is non-mutating.

## Progress

- [x] (2026-05-05) Archived the completed stream observation controls ExecPlan.
- [x] (2026-05-05) Added this ExecPlan.
- [x] (2026-05-05) Added stream source taxonomy metadata to sample and heartbeat events.
- [x] (2026-05-05) Updated docs and runtime skills for the stream event source fields.
- [x] (2026-05-05) Ran Rust, docs, packaging, and hygiene validation.
- [x] (2026-05-05) Attempted skill validation through `uvx --with PyYAML`; blocked because `uvx` is not installed in the active shell.
- [x] (2026-05-05) Commit the related changes.

## Surprises & Discoveries

- Observation: the active shell does not have `uvx` or `uv` on `PATH`.
  Evidence: each `uvx --with PyYAML python ... quick_validate.py ...` command failed with `zsh:1: command not found: uvx`, and `command -v uvx` / `command -v uv` returned no path.

## Decision Log

- Decision: Keep stream source metadata additive under `data`.
  Rationale: downstream consumers already parse stream JSONL sample fields; adding fields avoids a command or envelope change.
  Date/Author: 2026-05-05 / Codex

- Decision: Validate skills with `uvx --with PyYAML python ...` in this environment.
  Rationale: the local Python may not have `yaml`; using `uvx` supplies the dependency without adding a repository dependency or local virtualenv.
  Date/Author: 2026-05-05 / Codex

## Outcomes & Retrospective

Implemented additive source taxonomy metadata for stream sample and heartbeat events. Each event now reports `source: "desktop_chart_stream"`, `source_category: "desktop_backed_read"`, `requires_desktop: true`, and `non_mutating: true`.

Updated README, taxonomy docs, internal API/boundary docs, the v0.6 roadmap, development validation guidance, and runtime skills so downstream agents can distinguish Desktop-backed stream observations from Desktop-free scanner reads and lab browserless bars.

Rust validation, packaging script syntax, metadata generation, and diff whitespace checks passed. Skill validation was attempted with `uvx --with PyYAML`, as planned, but the command is not installed in this shell. No Python dependency, virtualenv, or local package install was added.

## Context and Orientation

The previous stream slice added bounded observation controls and heartbeat events. It intentionally left stream as a Desktop-backed read surface. This slice does not add browserless streaming, a new `observe` command, or additional stream options.

Relevant files:

- `crates/cli/src/ops/stream.rs` owns stream sample and heartbeat payload metadata.
- `docs/command-source-taxonomy.md` defines `Desktop-backed read`.
- Runtime skills under `.agents/skills/` explain how downstream agents should interpret stream JSONL.

## Plan of Work

Add the following fields to both sample and heartbeat event payloads:

- `source: "desktop_chart_stream"`
- `source_category: "desktop_backed_read"`
- `requires_desktop: true`
- `non_mutating: true`

Keep `_event`, `_stream`, `_ts`, practical stream fields, bounded controls, infinite default behavior, and runtime stderr error behavior unchanged. Do not count heartbeat events as samples.

Update docs and runtime skills to tell agents to read `data._event` first and then use the source taxonomy fields to distinguish Desktop-backed stream observations from Desktop-free reads and lab browserless bars.

## Validation and Acceptance

Run:

    cargo test -p tradingview-cli stream -- --nocapture
    cargo test -p tradingview-cli --test cli_contract stream -- --nocapture
    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1
    git diff --check

Validate changed runtime skills with `uvx`:

    uvx --with PyYAML python <skill-creator>/scripts/quick_validate.py .agents/skills/market-data-interpretation
    uvx --with PyYAML python <skill-creator>/scripts/quick_validate.py .agents/skills/chart-analysis
    uvx --with PyYAML python <skill-creator>/scripts/quick_validate.py .agents/skills/multi-symbol-scan
    bash -n scripts/stage-release-package-files.sh

Optional live smoke:

    target/debug/tv stream quote --duration-ms 3000 --heartbeat-ms 1000
    target/debug/tv stream bars --max-events 2 --interval 500

Acceptance is met when sample and heartbeat events carry the additive source taxonomy fields, existing stream controls still behave the same way, and full Rust validation passes. Changed skills should be validated through `uvx --with PyYAML` when `uvx` is available; if the command is unavailable, record that explicitly and do not add a repository Python dependency or local virtualenv.

## Idempotence and Recovery

This slice is safe to rerun. If live smoke is attempted, it reads the selected Desktop chart and does not mutate account state. Do not record raw live payloads, target ids, account-local metadata, cookies, tokens, or local absolute paths in tracked docs.

## Interfaces and Dependencies

No new dependencies. No new top-level command. No stream option changes. JSONL additions are additive under `data`.

## Open Questions

None.
