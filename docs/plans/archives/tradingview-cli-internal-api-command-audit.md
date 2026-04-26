# Audit internal API replacement candidates across commands

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document follows `.agents/PLANS.md`.

## Purpose / Big Picture

The Rust `tv` CLI now covers the known old JavaScript CLI surface and a broad Screener follow-up surface. Some commands still operate by reading or clicking TradingView's rendered DOM. DOM automation is sometimes the correct contract, but the recent Screener work showed that saved-screen storage APIs can be more reliable for durable account-state mutation.

After this audit, a contributor should know which existing commands are already using non-public APIs, which DOM-backed commands are worth a bounded replacement investigation, and which commands should intentionally stay DOM-backed. This slice does not implement replacements. It records the evidence and updates the public-safe internal API reference so future work does not add retries where an API-backed path is likely better.

## Progress

- [x] (2026-04-27 01:12Z) Checked the working tree and confirmed it was clean.
- [x] (2026-04-27 01:15Z) Searched `src/ops/` and top-level command code for DOM selectors, click paths, fetch calls, storage references, page objects, and CDP input events.
- [x] (2026-04-27 01:22Z) Reviewed representative modules: alerts, watchlist operations, Pine editor, tab operations, data depth, screenshots, scanner reads, indicators, drawings, replay, saved layouts, and Screener storage helpers.
- [x] (2026-04-27 01:34Z) Classified DOM-backed commands by replacement feasibility.
- [x] (2026-04-27 01:42Z) Updated the internal TradingView API reference with cross-command replacement guidance.
- [x] (2026-04-27 01:45Z) Updated handoff docs, README, CHANGELOG, and `CONTINUITY.md`.
- [x] (2026-04-27 01:49Z) Ran docs validation.
- [x] (2026-04-27 01:52Z) Committed the related tracked changes without pushing.

## Surprises & Discoveries

- Observation: `alert create` still opens and submits the visible alert dialog even though alert list and delete already use TradingView alert endpoints.
  Evidence: `src/ops/alert.rs` uses `pricealerts.tradingview.com/list_alerts` and `delete_alerts` for list/delete, but `alert_create` uses DOM dialog selectors and button clicks. This is a high-value API replacement candidate, but it needs live request-shape evidence before implementation because create payloads are more complex and account-mutating.

- Observation: Watchlist add/remove remain among the most DOM-heavy account mutations.
  Evidence: `src/ops/layout.rs` reads the visible right-panel rows and clicks add/remove controls, then uses CDP text/key input for add. No watchlist REST/storage endpoint is currently implemented in Rust. This is a high-value replacement candidate if a saved-list endpoint can be safely identified.

- Observation: Some DOM usage is not a problem to solve.
  Evidence: `screenshot --region chart` only uses DOM to compute the visible chart bounds before CDP screenshot capture, and `data depth` reads a visible DOM/Depth of Market panel by design because no structured data source is known. These should not receive generic retry logic unless new API evidence appears.

- Observation: Tab new/close are DOM-backed because they operate the TradingView Desktop app-window tab strip, not the chart page.
  Evidence: `src/ops/tab.rs` uses CDP target activation for chart target switching, but creates/closes app tabs by clicking `.tabs-container` controls in `/app/window/index.html`. A non-DOM application command may exist, but it is not present in the current Rust implementation.

## Decision Log

- Decision: Do not update runtime skills in this slice.
  Rationale: Skills should teach stable operator workflows. The command/API boundary should be audited first so skill text does not encode soon-to-change DOM-specific guidance.
  Date/Author: 2026-04-27 / Codex

- Decision: Treat `watchlist add/remove`, `alert create`, and Screener `filters add/modify` plus `screens create/rename/save-as/save/switch` as the highest-value replacement candidates.
  Rationale: These are durable account or saved-screen mutations where DOM timing and localization can cause practical failures, and nearby implemented code already proves related endpoint/storage surfaces exist.
  Date/Author: 2026-04-27 / Codex

- Decision: Keep `data depth`, `screenshot --region chart`, visible strategy fallback rows, and generic `ui` commands DOM-backed for now.
  Rationale: These commands either intentionally read visible UI state, need visible coordinates, or exist as compatibility automation. No safer structured API is known from current code evidence.
  Date/Author: 2026-04-27 / Codex

- Decision: This slice remains docs and research only.
  Rationale: API replacements require live request-shape evidence and account-state safety rules. Mixing implementation with the first cross-command audit would make the result harder to review.
  Date/Author: 2026-04-27 / Codex

## Outcomes & Retrospective

The audit classifies the current command surface into API-backed, high-value replacement candidates, research-only candidates, and intentional DOM boundaries. The most actionable next implementation plan is a bounded API evidence slice for either watchlist storage/API mutation or alert create endpoint mutation. Screener filter add/modify storage editing remains plausible but needs schema evidence before replacing the visible UI path.

## Context and Orientation

This repository implements a Rust-native `tv` command. Commands that need the running TradingView Desktop page use Chrome DevTools Protocol, abbreviated CDP, through `src/cdp.rs`. Most operation code lives under `src/ops/`. A DOM-backed command evaluates JavaScript that reads or clicks HTML elements such as buttons and rows. A page-session API-backed command calls a TradingView page object or endpoint from inside the logged-in page context, usually with `fetch(..., { credentials: 'include' })`. A direct HTTP command uses `reqwest` outside CDP.

