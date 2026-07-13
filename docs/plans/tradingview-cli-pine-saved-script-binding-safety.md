# Make saved Pine script opening verify its active slot

This ExecPlan is a living document. The sections `Progress`, `Surprises &
Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to
date as work proceeds.

This document follows `.agents/PLANS.md` and must be maintained in accordance
with that file.

## Purpose / Big Picture

After this change, `tv pine open <NAME...>` will succeed only when TradingView
Desktop has opened the requested saved Pine script through its own script
manager and the CLI has verified that the editor's active saved-script slot
matches the request. A later explicit `tv pine save` will therefore target the
script that TradingView reports as active rather than an older slot left from a
previous editor session.

Before this implementation, the command fetched a saved script and called
Monaco `setValue`. Monaco is the text editor embedded in Pine Editor. Replacing
its visible text did not prove that TradingView changed the separate
account-linked saved-script slot used by Save. The Rust implementation has not
reproduced an overwrite live, so the concrete Rust failure remains
`UNCONFIRMED`, but the old operation could not establish the safety property it
claimed. The corrected command fails closed: if the internal open-script
operation or identity readback is not available, it returns a structured error
and does not inject the fetched source as a fallback.

## Progress

- [x] (2026-07-14) Verified that upstream merged `main` remains at the prior
  survey boundary and reviewed the unmerged saved-script binding evidence.
- [x] (2026-07-14) Confirmed that current Rust `pine_open_expression` fetches
  source and calls Monaco `setValue` without active-slot readback.
- [x] (2026-07-14) Created the v0.28 roadmap, ordered inventory, and this
  implementation plan.
- [x] (2026-07-14) Inspected all current chart targets without changing a
  saved script; each exposes the Pine test API and open methods, while active
  readback is initialized only after the editor is opened.
- [x] (2026-07-14) Implemented fail-closed saved-script opening and additive
  binding diagnostics without a source-only fallback.
- [x] (2026-07-14) Added deterministic Rust and pinned executable JavaScript
  contract tests while keeping ordinary Cargo tests independent of Node.js.
- [x] (2026-07-14) Updated help, stable Pine workflow documentation, packaged
  guidance, and the Pine development skill without expanding its core steps.
- [x] (2026-07-14) Ran focused tests, the full Rust baseline, both pinned
  JavaScript contract gates, workflow parsing, public hygiene, package-script
  syntax, contributor-guide parity, diff checks, and the Pine skill validator.
- [x] (2026-07-14) Applied the initial independent-review corrections: runtime
  evaluation errors now cross a fixed public-safe boundary, Rust rejects
  contradictory success payloads, and executable fixtures prove awaited opens,
  rejection handling, isolated identity mismatches, partial resolution, and
  ambiguity before mutation. Re-ran focused and full local validation green.
- [x] (2026-07-14) Focused independent re-review found no remaining findings
  and approved proceeding to the separately owner-authorized live matrix.
- [x] (2026-07-14) Received owner approval to use `Testスクリプト` and
  `Testスクリプト2` on the ELVN chart without cloud-script cleanup, confirmed
  both scripts, and attempted the live matrix. The command failed closed before
  any set or save because current-build active-slot readback was unavailable.
- [ ] Complete the separate current-build Active Pine Editor compatibility
  plan, then rerun the already authorized two-script live matrix and record
  only a public-safe summary.
- [ ] Archive this plan only after the required live evidence is green.

## Surprises & Discoveries

- Observation: Before this implementation, the command's `opened: true` meant
  only that source was fetched and copied into Monaco.
  Evidence: The prior `crates/cli/src/ops/pine/editor/scripts.rs` called
  `m.editor.setValue(source)`, compares `getValue`, and returns success without
  reading TradingView's active saved-script state.

- Observation: Current upstream evidence identifies
  `window.TradingViewApi.pineEditorTestApi().openScript(...)` as the same
  script-manager path used by TradingView's UI and reports successful
  disposable-script verification. This is useful evidence, not an API
  guarantee for the Rust implementation or every Desktop build.
  Evidence: upstream pull request 158 is unmerged and includes one current
  implementation plus live notes; Rust still needs its own bounded probe and
  tests.

- Observation: Every current chart target exposes `pineEditorTestApi`,
  `openEditor`, and `openScript`, but none exposes the active-script provider
  before Pine Editor initialization.
  Evidence: A read-only capability expression returned the three method
  booleans as true and active-readback availability as false for all four
  current chart targets. No target ID, script ID, source, or raw object was
  retained.

- Observation: The generated asynchronous binding expression needs executable
  coverage because fake Rust runtime responses cannot prove promise ordering,
  method absence, or throwing readback behavior.
  Evidence: The dedicated pinned-Node contract executes the production
  expression against matching, missing-method, mismatched, and throwing
  synthetic editor-manager states; ordinary Cargo tests keep it ignored.

- Observation: Fail-closed page JavaScript is not sufficient by itself because
  a CDP evaluation failure and a malformed synthetic success can bypass the
  expression's own checks.
  Evidence: Independent review identified both outer boundaries. The Rust
  adapter now replaces evaluation errors with a fixed message and whitelisted
  details, and independently requires a success marker plus matching resolved
  and observed display name/version before returning success.

- Observation: `scripts.rs` is now 1,211 lines, of which 663 lines are the
  deterministic test module; the 548-line production portion still owns one
  coherent saved-script open/save/list boundary.
  Evidence: The Pine editor module size inspection places `#[cfg(test)]` at
  line 549. A future
  completion audit should reconsider an `open`/`save` module split if another
  Pine persistence feature lands, but a behavior-neutral split is not required
  to review this safety correction.

