# Saved-screen changes

Inspect `tv spec screener screens <action>` when available, and use help on older
binaries. Select the intended Desktop target and establish the requested change
before executing it. Official MCP screening does not manage saved Desktop screens.

Dry-run still connects. Switch/save inspect menus; create/rename/save-as can open
and focus name dialogs without submitting. Delete preview fetches account storage.
Cleanup attempts do not reconstruct prior popups or roll back selection/edits.
`restored_open_state` is the initial panel state, not a cleanup-success flag.
After a confirmed change, a failed panel close returns success with
`cleanup.restored: false` and `cleanup.error`; `cleanup` is otherwise omitted.
Other close failures remain errors.
A delete whose readback fails adds `details.write_outcome: "uncertain"` and
`details.delete_result`. A delete request that times out, loses its connection
or throws before an answer adds only `details.write_outcome: "uncertain"`.
A list response that is not a readable list counts as a failed readback. A list
that is read and still contains the screen is an error without `write_outcome`.
Inspect storage before retrying.

| Action | Execution boundary and evidence |
| --- | --- |
| switch | Resolves one exact name in the title menu or `--catalog` scope. Missing/duplicate matches fail. An already-active title returns `switched: false`; otherwise the new title is observed and left active. No test-name restriction. |
| save | Targets the active screen, with no name argument or test-name restriction. Requires an enabled save action. `save_requested: true` with `confirmation: not_observable` confirms the request, not durable storage. |
| create / save-as | Execution requires a destination name containing case-sensitive `CLI-Test` or `テスト`. Opens create/copy dialog, submits and waits for the new active title; this is not a uniqueness, content-equivalence or storage audit. |
| rename | `--name` must match the active title even in preview; `--to` must differ after trimming. Execution requires the test substring in both names. Success observes the new title, not independent persistence. |
| delete | Resolves an exact saved name through storage. Execution requires a test name and `--confirm-delete`, and refuses an active target. Preview can report an active target without rejecting it. After deletion by ID, checks name absence in a fresh list. |

Names are trimmed and must be nonempty. Test-name limits are enforced by the CLI,
not merely a recommendation for verification. User permission does not remove
these implementation restrictions. A failed post-check can follow a completed
write; inspect current state before deciding what to do next. Do not blindly
repeat create/copy/rename/delete or claim that an error rolled the change back.
Keep real names, IDs and account storage replies private.
