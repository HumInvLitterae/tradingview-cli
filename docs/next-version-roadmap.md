# Development direction

Prioritize reliable outcomes for commands that change account state. Keep data
sources explicit and distinguish confirmed writes, failed preflight checks and
uncertain outcomes. Public contracts belong in the [CLI specification](cli-spec.md)
and [Rust API reference](rust-api.md), rather than in execution logs.

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
