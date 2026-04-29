# Operation adapter boundary audit

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This plan follows `.agents/PLANS.md` in this repository. It is self-contained and describes a documentation and boundary-classification slice after the `tradingview-model` crate extraction.

## Purpose / Big Picture

The repository now has clean internal crates for core contracts, I/O-free model logic, Desktop-free read clients, Pine static analysis, and CDP transport. The remaining question is what to do with `crates/cli/src/ops/`: whether more of it should move into crates, whether it should remain as operation adapters, and which UI-heavy commands should be replaced by safer non-public APIs or storage payloads.

After this change, future contributors can read one stable document to understand which operation adapters are intentionally retained in the CLI package, which pieces belong in `tradingview-model`, which pieces belong in service/client crates, and which UI or DOM dependencies are replacement candidates. This slice changes no CLI behavior.

## Progress

- [x] (2026-04-29) Inspected current operation adapter files, largest modules, and internal API reference.
- [x] (2026-04-29) Archived the completed model crate extraction plan.
- [x] (2026-04-29) Added a stable operation adapter boundary document.
- [x] (2026-04-29) Updated architecture, development, roadmap, changelog, plans index, and continuity ledger.
- [x] (2026-04-29) Ran docs validation and hygiene checks.
- [x] (2026-04-29) Committed the related changes as one documentation refactor.

## Surprises & Discoveries

- Observation: Most large remaining adapters are large because they execute live TradingView work, not because they still hide obvious model logic.
  Evidence: The largest files are Screener filters/screens, watchlist, indicator alerts, Screener engine/columns, alert create/delete, tabs, launch, panes, saved layouts, data reads, chart reads, and UI input. These use `RuntimeEvaluator`, page-session storage, DOM clicks, post-checks, or CDP target APIs.

- Observation: The strongest remaining replacement candidates are already concentrated in `docs/internal-tradingview-apis.md`.
  Evidence: Screener filters add/modify, Screener screen lifecycle, and app-tab new/close are listed as evidence-gated candidates; generic UI, screenshots, data depth, and visible strategy fallbacks are documented intentional DOM boundaries.

## Decision Log

- Decision: Do not create a generic `tradingview-ops`, `tradingview-ui`, or `tradingview-account` crate in this slice.
  Rationale: The remaining operation adapters are coupled to live TradingView execution, page-session APIs, DOM/UI fallback, active chart state, and post-check behavior. Moving them into a crate now would mostly move complexity rather than clarify responsibility.
  Date/Author: 2026-04-29 / Codex.

- Decision: Keep `ops` as the executable adapter layer inside `tradingview-cli`.
  Rationale: `ops` is the boundary that turns validated command/model data into live TradingView effects. That boundary remains useful even after `tradingview-model` extraction.
  Date/Author: 2026-04-29 / Codex.

- Decision: Future work should prioritize replacement evidence before more UI retries.
  Rationale: Past stability gains came from watchlist, alert, and Screener storage/API paths. Remaining DOM-heavy commands should first be checked for API/storage alternatives when the command is not intentionally about visible UI.
  Date/Author: 2026-04-29 / Codex.

## Outcomes & Retrospective

This slice leaves the code unchanged and clarifies the next refactoring boundary. The architecture is now:

    cli/app dispatch -> tradingview-model for I/O-free validation and shaping
    cli/app dispatch -> ops for executable TradingView work
    ops -> cdp/page-session/DOM/storage/direct clients as needed

The next useful implementation work is not another mechanical crate split. It is either a focused API/storage replacement slice for a named command family, or a small shared helper extraction where repeated execution patterns are already proven.

## Context and Orientation

The workspace has these internal crates:

- `tradingview-core`: typed errors, JSON envelopes, and exit codes.
- `tradingview-model`: I/O-free request models, validation, target resolution, payload shaping, and fallback policy.
- `tradingview-market`: Desktop-free symbol search, symbol info, and quote reads.
- `tradingview-scanner`: Desktop-free scanner HTTP reads.
- `tradingview-pine`: Desktop-free Pine static analysis, alertcondition discovery, and Pine check helpers.
- `tradingview-cdp`: CDP client, target discovery, runtime evaluation, screenshot capture, and input events.
- `tradingview-cli`: CLI surface, application dispatch, and live operation adapters under `crates/cli/src/ops/`.

