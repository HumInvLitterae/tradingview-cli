# Replay attachments and end state

Add `--attach-ohlcv-summary [--ohlcv-count <N>]` to `tv replay log --steps <N>`
when selected-chart OHLCV summary should accompany each step. Add
`--attach-chart-screenshot --screenshot-output-dir <DIR>` when each successful
step needs a deterministic chart PNG. Existing files are not overwritten;
attachment failure is separate from Replay step failure. Inspect the JSONL
record and attachment result rather than assuming one proves the other.

For a one-off image outside the step log, use
`tv screenshot --region chart --output <PATH>`.

Before changing Replay, note whether it is already active and what end state
the user requested. At the agreed endpoint, use `tv replay status` and relevant
trade readback to confirm it. Execute `tv replay trade close` or `tv replay stop`
only when that effect is requested or included in the approved practice scope.
Report failed or unknown outcomes without silently retrying a state-changing
step. Record intentionally retained Replay state for the user.
