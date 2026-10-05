# Pine editor identity and persistence

Resolve the [Desktop session](desktop-session.md)
when necessary, then preserve the intended script and source before editing.
Reuse authorization for the same script and effects; a local coding request
alone does not authorize changing the editor or saving cloud state.

## Open, edit, validate, save

`tv pine open <NAME...>` resolves an exact or unique partial saved-script name.
Proceed only when `slot_rebound` and `binding_verified` are both true.
`switch_performed: false` means it was already active and verified. Open changes
the saved-script binding; it does not save or compile.

For a non-active script, the command requires one exact row in the popup
semantically linked to Pine's saved-script trigger. If identity cannot be
verified, stop the editor-write path. Do not substitute `pine set` followed by
save, which could overwrite another script.

`tv pine set --file <PATH>` (or stdin) replaces the local editor buffer.
`tv pine new [indicator|strategy|library]` starts an unsaved template. Neither
persists cloud state. Run `tv pine compile` when live editor verification is
requested; it can add/update a chart-local study and deliberately refuses
save-related buttons. Inspect `tv pine errors` or `tv pine console` as needed.

`tv pine save` persists only the current verified existing saved script. Claim
saved state only after successful command readback or user verification.
It does not name an unsaved editor buffer; a naming dialog can be outside the
CDP page target. Do not invent a keyboard fallback. Use the explicit file-based
create operation below when new cloud persistence is intended. `tv pine raw-compile` retains broad legacy compile behavior and can
click save-related actions; it requires explicit acceptance of that effect and
disposable Pine state, and is not an ordinary compile recovery step.


## Create a new saved script

`tv pine create --name <NAME> --file <PATH>` saves the explicit UTF-8 file under a
new name through the authenticated Desktop session. It does not open/replace
the editor or add a study. New cloud persistence must be authorized; a local
source-edit request alone does not authorize it. Reuse existing approval for
the same name, source, and effects.

The command rejects a name collision, invokes native saving without overwrite,
and verifies the new catalog ID/name/version and exact revision source. Only
line-ending differences are accepted. A successful save reports saved true and
source_verified true. Check compilation.compiled separately: false means saved
with compile errors; null means compilation evidence is unavailable. Saving
and server compilation do not prove chart execution.

Operation preflight failures report saved false. A lost save response or failed
readback reports saved null / save_outcome unknown and is never retried
automatically. Inspect the saved catalog before another attempt. Do not replace
an uncertain create with a different name, overwrite, editor save, or deletion.
The result contains account-local IDs; keep them out of public artifacts.
