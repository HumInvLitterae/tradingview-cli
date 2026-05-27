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
- Keep shared package metadata such as version, edition, license, and publish
  policy in the workspace root `[workspace.package]` table.
- Keep dependency versions and internal crate paths in the workspace root
  `[workspace.dependencies]` table. Member crates should normally use
  `workspace = true` in their own `[dependencies]` or `[dev-dependencies]`
  entries and add only crate-specific feature selections there.
- Prefer facade files with same-named submodule directories for large
  capabilities.
- Keep top-level CLI package module declarations in `crates/cli/src/lib.rs`.
- Keep `crates/cli/src/cli.rs` focused on command and argument shape.
- Keep operation adapter implementations under `crates/cli/src/ops/` by
  capability.
- Put shared I/O-free command model logic in `crates/model/`. The
  `tradingview-model` crate owns validation, request interpretation, selector
  and target resolution, payload normalization/shaping, and fallback policy
  decisions. It must not depend on clap command enums, CDP runtime objects,
  HTTP clients, page-session execution, or UI automation. Accepted examples are
  `tradingview_model::watchlist`, `alert`, `replay`, `drawing`, and
  `screener`.
- Let `crates/cli/src/app/dispatch.rs` call `tradingview_model::*` directly for pure
  validation, request interpretation, target resolution, and payload shaping.
  Use `ops::*` from dispatch only for executable TradingView operations or
  adapter-specific request types. Do not re-export model helpers through
  `ops.rs` solely for dispatch convenience.
- When an operation adapter grows too large, split it behind a facade file and
  a same-named directory before creating a new workspace crate. `screener` is
  the current model: stable public adapter exports at the facade, sub-surface
  implementation modules underneath, and shared runtime/page-session helpers in
  a narrow common module.
- Prefer moving CDP-free input boundaries before runtime/storage/UI code.
  Screener is the larger example: validation, target resolution, and storage
  payload shaping live in `tradingview_model::screener`, while page-session storage
  fetch/save and UI operations remain in `ops/screener`.
- Storage-backed sub-surfaces are the next-best split candidates once
  validation is isolated. Screener columns live in
  `crates/cli/src/ops/screener/columns.rs`; Screener filters and screens now
  also own their operation bodies while shared open-state, storage fetch, click
  dispatch, and JavaScript helper expansion remain in `engine.rs`.
- Keep mixed page-session adapters split by user-visible sub-surface before
  extracting crates. Alert is the current model: list, normal create,
  indicator-alert create, delete, and public-safe payload normalization live
  under `crates/cli/src/ops/alert/`, while `alert.rs` preserves the adapter
  exports used by dispatch.
- Keep historical adapter names as facades when that avoids churn. Layout is
  now a facade over `crates/cli/src/ops/layout/watchlist.rs` and
  `crates/cli/src/ops/layout/pane.rs`; do not mix new watchlist and pane
  implementation bodies back into the facade file.
- Keep CDP-dependent Pine Editor operations in the CLI package, but split them
  by Editor sub-surface. `crates/cli/src/ops/pine/editor.rs` is now a facade
  over `runtime`, `source`, `scripts`, and `compile` modules. Desktop-free
  Pine static analysis and facade checks still belong in `crates/pine/`.
- Keep medium adapters behind the same facade pattern once they mix validation,
  reads, mutation, and payload shaping. Drawing, Replay, and chart-dependent
  Market now use same-named implementation directories under
  `crates/cli/src/ops/`. Do not gather new Drawing/Replay/Market operation
  bodies back into the facade files.
- Treat selected-chart historical export as Desktop-backed feasibility work
  until a contract proves the requested visible range and returned chart bars
  line up. Do not implement it as a fallback inside Desktop-free `tv bars`, and
  do not assume `tv range` changes the range returned by `tv ohlcv --count`.
  `tv ohlcv` selected-chart feasibility readback should stay additive and
  public-safe: `chart_context`, `returned_bars_range`, and
  `selected_chart_range_match` are diagnostics, not export guarantees.
