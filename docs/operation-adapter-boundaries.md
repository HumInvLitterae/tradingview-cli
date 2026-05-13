# Operation Adapter Boundaries

This document records how the repository treats the remaining `ops` layer after
the workspace split and `tradingview-model` extraction.

The short version: `ops` is not leftover CLI surface. It is the executable
TradingView adapter layer. Code should leave `ops` only when the new location
has a clearer dependency boundary than live TradingView execution.

For user-facing source terminology, use `docs/command-source-taxonomy.md`.
That document classifies commands by source and side-effect boundary while this
document explains implementation placement. The project keeps one `tv` binary
for now; Desktop-free and Desktop-backed commands are separated by taxonomy,
not by executable name.

## Layer model

The current internal layers are:

- `tradingview-core`: typed errors, JSON envelopes, and exit-code mapping.
- `tradingview-model`: I/O-free request models, validation, target resolution,
  payload shaping, and fallback policy.
- `tradingview-market`, `tradingview-scanner`, and `tradingview-pine`:
  credential-safe direct reads or local/static analysis that do not require
  TradingView Desktop.
- `tradingview-cdp`: CDP connection, target discovery, runtime evaluation,
  screenshot capture, and input event primitives.
- `tradingview-cli` `ops`: executable adapters that call TradingView Desktop,
  page-session APIs, storage APIs, DOM/UI surfaces, or direct client crates and
  then shape the command result.

Do not create a generic `tradingview-ops` crate. A crate boundary should prove
that the extracted code is reusable without dragging along CLI command enums,
CDP runtime objects, page-session state, or DOM behavior.

## Placement rules

Move logic to `tradingview-model` when it is:

- validation or request interpretation;
- selector, target, or storage payload resolution over already-read data;
- public-safe payload normalization or shaping;
- fallback-policy decisions that do not execute a fallback;
- independent of clap, CDP, reqwest, page-session APIs, DOM, and live chart
  state.

Move logic to a service/client crate only when it is:

- credential-safe and usable without TradingView Desktop;
- a direct read or local analysis path with stable enough inputs and outputs;
- useful outside a single CLI operation adapter.

Keep logic in `ops` when it:

- calls CDP or a `RuntimeEvaluator`;
- reads active chart, Replay, Pine Editor, Screener, watchlist, or layout state;
- executes page-session APIs or saved storage fetch/save calls;
- clicks DOM, sends keyboard/mouse input, or computes visible geometry;
- performs mutation post-checks against live TradingView state;
- exists to preserve generic UI automation compatibility.

## Current adapter classification

The following command families should stay in `ops` as executable adapters for
now:

- Chart state commands: `state`, current-chart `info`, `symbol`, `timeframe`,
  `type`, `range`, `scroll`, and chart-sourced quote reads. These depend on
  live chart page objects and chart readiness. `tv quote <SYMBOL>` defaults to
  scanner REST, `--source chart` explicitly uses this chart adapter, and
  `--source auto` is a chart-first compatibility mode that falls back to
  scanner only if chart access fails before any chart mutation. Core
  Desktop-backed read payloads expose `source_category:
  "desktop_backed_read"`, `requires_desktop: true`, and `non_mutating` so
  agents can distinguish reads from chart/account operations.
- OHLCV reads: `ohlcv` reads the active chart's main-series bars. Symbol-level
  quote and info reads are Desktop-free. Historical bars now have a separate
  stable `tv bars <SYMBOL>` command through an undocumented WebSocket
  chart-session path, but `ohlcv` remains chart-dependent. Raw and summary
  OHLCV payloads retain `source:
  "direct_bars"` and report Desktop-backed read metadata.
- Desktop-free market and scanner reads: `search`, symbol-targeted `info`,
  scanner-source `quote`, `quotes`, `fundamentals`, `scanner scan`,
  `scanner hotlist`, and `scanner metainfo` live in the reusable market and
  scanner read crates. Their success payloads expose `source_category:
  "desktop_free_read"`, `requires_desktop: false`, and `non_mutating: true`
  so agents can keep REST evidence separate from Desktop chart evidence.
- Drawing and indicator chart operations: these execute chart APIs and verify
  newly created, updated, hidden, or removed entities.
- Replay operations: these use Replay page APIs and chart-local Replay state.
- Pine Editor operations: editor source, save/open/new, compile, errors, and
  console depend on Monaco/editor UI state. Pine static analysis and `pine
  check` stay in `tradingview-pine`.
- Data reads from chart, drawing, strategy, Depth of Market, and visible
  panels: these intentionally read live chart/page state.
- Stream reads: `tv stream ...` repeatedly samples live chart/page state and
  emits JSONL observation events. Bounded controls such as `--duration-ms`,
  `--max-events`, and `--heartbeat-ms` belong in the CLI stream runner because
  they manage process observation behavior rather than reusable domain logic.
  Sample and heartbeat events carry `source: "desktop_chart_stream"`,
  `source_category: "desktop_backed_read"`, `requires_desktop: true`, and
  `non_mutating: true`.
- Screenshots: chart-region screenshots use DOM geometry before CDP screenshot
  capture. This is intentional visual evidence, not a TradingView data API.
  Screenshot payloads are Desktop-backed reads with `non_mutating: true`,
  `writes_file: true`, and `visual_evidence: true`.
