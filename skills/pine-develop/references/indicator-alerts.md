# Pine indicator alerts

`tv alert create-indicator` uses Desktop and a saved Pine script; official MCP
simple-price alerts do not replace its alertcondition logic. Inspect
`tv spec alert create-indicator` when available. Supply UTF-8 source via `--file`
or nonterminal stdin, `--script` matching exactly one saved name/title, and
exactly one of `--condition-title` or `--alert-cond-id`. Use local
`tv pine alertconditions --file <source.pine>` for best-effort candidates.

Add `--verify` to check the saved compiled condition and required chart inputs.
Add `--study-id <entity_id>` to select one exact instance; it implies `--verify`.
The ID is case-sensitive, trimmed, and nonblank. Use `tv state` to find study IDs.
Existing invocations without either flag retain their behavior and JSON.

```sh
tv --target-id <target_id> alert create-indicator \
  --script "Example indicator" --file <source.pine> \
  --condition-title "Example condition" --verify --dry-run
```

Both dry-run and normal creation read the exact saved script ID/version and
compare that source with the supplied local text. Only CRLF, LF, and lone CR
are normalized. Other differences, including spaces, comments, BOM, or a terminal
newline, stop before alert listing or creation. Missing ID/version or unavailable
source also stops the operation; no default version or source fallback is used.
The command does not edit/save source, compile through the Pine Editor, add a
study, alter its inputs, or switch chart symbol/timeframe. Source text is not
uploaded. Normal execution sends derived feature/condition metadata, saved
script identity/version and the prepared study inputs.

## Opt-in verification

Verified preview and creation read compiled metadata for that exact saved
revision through the existing Desktop session. They require agreement on saved
ID/version, the selected alertcondition's plot ID/type, and its title when the
local parser knows a literal title. Explicit condition-ID selection with an
unknown local title verifies that exact ID/type without claiming title agreement.
The metadata service is internal and has no stability guarantee. Unavailable
metadata fails the operation rather than falling back to legacy behavior.

With zero compiled user inputs, no chart study is required unless `--study-id`
was explicitly supplied. With inputs, automatic selection requires exactly one
chart instance matching the saved ID/version. Multiple matches require
`--study-id`; an explicit ID must exist and match the saved revision. Native
identity representations and compiled/native input declarations must agree.
Unreadable potentially matching metadata stops automatic selection; explicit
selection need not read unrelated instances. Saved defaults never replace the
selected instance's current values. Values are copied before account requests.

Verified dry-run performs this same preflight, resolves symbol/resolution, and
stops before every alert-list/create request. It is a live read, not an offline
check or proof of provider acceptance. Only verified successes add `verification`:

```json
{"condition_source":"saved_compilation","input_source":"none","input_count":0,"study":null}
```

For chart inputs, `verification.input_source` is `active_chart_study` and `study`
contains `entity_id` plus `selection: explicit` or `unique`. An explicitly
selected no-input study is also reported, with input_source none and count zero.
Preview's request contains resolved symbol/resolution. Raw compiled results and
saved IDs are not returned. Stronger preflight does not expand post-create
readback or notification guarantees.

## Existing behavior without verification

Legacy dry-run reads the saved catalog and source only. It does not check chart
inputs, compiled conditions, or chart defaults. `would_create` and
`mutation_supported` describe preview assembly, not provider acceptance.

Legacy normal creation takes inputs from the first chart study matching a
saved/requested name/title. It does not resolve duplicates by entity ID or verify
that study's saved version. Textual input detection (`input.` or `input(`) can
be affected by comments or formatting. If inputs are detected and no matching
study is available, execution fails; otherwise base metadata can suffice without
a study. Source equality alone does not prove compiled condition or study identity.

## Input validation and failures

Legacy and verified chart-input preparation both require readable native
declarations and values with unique, recognized IDs. User IDs such as in_0 are preserved regardless
of array order. Missing/undeclared inputs, duplicate IDs, unknown shapes, and
absent values fail before alert operations with
`phase: study_input_metadata_unavailable` and `created:false`. Defaults do not
fill missing values. System text, pineId, pineVersion, pineFeatures, __fast_calc,
and __profile entries are excluded from user-input assignment. Generated base
metadata retains its handling; input_metadata.input_count counts user inputs.

Source mismatch returns validation with `phase: saved_source_verification` and
`reason: source_mismatch`. Missing identity/version or unavailable source returns
internal_api_unavailable with saved_identity_unavailable or
saved_source_unavailable; these errors retain their existing format.

Verified compiled failures use `phase: compiled_condition_verification` with
saved_revision_mismatch, condition_mismatch, or compiled_metadata_unavailable.
Study failures use `phase: study_identity_verification` with study_not_found,
saved_revision_mismatch, no_matching_study, ambiguous_study, or
study_metadata_unavailable. Ambiguity includes match_count. Mismatches are
validation errors; unavailable evidence is internal_api_unavailable. These
returned preflight errors have created:false. Verified preview evaluation
failures also have created:false and never an unknown account-write outcome.

## Creation defaults and readback

Optional symbol/resolution overrides do not change the source of study inputs.
Missing resolution and currency can default to 1 and USD; saved version is required.
A trimmed nonblank message overrides the source candidate message, then `(none)`
is used. The request uses dividends adjustment, on-bar-close, approximately
30-day expiry, auto-deactivation off and all notification channels off.

Readback matches an alert not previously seen by ID, its alert_cond type,
condition ID and message; symbol is checked only when reported. It does not
verify every input, saved version, resolution or notification delivery. Failed
readback does not prove no alert was created. Once a creation POST is attempted,
request/response failures and failed readback return error details with
`created:null` and `creation_outcome:"unknown"`. This also covers unavailable or
malformed normal creation-evaluation results. HTTP/provider errors alone do not
prove absence of a write. Returned chart/input/list preflight failures retain
created:false. Confirmed normal success returns created:true; only opt-in
successes add the verification summary. No automatic retry, deletion, DOM or MCP
fallback is performed. Inspect account state before retrying and keep
script/account details private.
