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
- [x] (2026-07-14) Proved that `[data-name="pine-dialog"]` uniquely owns the
  visible focused Monaco editor and that the same editor's React chain exposes
  one Redux store containing active script, open, and save state.
- [x] (2026-07-14) Recorded a full-go decision after exact UI selection of both
  approved disposable scripts produced matching internal identity, version,
  and public display name in the Save-bound store.
- [x] (2026-07-14) Implemented shared Pine-owned visible-editor selection,
  one bounded readiness deadline, overlay menu selection, and active-store
  post-checks without using the legacy test API or a source-only fallback.
- [x] (2026-07-14) Added deterministic Rust and pinned executable JavaScript
  coverage for focused preference, hidden stale editors, ambiguous visible
  editors, missing store/menu state, binding mismatch, and sanitization.
- [x] (2026-07-14) Ran focused Pine tests, Desktop CLI contracts, the full Rust
  baseline, both pinned JavaScript gates, public hygiene, packaging syntax,
  contributor-guide parity, diff checks, and Pine skill validation; all green.
- [x] (2026-07-14) Obtained initial independent review; it identified public
  script-ID exposure, globally scoped menu-row lookup, cross-registry Monaco
  ambiguity, and one stale stable-doc description.
- [x] (2026-07-14) Applied all four corrections with production-expression
  fixtures for unrelated exact-name rows and cross-registry ambiguity.
- [x] (2026-07-14) Obtained focused independent re-review of the corrected
  implementation, contracts, public-safe diagnostics, docs, and live-evidence
  boundary.
- [x] (2026-07-14) Reran the authorized ELVN matrix through the first explicit
  edit. Both bindings verified, but `pine set` stopped before save because
  Monaco normalized mixed line endings to CRLF and the post-check compared raw
  strings. Cloud versions and modification markers remained unchanged.
- [x] (2026-07-14) Updated `pine set` and `pine new` verification to compare
  source after normalizing CRLF and lone CR line endings, while continuing to
  reject content differences; added deterministic regression coverage.
- [x] (2026-07-14) Obtained focused independent review of the line-ending verification
  correction before resuming the live matrix.
- [x] (2026-07-14) Resumed the matrix and proved the normalized `pine set`
  post-check live. The subsequent explicit `pine save` stopped before keyboard
  mutation because its preflight expression contained invalid doubled braces;
  cloud versions and modification markers remained unchanged. The CDP error
  also exposed internal exception metadata in the normal error envelope.
- [x] (2026-07-14) Corrected the generated save-preflight JavaScript, added
  executable pre/post-save expression coverage to the pinned Node gate, and
  sanitized preflight/post-shortcut evaluation failures while preserving
  `ErrorKind`.
- [x] (2026-07-14) Obtained focused independent review of the Pine save
  correction; generated-expression execution, fail-before-shortcut behavior,
  public-safe diagnostics, and existing save boundaries are green.
- [x] (2026-07-14) Resumed the ELVN two-script matrix. A fixed Control modifier
  failed closed on macOS with dirty state preserved; the target script was not
  reported saved.
- [x] (2026-07-14) Selected Command/Meta on macOS and Control on Windows/Linux,
  added deterministic modifier coverage, and reran the authorized matrix. Only
  `Testスクリプト2` advanced and retained the edited source after reopening;
  `Testスクリプト` remained unchanged. Both scripts were retained.
- [x] (2026-07-14) Obtained closeout review; it found that unknown dirty state
  could be reported saved, modifier coverage reused the production helper, and
  the paused binding-plan state was stale.
- [x] (2026-07-14) Required explicit `saved: true` and `dirty_after: false`,
  added executable unknown-dirty and helper-independent modifier coverage, and
  synchronized the paused plan and current project state.
- [x] (2026-07-14) Focused re-review confirmed explicit verification and
  modifier coverage, then found raw malformed-outcome details and stale upper
  binding summaries.
- [x] (2026-07-14) Replaced all Pine save outcome/page-error details with
  public-safe whitelists, added private-value fixtures, and synchronized the
  roadmap and work inventory binding summaries.
- [x] (2026-07-14) Focused re-review found no remaining findings in explicit
  save verification, modifier coverage, diagnostic whitelisting, live evidence,
  or project-state documentation.
- [x] (2026-07-14) Synchronized durable docs and skills and completed this plan
  for archival alongside the saved-script binding prerequisite.

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