- Generic `ui` commands: these are compatibility automation by definition and
  should not become a broader domain API.
- Launch, status, and tab diagnostics: these are process/CDP/app-window
  boundaries rather than reusable model logic.

The following adapters already use safer API or storage paths where evidence
exists, but remain in `ops` because they still execute live page-session work
and post-checks:

- Watchlist add/remove/add-bulk: prefer the logged-in symbols-list API and
  keep DOM fallback only before mutation.
- Alert list/create/delete and indicator-alert creation: use alert endpoints
  where possible and verify through readback.
- Screener screen delete, column config/add/remove/reorder, and filter
  remove/clear: use saved-screen storage payloads and exact post-checks.

## Replacement candidates

Future stabilization should prefer API or storage evidence before adding DOM
retries. The current high-value candidates are:

- Screener filters add and option modify: storage schema evidence may remove
  remaining popover clicks in the future. The 2026-04-29 bounded audit
  implemented storage-backed numeric range modify for simple `Condition`
  filters selected by index, but add and option modify remain `research_only`.
- Screener screens create, rename, save-as, save, and switch: storage or
  command evidence may reduce catalog/menu/dialog dependency.
- Pine compile/save replacement: keep as `research_only` unless a safe endpoint
  preserves the same editor/account semantics without raw account metadata.
- Browserless historical bars: comparable-project evidence exists through an
  undocumented TradingView WebSocket chart-session protocol. The Rust CLI now
  has a bounded stable `tv bars <SYMBOL>` command, but it remains separate
  from `tv ohlcv` and does not guarantee realtime or entitlement status.

The scanner REST watchlist-style read lane is not a current replacement
candidate because it is already practically covered by `scanner scan`,
extended-hours columns, `scanner metainfo`, `quote`, `quotes`, and
scanner-backed `fundamentals` field groups. Add more scanner REST reads only
for a concrete workflow and clear endpoint evidence.

These are not current crate-extraction tasks. Each needs its own evidence-gated
ExecPlan before behavior changes.

For `v0.16.0`, `tv bars` is a stable CLI-owned Desktop-free historical bars
read rather than a `tradingview-market` typed API. It still depends on an
undocumented WebSocket chart-session protocol, so callers must keep its source
metadata and data-quality boundary visible. A broad diagnostic command remains
deferred until existing diagnostics prove insufficient. See
`docs/v0.5-roadmap.md` for the current roadmap.

For `v0.6.0`, source taxonomy and observation-first planning are recorded.
Existing Desktop-backed `tv stream ...` commands are current-chart JSONL
polling reads. They now support bounded observation controls and heartbeat
events, and the emitted events carry source taxonomy metadata, but they remain
Desktop-backed reads rather than browserless streams.
Future work may add browserless observation candidates, but it should not blur
the source categories or hide readiness failures.

`tv readiness` is the narrow Desktop-backed readiness read. It aggregates CDP
target discovery, selected-target handoff, chart API readiness, and a one-bar
OHLCV readiness check without switching symbols, activating tabs, or capturing
screenshots. If CDP is reachable but target selection or bars are not ready,
it returns `success: true` with `ready: false` and public-safe next-action
hints. Broad `tv diagnose` behavior remains deferred.

`tv screenshot` is the portable visual evidence follow-up when structured
fields do not explain the visible state. It does not mutate TradingView state,
but it writes the requested local output file, so payloads expose
`writes_file: true`.

For the operator-facing order of these reads, keep
`docs/observation-workflows.md` aligned with this boundary document: broad
screening starts with Desktop-free reads, chart observation starts with
`tv readiness` or `tv observe chart`, and screenshots are evidence follow-up
only when structured fields are insufficient.

Computer Use is not a general runtime dependency for these boundaries. Portable
agent guidance should use structured `tv` diagnostics and `tv screenshot` for
visual evidence. Computer Use may be mentioned only as an optional Codex app
aid when the current environment explicitly provides it.

## Intentional DOM boundaries

Do not start replacement work from these unless new evidence changes their
observable contract:

- `tv ui ...`, because it is generic UI automation.
- `data depth`, because it reads the visible Depth of Market panel.
- chart-region screenshot geometry, because it is about the visible chart
  rectangle.
- visible Strategy Tester fallback rows, because the fallback explicitly
  reports currently rendered rows.
- diagnostic UI-state reads, because their purpose is rendered UI
  troubleshooting.

## Shared helper candidates

If the next refactor is not API/storage replacement, prefer small helper
extractions over new crates. Good candidates are:

- page-session request wrappers that standardize error mapping without
  exposing raw payloads;
- shared Desktop app-window helpers when two adapters need the same app-tab or
  new-tab launcher behavior. The first helper lives in
  `crates/cli/src/ops/desktop.rs` and is used by `tab` and Screener full-page
  open;
- post-check helper patterns for mutation success boundaries;
- safe JavaScript serialization helpers;
- readiness diagnostics for chart, Screener, Pine Editor, and Replay state.

Do not extract a helper until at least two operation adapters use the same
pattern and tests can preserve the existing JSON contract.