- Once an adapter split exposes CDP-free request interpretation or validation,
  move that logic into `crates/model/` if it is reusable and not tied
  to clap or live page state. Drawing is the request-boundary example:
  `tradingview_model::drawing` owns the request structs and position validation, while
  `ops/drawing` owns shape creation, entity post-checks, reads, and cleanup.
- Keep generic UI automation safety-aware. `crates/cli/src/ops/ui.rs` is a
  facade over `dom`, `input`, `selectors`, and `eval`; do not move the
  `TV_ALLOW_UNSAFE_UI_EVAL` gate out of the application safety/dispatch layer
  or hide new unsafe behavior inside the adapter.
- Keep `crates/cli/src/main.rs` as a thin process entrypoint. Put CLI parsing,
  command dispatch, JSON envelope output, stream loops, input conversion, and
  target connection orchestration under `crates/cli/src/app/`.
- Put reusable command logic and transport helpers in root library modules
  rather than adding binary-only code to `crates/cli/src/main.rs`.
- Put cross-crate contract types in `crates/core/` only when they are small,
  low-dependency, and broadly shared. Current examples are typed errors, JSON
  envelopes, and exit-code mapping.
- Put shared I/O-free request models, validation, normalization, target
  resolution, and public-safe payload shaping in `crates/model/`. The model
  crate may use `tradingview-core` and `serde_json`, but it must stay free of
  network, CDP, clap, and UI dependencies.
- Put credential-free, Desktop-free market reads in `crates/market/` when they
  do not depend on CDP, chart state, or UI automation. Prefer typed result
  structs for reusable Rust APIs; keep JSON wrappers only for CLI payload
  compatibility. Document reusable typed APIs in rustdoc and
  `docs/rust-api.md`.
  Browserless historical `tv bars` is part of this boundary: the WebSocket
  read, request validation, payload shaping, and source-availability details
  live in `tradingview-market`, while CLI `ops` remains a thin command
  adapter. Keep the public `bars_symbol` facade stable and place internal
  bars responsibilities in same-named private modules such as validation,
  protocol, transport, payload, and types.
- Put credential-free, Desktop-free scanner reads in `crates/scanner/` when
  they can be exercised without TradingView Desktop. Prefer typed result
  structs for reusable Rust APIs; keep JSON wrappers only for CLI payload
  compatibility. Document reusable typed APIs in rustdoc and
  `docs/rust-api.md`.
- Put Desktop-free Pine helpers in `crates/pine/` when they are local source
  analysis or Pine facade checks. Keep Pine Editor operations in the CLI
  package because they depend on CDP, Monaco, and visible TradingView UI state.
- Put shared TradingView Desktop CDP connection code in `crates/cdp/`. Do not
  duplicate target discovery, `RuntimeEvaluator`, screenshot/input event
  primitives, or target handoff helpers inside operation modules.
- Put shared TradingView Desktop app-window helpers in
  `crates/cli/src/ops/desktop.rs` when multiple operation adapters need the
  same Desktop shell behavior, such as app-tab reads or new-tab launcher
  clicks. Keep product-specific launch behavior, such as opening the Screener
  tile from the Desktop new-tab page, in the owning operation adapter.
- Keep each library crate's `lib.rs` as a facade. When implementation grows,
  split into same-directory modules rather than gathering everything in
  `lib.rs`.
- For Desktop-free read crates such as `tradingview-market` and
  `tradingview-scanner`, prefer splitting a grown read surface into field or
  request selection, endpoint request construction, and response normalization
  modules before release. Keep the crate-level public API stable and expose the
  split modules only when a later plan intentionally makes them reusable.
- Do not move chart-dependent market reads, Screener code, account mutation,
  or UI automation into another workspace crate merely because they are
  reusable in theory. Extract them only when a concrete follow-up plan proves
  the boundary and dependency set are useful.
- Before extracting more `ops` code, consult
  `docs/operation-adapter-boundaries.md`. Keep executable TradingView work in
  `ops` when it needs CDP/runtime access, page-session APIs, storage fetch/save,
  DOM/UI fallback, live chart state, or post-checks.
