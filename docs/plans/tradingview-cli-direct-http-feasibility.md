# Direct HTTP feasibility

This plan records the first direct HTTP feasibility investigation after the
upstream PR follow-up reached a stable closure boundary. It is research-only:
no command implementation or mutation path changes are part of this slice.

## Purpose / Big Picture

Some TradingView surfaces used by the Rust `tv` CLI are non-public endpoints or
page-session APIs. The scanner commands already show that direct HTTP reads can
be faster and simpler when authentication and active UI context are not needed.
This pass investigates whether more read-oriented operations can move from
TradingView Desktop CDP/page-session calls to direct HTTP without changing the
project's safety boundary.

The goal is not to build an authenticated TradingView API client. The goal is
to identify low-risk direct HTTP reads, if any, while keeping account mutation
and session-bound operations inside the user's own logged-in TradingView
Desktop page session.

## Progress

- [x] (2026-04-28) Re-checked upstream PRs and confirmed no new open PRs beyond
  the current 54-item inventory.
- [x] (2026-04-28) Searched current Rust code and docs for direct HTTP clients,
  page-session `fetch()` usage, scanner reads, symbol search, alert endpoints,
  and Pine facade usage.
- [x] (2026-04-28) Classified current direct HTTP candidates and found no
  additional implementation candidate for this slice.

## Decisions

- Prefer direct HTTP only for read-only surfaces that do not require extracting
  cookies, tokens, account ids, or active page state.
- Keep account mutations such as alerts, watchlists, saved Screener screens,
  Pine saves, and layout switches page-session backed unless a future plan
  proves a credential-safe alternative.
- Do not implement cookie import, session export, login automation, or token
  storage in this repository.
- Do not turn `docs/internal-tradingview-apis.md` into a third-party integration
  guide or copy-paste endpoint manual.

## Candidate Areas

| Area | Classification | Outcome |
| --- | --- | --- |
| `scanner hotlist` | `already direct HTTP` | Implemented through TradingView scanner preset REST reads. Use as the reference pattern. |
| `scanner scan` | `already direct HTTP` | Implemented through TradingView scanner REST reads with explicit markets, fields, filters, and sort validation. |
| `quote <SYMBOL>` | `already direct HTTP first` | Implemented through scanner REST before chart-switch fallback. Treat as done, not an open candidate. |
| `search <QUERY>` | `already direct HTTP` | Uses TradingView symbol-search HTTP and remains credential-free. |
| `pine check` | `already direct HTTP` | Posts source to the Pine facade compile endpoint without CDP or editor mutation. It remains read-only from the editor/account perspective. |
| Additional market discovery reads | `direct HTTP safe if evidence appears` | No concrete new endpoint or command surface was identified in this pass. Future work should start from a specific operator need. |
| Alert reads and mutations | `page-session only` | Alert list/create/delete depend on logged-in page-session endpoints and post-mutation readback. Do not move to standalone direct HTTP without a separate safety plan. |
| Watchlist and saved Screener mutation | `page-session only` | They depend on account/session state, saved-screen storage shape, or active custom watchlist context. |
| Pine saved-script list/open/save | `page-session or UI-bound` | Saved script metadata is account-linked; editor semantics remain tied to TradingView Desktop. |
| Visual chart/table reads, screenshots, data depth, generic UI | `intentionally UI-bound` | The visible UI or chart state is the observable contract. |

Do not start with:

- `alert create/delete` or `watchlist add/remove`: currently safer through the
  logged-in page session with readback post-checks.
- Screener saved-screen mutation: currently depends on page-discovered storage
  URL, schema version, active screen data, and test/disposable safety guards.
- Pine save/open/editor operations: tied to editor/session semantics.
- Generic UI, data depth, screenshots, tab new/close, or visible strategy
  fallbacks: their current UI dependency is part of the observable contract or
  still research-only.

## Investigation Steps

1. Review `docs/internal-tradingview-apis.md` and current scanner modules.
2. Search the codebase for direct HTTP clients and page-session `fetch` usage.
3. Classify each candidate as:
   - direct HTTP safe now
   - page-session only
   - intentionally UI-bound
   - not worth pursuing
4. For any direct HTTP candidate, define the exact read contract, error shape,
   rate/limit assumptions, and whether it changes the need for TradingView
   Desktop.
5. Create a separate implementation ExecPlan only for candidates that are
   read-only, credential-safe, and useful to core CLI workflows.

## Validation

This future slice should be docs/research first:

```bash
git diff --check
git grep -nE '(/Users/|C:\\|USER;|sessionid|cookie|authorization|bearer)' -- README.md CHANGELOG.md docs .agents/skills packaging scripts || true
```

If implementation follows, run the relevant Rust tests plus the standard
baseline. Direct HTTP implementation must not require live TradingView Desktop
unless the command remains explicitly page-session based.

## Outcome

No additional direct HTTP implementation candidate is selected from this pass.
The useful credential-safe direct HTTP reads are already represented by
`search`, `scanner hotlist`, `scanner scan`, symbol-targeted `quote`, and
`pine check`. The next implementation work should not invent a new direct HTTP
surface without a concrete command need and endpoint evidence.
