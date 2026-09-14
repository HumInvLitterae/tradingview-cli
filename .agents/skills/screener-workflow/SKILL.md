---
name: screener-workflow
description: Inspect or change TradingView Desktop Screener screens, filters, and columns with tv.
---

# Desktop Screener workflow

Use this skill for a visible or saved Desktop Screener. Desktop-free scanner
queries and result interpretation belong to [market-data](../market-data/SKILL.md).

## Target and first read

Reuse a confirmed Screener target. Otherwise use `tv tab list` and its
`screener_targets`, then pass the intended target's `target_cli_args` to later
commands. If connection is unclear, consult
[Desktop session guidance](../chart-analysis/references/desktop-session.md).
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
For why rows matched, use the
[screen interpretation reference](../market-data/references/screening-and-comparison.md).

## Saved-state changes

Get the concrete target and requested effect clear, reuse existing authorization,
and use `--dry-run` where supported before applying the change. A read request
alone does not authorize changing screens, filters, columns, or watchlists.

- `screens switch --name <NAME> [--catalog] --dry-run` previews selection.
- For implementation/testing, use disposable names containing `CLI-Test` or
  `テスト`. Real saved-screen create/rename/save-as/delete/save needs explicit
  intent to change that account state. Save only to the verified intended screen.
- Use filter add/modify/remove/clear with target checks. Broad multi-option and
  free-text editing are not genericized.
- Inspect column config before add/remove/reorder. Add requires a known storage
  column ID and optional JSON-object params, not display-name search.
  `columns reset` remains deferred because a reliable default source is unknown.

Read back the requested after-state. Report remaining changed state and any
agreed cleanup. Keep real screen IDs, account-local names, storage payloads, and
target IDs out of shared artifacts.