- Do not create a generic `ops` crate just to move files. Current `ops` modules
  are operation adapters inside the CLI package. Split large modules internally
  first, then extract domain-specific crates only when their dependency
  boundary is clear.
- Treat the workspace library crates as internal and unstable until a future
  plan explicitly defines a stable Rust API.
- Keep helpers as private as possible; use `pub(super)` for sibling operation
  modules when needed.
- Avoid unrelated cleanup while migrating commands or fixing behavior.

## Integration test organization

Large CLI contract suites should be split by command family. Keep shared
integration-test helpers under `crates/cli/tests/support/`, keep root-level
CLI contracts in `cli_contract.rs`, and add focused `cli_contract_*` test
targets when a command family grows.

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

Some live checks are available as ignored integration tests. They are opt-in
only and must not become CI requirements. For chart-source quote endurance
checks, build the CLI and run:

```bash
TV_LIVE_CHART_QUOTE_SMOKE=1 cargo test -p tradingview-cli --test live_chart_quote -- --ignored --nocapture
```

Optional environment variables:

- `TV_LIVE_CHART_QUOTE_SYMBOLS`: comma-separated public symbols, defaulting to
  `PLUG,AAPL,MSFT,IONQ,MU,PLUG`.
- `TV_LIVE_CHART_QUOTE_RUNS`: positive repeat count, defaulting to `1`.
- `TV_LIVE_CHART_QUOTE_TARGET_ID`: explicit CDP target id when multiple chart
  targets are open. Do not paste live target ids into tracked docs.

The ignored test validates public-safe summary fields only: requested symbol,
observed quote symbol, chart symbol, `freshness_check`, stable sample count,
and restore status. Switched-symbol reads require at least two stable samples;
same-symbol fast-path reads may report one stable sample because no chart
switch occurred.

For Desktop quote-session extended-hours evidence checks, run this only during
the relevant market phase:

```bash
TV_LIVE_QUOTE_SESSION_SMOKE=1 TV_LIVE_QUOTE_SESSION_EXPECT_PHASE=postmarket cargo test -p tradingview-cli --test live_quote_session_extended_hours -- --ignored --nocapture
```

Do not run the postmarket or premarket smoke early and treat the result as
evidence. If the observed quote-session phase does not match
`TV_LIVE_QUOTE_SESSION_EXPECT_PHASE`, the test prints
`phase_result=not_yet_in_expected_phase`; that is only a timing guard telling
you to wait for the relevant U.S. session.

Optional environment variables:

- `TV_LIVE_QUOTE_SESSION_TARGET_ID`: explicit CDP target id when multiple
  chart targets are open. Do not paste live target ids into tracked docs.
- `TV_LIVE_QUOTE_SESSION_SYMBOL`: scanner quote symbol, defaulting to `OKLO`.
- `TV_LIVE_QUOTE_SESSION_QUALIFIED_SYMBOL`: quote-session symbol, defaulting
  to `NYSE:OKLO`.
- `TV_LIVE_QUOTE_SESSION_CHART_SYMBOL`: optional current-chart symbol for
  quote-session variants. If omitted, the test tries to read the current chart
  symbol through chart-source quote.
- `TV_LIVE_QUOTE_SESSION_EXPECT_PHASE`: optional expected
  `market-status.phase`, such as `postmarket` or `premarket`. The test treats
  TradingView's hyphenated phase names `post-market` and `pre-market` as
  aliases for those expected values.

The ignored test compares scanner-backed extended-hours fields with selected
TradingView Desktop quote-session fields. Scanner equality is not required:
scanner reads may be delayed while Desktop quote-session values may be
streaming or entitlement-dependent. The test prints only public-safe selected
field summaries and must not become a CI requirement.

For explicit Desktop quote-data source contract checks, run:

```bash
TV_LIVE_QUOTE_DATA_SMOKE=1 cargo test -p tradingview-cli --test live_quote_data_source -- --ignored --nocapture
```

Optional environment variables:

- `TV_LIVE_QUOTE_DATA_TARGET_ID`: explicit CDP target id when multiple chart
  targets are open. Use `tv tab list` to choose the target, but do not paste
  live target ids into tracked docs.
