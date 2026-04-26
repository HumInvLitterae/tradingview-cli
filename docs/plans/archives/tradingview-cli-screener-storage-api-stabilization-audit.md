# Audit Screener storage and internal API stabilization paths

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document follows `.agents/PLANS.md`.

## Purpose / Big Picture

The implemented Screener surface works, but several commands still rely on visible DOM popovers and localized UI labels. Previous Screener work became more stable when it moved from DOM clicks to TradingView's saved Screener storage API: `screens delete` and `columns config/add/remove/reorder` are the clear examples. This audit checks whether the remaining fragile Screener operations can be stabilized the same way.

After this slice, a future contributor should know which Screener commands are already storage-backed, which are good candidates for storage/API replacement, and which should stay DOM-backed until new evidence appears.

## Progress

- [x] (2026-04-26 18:05Z) Reviewed current Screener implementation, stabilization note, and internal API usage across the repository.
- [x] (2026-04-26 18:12Z) Ran read-only full-page Screener evidence on `米国株（テスト用）`.
- [x] (2026-04-26 18:18Z) Added `docs/internal-tradingview-apis.md` as the safe internal API reference.
- [x] (2026-04-26 18:18Z) Recorded Screener storage/API stabilization findings in this ExecPlan.
- [x] (2026-04-26 18:32Z) Ran docs-only validation.
- [x] (2026-04-26 18:35Z) Updated `CONTINUITY.md`.
- [x] (2026-04-26 18:38Z) Committed the docs and audit changes without pushing.

## Surprises & Discoveries

- Observation: The active full-page Screener target exposes the storage URL, storage version, standalone type, and screen data through `window.initData`.
  Evidence: A read-only `ui eval` summary returned seven Screener-related `initData` keys, including storage URL/version, standalone type, and `screen_data`.

- Observation: The active saved screen storage payload includes filter information, not only column information.
  Evidence: A read-only active-screen storage fetch summary returned top-level keys for filters, default custom columns, watchlists, sort, market, view mode, id, title, and version. It reported 17 filters and 13 custom columns.

- Observation: The active storage filter shape is present but not yet enough to safely rewrite filter commands in this slice.
  Evidence: The read-only summary exposed a filter key union containing `type`, `subtype`, and `operation`, but this audit intentionally did not record raw filter payloads or attempt storage writes.

- Observation: Fetching the current standalone Screener bundle text from the page failed in this live session.
  Evidence: The read-only bundle keyword summary found one candidate script and returned `Failed to fetch`.

## Decision Log

- Decision: Keep this slice docs and read-only evidence only.
  Rationale: The storage payload proves filter data exists, but the safe mutation schema and post-check strategy require a separate implementation plan.
  Date/Author: 2026-04-26 / Codex

- Decision: Document internal TradingView APIs in a public-safe reference instead of hiding them.
  Rationale: The code already depends on these surfaces. A documented dependency inventory with safety boundaries is safer than scattered implicit knowledge, as long as it avoids credentials, account ids, and raw mutation recipes.
  Date/Author: 2026-04-26 / Codex

- Decision: Treat filter storage-backed mutation as the highest-value next candidate.
  Rationale: `filters add/modify/remove/clear` are the remaining Screener operations most affected by DOM popover fragility, and live storage evidence shows filters are part of the saved screen payload.
  Date/Author: 2026-04-26 / Codex

## Outcomes & Retrospective

This audit found a promising path for future filter stabilization: filters are present in the active saved screen storage payload. It did not prove a safe write schema, so no command implementation changed. The recommended next implementation slice is a bounded `filters storage-backed mutation feasibility` plan that starts with dry-run storage filter parsing and one reversible test-screen mutation only if the schema can be mapped without guessing.

Follow-up implementation in `docs/plans/archives/tradingview-cli-screener-filter-storage-mutations.md` adopted storage-backed `filters remove` and `filters clear` only. `filters add` and `filters modify` remain DOM/post-check guarded because they still require schema-aware filter payload creation or editing.

## Context and Orientation

The Rust CLI is the `tv` binary. Screener command parsing lives in `src/cli.rs`, dispatch lives in `src/main.rs`, and Screener operations live in `src/ops/screener.rs`. Screener commands use Chrome DevTools Protocol, abbreviated CDP, to evaluate JavaScript in a running TradingView Desktop page.

The phrase "storage API" in this plan means TradingView's logged-in saved Screener storage endpoint as discovered through `window.initData` in the page session. It is not a public stable API. The CLI must treat missing metadata, failed requests, and failed post-checks as `internal_api_unavailable`.

The full-page Screener target is preferred for evidence because it is a standalone Screener page exposed by `tv tab list` under `screener_targets`.

## Plan of Work

Create a durable internal API reference at `docs/internal-tradingview-apis.md`. Keep it high-level and public-safe: categories, dependent commands, read/write boundary, failure behavior, and mutation guards. Do not include credentials, live account ids, raw payloads, or direct mutation recipes.