- Observation: Monaco normalizes editor-buffer line endings on `setValue`.
  The first reviewed live-matrix retry supplied a harmless-comment source with
  mixed CRLF/LF endings; readback contained equivalent content with CRLF
  normalization, so the previous raw-string post-check failed before save.
  Evidence: public-safe expected/observed character counts differed by two,
  the cloud script version and modification marker remained unchanged, and no
  source text or saved-script identifier was retained.

- Observation: The existing `pine save` preflight used doubled JavaScript
  braces even though that raw body does not pass through Rust `format!`.
  Current Desktop therefore returned a syntax error before shortcut dispatch.
  The unsanitized Runtime evaluation error also included CDP-local exception
  metadata. Cloud versions and modification markers remained unchanged.

- Observation: The inherited Pine save path used CDP's Control modifier on all
  platforms. Current macOS Desktop left the buffer dirty after that shortcut,
  while Command/Meta cleared dirty state and persisted the selected script.
  Evidence: the first reviewed attempt failed closed; after the platform
  correction, only the intended script advanced and retained its edited line
  and character counts after a verified reopen. The other disposable script's
  version, modification marker, and source counts remained unchanged.

- Observation: Dirty-state unavailability is not equivalent to a clean editor.
  The prior JavaScript used `dirtyAfter !== true`, and Rust defaulted missing
  `saved` to true, so DOM drift could report an unverified shortcut as saved.
  Evidence: closeout review identified the path; both page-side and Rust-side
  success now require explicit booleans, with malformed and contradictory
  fixtures rejected.

- Observation: Failing closed is insufficient if the rejected page payload is
  attached verbatim to the error envelope.
  Evidence: focused re-review found that malformed save outcomes could carry
  raw source or account-local identifiers through `.with_details(raw)`. Save
  outcome and page-error diagnostics now expose only fixed operation/stage,
  booleans, a known source marker, and a fixed next-action hint.

- Observation: The visible current-build editor is owned by the
  locale-independent `[data-name="pine-dialog"]` surface, while the other
  Monaco instance is hidden and outside that owner.
  Evidence: A capability-only probe observed one visible Pine-owned editor and
  one hidden stale editor. The corrected `tv pine get` then succeeded on ELVN;
  only line count and editor-open state were retained.

- Observation: The visible editor's React ancestry exposes one Redux store
  whose `script`, `openScript`, and `saveScript` state represent the active
  saved slot used by the visible overlay.
  Evidence: Selecting each approved disposable script through the exact UI
  menu produced one exact facade candidate and matching internal identity,
  version, and public display name. Save state reported saved. No source or
  account-local identifier was retained.

- Observation: A page-coordinate click can reach content behind a transient
  menu and change unrelated chart state.
  Evidence: One feasibility attempt changed the selected chart symbol; it was
  immediately restored to ELVN. The accepted implementation uses an exact
  Pine-owned menu trigger and exact menu row in one page evaluation and does
  not use coordinates.

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

- Decision: Take the full-go path using the visible overlay's own menu and
  Redux store; retire the legacy `pineEditorTestApi()` path for saved-script
  opening on the current build.
  Rationale: The menu changes the same state later consumed by Save, and the
  store permits internal ID/version/name comparison without returning private
  values. The legacy API creates a separate embedded editor and can remain
  pending indefinitely.
  Date/Author: 2026-07-14 / Codex.

- Decision: Use semantic DOM ownership and exact menu elements, never page
  coordinates, for the accepted open path.
  Rationale: Pine dialog ownership is locale-independent and deterministic;
  transient coordinate targets are not.
  Date/Author: 2026-07-14 / Codex.

- Decision: Compare Pine source after normalizing line endings only.
  Rationale: Current-build Monaco converts mixed LF/CRLF input to its buffer
  convention. Treating CRLF, LF, and lone CR as equivalent avoids a false
  negative without accepting any source-token or whitespace difference beyond
  the line-ending encoding itself.
  Date/Author: 2026-07-14 / Codex.

- Decision: Execute both generated Pine save inspection expressions in the
  pinned JavaScript contract gate and sanitize Runtime evaluation failures at
  the operation boundary.
  Rationale: Fake Runtime payload tests did not parse the production-generated
  JavaScript, and raw CDP exception details are not part of the public Pine save
  contract.
  Date/Author: 2026-07-14 / Codex.

