# Watchlist API-backed mutation feasibility

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document follows `.agents/PLANS.md`. It is self-contained so a future contributor can understand why the watchlist mutation path changed and how to validate it without reading chat history.

## Purpose / Big Picture

The user-facing goal is to make `tv watchlist add`, `tv watchlist remove`, and therefore `tv watchlist add-bulk` less dependent on TradingView's visible right-panel DOM. Before this change, add and remove needed the Watchlist sidebar to be open, found buttons by selector, sent CDP input events, and verified visible rows afterward. After this change, add and remove prefer TradingView's logged-in page-session symbols-list API, target the active saved watchlist, and verify the mutation by re-fetching the active list. If the internal API is not available, the previous DOM path remains as a fallback.

## Progress

- [x] (2026-04-27) Read the existing Rust watchlist implementation in `src/ops/layout.rs` and confirmed `watchlist get` is visible-DOM readback while `watchlist add/remove` were DOM mutation paths.
- [x] (2026-04-27) Reviewed upstream PR #89 evidence. The upstream branch records REST-backed watchlist list, switch, append, remove, create, rename, and delete operations under TradingView's symbols-list API family.
- [x] (2026-04-27) Took read-only live evidence from an authenticated TradingView page session. The page exposes a watchlist URL hint and the symbols-list all endpoint returned a list summary with one active custom list and symbol arrays.
- [x] (2026-04-27) Implemented API-backed `watchlist add` and `watchlist remove` as the preferred path, with DOM fallback only when the API list or active list cannot be used.
- [x] (2026-04-27) Added unit coverage for API success, already-present, absent-symbol validation, and DOM fallback.
- [x] (2026-04-27) Live-smoked API-backed add/remove with one disposable symbol. The symbol was added and then removed in the same run; no residual watchlist entry is known.
- [x] (2026-04-27) Updated public-safe internal API docs, upstream PR notes, README, CHANGELOG, handoff notes, and `CONTINUITY.md`.
- [x] (2026-04-27) Validation passed with `cargo fmt --check`, `cargo test watchlist -- --nocapture`, `cargo test --test cli_contract watchlist -- --nocapture`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`, `git diff --check`, and the tracked-doc path/credential grep. The grep returned only existing validation-command examples in plan documents.

## Surprises & Discoveries

- Observation: The currently loaded TradingView page exposes a watchlist URL hint on `window` and the authenticated symbols-list all endpoint can summarize saved watchlists without opening the sidebar.
  Evidence: read-only eval returned a successful list summary with a custom active list, symbol arrays, and no need for visible Watchlist DOM.

- Observation: API-backed mutation can be verified without relying on the right sidebar.
  Evidence: `tv watchlist add NASDAQ:CELH` returned `source: "watchlist_api"`, count increased by one, and `tv watchlist remove NASDAQ:CELH` returned `source: "watchlist_api"`, `remove_method: "api"`, and count returned to its prior value.

## Decision Log

- Decision: Keep `watchlist get` as visible-sidebar DOM readback.
  Rationale: `watchlist get` answers what is currently visible in the UI, including row display fields such as last/change values. Changing it to an account-list API read would silently alter the command's meaning.
  Date/Author: 2026-04-27 / Codex.

- Decision: Prefer API-backed add/remove, but keep DOM fallback for API-unavailable cases.
  Rationale: The API path is more stable and does not require button/input automation, but the old DOM path already works in some UI states and preserves backward behavior if the internal endpoint disappears or the active list is unsupported.
  Date/Author: 2026-04-27 / Codex.

- Decision: Treat API precheck absence for remove as a validation error and do not fall back to DOM.
  Rationale: Once the active watchlist is resolved through the API, removing an absent symbol should not try a different visible UI list and risk deleting from an unintended list.
  Date/Author: 2026-04-27 / Codex.

- Decision: Do not implement broad watchlist list/switch/create/rename/delete in this slice.
  Rationale: The requested reliability problem is add/remove/add-bulk. Broader watchlist lifecycle commands are account-state management and need their own safety design, especially delete.
  Date/Author: 2026-04-27 / Codex.

## Outcomes & Retrospective

The slice succeeded. `watchlist add` and `watchlist remove` now use an API-backed path when TradingView's logged-in symbols-list API is available, and both commands still verify their result before reporting success. `watchlist add-bulk` inherits the improved single-symbol path because it already calls `watchlist_add` sequentially. The main remaining boundary is that `watchlist get` intentionally remains visible-DOM readback, and broader watchlist list/switch/create/rename/delete surface remains future feature research rather than part of this reliability slice.

## Context and Orientation

The Rust CLI prints JSON envelopes such as `{ "success": true, "command": "watchlist", "data": ... }`. Most command logic lives under `src/ops/`. Watchlist and pane operations currently share `src/ops/layout.rs`.

Before this plan, `watchlist add` opened the Watchlist panel, clicked an add button, inserted text through CDP, pressed Enter and Escape, then scanned visible rows to confirm the symbol appeared. `watchlist remove` opened the Watchlist panel, found an exact visible row, revealed row controls, clicked a row remove button, then scanned visible rows to confirm absence. This was practical but fragile because TradingView changes DOM class names and lazy-renders sidebar widgets.

The replacement path uses TradingView's authenticated page session from inside the already logged-in page. It fetches a high-level saved-watchlist list, finds the active custom watchlist, sends a one-symbol append or remove request, then fetches the list again for post-check. The implementation deliberately avoids returning account-linked list ids in the CLI payload; it reports a public summary of the target list and count changes instead.

## Plan of Work

Update `src/ops/layout.rs` so `watchlist_add` and `watchlist_remove` first call API-backed helpers. Add `watchlist_add_via_api`, `watchlist_remove_via_api`, a shared `watchlist_mutate_via_api`, and normalization helpers. The API helper must:

- validate blank symbols before CDP connection using the existing validation behavior
- fetch the active watchlist list summary inside the page context
- refuse to use the API if no active custom list is available, and allow DOM fallback in that case
- return `already_present` for add when the symbol is already in the active list
- return a validation error for remove when the symbol is absent from the active list
- append or remove one symbol and then re-fetch the list to verify presence or absence
- return `source: "watchlist_api"` for API-backed success
- avoid falling back to DOM after an ambiguous post-check failure

Preserve the existing DOM implementation as fallback. Do not add new CLI flags.

Update documentation to record the boundary:

- `docs/internal-tradingview-apis.md` should classify watchlist add/remove as API-backed with DOM fallback and keep `watchlist get` as visible DOM readback.
- `docs/notes/upstream-pr-89-hidden-surface-audit-2026-04-25.md` should record that targeted add/remove has now been adopted, but broader list lifecycle remains future research.
- README and CHANGELOG should describe the user-visible reliability change.
- `docs/notes/next-agent-handoff-prompt-2026-04-24.md` should no longer call watchlist add/remove a top replacement candidate.

## Concrete Steps

From the repository root, inspect the current status:

    git status --short

Take read-only evidence with an explicit chart target:

    tv tab list
    TV_CDP_TARGET_ID=<chart-target> tv watchlist get
    TV_ALLOW_UNSAFE_UI_EVAL=1 TV_CDP_TARGET_ID=<chart-target> tv ui eval '<read-only summary expression>'

The read-only eval must summarize only key names, list counts, list types, and whether symbol arrays exist. Do not paste raw watchlist arrays, list ids, cookies, tokens, or live account names into tracked docs.

Implement the helper functions in `src/ops/layout.rs`. Then run:

    cargo test watchlist -- --nocapture
    cargo test --test cli_contract watchlist -- --nocapture

If those pass, run one bounded live smoke only when an explicit chart target is available. First confirm the disposable symbol is absent from the active API list, then add and remove the same symbol:

    TV_CDP_TARGET_ID=<chart-target> tv watchlist add <DISPOSABLE_SYMBOL>
    TV_CDP_TARGET_ID=<chart-target> tv watchlist remove <DISPOSABLE_SYMBOL>

Do not remove symbols that existed before the smoke.

## Validation and Acceptance

Acceptance for the code change is:

- `tv watchlist add <SYMBOL>` returns `source: "watchlist_api"` when the page-session API is available and the symbol was added or already present.
- `tv watchlist remove <SYMBOL>` returns `source: "watchlist_api"` and `remove_method: "api"` when the API removal succeeds.
- both add and remove verify the post-mutation state before success
- `tv watchlist add-bulk` continues to report per-symbol results and inherits the improved single-symbol add path
- if the API list cannot be used, the old DOM path remains available

Validation commands:

    cargo test watchlist -- --nocapture
    cargo test --test cli_contract watchlist -- --nocapture
    cargo fmt --check
    cargo clippy --all-targets --all-features -- -D warnings
    cargo test
    git diff --check
    git grep -nE '(/Users/|C:\\|USER;|sessionid|cookie|authorization|bearer)' -- README.md CHANGELOG.md docs .agents/skills || true

These validation commands passed on 2026-04-27. The grep output contained only existing validation-command examples in checked-in plan documents, not new live account data.

## Idempotence and Recovery

The code path is idempotent for `watchlist add`: if the symbol is already in the active API list, it returns `already_present` and does not append. `watchlist remove` is intentionally not idempotent: if the symbol is absent from the active API list, it returns a validation error and does not attempt DOM fallback.

If a live smoke add succeeds but remove fails, record the remaining symbol name in this plan and in `CONTINUITY.md`. Do not try to remove unrelated existing symbols. If the API endpoint stops working, the command should either fall back to the existing DOM path before any mutation or fail with `internal_api_unavailable` after an ambiguous mutation boundary.

## Artifacts and Notes

Read-only evidence summary:

    watchlist list endpoint: reachable from logged-in page session
    list summary: custom and colored lists are present; exactly one active list was observed
    active list: custom list with symbol array

Live smoke summary:

    add smoke: tv watchlist add NASDAQ:CELH
    observed: source watchlist_api, action added, count +1
    remove smoke: tv watchlist remove NASDAQ:CELH
    observed: source watchlist_api, remove_method api, count -1
    residue: none known

## Interfaces and Dependencies

The new helper functions live in `src/ops/layout.rs`:

    pub async fn watchlist_add_via_api(
        runtime: &mut impl RuntimeEvaluator,
        symbol: &str,
    ) -> Result<Value, AppError>

    pub async fn watchlist_remove_via_api(
        runtime: &mut impl RuntimeEvaluator,
        symbol: &str,
    ) -> Result<Value, AppError>

They use `RuntimeEvaluator::evaluate` with `await_promise: true` to run `fetch()` inside the authenticated TradingView page context. They do not add new crates. They keep the public CLI subcommands unchanged.

## Open Questions

Broader watchlist lifecycle management remains open:

- Should Rust add read-only `watchlist list` using the same API, or would that confuse the visible-sidebar meaning of `watchlist get`?
- Should Rust add guarded `watchlist switch/create/rename/delete`, and if so what safety gates should destructive delete require?
- Should future commands allow targeting a named watchlist instead of only the current active list?

These questions are intentionally outside this slice.
