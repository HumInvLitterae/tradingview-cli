---
name: screener-workflow
description: Operate TradingView Stock Screener with the Rust `tv` CLI. Use when the user wants Screener reads, scanner-to-screener workflow, saved screen switching, test screen setup, filter or column changes, or cleanup of disposable Screener state.
---

# Screener Workflow

Use this skill for TradingView Stock Screener work through the Rust `tv` CLI.

## Start With The Target

1. Run `tv status`.
2. Run `tv tab list` and prefer a full-page Screener target from
   `screener_targets`.
3. Use the returned `target_cli_args`, for example
   `tv --target-id <ID> screener ...`, for follow-up commands.
4. If no full-page Screener target is available, run
   `tv screener open --full-page` and then use the returned `target_cli_args`.

## Read First

Start with read-only commands before changing saved state:

- `tv screener status`
- `tv screener get --limit <N>`
- `tv screener screens active`
- `tv screener screens list`
- `tv screener screens list --catalog`
- `tv screener filters list`
- `tv screener filters actions`
- `tv screener columns list`
- `tv screener columns config`
- `tv screener columns actions`

Use `tv scanner hotlist` and `tv scanner scan` for broad market discovery. Use
`tv screener get` when the user needs the currently visible Screener rows.
Use `tv scanner metainfo --field <FIELD>` when you need to confirm whether a
scanner field exists before building a scan.

## Interpreting Results

For result explanation, use `screener-result-analysis`. Keep this skill focused
on operating the Screener target and saved state. A compact first pass is
usually enough: source, filters or screen name, columns, sort key, result count,
top rows, and any intended next read.

## Mutations

Use `--dry-run` first whenever the command supports it. Explain the expected
saved-state change and get user approval before normal mutation.

- Screen switching: `tv screener screens switch --name <NAME> [--catalog] --dry-run`.
- Screen lifecycle: use disposable names containing `CLI-Test` or `テスト` for
  create, rename, save-as, and delete.
- Screen save: use only on prepared test or disposable screens.
- Filters: use `filters add`, `modify`, `remove`, and `clear` with dry-run
  target checks first.
- Columns: use `columns config` before `columns add`, `remove`, or `reorder`.
  `columns add` requires a known storage column id and optional JSON-object
  params; it is not display-name search.

## Boundaries

- `columns reset` is deferred because no reliable default source is confirmed.
- Broad multi-option and free-text filter editing are not genericized.
- Normal screen lifecycle, storage-backed filter cleanup, and column mutations
  should stay on test or disposable screens unless the user explicitly accepts
  account-state changes.
- Do not record real screen ids, account-local names, raw storage payloads, or
  target ids in shared notes.

## Reporting

Report which target was used, which commands were read-only or dry-run, what
mutated, and whether post-checks confirmed the requested after-state. If any
test screen, filter, or column remains changed, name only the visible disposable
object and the intended cleanup command.