- `TV_LIVE_QUOTE_DATA_SYMBOL`: public symbol to pass to
  `tv quote <SYMBOL> --source quote-data`, defaulting to `NASDAQ:RKLB`.
- `TV_LIVE_QUOTE_DATA_RUNS`: positive repeat count, defaulting to `1`.
- `TV_LIVE_QUOTE_DATA_EXPECT_PHASE`: optional reporting hint such as
  `postmarket` or `premarket`. The test reports observed quote-data phase
  fields when a success payload is available; phase equality is not a scanner
  comparison.
- `TV_LIVE_QUOTE_DATA_ALLOW_UNAVAILABLE`: defaults to `1`. Set to `0` only
  when you expect a matching `qsd.rtc` frame during the bounded window.

The ignored test validates public contract fields only. A bounded no-frame
result is acceptable by default when it returns structured
`internal_api_unavailable` details with `raw_frame_included: false`. Do not
paste raw WebSocket frames, live payloads, or target ids into tracked docs.

For a multi-target premarket check, keep the target id as a local environment
value and record only public-safe summaries:

```bash
TV_LIVE_QUOTE_DATA_SMOKE=1 \
  TV_LIVE_QUOTE_DATA_TARGET_ID=<ID> \
  TV_LIVE_QUOTE_DATA_EXPECT_PHASE=premarket \
  TV_LIVE_QUOTE_DATA_SYMBOL=NASDAQ:RKLB \
  TV_LIVE_QUOTE_DATA_ALLOW_UNAVAILABLE=1 \
  cargo test -p tradingview-cli --test live_quote_data_source -- --ignored --nocapture
```

For chart-source quote concurrency checks, run:

```bash
TV_LIVE_CHART_QUOTE_CONCURRENCY_SMOKE=1 cargo test -p tradingview-cli --test live_chart_quote_concurrency -- --ignored --nocapture
```

Optional environment variables:

- `TV_LIVE_CHART_QUOTE_CONCURRENCY_SYMBOLS`: comma-separated public symbols,
  defaulting to `PLUG,AAPL,MSFT,IONQ,MU,PLUG`.
- `TV_LIVE_CHART_QUOTE_CONCURRENCY_RUNS`: positive repeat count, defaulting to
  `1`.
- `TV_LIVE_CHART_QUOTE_CONCURRENCY_TARGET_ID`: explicit CDP target id when
  multiple chart targets are open. Do not paste live target ids into tracked
  docs.
- `TV_LIVE_CHART_QUOTE_CONCURRENCY_WIDTH`: number of near-concurrent child
  `tv quote <SYMBOL> --source chart` processes per batch, defaulting to `2`.

The ignored test checks whether near-concurrent chart-source quote processes
serialize cleanly or expose mismatch/restore failures. It validates
public-safe summary fields only and must not become a CI requirement.

For `tv observe chart` JSONL contract checks, run:

```bash
TV_LIVE_OBSERVE_CHART_SMOKE=1 cargo test -p tradingview-cli --test live_observe_chart -- --ignored --nocapture
```

Optional environment variables:

- `TV_LIVE_OBSERVE_CHART_TARGET_ID`: explicit CDP target id when multiple chart
  targets are open. Do not paste live target ids into tracked docs.
- `TV_LIVE_OBSERVE_CHART_DURATION_MS`: bounded observation duration,
  defaulting to `3000`.
- `TV_LIVE_OBSERVE_CHART_HEARTBEAT_MS`: heartbeat interval, defaulting to
  `1000`.
- `TV_LIVE_OBSERVE_CHART_MAX_EVENTS`: optional sample event cap.

The ignored test validates public-safe JSONL summaries only: the first event is
readiness, later events use `command: "observe"`, sample events are bar stream
samples, heartbeat events preserve sample counts, the final summary event
reports counts and end reason, and source metadata marks the events as
Desktop-backed non-mutating reads.