- Observation: The current Desktop build can display a visible Pine Editor and
  a visible Monaco instance while the legacy `.pine-editor-monaco` selector and
  the Pine test API's active editor/store provider both remain unavailable.
  Evidence: On the owner-selected ELVN chart, a capability-only CDP probe found
  two Monaco instances, one visible, and found the Pine test API factory and
  open methods. `tv pine get` still reported that Monaco was unavailable, and
  the test API did not expose active-slot readback.

- Observation: Calling the current build's internal `openEditor()` does not
  settle or initialize active-slot readback within a bounded 15-second
  observation, even when the editor is visibly open.
  Evidence: `tv pine open Testスクリプト` reached its normal evaluation timeout
  with the fixed public-safe error, and a separate capability-only same-instance
  probe observed neither promise settlement nor active readback. No source,
  script ID, target ID, or raw object was retained.

## Decision Log

- Decision: Fail if verified slot rebinding is unavailable; never fall back to
  Monaco `setValue` while reporting `pine open` success.
  Rationale: A warning does not prevent a later explicit save from overwriting
  a different account-linked script. The old source-only behavior cannot meet
  the command's saved-script-open meaning.
  Date/Author: 2026-07-14 / Codex.

- Decision: Keep name resolution compatible in this slice: exact name or
  title is preferred, one unique partial match is allowed, and ambiguous or
  missing matches fail before editor mutation.
  Rationale: The safety defect is binding, not user-facing name matching. A
  separate breaking change is unnecessary if the resolved internal identity
  is verified after opening.
  Date/Author: 2026-07-14 / Codex.

- Decision: Compare account-local script identifiers internally, but do not
  add them to new diagnostics, tracked evidence, or panic text.
  Rationale: Internal IDs are the strongest equality key, while names alone
  can collide. The CLI can report requested and observed display names,
  versions, and boolean match status without broadening private output.
  Date/Author: 2026-07-14 / Codex.

- Decision: Do not change `tv pine save`, `tv pine compile`, `tv pine
  raw-compile`, named new-script save, or REST overwrite behavior in this
  implementation unless a direct regression is required for binding safety.
  Rationale: Those operations have distinct persistence and dialog risks. The
  first slice should repair one existing command and preserve its surrounding
  boundaries.
  Date/Author: 2026-07-14 / Codex.

