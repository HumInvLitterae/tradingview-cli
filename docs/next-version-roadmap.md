# Development direction

Prioritize reliable outcomes for commands that change account state. Keep data
sources explicit and distinguish confirmed writes, failed preflight checks and
uncertain outcomes. Public contracts belong in the [CLI specification](cli-spec.md)
and [Rust API reference](rust-api.md), rather than in execution logs.

## Next release: Screener outcome reliability

A Screener mutation must report what happened to saved data and what happened to
the temporary UI as separate facts, because retrying after a misleading report can
repeat a write. This comes before performance and structural work.

The release keeps cleanup best effort. It does not roll back saved changes, resend
failed writes or close a panel the command did not open.

Two smaller changes can accompany it because each is independent and useful alone:
removing Screener JavaScript that an expression cannot reach, and reusing HTTP
connections across `watch compare` polls. Neither claims a speedup without
measurement, and the connection change must not delay the Screener work.
Documentation also moves to the implemented bars deadlines; the behavior stays.
Order and acceptance are in the [work inventory](next-version-work-items.md).

## Later structural work

Change bars deadlines, move Screener JavaScript into standalone files, or restructure
snapshot and compare only when the inventory's stated condition is met. File size
alone is not a reason to split code.

## Remaining alert work

Indicator creation binds its readback to the response ID and compares the saved
settings with the submitted snapshot. Desktop price creation still uses content
matching; response-ID attribution is a separate improvement requiring provider
qualification. See the [work inventory](next-version-work-items.md).

Notification delivery cannot be inferred from saved settings. Automatic retries,
reactivation and waiting are not implied by successful creation confirmation.

## Release maintenance

Follow the [release packaging contract](release-packaging.md). Keep feature fixes,
dependency updates and version preparation in separate commits. Release notes
explain changes and compatibility limits; local execution history is not part
of the public documentation.