The `v0.18` JSONL observation contract keeps these events additive and
public-safe: `observe_chart.v1` marks observe readiness / sample / heartbeat /
summary events, `stream.v1` marks lower-level stream sample / heartbeat /
summary events, and source metadata plus bounded controls stay intact. Summary
events are observation-window readbacks, not market-data samples. Do not paste
raw JSONL live output, target ids, raw WebSocket frames, account-local
metadata, or local validation paths into tracked docs.

For bounded Desktop-free watch compare contract checks, run:

```bash
cargo test -p tradingview-cli watch -- --nocapture
cargo test -p tradingview-cli --test cli_contract_quote watch -- --nocapture
```

`tv watch compare <SYMBOL>...` emits JSONL readiness, sample, heartbeat, and
summary events with `contract_version: "watch_compare.v1"` and scanner-backed
source metadata. If live smoke is attempted, record only public-safe summary
counts such as symbols, sample count, heartbeat count, poll count, end reason,
and source marker. Do not paste raw JSONL output into tracked docs.

For Replay extraction feasibility checks, treat `tv replay status` as a
Desktop-backed read and Replay controls as Desktop-backed operations. Payloads
should expose `replay_context`, selected-chart `chart_context` when available,
source metadata, and operation metadata without creating a stable export
command. If live smoke is attempted, record only public-safe fields such as
Replay started state, current date, operation, and whether Replay was stopped.
Do not paste raw DOM, raw payloads, target ids, account-local metadata, or
local absolute paths into tracked docs.

For `tv snapshot <SYMBOL>` live contract checks, run:

```bash
TV_LIVE_SNAPSHOT_SMOKE=1 cargo test -p tradingview-cli --test live_snapshot -- --ignored --nocapture
```

Optional environment variables:

- `TV_LIVE_SNAPSHOT_SYMBOLS`: comma-separated public symbols, defaulting to
  `NASDAQ:AAPL,NYSE:IONQ`.
- `TV_LIVE_SNAPSHOT_GROUPS`: comma-separated fundamentals groups to pass as
  repeated `--group` options.
- `TV_LIVE_SNAPSHOT_FIELDS`: comma-separated fundamentals fields to pass as
  repeated `--field` options.
- `TV_LIVE_SNAPSHOT_RUNS`: positive repeat count, defaulting to `1`.

The ignored test validates only public contract fields: source metadata,
requested symbol, section success/error shape, top-level error summaries, and
next-action hints. Do not paste raw snapshot output or live response payloads
into tracked docs.

For `tv compare <SYMBOL>...` live contract checks, run:

```bash
TV_LIVE_COMPARE_SMOKE=1 cargo test -p tradingview-cli --test live_compare -- --ignored --nocapture
```

Optional environment variables:

- `TV_LIVE_COMPARE_SYMBOLS`: comma-separated public symbols, defaulting to
  `NASDAQ:AAPL,NYSE:IONQ`.
- `TV_LIVE_COMPARE_RUNS`: positive repeat count, defaulting to `1`.

The ignored test validates only public contract fields: source metadata,
requested count, ordered items, section success/error shape, top-level error
summaries, and next-action hints. Do not paste raw compare output or live
response payloads into tracked docs.

For `tv bars` WebSocket contract evidence checks, run:

```bash
TV_LIVE_BARS_SMOKE=1 cargo test -p tradingview-cli --test live_bars -- --ignored --nocapture
```

Optional environment variables:

- `TV_LIVE_BARS_SYMBOLS`: comma-separated exchange-qualified public symbols,
  defaulting to `NASDAQ:AAPL,NYSE:IONQ`.
- `TV_LIVE_BARS_TIMEFRAME`: timeframe passed to `tv bars`, defaulting to
  `1D`.
- `TV_LIVE_BARS_COUNT`: positive bounded bar count, defaulting to `5`.
- `TV_LIVE_BARS_RUNS`: positive repeat count, defaulting to `1`.

The ignored test validates only public contract fields: `bars.v1` source
metadata, requested symbol, timeframe, bounded count, non-empty bars,
`summary`, `range`, `range_alignment`, `range_fetch_summary`,
`source_availability`, public-safe `wait_summary`, and `data_quality`. Do not
paste raw WebSocket output or live response payloads into tracked docs.

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