In this plan, an operation adapter is code that executes a `tv` command against TradingView. It may call CDP, page-session endpoints, storage APIs, DOM selectors, mouse/keyboard input, or direct read clients. It is different from `tradingview-model`, which has no I/O and does not know about clap, CDP, reqwest, or DOM.

## Plan of Work

Add a root documentation file, `docs/operation-adapter-boundaries.md`, that classifies the remaining operation adapter families. The document should name what stays in `ops`, what belongs in `tradingview-model`, what belongs in service/client crates, and what needs API/storage research before replacement.

Update `docs/architecture.md` to link to the new boundary document instead of trying to hold every detailed classification inline. Update `docs/development.md` with the practical rule: do not create crates for UI/live-state execution just to shorten `ops`; first extract I/O-free model logic or prove a safer API/storage path. Update `docs/v0.3-roadmap.md` to say the current crate split is a good release boundary and the next work is adapter stabilization/replacement evidence or release readiness.

Update `docs/plans/README.md` so this plan is current and the model crate extraction is archived. Update `CHANGELOG.md` with a documentation/refactor note. Update `CONTINUITY.md` as the local ledger, but do not commit it.

## Concrete Steps

Run docs validation:

    git diff --check
    rg -n '(/Users/|C:\\|USER;|sessionid|cookie|authorization|bearer)' README.md CHANGELOG.md docs .agents/skills packaging scripts || true

Because this slice does not change Rust code, `cargo test` is not required. If code changes are introduced unexpectedly, run:

    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace

Commit:

    git add CHANGELOG.md docs
    git commit -m "docs(architecture): Record operation adapter boundaries"

`CONTINUITY.md` is a local ledger and is intentionally not staged. Do not push.

## Validation and Acceptance

The change is accepted when:

- `docs/operation-adapter-boundaries.md` exists and classifies current `ops` surfaces by placement and replacement policy;
- architecture and development docs point future contributors to the boundary policy;
- the roadmap records that more crate extraction is not the next default move;
- docs validation passes;
- no machine-specific paths, account-local ids, cookies, tokens, or raw live payloads are added to tracked docs.

## Idempotence and Recovery

This is a docs-only slice. It is safe to repeat searches and validation. If a classification is uncertain, mark it `UNCONFIRMED` and keep the command in `ops` until a future evidence-gated ExecPlan proves a better path.

## Artifacts and Notes

Initial inspection:

    for f in crates/cli/src/ops/*.rs crates/cli/src/ops/*/*.rs; do [ -f "$f" ] && printf "%5d %s\n" "$(wc -l < "$f")" "$f"; done | sort -nr | sed -n '1,40p'

Largest remaining adapters include Screener filters/screens, watchlist, indicator alert create, Screener engine/columns, alert create/delete, tab, launch, pane, saved layout, data reads, chart reads, and UI input.

Validation evidence:

    git diff --check
    result: passed.

    rg -n '(/Users/|C:\\|USER;|sessionid|cookie|authorization|bearer)' README.md CHANGELOG.md docs .agents/skills packaging scripts || true
    result: only existing policy language, archived validation-command examples, and secret-safety wording were reported. No new machine-specific path, account-local id, cookie, token, or authorization value was added.

## Interfaces and Dependencies

No Rust interfaces change in this slice. The new stable documentation should define these placement rules:

- `tradingview-model`: I/O-free validation, request interpretation, target resolution, payload shaping, and fallback policy.
- service/client crates: credential-safe direct reads and source analysis that do not require TradingView Desktop.
- `tradingview-cdp`: CDP transport primitives only.
- `ops`: executable TradingView adapters that use runtime access, page-session APIs, storage fetch/save, DOM/UI fallback, live chart state, or post-checks.

## Open Questions

- Whether Screener filters add/modify can become fully storage-backed remains `UNCONFIRMED`.
- Whether Screener screen create/rename/save-as/save/switch can become storage/API-backed remains `UNCONFIRMED`.
- Whether app-tab new/close have a non-DOM command path remains `UNCONFIRMED`.