- Decision: Any Node.js execution test is a separate pinned contract gate and
  not an ordinary `cargo test` prerequisite.
  Rationale: The repository already keeps generated page-JavaScript execution
  outside the Rust-only Cargo baseline. Repeating that pattern avoids an
  undeclared cross-platform test dependency.
  Date/Author: 2026-07-14 / Codex.

- Decision: Preserve the shipped top-level `script_id` success field unchanged
  in this safety slice, while excluding internal IDs from every new nested
  diagnostic, failure detail, tracked artifact, and test report.
  Rationale: Removing an existing response field would be a separate breaking
  public-safety migration. The current correction can stop adding new exposure
  and sanitize failures without silently breaking compatible consumers.
  Date/Author: 2026-07-14 / Codex.

- Decision: Treat opening Pine Editor as an allowed UI effect while still
  refusing saved-script rebinding when active identity readback is unavailable.
  Rationale: Current Desktop exposes active-script readback only after
  `openEditor()`. The command may reveal the editor panel, but it does not call
  `openScript` until readback capability is present and never injects source as
  a fallback. A rejected or mismatched private operation is reported as
  unverified and the user is told not to save from that state.
  Date/Author: 2026-07-14 / Codex.

- Decision: Record the owner-approved live matrix as a current-build no-go and
  stop before `pine set` or `pine save` rather than weakening binding
  verification.
  Rationale: The visible editor does not provide the active-slot evidence
  required by this plan, and the internal editor-open operation does not finish
  within the bounded command deadline. The fail-closed result is the intended
  safety behavior; saving from this state would defeat the purpose of the
  correction.
  Date/Author: 2026-07-14 / Codex.

- Decision: Promote Active Pine Editor targeting and readback compatibility
  ahead of indicator insertion, then return to this plan's live matrix.
  Rationale: The roadmap already allowed that work to move earlier when it
  blocks Pine binding validation. The current live evidence establishes that
  condition without justifying an unsafe source-only fallback.
  Date/Author: 2026-07-14 / Codex.

## Outcomes & Retrospective

Implementation, focused Rust tests, executable JavaScript coverage, help, docs,
packaged guidance, and skill synchronization are complete. `tv pine open` no
longer calls Monaco `setValue`; success requires the internal open operation
and matching active ID/version/name readback. Failure details are whitelisted
and do not retain source, raw page values, or internal script identity.

Full local validation after the initial review corrections and focused
independent re-review are green. The owner-approved disposable-script matrix
was attempted on ELVN with the requested two test scripts and stopped safely
before any source change or save. The current Desktop build did not provide a
bounded active-slot readback path, so the overwrite-safety matrix remains
incomplete and the Rust overwrite risk remains `UNCONFIRMED`. A separate
current-build editor-targeting/readback compatibility plan now blocks the
matrix; deterministic coverage and the live fail-closed result do not replace
proof that a later save updates only the intended disposable slot.

## Context and Orientation

The repository builds one Rust binary named `tv`. CLI command definitions live
in `crates/cli/src/cli.rs`, dispatch lives in
`crates/cli/src/app/dispatch.rs`, and Desktop-backed operation adapters live
under `crates/cli/src/ops/`. Pine Editor operations are re-exported by
`crates/cli/src/ops/pine.rs`; saved-script open and save behavior is in
`crates/cli/src/ops/pine/editor/scripts.rs`; shared Monaco discovery and input
dispatch are in `crates/cli/src/ops/pine/editor/runtime.rs`.

TradingView Desktop exposes a chart page over Chrome DevTools Protocol, called
CDP. The CLI evaluates JavaScript inside that page through
`tradingview_cdp::RuntimeEvaluator`. Pine Editor uses Monaco for visible source
text, but TradingView separately tracks the saved script that Save will update.
This plan calls that account-linked selection the active saved-script slot.

