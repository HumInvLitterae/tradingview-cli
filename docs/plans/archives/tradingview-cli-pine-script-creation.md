# Verified creation of a saved Pine script

Status: implemented and accepted locally, 2026-10-05. One executor; no
additional agents or sessions. The owner approved the new command and one live
cloud script named CLI Save Fixture, retained afterward for reuse. Existing
scripts, editor buffers, chart entities, and installed binaries are protected.
This is a required fix before preparing the next release, in the same scope as
the indicator changes. No publication or dependency change is included.

## Outcome and contract

Before: pine new creates an editor buffer; pine save invokes the current
editor's save action and cannot name a new saved script. A local source cannot
be saved as a new script through the CLI without manual naming.

After:

```sh
tv pine create --name 'CLI Save Fixture' --file fixture.pine
```

The new Desktop-backed operation requires an explicit UTF-8 file and nonblank
name/source. It trims the requested name, reads the file before connecting,
checks a fresh saved catalog for a name collision, and invokes the existing
native saveNewScript service once. The service disables overwrite. It then
checks the fresh catalog's new ID/name/version and fetches that exact revision.
Source verification permits line-ending normalization only, using the existing
Pine comparison. It neither opens nor replaces the editor buffer and does not
insert a study. The native save compiles submitted source; compilation is not
chart execution and its result is separate from persistence.

Successful data reports operation pine_create, saved true, script ID/name/version,
source_verified true, compilation evidence, and internal_api / Desktop effects.
Compilation can be false or unknown even when persistence is verified. Explicit
operation preflight failures report saved false. Local argument/file failures
precede Desktop connection. Once dispatch may have occurred, missing,
rejected, malformed, or unverifiable results report saved null and
save_outcome unknown. No automatic retry, delete, rename, or overwrite fallback
is allowed. Error messages do not echo raw provider payloads or source.

## Grounding and design

The current dispatch reads source before connecting for other file-based Pine
operations. Existing editor adapters and the shared pine_sources_match function
own their Desktop boundary. The
[new adapter](../../../crates/cli/src/ops/pine/create.rs) belongs beside those
operations, not in the credential-free Pine HTTP crate. Private tagged result types encode
not_saved, saved, and unknown so error normalization cannot infer non-creation
from a failed request. Compilation diagnostics are independently normalized.

Read-only inspection found TradingViewApi._pineEditorApi.saveNewScript on the
current Desktop. It checks plan limits, invokes saveNew with allowOverwrite false,
and processes compilation metadata. The native updater only updates chart
studies matching the returned script identity. Fresh catalog/source GETs provide
independent readback; listSavedScripts alone caches results. The inspected Pine
fetch helper calls window.fetch once; no native request retry loop was found on
this path. No private identifiers or raw live payloads belong
in this record.

Alternatives considered: extending the editor save/naming dialog would couple
file creation to current editor state; posting directly to save/new would bypass
the application's existing limit checks and handling. Use the observed native
service, with bounded verification around it. Neither alternative is needed.

## Work and throughput checkpoint

- [x] how over the affected subsystem. Trace parser, dispatch, source reader,
  runtime evaluator, native save, and output shaping.
- [x] architect for parallel design exploration. Sequential comparison above;
  skip parallel agents because owner instructions prohibit delegation.
- [x] Blocking first steps. Confirm native service and readback before writes;
  preserve the one-script live allowance until deterministic checks pass.
- [x] Independent workstreams. n/a: one executor owns the coupled adapter,
  CLI/spec, tests, and guidance.
- [x] Shared mutable state. Run Cargo checks serially. Live creation has a single
  writer; capture catalog/editor/chart state before the one permitted save.
- [x] Smallest safe decomposition. One owner keeps dispatch, error handling,
  and mutation evidence aligned without a parallel integration boundary.
- [x] Implement the new operation and affected CLI/spec/docs consumers.
  Delegate code-writing: skip because the owner prohibits additional agents.
- [x] Verify on the matching surface. Execute production JavaScript against
  deterministic fixtures, Rust envelope/CLI tests, and one real creation.
- [x] Commit coherent verified changes before release preparation; no version
  increment or rewrite of unrelated history is required.
- [x] Review affected design and simplify avoidable complexity. Interrogate:
  no delegated review; perform the review directly.
- [x] Opening a PR: skip because remote publication was not requested.

## Acceptance

Fixtures must cover collision/invalid catalog/unavailable service with zero saves;
exactly one call after dispatch; source and identity/version readback failures;
malformed or lost evaluation results; compiled success, failure, and unavailable
metadata independent of saved state; source escaping and CRLF equivalence; and
no editor/chart methods invoked. Every fixture result crosses the real Rust
normalizer. CLI tests cover required flags and local failure before connection.
Spec examples must parse and state the cloud/source effects accurately.

The live check creates only CLI Save Fixture from the existing synthetic source,
then verifies the returned ID/version/source, rejects a same-name invocation
before saving, and compares catalog/editor/chart state with its baseline. Leave
that script saved. No cloud deletion, existing-script update, or alert operation
is authorized. Reuse prior deterministic evidence for unchanged properties;
run affected tests and mandatory checks for changed code.


## Acceptance evidence

The complete local baseline passed with CARGO_BUILD_JOBS=1 and
RUST_TEST_THREADS=1: formatting, workspace/all-target/all-feature Clippy with
warnings denied, 1,183 Rust tests with zero failures and 38 ignored opt-in tests,
and diff checks. The focused Pine module had 52 passing Rust tests. The pinned
Node 24.21.0 Pine gate passed all three tests, including 33 new creation cases;
those ignored JavaScript tests were exercised separately from the Rust baseline.
All generated creation-case results crossed the real Rust operation boundary.
Required-flag, blank-name/source, missing-file, connection, and offline-spec
checks passed. An initial collapsible-if Clippy finding was corrected before
the successful full baseline. No formatter or Clippy suppressions were added.

Runtime resource checks passed all 14 validator self-tests, the standalone Pine
skill, and staged package references/parity for seven skills per root. The README
link uses the repository web reference because cli-spec is not bundled. Public
hygiene and changed-document link checks passed; temporary live data is excluded
from tracked files. No external dependency changed. The test run used temporary package version
0.37.0; the later history correction restores 0.36.0 without changing production
source. Version-specific metadata checks are separate from these feature tests.

The actual new command created exactly the one approved synthetic script and
reported saved true, source_verified true, version 1.0, and compiled true.
An independent versioned source GET matched the local fixture after line-ending
normalization. A same-name invocation returned validation / name_conflict,
saved false, and phase preflight; subsequent state matched the successful save.
The existing four catalog entries retained ID/name/version/modified fields.
The editor's full source, saved binding, modified flag, and draft state matched
its baseline. The original 27 study IDs and input values, symbol metadata, and
resolution also matched. No editor-open, source replacement, chart insertion,
alert operation, or cloud deletion was performed. The new fixture is retained.
Private snapshots and the local probe script were removed after verification.

Real compilation-failure persistence, unavailable services, and lost responses
remain fixture-qualified, not additional cloud-write trials. No cross-platform
live observation or publication occurred. The feature belongs to the same
Unreleased scope as the indicator changes. The premature candidate that omitted
this fix has been withdrawn; the installed binary remains unchanged.

The final review retained one native service call and fresh catalog/source reads.
Tagged outcomes prevent failed or malformed dispatch results from becoming a
false non-save claim. No keyboard/dialog fallback, direct-POST adapter, retry
framework, shared cache, or new dependency was needed.
