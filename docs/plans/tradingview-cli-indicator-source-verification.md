# Verify source before creating a Pine indicator alert

Status: contract and implementation approved on 2026-10-05; implementation and
focused local checks and bounded live dry-run qualification complete. The
authorized saved revision passed exact-source and line-ending checks and rejected
a textual mismatch. No alert creation is authorized by this record. The next
release version is not selected. The [inventory](../next-version-work-items.md)
owns priority.

## Outcome and demonstrated workflow

Prevent `alert create-indicator` from combining a condition inferred from one
local source revision with a different saved script revision. The existing
[indicator-alert guide](../../skills/pine-develop/references/indicator-alerts.md)
asks users to supply local source and a saved script name. This is a supported
CLI workflow; its actual use frequency and a current affected account are
UNCONFIRMED. No real incorrect alert was observed during this investigation.

Before this change, the [adapter](../../crates/cli/src/ops/alert/indicator.rs)
selected the condition from local source and combined it with saved ID/version
metadata. Dry-run could report would_create:true after catalog matching without
reading saved source. The new preflight verifies source before either mode.

Two synthetic sources, run through the existing local `pine alertconditions`
command on 2026-10-05, demonstrate the revision hazard:

```pine
//@version=6
indicator("Source identity probe")
alertcondition(close > open, "Long", "Long message")
```

This produces the best-effort candidate `Long` with ID `plot_0`. Inserting
`plot(close)` immediately before the condition preserves the script and
condition names but changes its local candidate ID to `plot_1`. Name agreement
therefore does not establish source or condition-index agreement. These are
local parser observations, not provider compilation or alert-creation evidence.

## Existing facilities and limits

[Saved-script opening](../../crates/cli/src/ops/pine/editor/scripts.rs) already
reads `pine-facade/get/<encoded ID>/<encoded version>` through the selected
Desktop session and consumes the response's source field. Reuse that endpoint
pattern, not the entire `pine open` workflow: opening also changes editor
selection and verifies a saved-script binding. `pine get` reads an editor buffer;
`pine list` returns metadata. Neither substitutes for reading a selected saved
revision without altering the editor.

The existing Pine-open fixture returns synthetic source for any non-list fetch.
It does not establish current provider support or validate the requested ID and
version. The bounded live evidence below now confirms source retrieval for one
authorized saved revision. Test the exact encoded path and selected version with
production-generated JavaScript; a canned Rust Runtime result alone is not
sufficient evidence.

The [shared Pine source comparison](../../crates/cli/src/ops/pine.rs) treats
CRLF, LF, and lone CR as equivalent. The editor and new preflight reuse it.
Do not add hashing, semantic Pine comparison, an editor workaround, a generic
source service, or a dependency for this bounded operation.

## Approved public behavior

Apply the same preflight to dry-run and normal creation after the existing local
selector validation and unique saved-script match. Read that exact saved
ID/version once, compare locally, and proceed only on a match. Never guess a
missing version as 1.0. Keep local source out of provider requests. Do not expose
saved source, IDs, digests, or raw provider errors in public output.

| Input/evidence | Before | Approved behavior |
| --- | --- | --- |
| Local and saved source match after line-ending normalization | Existing preview/create path | Same successful JSON shape and path |
| Same names, but local revision has an extra plot | Preview can report would_create:true with locally inferred plot_1; creation can advance with the saved revision's identity | Validation error before alert listing or creation, with phase:saved_source_verification and reason:source_mismatch |
| Only CRLF/LF/lone-CR differs | No comparison | Accept as a match |
| Spaces, comments, BOM, or a terminal newline differ | No comparison | Reject as source_mismatch; no automatic source repair or save |
| Missing saved ID/version, denied GET, absent/empty/non-string source, or evaluation failure | Dry-run can succeed without an ID; normal mode can default a missing version | Fail before any alert operation with internal_api_unavailable and phase:saved_source_verification; use a fixed public-safe reason |

For a source mismatch, the approved error is:

```json
{
  "success": false,
  "command": "alert",
  "error": {
    "kind": "validation",
    "message": "Local Pine source does not match the saved script version",
    "details": {
      "phase": "saved_source_verification",
      "reason": "source_mismatch"
    }
  }
}
```

For unavailable evidence, use reason:saved_identity_unavailable when ID/version
is missing and reason:saved_source_unavailable when the source cannot be read.
Those cases use error kind internal_api_unavailable. No retry, alternate source,
new opt-out flag, or changed success metadata is proposed.

This deliberately tightens both modes: scripts whose saved source cannot be
read will no longer produce a successful preview or creation attempt. Textual
comparison also rejects harmless edits other than line endings. The alternative
is retaining the current manual alignment requirement. Warning-only comparison
would still allow the mismatched combination and would not achieve this outcome.

Matching source does not prove compiled condition IDs, chart-study identity,
input completeness, or complete post-create readback. Those existing limits
remain explicit. Dry-run still is not creation acceptance.

## Work and acceptance

1. Add the saved-revision preflight using the existing Runtime and endpoint
   facilities. Keep source comparison local and preserve local validation order.
   Remove the now-invalid saved-version fallback in the affected creation path.
2. Cover exact match, line-ending equivalence, changed outputs/logic, harmless
   textual differences, missing identity/version, and malformed/error source
   responses. Assert that failure performs no alert-list/create operation and
   never includes private source/IDs in public errors. Test the exact versioned
   GET and absence of editor/account mutation using generated-JavaScript fixtures.