- Decision: Select the Pine save shortcut modifier from the CLI host platform:
  Meta on macOS and Control on Windows/Linux.
  Rationale: TradingView Desktop and the CLI run on the same host, CDP defines
  distinct modifier bits, and current macOS live evidence rejected the
  inherited fixed-Control shortcut while accepting Command+S. Dirty-state
  verification remains the authoritative success check.
  Date/Author: 2026-07-14 / Codex.

- Decision: Treat Pine save as successful only when page-side readback reports
  both `saved: true` and `dirty_after: false` explicitly.
  Rationale: A missing or malformed dirty-state observation cannot prove cloud
  persistence. The shortcut may have done nothing even when no evaluation error
  occurred, so unknown and contradictory outcomes must fail closed.
  Date/Author: 2026-07-14 / Codex.

- Decision: Whitelist Pine save success and failure fields at the Rust operation
  boundary instead of forwarding the page payload.
  Rationale: Runtime payloads are untrusted even when generated JavaScript is
  expected to return a narrow shape. Malformed success, dirty-state failure,
  and page-error paths must not expose source, account-local identity, target
  identity, or exception-shaped values.
  Date/Author: 2026-07-14 / Codex.

## Outcomes & Retrospective

Feasibility, implementation, full local validation, and focused binding review
are complete. The accepted path selects only a focused Pine-owned editor, or
the sole visible Pine-owned editor, and fails on ambiguity. Saved-script open
uses an exact row from the popup linked to the Pine-owned trigger and verifies
the resulting active slot through the same overlay's Save-bound store. The
legacy test API and source-only Monaco replacement are absent from the success
path. The resumed live matrix stopped before save on Monaco line-ending
normalization; that correction passed focused review and succeeded live. The
next save attempt stopped before shortcut dispatch on invalid preflight
JavaScript. Its syntax and public-safe evaluation boundary are corrected and
passed focused review. The resumed matrix then failed closed because the
inherited shortcut used Control on macOS. After selecting the platform-specific
modifier, the intended disposable script saved, remained changed after a
verified reopen, and the other script remained unchanged. Closeout-review
corrections passed focused re-review with no remaining findings. A
non-active script must be present as one unique exact row in that linked popup;
missing ownership, absence, or ambiguity is an explicit fail-closed boundary.

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

Before this compatibility change, `FIND_MONACO` accepted a global editor only
when its container was inside `.pine-editor-monaco`, then fell back to a
React-fiber walk starting from `.monaco-editor.pine-editor-monaco`. The visible
current-build Pine editor uses generic Monaco markup, so both branches failed.
The former `pine_open_expression` also awaited
`pineEditorTestApi().openEditor()` and then attempted to read
`_pineEditor._storeProvider.getEditorActiveScript`; live evidence showed that
the open promise remained pending and the provider did not appear. The current
implementation instead uses semantic Pine ownership, exact rendered-menu
selection, and the overlay's Save-bound store.

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
than resetting a full wait after every UI attempt. Sanitize runtime evaluation
failures at this readiness boundary while preserving their `ErrorKind` and
return only public-safe stage booleans; do not expose exception descriptions,
DOM content, or account-local values.

On full identity go, update `pine_open` in
`crates/cli/src/ops/pine/editor/scripts.rs` so editor readiness and saved-script
binding are separate bounded stages. Use the selected overlay's exact visible
menu row and preserve the same Save-bound state provider throughout one
attempt. Do not use the unavailable legacy test API, page coordinates, or a
Monaco source-only fallback. Keep menu discovery and binding readback within
one finite page-side deadline. After selection, require requested and observed
identity, version, and public display name to match before returning success.
Preserve the fixed sanitized outer evaluation error and existing exit-code
classification.

If feasibility finds only an editor targeting path, limit production changes
to editor selection and keep the saved-script binding code fail-closed. If no
go path exists, make no production mutation change merely to satisfy a live
smoke. Update help and stable docs to state current-build unavailability where
necessary.

### Milestone: prove behavior and rerun the live matrix

Add Rust tests for focused-versus-visible precedence, hidden stale editors,
multiple visible Pine candidates, panel open retry, one absolute deadline, and
sanitized readiness evaluation errors. Extend the pinned executable JavaScript
gate to run the production-generated selector and binding expression.
Synthetic fixtures must cover editor ambiguity, missing owner/store/menu,
exact menu selection, and post-selection identity mismatch. Ordinary Cargo
tests must remain independent of Node.js.

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

