---
name: screener-workflow
description: Inspect or change TradingView Desktop Screener screens, filters, and columns with tv.
---

# Desktop Screener workflow

Use this skill for a visible or saved Desktop Screener. For a data-only query,
prefer `tv mcp screener --market <MARKET> --limit <N>` when authenticated and
capable of answering the request. It does not operate a saved Desktop screen.
Use [MCP connection guidance](references/mcp-connection.md) if setup is needed.
Keep returned rows distinct from provider totals; no pagination/completeness
claim or automatic fallback. Explicit saved-screen work still follows below.
Desktop-free scanner queries and result interpretation belong to the optional `market-data` skill.

## Target and first read

Reuse a confirmed Screener target. Otherwise use `tv tab list` and its
`screener_targets`, then pass the intended target's `target_cli_args` to later
commands. If connection is unclear, consult
[Desktop session guidance](references/desktop-session.md).
If no target exists, `tv screener open --full-page` opens one; use it only when
opening a Screener is part of the requested workflow.

| Need | First command | Read next when needed |
| --- | --- | --- |
| Current rows | `tv screener get --limit <N>` | `tv screener status` for unclear context |
| Current/saved screen | `tv screener screens active` / `tv screener screens list` | Add `--catalog` for catalog screens |
| Filters | `tv screener filters list` | `tv screener filters actions` for supported actions |
| Columns | `tv screener columns list` | `tv screener columns config` / `actions` before editing |
| Visual evidence | `tv screenshot --region full --output <PATH>` | Only when structured state does not answer the question |

Report source, screen/filters, columns, sort, and coverage relevant to the task.
Explain why rows matched using returned filter, column, source and coverage
evidence. Preserve missing values; do not assert criteria unsupported by those fields.

For menu probes and storage-column identity, read
[discovery limits](references/discovery.md). Capability flags describe the probe,
not every operation implemented by the CLI.

## Saved-state changes

Get the concrete target and requested effect clear, reuse existing authorization,
and use `--dry-run` where supported before applying the change. A read request
alone does not authorize changing screens, filters, columns, or watchlists.

- `screens switch --name <NAME> [--catalog] --dry-run` previews selection.
- Create/rename/save-as/delete execution is restricted by code to names containing
  `CLI-Test` or `テスト`; rename requires it in both names. Save/switch have no
  test-name restriction. Read [saved-screen guidance](references/saved-screens.md)
  for dry-run effects and what post-checks establish. Save only to the verified
  intended screen.
- Use filter add/modify/remove/clear with target checks. UI and storage paths
  have different test-name guards, preview effects and readback; consult
  [filter-change guidance](references/filters.md). Option changes can clear other
  selections; broad multi-option and free-text editing are not genericized.
- Inspect column config before add/remove/reorder. Add requires a known storage
  column ID and optional JSON-object params, not display-name search. These writes
  require a test-named active screen and verify storage, not refreshed UI; read
  [column-change guidance](references/columns.md).
  `columns reset` remains deferred because a reliable default source is unknown.

Read back the requested after-state. Report remaining changed state and any
agreed cleanup. Keep real screen IDs, account-local names, storage payloads, and
target IDs out of shared artifacts.


## Visible read limits

When supported, inspect the chosen `tv spec screener` command path before use.
Status reads current state without opening the panel. Get, screens active,
filters list and columns list can temporarily open a closed panel, then attempt
to close it with Escape. `opened_for_read` records the opening;
`restored_open_state` is the initial open boolean, not a success flag. False is
expected after a successful closed-to-open-to-closed read. Returned `open` is the
captured state, not necessarily the final state. After errors, check actual UI
state because opening or cleanup can fail.

Get defaults to 20 rows and clamps positive limits to 100; zero is rejected. It
does not scroll or paginate. `visible_row_count` counts DOM table rows, not total
matches or strictly viewport-visible rows. Cells contain localized display text.
`field_values` uses displayed column labels, so duplicate labels overwrite keys
and missing headers can misalign values. Row `text` is truncated to 500 characters.

Screens active returns title text, not a saved-screen ID. Filter pills are not a
full filter definition. Columns list returns displayed names and positional
indexes, not storage column IDs. Inspect config/actions before editing. For
data-only screening use official MCP when it meets the requested criteria; it
does not reproduce saved Desktop screen state. Never silently substitute sources.