Before this implementation, `pine_open` first called
`ensure_pine_editor_open`, listed saved scripts through Pine facade, fetched
source by internal ID and version, wrote that source with Monaco `setValue`,
and verified only that Monaco returned the same text. It returned `name`,
`script_id`, `version`, line count, `source: "internal_api"`, and
`opened: true` without an active-slot post-check.

Upstream evidence indicated that the page may expose
`window.TradingViewApi.pineEditorTestApi()`. Its `openEditor()` and
`openScript({scriptIdPart, version})` methods reportedly route through
TradingView's script manager. The active script may be readable from the
editor store. The read-only probe confirmed the open methods on current chart
targets and confirmed that readback appears after editor initialization. These
remain private current-build interfaces rather than a universal API guarantee.

No public command, source, dependency, or version is added. The existing
`tv pine open` behavior becomes stricter because a source-only open can no
longer return success. This is an intentional safety correction.

## Plan of Work

Start with a read-only current-build probe. In a running TradingView Desktop
session with Pine Editor available, inspect only capability booleans and
public-safe active-script fields: whether `pineEditorTestApi`, `openEditor`,
`openScript`, and active-script readback are callable, plus whether an active
display name and version are available. Do not print object dumps, source,
internal IDs, target IDs, Redux state, or raw exceptions. Record the minimum
verified API shape in `Surprises & Discoveries` and update the implementation
description if the current build differs.

Refactor `crates/cli/src/ops/pine/editor/scripts.rs` so saved-script discovery
remains separate from editor mutation. The page expression may continue using
Pine facade to resolve the requested name and obtain the internal ID and
version. Before changing the saved-script binding, confirm the script-manager
open methods are available. Open Pine Editor if needed, then confirm active
identity readback before calling `openScript`. If readback is unavailable,
return an
`internal_api_unavailable` result with public-safe capability booleans and a
next action hint. Opening the editor panel is an allowed UI effect; source
injection and saved-script rebinding are not. Do not call Monaco `setValue` in
this path.

Invoke TradingView's open-editor and open-script operations as awaited
promises. After they resolve, read active-script identity from the same editor
manager/store. Compare internal requested and observed IDs, normalized
versions, and display names inside the page. The operation succeeds only when
all three match. A method rejection, missing
readback, mismatched identity, or ambiguous state is
`internal_api_unavailable`; user name ambiguity or absence remains
`validation`.

Shape the Rust payload additively. Preserve existing practical fields where
safe and add:

    operation: "pine_open"
    source_category: "desktop_backed_operation"
    requires_desktop: true
    non_mutating: false
    slot_rebound: true
    binding_verified: true
    requested_script: { name: <user request> }
    observed_script: { name: <display name>, version: <version or null> }
    binding_method: "pine_editor_internal_api"

Do not add a raw internal ID to the new nested objects. Inspect the existing
top-level `script_id` contract before editing it. This safety slice should not
silently remove a shipped field; if public hygiene or compatibility requires a
change, record the decision and cover it explicitly rather than making an
incidental deletion.

Sanitize all failures before returning them through `AppError`. Permitted
details are operation name, requested display name, candidate count capped at
the existing public limit, capability booleans, binding status, observed
display name/version when available, and a short next action. Do not include
source, request URLs, raw page values, raw exception text, internal script IDs,
session or target identifiers, or account-local state.

Add deterministic Rust tests in the Pine editor module. Fake runtime responses
must cover exact and unique partial resolution, ambiguity before mutation,
missing methods before mutation, successful internal open and matching
readback, method rejection, mismatched readback, absent readback, and proof
that no generated success path contains `setValue`. Tests must also prove that
`tv pine open` still validates an empty name before CDP connection and that
unrelated Pine command contracts do not change.

Execute the generated page JavaScript in a synthetic Node.js harness if the
binding logic cannot be represented by small independently tested Rust
shaping. Follow the existing managed-gate pattern: add a dedicated ignored
test and a small version-checking script under `scripts/`, pin Node.js
`24.18.0` in `mise.toml`, CI, and release workflow, and keep the normal Cargo
suite runnable when Node is absent. The synthetic objects must cover matching
identity, rejection, missing method, mismatched identity, and throwing getter
behavior. If the implementation is instead split into sufficiently small
nonthrowing JavaScript helpers already exercised by an existing pinned gate,
record why a new gate is unnecessary.

