# Verify source before creating a Pine indicator alert

Status: investigation complete; contract proposal awaiting owner agreement.
No implementation or new live Desktop/account operation is authorized by this
record. The next release version is not selected. The
[inventory](../next-version-work-items.md) owns priority.

## Outcome and demonstrated workflow

Prevent `alert create-indicator` from combining a condition inferred from one
local source revision with a different saved script revision. The existing
[indicator-alert guide](../../skills/pine-develop/references/indicator-alerts.md)
asks users to supply local source and a saved script name. This is a supported
CLI workflow; its actual use frequency and a current affected account are
UNCONFIRMED. No real incorrect alert was observed during this investigation.

The [adapter](../../crates/cli/src/ops/alert/indicator.rs) selects the condition
from local source, resolves a saved script by name/title, and combines the local
condition ID with the saved ID/version. Dry-run stops after catalog matching and
can report would_create:true without reading saved source.

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
version. Current live response shape and version binding remain UNCONFIRMED.
An implementation must test the exact encoded path and selected version with
production-generated JavaScript; a canned Rust Runtime result alone is not
sufficient evidence.

The [editor source comparison](../../crates/cli/src/ops/pine/editor/source.rs)
already treats CRLF, LF, and lone CR as equivalent. Reuse that comparison rule.
Do not add hashing, semantic Pine comparison, an editor workaround, a generic
source service, or a dependency for this bounded operation.

## Proposed public behavior

Apply the same preflight to dry-run and normal creation after the existing local
selector validation and unique saved-script match. Read that exact saved
ID/version once, compare locally, and proceed only on a match. Never guess a
missing version as 1.0. Keep local source out of provider requests. Do not expose
saved source, IDs, digests, or raw provider errors in public output.

| Input/evidence | Current behavior | Proposed behavior |
| --- | --- | --- |
| Local and saved source match after line-ending normalization | Existing preview/create path | Same successful JSON shape and path |
| Same names, but local revision has an extra plot | Preview can report would_create:true with locally inferred plot_1; creation can advance with the saved revision's identity | Validation error before alert listing or creation, with phase:saved_source_verification and reason:source_mismatch |
| Only CRLF/LF/lone-CR differs | No comparison | Accept as a match |
| Spaces, comments, BOM, or a terminal newline differ | No comparison | Reject as source_mismatch; no automatic source repair or save |
| Missing saved ID/version, denied GET, absent/empty/non-string source, or evaluation failure | Dry-run can succeed without an ID; normal mode can default a missing version | Fail before any alert operation with internal_api_unavailable and phase:saved_source_verification; use a fixed public-safe reason |

For a source mismatch, the proposed error is:

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

## Work and acceptance after agreement

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