3. Update indicator-alert spec, CLI docs, and the standalone Pine guide together.
   Use focused serial Cargo tests, the pinned Node gate, scoped Clippy, formatting,
   public hygiene, and resource parity. No full rebuild or new dependency is needed.
4. Before adopting this tightening as release-ready, qualify one explicitly
   authorized saved script, local source file, and Desktop target with the new
   dry-run. This is catalog/source read authority only, not alert creation.
   If those inputs or permissions are absent, report fixture-only acceptance;
   do not claim current provider compatibility or add a fallback.

## Investigation evidence

On 2026-10-05, the two local parser probes above returned plot_0 and plot_1.
All six existing indicator-alert adapter tests passed with one Cargo job and one
test thread. Their FakeRuntime payloads cover existing behavior, not saved-source
verification. Source inspection located the versioned GET and line-ending rule.
No saved-source retrieval, Desktop operation, account mutation, new provider
request, or pinned JavaScript gate was run for this investigation.

## Implementation decisions and progress

The owner approved the preceding concrete behavior on 2026-10-05. One executor
performs this work without additional agents. The focused mismatch regression failed before implementation because dry-run
returned would_create:true for the different local revision. It now passes for
both dry-run and normal creation, with no alert operations.

Two private layouts were considered: keep lookup and comparison in the already
large indicator adapter, or group catalog lookup and source verification in an
adapter-local saved_script module. Use the latter so normal creation accepts
only a verified saved-script value with required ID and version. It is not a
new public API or general source service. Preserve the provider's numeric/string
version representation for preview while using the same explicit version for
GET and creation. Share the existing line-ending comparison within CLI Pine
operations; do not add a dependency or expose a new Rust library API.

The existing pinned account-JavaScript gate will execute the new generated
source-fetch expression. No new test runner or CI lane is needed.

The source GET uses the native fetch redirect:error option so a redirect cannot
silently select another resource. Lookup and evaluation errors become fixed
public-safe errors; raw provider failures and saved source are not exposed.
The verified saved-script value is constructed only after comparison. Successful
preview keeps the original numeric/string version representation, while GET and
creation use the same explicit version.

Acceptance on 2026-10-05:

- Eleven indicator-alert adapter tests passed, covering mismatch in both modes,
  exact/line-ending matches, textual differences, missing/malformed identity or
  source, sanitized evaluation failures, and unchanged creation/post-check paths.
- Fifteen existing Pine editor source tests passed after sharing the comparator.
- The pinned Node 24.18.0 account gate passed all three tests. The new test executes
  production-generated lookup and source-fetch expressions, checks the exact
  encoded ID/version URL, requires a single read, and rejects failed/malformed
  responses. Existing alert-list and watchlist gates also passed.
- The indicator-alert spec CLI test and example parser test passed. Scoped CLI
  Clippy for the library/tests passed with warnings denied; formatting passed.

All Cargo checks used one build job and one test thread, without a full workspace
suite, optimized build, dependency update, or installed-binary replacement.
Upstream CI has not run for this change.

Live qualification completed on 2026-10-05 after the owner manually restarted
Desktop. Earlier target discovery failed because no CDP listener was available.
An authorized CLI restart returned launched:true but cdp_ready:false after the
macOS launcher fallback; it did not establish readiness. Subsequent discovery
confirmed connectivity after the owner's restart. No launch defect was diagnosed.

Opening the authorized test script through pine open timed out at editor_readiness;
no verified editor binding was obtained. The check therefore used a scoped,
read-only CLI ui eval fetch of the catalog-selected saved ID/version to obtain
source in a private temporary file. This was test input preparation, not a
production fallback. The implementation itself used its normal catalog/source
preflight, independently of the editor.

Using the local debug binary with the implementation:

- Exact saved source: dry-run succeeded with would_create:true.
- A local comment addition: dry-run failed with validation and the approved
  saved_source_verification/source_mismatch details.
- Only line endings changed to CRLF: dry-run succeeded with would_create:true.

This qualifies the observed provider revision retrieval and dry-run source check.
It does not qualify Pine editor opening, compiled plot IDs, study inputs, normal
alert creation, or post-create readback. No script save/compile or alert creation
was performed. Raw source, account identifiers, target identifiers, and local
paths remain outside tracked evidence. Upstream CI and release packaging remain
separate gates; no release version has been selected.

Standalone Pine skill references, disposable resource staging with seven skills
per root, public hygiene/self-test, and diff/link checks passed. Source review
also confirmed that the alert-mutation JavaScript body is byte-identical; the
new verification occurs before entering that body.

## Final review correction, 2026-10-05

Final review found that the source comparator accepted lone CR while local
condition discovery only ended line comments at LF. A mixed-ending synthetic
source therefore matched the saved text after normalization but produced plot_0
instead of plot_1. An all-CR source beginning with a version directive produced
zero candidates. No live source containing those endings was observed.

The Pine call scanner now recognizes CR as a line boundary without changing
source byte offsets, and its line counter treats CRLF as one boundary. New
regressions failed before the correction and passed afterward for local LF,
CRLF, lone CR, and mixed endings. The adapter regression verifies both the
preview ID and the condition ID sent to the normal creation expression through
FakeRuntime; it does not create a real alert. All 26 Pine library tests and 12
indicator-alert tests passed. Scoped Pine/CLI Clippy passed with warnings denied.
The approved source-comparison contract and successful output shape are unchanged.

The account-management copy of the indicator-alert guide now matches the Pine
copy. The existing package validator also rejects drift between those copies;
its new regression failed before the validator change and passed afterward.