Update the `tv pine open --help` wording, README Pine section,
`docs/development.md`, packaged agent guidance, and
`.agents/skills/pine-develop`. Correct the old claim that `pine open` only
changes the local buffer. Keep the skill's core workflow short and place
uncommon recovery or binding diagnostics in its existing reference file.
Validate every changed skill.

Finally run the owner-approved live matrix with `Testスクリプト` and
`Testスクリプト2`. Capture only
public-safe preconditions: requested display name, version, and a digest or
boolean unchanged marker computed without storing source. Open the first
script through the TradingView UI or verified internal path, run `tv pine open`
for the second, verify `slot_rebound` and observed display identity, then make
an explicit harmless edit and save. Confirm the first script remains unchanged
and the second is the only script whose version or digest changes. Leave both
cloud scripts in their resulting disposable test state; the owner requested no
cleanup. Do not run this matrix against any other script or infer safety from
source text copied into tracked files.

## Concrete Steps

Run all commands from the repository root. Ground the implementation state:

    git status --short --branch
    git stash list
    rg -n "pine_open|pine_open_expression|setValue|pineEditorTestApi" crates/cli/src crates/cli/tests
    cargo test -p tradingview-cli pine -- --nocapture

During implementation, run focused checks:

    cargo test -p tradingview-cli ops::pine -- --nocapture
    cargo test -p tradingview-cli --test cli_contract_desktop pine -- --nocapture

If a dedicated executable JavaScript contract is added, run it under the
pinned runtime using its new `mise` task. Its wrapper must reject any Node
version other than `24.18.0`, and ordinary `cargo test --workspace` must still
pass with Node removed from `PATH`.

Before review, run:

    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1
    python3 scripts/check-public-hygiene.py --self-test
    python3 scripts/check-public-hygiene.py
    bash -n scripts/stage-release-package-files.sh
    cmp -s AGENTS.md CLAUDE.md
    git diff --check

Validate `.agents/skills/pine-develop` with the repository's skill validator.
If CI or release workflow changes, parse both YAML files locally and verify
that every release build depends on the new JavaScript contract gate.

The live matrix command names depend on the disposable scripts prepared by the
owner. Record commands here only with placeholders or public-safe display
names; never persist internal IDs or source. The final evidence should state
the command, `slot_rebound`, `binding_verified`, observed display-name match,
which disposable changed, whether the other remained unchanged, and that no
cloud-script cleanup was requested or performed.

## Validation and Acceptance

The command is accepted when deterministic tests prove all failure paths stop
without Monaco source injection and the success path requires awaited internal
open plus matching active-script readback. The ordinary Rust baseline must not
require Node. Any executable JavaScript gate must be pinned and required by CI
and release builds.

User-visible acceptance requires:

- `tv pine open <EXACT_DISPOSABLE_NAME>` succeeds with `slot_rebound: true`
  and `binding_verified: true`.
- The observed public-safe script name matches the resolved requested script.
- Ambiguous and missing names fail before mutation.
- Missing script-manager API, rejected open, absent readback, and mismatched
  readback fail with `internal_api_unavailable` and no source-only fallback.
- Existing `tv pine save`, safe compile, raw compile, list, get, set, and new
  command contracts remain unchanged except for documentation needed to
  explain the corrected open semantics.
- The owner-approved two-disposable-script matrix proves that an explicit save
  after open changes only the intended disposable script.

If the current Desktop build does not expose a trustworthy open and readback
path, record a no-go outcome. In that case the command should be changed to
fail safely or be documented as unavailable on that build; do not restore the
old source-only success path to make the smoke pass.

## Idempotence and Recovery

