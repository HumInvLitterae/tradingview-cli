# Operation Adapter Boundaries

This document records how the repository treats the remaining `ops` layer after
the workspace split and `tradingview-model` extraction.

The short version: `ops` is not leftover CLI surface. It is the executable
TradingView adapter layer. Code should leave `ops` only when the new location
has a clearer dependency boundary than live TradingView execution.

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
  `type`, `range`, `scroll`, and current-chart quote fallback. These depend on
  live chart page objects and chart readiness.
- OHLCV reads: `ohlcv` reads the active chart's main-series bars. Symbol-level
  quote and info reads are Desktop-free, but historical bars remain
  chart-dependent until a credential-safe endpoint is proven.
- Drawing and indicator chart operations: these execute chart APIs and verify
  newly created, updated, hidden, or removed entities.
- Replay operations: these use Replay page APIs and chart-local Replay state.
- Pine Editor operations: editor source, save/open/new, compile, errors, and
  console depend on Monaco/editor UI state. Pine static analysis and `pine
  check` stay in `tradingview-pine`.
- Data reads from chart, drawing, strategy, Depth of Market, and visible
  panels: these intentionally read live chart/page state.
- Screenshots: chart-region screenshots use DOM geometry before CDP screenshot
  capture.
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
- App-tab new and close: a non-DOM application command may exist, but exact
  target and tab-count post-check evidence is required.
- Pine compile/save replacement: keep as `research_only` unless a safe endpoint
  preserves the same editor/account semantics without raw account metadata.

These are not current crate-extraction tasks. Each needs its own evidence-gated
ExecPlan before behavior changes.

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
- post-check helper patterns for mutation success boundaries;
- safe JavaScript serialization helpers;
- readiness diagnostics for chart, Screener, Pine Editor, and Replay state.

Do not extract a helper until at least two operation adapters use the same
pattern and tests can preserve the existing JSON contract.