Full-go feasibility evidence:

    Pine-owned visible editor count: 1
    Save-bound state store count: 1
    Exact facade candidate count per approved script: 1
    Identity, version, and public display-name matches: true
    Exact DOM menu switching succeeded for both scripts: true
    Corrected tv pine get on ELVN: success, 4 lines
    Source or account-local identifiers retained: false
    Saved scripts changed or saved during feasibility: 0

Local validation evidence:

    Focused Pine Rust tests: green
    Desktop Pine CLI contracts: green
    Workspace tests and strict Clippy: green
    Pinned study-value and Pine-open JavaScript gates: green
    Public hygiene and packaging checks: green
    Pine skill validation: green
    Readiness evaluation errors are sanitized with ErrorKind preserved: green
    One eight-second page-side menu/binding deadline: green
    Initial independent review: corrections applied
    Focused independent re-review of binding corrections: green
    First resumed matrix edit: stopped before save on CRLF-only post-check mismatch
    Line-ending-normalized post-check correction and full baseline: green
    Focused review of line-ending correction: green
    Normalized pine set live post-check: green
    First pine save attempt: stopped before shortcut on preflight syntax error
    Pine save expression and evaluation-sanitizer correction: implemented
    Focused review of Pine save correction: green
    First post-review save: failed closed with fixed Control modifier on macOS
    Platform-specific save modifier correction and focused tests: green
    Final owner-approved matrix: intended script saved; other script unchanged
    Initial closeout review: corrections required
    Explicit save/dirty verification and independent modifier tests: implemented
    Public-safe malformed-outcome and page-error diagnostics: implemented
    Upper binding summaries synchronized: complete
    Focused correction re-review: pending

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

None. Current-build indicator insertion and Windows launch behavior remain
separate roadmap work rather than unresolved questions in this completed plan.

Revision note (2026-07-14): Created after the owner-approved ELVN matrix failed
closed before mutation. The plan moves existing Active Pine Editor
compatibility work ahead of indicator insertion and requires separate
stop/go proof for visible editor ownership and saved-script identity.

Revision note (2026-07-14): Recorded the full-go feasibility result and the
implemented overlay menu plus Save-bound store path. The final authorized
edit/save matrix remains pending.

Revision note (2026-07-14): Recorded the green full local baseline and docs /
skill synchronization. The owner-approved edit/save matrix remains gated on
independent review.

Revision note (2026-07-14): Tightened the review candidate by sanitizing Pine
readiness evaluation failures, composing menu selection and identity readback
under one page-side deadline, and documenting the rendered-menu availability
boundary. Focused and full validation remain green after these corrections.

Revision note (2026-07-14): Applied initial-review corrections by removing the
account-local ID from success output, scoping menu rows to the trigger-linked
popup, aggregating global and fiber Monaco registries before selection, and
updating stable API documentation. Focused re-review remains before the live
matrix.

Revision note (2026-07-14): Recorded the green focused re-review and the live
matrix stop before save on semantically equivalent Monaco CRLF normalization.
The post-check now normalizes line endings only; focused review remains before
the authorized matrix resumes.

Revision note (2026-07-14): Recorded the green line-ending review and successful
live `pine set` post-check. The following save attempt exposed invalid preflight
JavaScript and unsanitized CDP exception details before shortcut dispatch; both
were corrected and subsequently passed focused review. The owner-approved
matrix may resume without expanding its mutation scope.

Revision note (2026-07-14): Recorded the reviewed save correction, the
fail-closed fixed-Control result on macOS, and the platform-specific modifier
correction. The final owner-approved matrix saved only `Testスクリプト2`, kept
`Testスクリプト` unchanged, retained both scripts, and exposed no private value.
Closeout review identified further corrections before plan completion.

Revision note (2026-07-14): Applied closeout-review corrections by requiring
explicit saved/clean booleans, exercising unknown dirty state in the generated
JavaScript contract, fixing modifier assertions to use literal platform bits
and event types, and synchronizing the paused binding plan. Focused re-review
remains before completion.

Revision note (2026-07-14): Applied the next focused-review corrections by
whitelisting all Pine save outcome and page-error details, adding private-value
non-leakage fixtures, and synchronizing the upper roadmap/inventory binding
summaries. Focused re-review remains before completion.

Revision note (2026-07-14): Recorded the green focused re-review with no
remaining findings. The compatibility and save-safety outcomes are complete,
and this plan is ready for archival.