The existing public-safe reference for non-public TradingView dependencies is `docs/internal-tradingview-apis.md`. That file must not include raw live payloads, account-linked ids, cookies, tokens, or copy-paste recipes for mutating another account. It can record categories, commands, read/write status, safety boundaries, and replacement candidates.

## Plan of Work

First, inspect code for current DOM and API usage. The useful searches are for `document.querySelector`, `querySelectorAll`, `.click()`, `dispatchMouseEvent`, `fetch(`, `initData`, known page objects such as `window.TradingViewApi`, and direct REST clients. Use the search result to classify behavior by command family rather than by every individual helper.

Second, update `docs/internal-tradingview-apis.md` so it is a true cross-command reference. Add a section that lists replacement feasibility. Keep it high-level and public-safe. Do not include live screen ids, script ids, alert ids, watchlist ids, cookies, authorization headers, or full request/response bodies.

Third, update handoff and user-facing docs to point future contributors at this audit before adding more DOM retry logic. The README should continue to link to the internal API reference without turning it into an integration guide.

Fourth, update `CONTINUITY.md` because this audit changes the recommended next work: before skill updates, pick a bounded API evidence slice for the most valuable candidate, or update skills only after deciding not to replace the relevant DOM paths.

## Concrete Steps

From the repository root, inspect state:

    git status --short

Search command dependencies:

    rg -n "document\\.querySelector|querySelectorAll|\\.click\\(|dispatchMouseEvent|Input\\.dispatch|fetch\\(|getSavedCharts|loadChartFromServer|initData|storage|pine-facade|_replayApi|createStudy|removeEntity" src/ops src/*.rs

Create this ExecPlan at:

    docs/plans/archives/tradingview-cli-internal-api-command-audit.md

Update:

    docs/internal-tradingview-apis.md
    docs/notes/next-agent-handoff-prompt-2026-04-24.md
    README.md
    CHANGELOG.md
    CONTINUITY.md

## Validation and Acceptance

Run docs validation:

    git diff --check
    git grep -nE '(/Users/|C:\\|USER;|sessionid|cookie|authorization|bearer)' -- README.md CHANGELOG.md docs .agents/skills || true
    git status --short

Acceptance is that the internal API reference clearly identifies at least these categories:

- API-backed today
- high-value replacement candidates
- research-only candidates
- intentional DOM boundaries

Acceptance also requires the next implementer to have a clear first implementation candidate without needing to re-read all code. Based on this audit, the recommended first implementation candidate is watchlist API/storage evidence if the user's priority is operator workflow reliability, or alert create endpoint evidence if the user's priority is replacing the remaining DOM-backed alert lifecycle command.

## Idempotence and Recovery

This slice edits docs only. It is safe to repeat the searches and docs validation. If the grep command finds only validation-command examples in old plan documents, record that as acceptable. If it finds a new live id, cookie, token, or local path in changed docs, remove that content before committing.

No live mutation smoke is part of this audit. If future work runs live evidence capture, use read-only discovery first and record only payload shapes and command categories, not raw account data.

## Artifacts and Notes

Static evidence summary after later replacement slices:

    alert list/create/delete: API-backed through pricealerts endpoints.
    watchlist add/remove: API-backed active custom watchlist mutation with DOM fallback before mutation only.
    screener storage: storage-backed for screen delete, filter remove/clear, and column add/remove/reorder.
    screener filters add/modify and screen create/rename/save-as/save/switch: DOM-backed or visible-UI-backed; storage replacement remains plausible but needs schema/action evidence.
    pine list/open/check: Pine facade and Monaco-backed.
    pine save/compile/raw-compile: still visible editor/button/shortcut operations; replace only with strong endpoint evidence.
    tab switch: CDP target activation; tab new/close: DOM app-window tab strip.
    data depth, screenshot chart, visible strategy table fallback, generic UI commands: intentional DOM boundaries for now.

No raw live payloads or account identifiers are included in this plan.

## Interfaces and Dependencies

No public CLI interface changes are introduced in this slice. No Rust code is changed.

Future implementation plans should name the exact command family they target and must preserve these boundaries:

- account-state mutations require dry-run where practical, test/disposable guards where appropriate, and post-action verification;
- page-session endpoints must be called only from the user's own logged-in TradingView page context unless the existing command already uses direct unauthenticated HTTP reads;
- future direct HTTP work should start with credential-safe read-only endpoints and must not introduce cookie import, session export, login automation, or token storage without a separate explicit decision;
- docs may describe endpoint categories and command dependencies but must not contain raw payloads or account-linked identifiers.

## Open Questions

- RESOLVED: watchlist add/remove can use the active custom watchlist symbols-list API with readback post-checks.
- RESOLVED: alert creation can use the alert endpoint family while preserving practical fields and readback post-checking created alerts.
- UNCONFIRMED: whether Screener `filters add` and `filters modify` can be storage-backed for all currently supported numeric and option cases without synthesizing unsafe payload internals.
- UNCONFIRMED: whether TradingView Desktop app tab creation/closure has a non-DOM command surface outside the visible app-window tab strip.
