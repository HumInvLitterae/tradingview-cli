# `tv bars` market crate internal split

This ExecPlan is a living document. Keep `Progress`,
`Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective`
current as work proceeds.

This document follows `.agents/PLANS.md` from the repository root. It records
the behavior-preserving internal refactor that splits the browserless
historical bars implementation inside `tradingview-market` after the CLI
operation adapter was restored to a thin wrapper.

## Purpose / Big Picture

`tv bars <EXCHANGE:SYMBOL>` now belongs to `tradingview-market`, which is the
right crate boundary for a Desktop-free historical OHLCV read. After that
move, however, `crates/market/src/bars.rs` still contained validation,
WebSocket transport, TradingView packet parsing, payload construction,
availability details, and tests in one large file.

This slice keeps the public Rust API and CLI JSON contract unchanged while
making the market crate implementation maintainable: `bars.rs` becomes the
facade, and private same-named submodules own validation, protocol parsing,
transport, payload shaping, and shared types.

## Progress

- [x] Create this ExecPlan and archive the completed bars crate-boundary
  refactor plan.
- [x] Update `docs/plans/README.md` and `docs/v0.17-roadmap.md` so the
  current plan is this market crate internal split.
- [x] Split `crates/market/src/bars.rs` into facade plus private
  `bars/validation.rs`, `bars/protocol.rs`, `bars/transport.rs`,
  `bars/payload.rs`, and `bars/types.rs`.
- [x] Keep `tradingview_market::bars_symbol(symbol, timeframe, count)` as the
  only public bars API.
- [x] Preserve the `bars.v1` JSON contract, validation behavior, and
  structured failure semantics.
- [x] Run focused tests, baseline checks, and docs hygiene.
- [x] Commit the refactor.

## Surprises & Discoveries

- No public JSON shape change was needed. The existing payload and failure
  tests moved cleanly to the module that owns each responsibility.
- The facade pattern matches existing repository guidance: large capabilities
  should keep a stable public entrypoint and move implementation detail into
  same-named private modules.

## Decision Log

- Decision: Split the market bars implementation by responsibility instead of
  keeping a single large `bars.rs`.
  Rationale: The earlier crate-boundary refactor fixed where the code lives;
  this follow-up fixes how it is organized inside that crate. Transport,
  protocol, payload, validation, and shared types change for different
  reasons and should not be edited as one large unit.
  Date/Author: 2026-05-14 / Codex.

- Decision: Keep module visibility private / `pub(super)` and expose only
  `bars_symbol` publicly.
  Rationale: v0.17 needs behavior-preserving maintainability work, not a new
  stable typed Rust API. Internal types can be promoted later if real callers
  need them.
  Date/Author: 2026-05-14 / Codex.

- Decision: Do not introduce a cross-command `source_availability` abstraction
  in this slice.
  Rationale: Bars is still the only Desktop-free WebSocket historical read
  using this exact availability packet. Premature sharing would hide the
  command-local contract instead of simplifying it.
  Date/Author: 2026-05-14 / Codex.

## Outcomes & Retrospective

`crates/market/src/bars.rs` is now a facade that validates the request, calls
the browserless transport, handles the no-bars error boundary, and returns the
existing payload.

Implementation details now live under `crates/market/src/bars/`:

- `types.rs`: internal bars structs, wait summary, availability state, and
  source constants;
- `validation.rs`: symbol / timeframe / count validation;
- `protocol.rs`: TradingView frame construction, packet parsing, ping/pong,
  bar parsing, time normalization, merge, and bar JSON conversion;
- `transport.rs`: WebSocket connection, session setup, bounded wait, and
  source-error mapping;
- `payload.rs`: `bars.v1` success payload and structured error details.

The public command contract remains unchanged: successful payloads and
structured failures still use `contract_version: "bars.v1"`,
`source: "tradingview_bars_ws"`, `source_category: "desktop_free_read"`,
`requires_desktop: false`, `non_mutating: true`, `summary`, `range`,
`data_quality.partial_result`, `source_availability`, and public-safe
`wait_summary`.

## Plan of Work

1. Archive the completed crate-boundary refactor plan and create this
   follow-up ExecPlan.
2. Replace the large `crates/market/src/bars.rs` with a thin facade.
3. Move validation, protocol parsing, transport, payload, and shared types
   into private same-named submodules.
4. Move existing unit tests into the owning modules.
5. Update durable docs to record the internal split and its non-goals.
6. Run focused tests and full workspace validation.

## Acceptance Criteria

- `tradingview_market::bars_symbol(...)` remains the public API.
- `tv bars` command behavior and JSON contract are unchanged.
- `bars.v1` success and structured failure fields are preserved.
- Raw WebSocket frames, raw payloads, session ids, credentials, account-local
  metadata, target ids, and local paths are not added to public docs, payloads,
  or panic messages.
- The next planned test-organization slice can proceed without having to
  untangle bars implementation responsibilities first.

## Validation

Run:

    cargo test -p tradingview-market bars -- --nocapture
    cargo test -p tradingview-cli market::bars -- --nocapture
    cargo test -p tradingview-cli --test cli_contract bars -- --nocapture
    cargo test -p tradingview-cli --test live_bars
    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1
    git diff --check
    bash -n scripts/stage-release-package-files.sh

Optional read-only smoke:

    target/debug/tv bars NASDAQ:AAPL --timeframe 1D --count 5
    target/debug/tv bars NASDAQ:RKLB --timeframe 1 --count 10

Live output must not be pasted into tracked docs.

## Interfaces and Dependencies

No public CLI interface changes. No new command, option, data source, payload
semantics, dependency, release version bump, realtime feed, automatic
fallback, source mixing, ranking, scoring, recommendation, or trading action
is introduced.
