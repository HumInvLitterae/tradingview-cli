# Pine editor identity and persistence

Resolve the [Desktop session](../../chart-analysis/references/desktop-session.md)
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
Naming and saving a new unsaved script remains deferred because the Desktop
naming dialog can be outside the CDP page target. Do not invent a keyboard
fallback. `tv pine raw-compile` retains broad legacy compile behavior and can
click save-related actions; it requires explicit acceptance of that effect and
disposable Pine state, and is not an ordinary compile recovery step.
