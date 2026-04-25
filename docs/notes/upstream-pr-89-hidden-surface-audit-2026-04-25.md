# Upstream PR #89 hidden surface audit 2026-04-25

This note audits upstream pull request
https://github.com/tradesdontlie/tradingview-mcp/pull/89 after the Rust CLI
completed the known old JavaScript CLI migration and first release follow-up
fixes.

The PR title is `Add dependency injection to drawing functions and update tests`,
but the diff contains a wider fork bundle. The purpose of this note is to
separate Rust-relevant evidence from downstream workflow material and avoid
copying a large mixed JavaScript patch into the Rust CLI.

## Sources checked

- `gh pr view 89 -R tradesdontlie/tradingview-mcp --json number,title,body,files,updatedAt,url`
- `gh pr diff 89 -R tradesdontlie/tradingview-mcp`
- Rust comparison points: `src/ops/alert.rs`, `src/ops/layout.rs`,
  `src/ops/data/drawings.rs`, `src/ops/drawing.rs`,
  `src/ops/saved_layout.rs`, and current CLI contract tests.

## Summary classification

PR #89 is not one feature. It is a fork note plus several unrelated capability
patches:

- drawing dependency-injection regression fixes
- Pine label read default-cap and truncation reporting
- watchlist panel lazy-render handling
- alert create/delete REST rewrites
- watchlist REST management and targeted insert/remove
- TradingView hotlist scanner reads
- quote/data REST fallback evidence
- source-audit and sanitization test updates

Rust should not cherry-pick the PR. The small read-only data-label contract
improvement from this audit has been addressed in Rust: `tv data labels` now
defaults to 500 returned labels and reports whether results were truncated.
Mutation-heavy REST rewrites should be planned separately only after stronger
Rust-specific evidence.

## Capability findings

### Drawing dependency injection

Upstream fixes JavaScript functions that lost access to injected dependencies
after a sanitizer/refactor change. Rust does not share this structure. Drawing
commands use Rust operation functions and fake runtime tests rather than a
JavaScript dependency-injection wrapper.

Disposition: no Rust action now. Keep as a reminder that drawing commands should
continue to have operation-level tests for list/get/remove/clear behavior.

### Data labels default cap and truncation

Upstream reports that a default cap of 50 labels silently drops useful older
labels in dense indicators and adds a `truncated` signal.

Disposition: addressed in Rust. `tv data labels` now defaults to 500 labels when
`--max <N>` is omitted, preserves `--max <N>` as an override, and reports
per-study `available_labels`, `limit`, and `truncated` metadata so downstream
callers can tell when older labels were omitted.

### Watchlist lazy render and REST list management

Upstream reports two separate watchlist themes. First, `watchlist_get` can return
empty data when another sidebar panel is active because TradingView lazily
renders the watchlist DOM. Second, the PR adds REST-backed watchlist list,
switch, insert, remove, create, rename, and delete flows.

Rust already has `watchlist get/add/remove`. `watchlist add/remove` can open the
panel and now use coordinate-based mouse events plus post-action verification.
`watchlist get` remains a read command that reports the currently visible
sidebar state and does not force-open the watchlist panel.

Disposition: do not implement the large REST management surface now. The lazy
render evidence is real, but changing `watchlist get` to click/open panels would
make a read command mutate UI state. Treat a safer future option as research:
either add an explicit `--open-panel` mode or add a separate read note explaining
that `watchlist get` is a visible-sidebar read. Targeted REST insert/remove
could be useful later for downstream list automation, but it is account
mutation, not old CLI migration closure.

### Alert create/delete REST rewrites

Upstream replaces DOM alert creation with a REST call to TradingView's alert
endpoint and rewrites delete to support individual, bulk-id, and all-alert
deletion. Rust already implements `alert list`, `alert delete --id`, and
`alert delete --all` through internal alert endpoints with dry-run and
post-delete verification for bulk delete. Rust `alert create` still uses DOM
dialog automation.

Disposition: possible Rust bugfix research, not immediate implementation in
this audit. If live smoke shows Rust `alert create` cannot open or submit the
current dialog reliably, plan a dedicated alert-create REST slice. That slice
must decide the exact request body, CORS/header behavior, message/price parity,
and post-create verification before changing account mutation behavior.

### Hotlist scanner reads

Upstream adds a `hotlist_get` tool backed by TradingView scanner preset
endpoints. This is useful for downstream watchlist refresh workflows, but it is
not old CLI migration backlog and is closer to scanner/product workflow surface
than TradingView chart-control CLI surface.

Disposition: future feature research only. Do not add to the Rust core CLI
unless downstream workflow evidence shows it belongs in `tv` rather than a
separate scanner/helper.

### Quote/data REST fallback evidence

Upstream notes a quote/data mismatch where an input symbol could be echoed while
values came from the active chart, and proposes scanner REST fallback evidence.
Rust `tv quote`, `tv search`, and OHLCV reads are separate implementations and
should not be changed from this PR alone.

Disposition: no action now. Revisit only if Rust live smoke shows a concrete
symbol mismatch or current-chart leakage.

### Sanitization tests

Upstream adds JavaScript sanitizer regression coverage around injected
evaluators and escaped entity ids. Rust already routes user strings through
structured JSON serialization helpers such as `js_string`, and numeric inputs
through finite validation helpers where applicable.

Disposition: no direct action now. Keep using PR #33/#89 as inspiration when
touching user-input-to-JavaScript paths.

## Recommended next work

No immediate PR #89 implementation candidate remains after the `tv data labels`
hardening slice. Do not combine future work with alert REST rewrites, watchlist
REST management, hotlist reads, or layout unsaved-dialog policy changes without
separate evidence and a dedicated plan.

## Assumptions

- This audit classifies PR #89 only. The read-only data-label follow-up has been
  implemented separately in Rust.
- PR #89's fork notes contain downstream workflow needs that are useful evidence
  but not automatically core CLI scope.
- MCP server implementation remains not planned.
- Account mutation surfaces require separate design and live smoke plans before
  implementation.
