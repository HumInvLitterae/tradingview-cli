# Pine indicator alerts

`tv alert create-indicator` uses Desktop and a saved Pine script; official MCP
simple-price alerts do not replace its alertcondition logic. Inspect
`tv spec alert create-indicator` when available. Supply UTF-8 source via `--file`
or nonterminal stdin, `--script` matching exactly one saved name/title, and
exactly one of `--condition-title` or `--alert-cond-id`. Use local
`tv pine alertconditions --file <source.pine>` for best-effort candidates.

The command does not save or compile the supplied source or compare it with
the saved script version. Plot IDs depend on preceding outputs. Verify that the
source corresponds to the intended saved version before creating an alert.
The source text is not uploaded; execution sends derived feature/condition
metadata, saved script identity/version and study inputs. It does not add a
study, alter its inputs or switch chart symbol/timeframe.

Dry-run is live: it connects and reads the saved-script catalog.
`would_create`/`mutation_supported` only describe preview assembly. Missing
script IDs can pass preview but fail execution. Input extraction, chart defaults
and provider creation/readback are not exercised by preview.

Execution takes inputs from the first chart study matching a saved/requested
name/title. It does not resolve duplicate names by entity ID or verify source
version. Returned input order becomes in_0, in_1, etc.; zero returned values do
not prove completeness. Textual input detection can be affected by comments or
formatting. When inputs are detected and no matching study is available, it
fails; otherwise base metadata can suffice without a study.

Optional symbol/resolution overrides do not change the source of study inputs.
Missing resolution, currency and saved version can default to 1, USD and 1.0.
A trimmed nonblank message overrides the source candidate message, then `(none)`
is used. The request uses dividends adjustment, on-bar-close, approximately
30-day expiry, auto-deactivation off and all notification channels off.

Readback matches an alert not previously seen by ID, its alert_cond type,
condition ID and message; symbol is checked only when reported. It does not
verify every input, saved version, resolution or notification delivery. Failed
readback does not prove no alert was created. No DOM/MCP fallback is performed;
inspect account state before retrying and keep script/account details private.