Deterministic tests and read-only capability probes are repeatable. The
implementation must inspect capabilities before editor mutation so unsupported
builds leave the editor unchanged. Re-running a successful open for the same
script should be idempotent with respect to cloud source and version.

The live save matrix is not idempotent because saving can create a new version.
The owner has approved it only for the two named disposable scripts; run it at
most once per reviewed candidate unless a correction requires another run.
Capture pre-operation public-safe digests in ignored `target/` files if
recovery needs them. Never place source or identifiers in tracked files. If
verification is ambiguous, stop without another save. Leave cloud-script state
as-is because the owner requested no cleanup.

The recovered indicator-search prototype stash is unrelated. Do not apply,
drop, rewrite, or include it in this slice.

## Artifacts and Notes

Planning evidence:

    Upstream merged main: unchanged from the previous survey boundary
    Current Rust open path: awaited internal open plus ID/version/name readback
    Source-only Monaco fallback: absent
    Focused Pine module tests: 14 passed, 1 ignored managed JavaScript fixture
    Pine CLI tests: 36 passed, 1 ignored managed JavaScript fixture
    Desktop Pine contract tests: 16 passed
    Pinned Pine open JavaScript contract: 1 passed on Node.js 24.18.0
    Full CLI unit suite: 414 passed, 2 managed fixtures ignored
    Full workspace baseline and public hygiene after review corrections: passed
    Owner-approved ELVN live matrix: stopped before mutation
    Correct-target pine open result: bounded fail-closed timeout
    Visible current-build Monaco instances: 1
    Active saved-script readback: unavailable
    Disposable scripts changed or saved: 0
    Rust live overwrite reproduction: UNCONFIRMED

Replace this section with concise test counts, JavaScript gate evidence, live
matrix summary, and independent-review outcome. Never paste raw source, page
objects, saved-script IDs, target IDs, or account-local metadata.

## Interfaces and Dependencies

The public command remains:

    tv pine open <NAME...>

The Rust operation remains conceptually:

    pub async fn pine_open(
        runtime: &mut impl RuntimeEvaluator,
        name: &str,
    ) -> Result<serde_json::Value, AppError>

Implementation stays in
`crates/cli/src/ops/pine/editor/scripts.rs` and uses the existing
`RuntimeEvaluator`; shared Monaco behavior remains in
`crates/cli/src/ops/pine/editor/runtime.rs`. Add a focused private result type
or shaping helper only if it makes capability, resolution, and binding states
explicit. Do not add a production crate dependency.

Use `ErrorKind::Validation` for missing or ambiguous user script names and
`ErrorKind::InternalApiUnavailable` for missing methods, rejected page
operations, unreadable active identity, or mismatched binding. Preserve the
normal JSON error envelope and existing exit-code mapping.

If a separate JavaScript contract gate is necessary, use the existing
development dependency on pinned Node.js `24.18.0`; do not make Node a Cargo
runtime or ordinary-test dependency.

## Open Questions

- UNCONFIRMED: The current build exposes `openEditor` and `openScript`, but the
  owner-approved ELVN probe found that `openEditor()` did not settle and active
  readback did not become available even while a visible Pine Editor was open.
  The compatibility plan must identify a trustworthy bounded path or record a
  durable no-go before this matrix can be retried.
- The owner approved changing and saving `Testスクリプト` and
  `Testスクリプト2` for the ELVN matrix and requested no cloud-script cleanup.
  That authorization does not relax the requirement to stop before saving when
  binding verification is unavailable.

Revision note (2026-07-14): Created after the v0.27 release and current
upstream-PR triage. The plan intentionally chooses fail-closed slot rebinding
over upstream's source-only fallback because a warning cannot prevent an
explicit later save from targeting the wrong account-linked script.

Revision note (2026-07-14): Recorded the owner-approved ELVN matrix no-go. The
current build showed a visible generic Monaco editor, but legacy editor
selection and active-slot provider readback were unavailable and the internal
editor-open operation did not settle. No script was changed or saved; Active
Pine Editor compatibility now blocks the remaining matrix.
