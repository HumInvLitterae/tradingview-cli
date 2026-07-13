# Make Pine Editor targeting and saved-script readback current-build compatible

This ExecPlan is a living document. The sections `Progress`, `Surprises &
Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to
date as work proceeds.

This document follows `.agents/PLANS.md` and must be maintained in accordance
with that file.

## Purpose / Big Picture

After this work, Desktop-backed Pine commands will identify the Pine Editor
that is actually visible on the selected TradingView chart instead of relying
on a legacy Monaco class or the first editor in a global list. In addition,
`tv pine open <NAME...>` will either use a bounded current-build path that can
verify the active saved-script slot or fail quickly with a public-safe stage
diagnostic. It must never report success from source injection or save from an
unverified binding.

The user can see the result by opening Pine Editor on the ELVN chart and
running `tv pine get` without exposing source in tracked evidence. The command
must find the visible editor. The already approved disposable-script matrix
using `Testスクリプト` and `Testスクリプト2` may resume only if a separate
active-slot probe proves that requested and observed internal identity can be
compared inside the page. If no trustworthy identity path exists on the
current Desktop build, this plan records a no-go and keeps `tv pine open`
fail-closed.

## Progress

- [x] (2026-07-14) Recorded the owner-approved ELVN live-matrix no-go from the
  saved-script binding plan without changing or saving either disposable
  script.
- [x] (2026-07-14) Confirmed through capability-only probes that the ELVN chart
  target has two Monaco instances, one visible, while the legacy
  `pine-editor-monaco` selector and test API active-store provider are absent.
- [x] (2026-07-14) Confirmed that the current test API exposes `openEditor` and
  `openScript`, but `openEditor()` neither settles nor makes active readback
  available within a bounded 15-second same-instance observation.
- [x] (2026-07-14) Compared the Rust runtime helper with upstream pull-request
  evidence and identified both an outdated editor selector and an unverified
  current-build saved-script provider boundary.
- [ ] Perform a bounded, public-safe feasibility probe for semantic visible
  Pine ownership and active saved-script identity on the current build.
- [ ] Record an explicit stop/go decision. Do not implement saved-script
  mutation when either editor ownership or active identity is ambiguous.
- [ ] On go, implement shared visible Pine Editor selection and bounded binding
  readback, then add deterministic Rust and executable JavaScript contracts.
- [ ] Run focused and full validation and obtain independent review.
- [ ] On reviewed go only, rerun the already authorized ELVN two-script matrix;
  modify and save only the intended disposable script and retain both scripts
  afterward as requested by the owner.
- [ ] Synchronize docs and skills, archive this plan when its stop/go outcome is
  complete, and return to the paused saved-script binding plan if live evidence
  becomes feasible.

## Surprises & Discoveries

- Observation: A current-build Pine Editor can be visibly open while the
  legacy selector `.monaco-editor.pine-editor-monaco` matches no usable editor.
  Evidence: The ELVN chart target contained two `.monaco-editor` elements and
  exactly one visible instance, while `tv pine get` reported that Monaco could
  not be found. Source text and DOM content were not retained.

- Observation: The Pine test API method names remain present, but method
  presence does not prove that its editor/store provider is initialized.
  Evidence: Capability-only inspection found the factory, `openEditor`, and
  `openScript`; the factory singleton had no active editor or store provider
  even while Pine Editor was visible.

- Observation: The current `pine_open_expression` awaits `openEditor()` and
  then calls the factory again. Upstream pull-request evidence retains one API
  instance, but even a retained-instance local probe did not settle or expose
  active readback on this build.
  Evidence: The normal command reached its bounded evaluation timeout. A
  separate same-instance probe started successfully but observed no settlement
  and no provider for 15 seconds. No saved-script open or save followed.

## Decision Log

- Decision: Split current-build compatibility into two independent proof
  obligations: visible Pine Editor ownership and active saved-script identity.
  Rationale: A correct Monaco editor is sufficient for source read/compile
  operations but does not prove which cloud saved-script slot a later Save will
  update. Conflating the two would recreate the overwrite risk this roadmap is
  intended to remove.
  Date/Author: 2026-07-14 / Codex.

- Decision: Treat method presence and a visible editor as insufficient for a
  saved-script go decision.
  Rationale: The live no-go demonstrated both conditions while the active
  provider was absent and the internal editor-open operation remained pending.
  A semantic identity comparison must be possible inside the page before
  `openScript` is called.
  Date/Author: 2026-07-14 / Codex.

- Decision: Move this work ahead of current-build indicator insertion.
  Rationale: The v0.28 roadmap already allows editor compatibility to move
  earlier when it blocks Pine binding validation. The owner-approved live
  evidence proved that blocker.
  Date/Author: 2026-07-14 / Codex.

- Decision: Preserve the owner's authorization for the two named disposable
  scripts, but do not treat it as permission to weaken verification.
  Rationale: Mutation authorization answers whether a safe test may modify the
  scripts. It does not make an ambiguous account-linked binding safe. The
  matrix remains gated on a reviewed implementation and verified readback.
  Date/Author: 2026-07-14 / Codex.

## Outcomes & Retrospective

Planning is complete and implementation has not started. The preceding live
matrix already produced a useful safety outcome: the command failed closed and
neither disposable script changed. The immediate compatibility gap is now
narrower than the original roadmap wording. The visible editor selector is
stale, and the saved-script test API cannot currently establish active-slot
readback even when the editor is visible. The first milestone must determine
whether a trustworthy current-build identity path exists before any mutation
implementation proceeds.

## Context and Orientation

The repository builds one Rust binary named `tv`. Desktop-backed operations
use Chrome DevTools Protocol, abbreviated CDP, to evaluate JavaScript in the
selected TradingView chart page. The CDP runtime adapter is represented by
`tradingview_cdp::RuntimeEvaluator`.

Pine Editor helpers live under `crates/cli/src/ops/pine/editor/`.
`runtime.rs` contains `FIND_MONACO`, the generated JavaScript that finds a
Monaco editor, and `ensure_pine_editor_open`, which attempts to reveal Pine
Editor and waits for that selector. `source.rs` uses the helper for `pine get`
and `pine set`. `compile.rs` uses it for compile operations. `scripts.rs`
contains saved-script list, open, and save behavior. The public facade is
`crates/cli/src/ops/pine.rs`.

Monaco is the code editor embedded in Pine Editor. TradingView may keep more
than one Monaco instance in the page, including hidden stale editors. A
visible editor is an editor whose container has non-zero rendered dimensions
and is not hidden by CSS. Pine ownership means that the editor can be tied to
the currently visible Pine surface by a stable, locale-independent DOM or
runtime relationship; merely choosing the first visible global Monaco is not
enough.

The active saved-script slot is separate from visible source text. It is the
account-linked identity and version that TradingView Save will update. The
existing safety implementation resolves a requested saved script through the
Pine facade, invokes an internal open path, and requires matching internal
identity, version, and public display name before success. It deliberately has
no Monaco `setValue` success fallback.

The current `FIND_MONACO` accepts a global editor only when its container is
inside `.pine-editor-monaco`, then falls back to a React-fiber walk starting
from `.monaco-editor.pine-editor-monaco`. On the current live build, the visible
Pine editor uses generic Monaco markup, so both branches fail. Separately,
`pine_open_expression` calls `pineEditorTestApi().openEditor()`, awaits it,
then attempts to read `_pineEditor._storeProvider.getEditorActiveScript`.
Current live evidence shows the open promise remains pending and the provider
does not appear within the command deadline.

The owner has approved live use of two existing disposable scripts named
`Testスクリプト` and `Testスクリプト2` on the ELVN chart. Both may be changed
and saved if verification is green. No cloud-script cleanup should be
performed. Never place their source, internal identity, target identity, raw
DOM, raw page objects, or account-local metadata in tracked files.

## Plan of Work

### Milestone: prove current-build editor and binding boundaries

Start with a read-only, bounded compatibility probe. Inspect the currently
selected chart target, but return only counts, booleans, stable member names,
and an enum-like outcome. Do not return source, saved-script identity, raw DOM,
or exception text.

For editor ownership, enumerate Monaco editor objects through the existing
global API when available and map each object to its container DOM node. Record
whether it is focused, visible, and owned by a stable Pine surface. Inspect the
current Pine overlay for locale-independent attributes or runtime ownership
that distinguish it from hidden or unrelated Monaco instances. A proposed
anchor is acceptable only if it survives closed, open, and reopened Pine
states and identifies at most one selected editor under deterministic
fixtures. If no stable ownership proof exists, record editor targeting as
no-go rather than using the first visible editor.

For saved-script identity, inspect the current Pine API and the state associated
with the visible Pine surface. The probe may test whether an internal provider
can compare a requested account-local identity, version, and public display
name entirely inside the page, but it must return only match booleans and
public display metadata. Do not call `openScript` in this read-only milestone.
Do not infer binding from Monaco source equality or the visible title alone.
An identity path is acceptable only when it is tied to the same save state used
by TradingView's Save operation and can distinguish two disposable scripts
with identical source.

Record a stop/go decision in this ExecPlan. A full go requires both unique
visible Pine ownership and semantic active-slot identity. Editor-only go may
permit a narrowly scoped correction for `pine get`, `pine set`, and compile
operations, but it does not unblock `pine open` or the save matrix. Identity
no-go keeps `pine open` fail-closed and records current-build saved-script open
as unavailable.

### Milestone: implement the reviewed go path

On full or editor-only go, refactor `FIND_MONACO` in
`crates/cli/src/ops/pine/editor/runtime.rs` into a total generated expression
that produces zero or one Pine-owned editor. Prefer the focused editor when it
is Pine-owned. Otherwise accept exactly one visible Pine-owned editor. Reject
zero or multiple candidates. Do not retain the current unqualified
`editors[0]` fallback, and do not select by localized button or heading text.
Keep source reads and writes inside existing helpers so raw source does not
enter new diagnostics.

Update `ensure_pine_editor_open` to use the same selector before and after the
existing bounded panel-opening attempts. Preserve one finite deadline rather
than resetting a full wait after every UI attempt. Return additive public-safe
diagnostics for candidate count, visibility/ownership status, and failure
stage only if they do not reveal DOM content or account-local values.

On full identity go, update `pine_open` in
`crates/cli/src/ops/pine/editor/scripts.rs` so editor readiness and saved-script
binding are separate bounded stages. Preserve the same test API or state
provider instance throughout one attempt. Do not await an unbounded or
indefinitely pending `openEditor()` as the sole readiness condition. Do not
call `openScript` until the page can read the active identity afterward. After
the internal open, require requested and observed identity, version, and public
display name to match before returning success. Preserve the fixed sanitized
outer evaluation error, existing exit-code classification, and the absence of
a Monaco source-only fallback.

If feasibility finds only an editor targeting path, limit production changes
to editor selection and keep the saved-script binding code fail-closed. If no
go path exists, make no production mutation change merely to satisfy a live
smoke. Update help and stable docs to state current-build unavailability where
necessary.

### Milestone: prove behavior and rerun the live matrix

Add Rust tests for focused-versus-visible precedence, hidden stale editors,
multiple visible Pine candidates, unrelated Monaco instances, panel open
retry, and one absolute deadline. Extend the pinned executable JavaScript gate
to run the production-generated selector and, on full go, the binding
expression. Synthetic fixtures must include a never-settling editor-open
promise and a factory that returns different objects so the current test blind
spots cannot recur. Ordinary Cargo tests must remain independent of Node.js.

After focused validation and independent review are green, rerun the existing
owner-approved ELVN matrix. Capture source only in a mode-appropriate temporary
file outside tracked paths, calculate digests without printing source, and
remove that temporary local file afterward. First open and fingerprint
`Testスクリプト`, then open `Testスクリプト2`, apply one harmless comment,
save, and verify that only the second script changes. Leave both cloud scripts
in place and perform no script cleanup, as requested by the owner. Stop before
set or save on any ambiguous binding, timeout, or mismatched readback.

Finally synchronize `README.md`, `docs/development.md`,
`docs/command-source-taxonomy.md`, the packaged agent guide, and
`.agents/skills/pine-develop` only where user or agent behavior changed. Keep
the skill's core workflow short and move uncommon compatibility diagnostics to
its reference material.

## Concrete Steps

Run commands from the repository root. Start by preserving the current state:

    git status --short --branch
    git stash list
    rg -n "FIND_MONACO|ensure_pine_editor_open|pineEditorTestApi|openEditor|getEditorActiveScript" crates/cli/src/ops/pine/editor

Perform only bounded, capability-only live probes during the first milestone.
Use `tv tab list` and `tv state` to select the ELVN chart by observed symbol,
not by app-tab position. Summarize counts and booleans; do not paste target IDs,
file URLs, script IDs, source, or raw objects into this plan.

During implementation, run:

    cargo test -p tradingview-cli ops::pine::editor::runtime -- --nocapture
    cargo test -p tradingview-cli ops::pine -- --nocapture
    cargo test -p tradingview-cli --test cli_contract_desktop pine -- --nocapture
    mise run check:pine-open-js

Before independent review, run:

    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1
    mise run check:study-values-js
    mise run check:pine-open-js
    python3 scripts/check-public-hygiene.py --self-test
    python3 scripts/check-public-hygiene.py
    bash -n scripts/stage-release-package-files.sh
    cmp -s AGENTS.md CLAUDE.md
    git diff --check

Validate every changed skill with the repository skill validator. If the
pinned JavaScript contract changes, parse both workflow YAML files and verify
that CI and every release build still depend on the pinned Node.js gate.

## Validation and Acceptance

The feasibility milestone is accepted only when it records one of three
explicit outcomes: full go, editor-only go, or no-go. Full go requires a
unique semantic Pine editor and an internal active-slot identity path tied to
Save. Editor-only go requires a unique semantic Pine editor but leaves saved
script binding unavailable. No-go means neither production selector nor
binding changes are justified.

Editor targeting acceptance requires deterministic fixtures proving that the
selected editor is focused and Pine-owned, or otherwise the only visible
Pine-owned editor. Hidden stale and unrelated Monaco instances must never win.
Zero or multiple candidates must fail without reading or changing source. On
the current ELVN chart, `tv pine get` must find the visible editor; tracked
evidence records only success, line count, and digest, never source.

Saved-script acceptance requires `tv pine open Testスクリプト` to return
`slot_rebound: true` and `binding_verified: true`, with the observed public
display name matching the request. A never-settling editor-open operation,
missing provider, factory-instance mismatch, rejected open, or contradictory
identity must fail within a finite deadline and before source injection or
save. The two-script matrix must prove that an explicit edit/save changes only
`Testスクリプト2`; `Testスクリプト` remains digest- and version-stable.

All existing Pine compile, raw compile, list, get, set, new, and save contracts
remain unchanged except for additive public-safe diagnostics and the corrected
editor selection. No new command, option, dependency, source, automatic
fallback, source mixing, ranking, recommendation, or trading judgment is
added.

## Idempotence and Recovery

Capability probes and deterministic tests are repeatable and do not change
saved scripts. Opening or revealing Pine Editor may change visible UI state but
must not change source or account data. A failed selector or identity probe
stops without calling `openScript`, `pine set`, or `pine save`.

The final live save matrix is intentionally non-idempotent because a save may
create a new script version. Run it once per reviewed candidate unless a
documented correction requires another attempt. Use only the two approved
disposable scripts and stop before save if any proof is ambiguous. Remove local
temporary source files after digest comparison, but leave both cloud scripts
unchanged from their resulting test state because the owner requested no
cleanup.

The unrelated recovered indicator-search prototype remains stashed. Do not
apply, drop, rewrite, or include it in this work.

## Artifacts and Notes

Initial public-safe evidence:

    Selected chart symbol: ELVN
    Approved disposable scripts present: 2
    Visible Monaco instances on selected chart: 1
    Legacy Pine Monaco selector usable: false
    Pine test API open methods present: true
    Same-instance openEditor settled within 15 seconds: false
    Active saved-script readback available: false
    Saved scripts changed or saved during initial matrix: 0

Replace this section as work proceeds with concise stop/go evidence, test
counts, independent-review outcome, and the final live-matrix summary if it
becomes feasible. Never paste source, raw DOM, raw page values, target IDs,
saved-script IDs, account-local metadata, credentials, or machine-specific
paths.

## Interfaces and Dependencies

The public commands remain unchanged:

    tv pine get
    tv pine set --file <PATH>
    tv pine open <NAME...>
    tv pine save

Keep the shared editor selector private to
`crates/cli/src/ops/pine/editor/runtime.rs`. It may return a private result
containing the selected editor and environment inside generated JavaScript,
but Rust diagnostics receive only counts, booleans, and enum-like status.
Continue using `tradingview_cdp::RuntimeEvaluator`; do not add a production
dependency.

Keep saved-script binding in
`crates/cli/src/ops/pine/editor/scripts.rs`. If the implementation needs staged
evaluation, define private request/result shaping helpers rather than exposing
raw page objects. Use `ErrorKind::InternalApiUnavailable` for absent or
untrusted current-build internals and preserve `ErrorKind::Validation` for
missing or ambiguous user names. Existing fixed output sanitization and exit
codes remain authoritative.

Executable JavaScript continues through the pinned Node.js `24.18.0` contract
gate invoked by `mise run check:pine-open-js`. Node remains outside ordinary
Cargo tests and production dependencies.

## Open Questions

- UNCONFIRMED: Which locale-independent DOM or runtime relationship proves that
  the one visible generic Monaco instance belongs to the current Pine surface?
- UNCONFIRMED: Does the current overlay-style Pine Editor expose an active
  account-linked saved-script identity through a state provider other than the
  non-initialized test API store provider?
- UNCONFIRMED: If the test API remains present but unusable, can a supported
  internal open and readback path be bounded without relying on source equality
  or visible title alone?

Revision note (2026-07-14): Created after the owner-approved ELVN matrix failed
closed before mutation. The plan moves existing Active Pine Editor
compatibility work ahead of indicator insertion and requires separate
stop/go proof for visible editor ownership and saved-script identity.
