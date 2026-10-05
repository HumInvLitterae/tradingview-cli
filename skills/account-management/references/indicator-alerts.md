# Pine indicator alerts

`tv alert create-indicator` uses Desktop and a saved Pine script; official MCP
simple-price alerts do not replace its alertcondition logic. Inspect
`tv spec alert create-indicator` when available. Supply UTF-8 source via `--file`
or nonterminal stdin, `--script` matching exactly one saved name/title, and
exactly one of `--condition-title` or `--alert-cond-id`. Use local
`tv pine alertconditions --file <source.pine>` for best-effort candidates.

Both dry-run and normal creation read the exact saved script ID/version and
compare that source with the supplied local text. Only CRLF, LF, and lone CR
are normalized. Other differences, including spaces, comments, BOM, or a terminal
newline, stop before alert listing or creation. Missing ID/version or unavailable
source also stops the operation; no default version or source fallback is used.
The command does not save, compile, or repair source. Matching text does not
prove compiled plot IDs or chart-study/input identity.
The source text is not uploaded; execution sends derived feature/condition
metadata, saved script identity/version and study inputs. It does not add a
study, alter its inputs or switch chart symbol/timeframe.

Dry-run is live: it reads the saved-script catalog and the selected revision's
source. `would_create`/`mutation_supported` describe preview assembly after a
source match, not provider creation acceptance. Input extraction, chart defaults,
and creation/readback are not exercised by preview.

Source mismatch returns a validation error with `phase: saved_source_verification`
and `reason: source_mismatch`. Missing identity/version or unavailable source
returns `internal_api_unavailable`, with reason `saved_identity_unavailable` or
`saved_source_unavailable`. These errors do not expose source text, saved IDs,
or raw provider failures.

Execution takes inputs from the first chart study matching a saved/requested
name/title. It does not resolve duplicate names by entity ID or verify that
study's source version. For that study, native input declarations and current
values must be readable arrays with unique, recognized IDs. User input IDs such
as in_0 are preserved, independently of array order. Missing or undeclared user
inputs, duplicate IDs, unknown input shapes, and absent values fail with
phase:study_input_metadata_unavailable and created:false before alert listing
or creation. Defaults do not fill missing values.

System entries text, pineId, pineVersion, pineFeatures, __fast_calc, and __profile
are excluded from user-input assignment. Generated base metadata retains its
existing handling. input_metadata.input_count counts verified user inputs, not
system entries. An empty user-input set is accepted only when declared metadata
and returned values agree on that set. This checks input completeness against
native declarations, not the saved script's version or compiled condition.

Textual input detection can be affected by comments or formatting. When inputs
are detected and no matching study is available, execution fails; otherwise base
metadata can suffice without a study. Dry-run does not check study inputs.

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
malformed creation-evaluation results, where the POST's progress cannot be
established. HTTP/provider error replies alone do not prove no side effect.
Errors returned by chart/input/list preflight before the POST retain
`created:false`; saved-source preflight errors keep their existing format.
Confirmed success still returns `created:true` with the existing shape.
No automatic retry, deletion, DOM or MCP fallback is performed. Inspect account
state before retrying and keep script/account details private.
