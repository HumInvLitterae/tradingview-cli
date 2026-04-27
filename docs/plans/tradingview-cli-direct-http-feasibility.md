# Direct HTTP feasibility after next release

This plan records a future investigation. It should not be implemented before
the next release unless the user explicitly reprioritizes it.

## Purpose / Big Picture

Some TradingView surfaces used by the Rust `tv` CLI are non-public endpoints or
page-session APIs. The scanner commands already show that direct HTTP reads can
be faster and simpler when authentication and active UI context are not needed.
After the next release, investigate whether more read-oriented operations can
move from TradingView Desktop CDP/page-session calls to direct HTTP without
changing the project's safety boundary.

The goal is not to build an authenticated TradingView API client. The goal is
to identify low-risk direct HTTP reads, if any, while keeping account mutation
and session-bound operations inside the user's own logged-in TradingView
Desktop page session.

## Decisions

- Defer this investigation until after the next release.
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

- Scanner reads: use as the reference model for direct HTTP, because these are
  already direct read commands.
- Symbol-targeted quote reads: `tv quote <SYMBOL>` now uses the scanner REST
  API as a non-mutating read before any chart-switch fallback. Future direct
  HTTP work should treat this as an implemented reference path, not an open
  candidate.
- Additional market-discovery reads: consider only if they are unauthenticated
  or work without copying browser session credentials.
- Read-only metadata endpoints: consider only if they do not expose
  account-linked ids or require active page state.

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

## Open Questions

- Which additional TradingView read endpoints work without authenticated page
  credentials?
- Would direct HTTP reads provide enough latency or reliability improvement to
  justify another command surface?
- Can any candidate avoid exposing or storing account-local identifiers in
  outputs, docs, tests, and logs?
