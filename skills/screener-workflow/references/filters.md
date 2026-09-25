# Filter changes

Inspect `tv spec screener filters <action>` when available. Filter operations
use different UI and storage paths; do not infer one persistence or permission
rule from the command family. UI add/modify has no test-name guard. Execution
of storage remove/clear/range-modify requires `CLI-Test` or `テスト` in the
screen title. These are code restrictions, separate from permission to edit.

Add searches the catalog and matches an existing range option. Supply a nonblank
name and at least one finite bound; with both bounds, max must exceed min.
Dry-run finds only the candidate, not the requested range option. Success checks
a new matching visible pill, not saved storage. Recognized greater/less labels
do not establish a uniform inclusive-bound interpretation.

Modify/remove require either a zero-based index or nonblank text, not both.
Text uses a case-insensitive substring and must match exactly one visible pill.
Modify accepts either a nonblank option or numeric bounds. Numeric presets are:
min-only 3, 5, 10, 15, 20, 30, 40, 50, 60, 70, 80, 90; or min 0 with max
3, 5, 10, 20, 30. Max-only is rejected for modify although add accepts it.
These are preset selectors, not arbitrary numeric input.

Option modification prefers a normalized exact match, then a unique substring
match in either direction. It can clear other selected options before selecting
the target. Preview opens the popover; it is not an additive multi-select API.
UI readback checks pill text, and an already-matching pill can return no change.

If both the panel and its open button are absent, numeric modify with an index
can first try current-screen storage. It supports only simple Condition
above/between filters and still applies the public preset bounds; stored bounded
ranges require exact zero. Unsupported preflight can proceed to the UI path,
but failed post-write verification does not. The storage path requests reload
without confirming visible readiness.

Remove and clear previews inspect visible targets only. Execution validates
the test-name guard and equal visible/storage counts, then maps by position.
Count agreement does not prove filter identity. Clear also requires
`--confirm-clear` and writes an empty saved filter list. Storage writes replace
filters in the fetched screen document and compare ordered definitions on
readback, without reconciling intervening edits.

For remove/clear, a full-page target can reload and poll filter count;
`visible_refresh.confirmed` checks only that count. False or skipped refresh can
accompany a successful storage change. Dialog targets skip reload. Neither this
flag nor pill text proves row coverage, freshness or every filter condition.

Dry-run can change panel/popover/focus state and can read storage.
`restored_open_state` is the initial panel state, not cleanup success. Failures
can leave UI or saved settings changed; inspect current state before retrying.
Official MCP can answer a suitable data-only query but does not edit these
Desktop filters. Keep account settings and IDs private.