Create this audit plan and record read-only evidence. Use the current full-page Screener target for `status`, `screens active`, `filters list`, and `columns config`. Use gated `tv ui eval` only for summarized key names, counts, and shape information from `window.initData` and the active saved-screen storage response.

Classify the remaining Screener commands:

- already storage/API-backed: `screens delete`, `columns config/add/remove/reorder`
- high-value storage/API candidates: `filters add/modify/remove/clear`
- possible but less-proven candidates: `screens create/rename/save-as/save/switch`, `columns reset`
- DOM-maintained boundaries: visible row reads, visible display-text reads, UI-only action discovery

Do not change command behavior in this slice.

## Concrete Steps

From the repository root, inspect state and find the full-page Screener target:

    git status --short
    tv tab list

Run read-only checks against the Screener target:

    TV_CDP_TARGET_ID=<screener-target> tv screener status
    TV_CDP_TARGET_ID=<screener-target> tv screener screens active
    TV_CDP_TARGET_ID=<screener-target> tv screener filters list
    TV_CDP_TARGET_ID=<screener-target> tv screener columns config

Run only read-only summarized `ui eval` checks:

    TV_ALLOW_UNSAFE_UI_EVAL=1 TV_CDP_TARGET_ID=<screener-target> tv ui eval '<read-only initData shape summary>'
    TV_ALLOW_UNSAFE_UI_EVAL=1 TV_CDP_TARGET_ID=<screener-target> tv ui eval '<read-only active storage shape summary>'

Do not run mutation smoke in this audit.

## Validation and Acceptance

Docs-only validation must pass:

    git diff --check
    git grep -nE '(/Users/|C:\\|USER;)' -- README.md CHANGELOG.md docs .agents/skills || true

Also scan the changed files manually for session credential wording, auth
header names, and token-like values. The tracked docs should describe those
categories without embedding live values or raw request headers.
    git status --short

Acceptance is that the repository contains a public-safe internal API reference, this plan records the Screener storage/API audit evidence, and no raw account-linked payloads or credentials are added.

## Idempotence and Recovery

All live commands in this audit are read-only. `tv ui eval` is gated by `TV_ALLOW_UNSAFE_UI_EVAL=1` but must only run expressions that summarize already-loaded page data or perform read-only fetches. If a read-only fetch fails, record the failure and do not retry with credentials or headers outside the page session.

If the full-page Screener target is not available, skip live evidence and mark the relevant findings `UNCONFIRMED` rather than using a chart-side panel as a substitute.

## Artifacts and Notes

Live read-only evidence from this slice:

    tv tab list
    # Found one full-page Screener target titled 米国株（テスト用）.

    tv screener status
    # Succeeded with open: true, screen_title: 米国株（テスト用）, 17 filters, 13 columns, and 100 visible rows.

    tv screener screens active
    # Succeeded with screen_title: 米国株（テスト用）.

    tv screener filters list
    # Succeeded with 17 visible filters. Filter names were recorded only as visible display labels in command output, not as raw storage payload.

    tv screener columns config
    # Succeeded with scope: screen_storage_api, active_column_set: custom, and 13 storage columns.

    read-only initData shape summary
    # Storage URL/version and standalone type were present. screen_data included active column set, default custom columns, filters, id, market settings, sort, title, version, view settings, and watchlists.

    read-only active storage shape summary
    # Active storage fetch succeeded. The high-level shape included filters, default custom columns, watchlists, sort, market, view mode, id, title, and version. Counts were 17 filters, 13 columns, and 0 watchlists.

    read-only bundle keyword summary
    # Candidate standalone Screener script fetch failed in this session, so bundle-level API method names remain unconfirmed here.

Docs-only validation from this slice:

    git diff --check
    # Passed.

    tracked-doc local path / USER marker / credential-keyword grep
    # Returned only existing validation-command examples in older plan documents.

    changed-file credential/token wording scan
    # No matches in the changed files.

## Interfaces and Dependencies

No CLI interface changes are introduced in this slice. No Rust code changes are required.

Future filter-storage work, if approved, should add a separate ExecPlan and start by adding internal helpers beside the existing storage-column helpers in `src/ops/screener.rs`. It must not reuse raw filter payloads from docs. It should parse the live storage shape at runtime, expose dry-run expected changes, guard normal writes to test/disposable screen names, and require a storage re-fetch post-check.

## Open Questions

- UNCONFIRMED: exact saved-screen filter write schema for each visible filter type.
- UNCONFIRMED: whether storage-backed filter changes immediately refresh visible Screener filters or require a page reload.
- UNCONFIRMED: whether screen create/rename/save-as can be safely represented through storage writes without hidden server-side side effects.
- UNCONFIRMED: whether `columns reset` has a default source outside the active saved screen payload.
