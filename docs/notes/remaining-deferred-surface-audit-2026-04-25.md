# Remaining deferred surface audit 2026-04-25

This note audits the old JavaScript CLI surfaces that remain deferred after the Rust `tv` CLI implemented the core read, chart setup, Pine, drawing, replay, tab, stream, and launch workflows.

The original goal was to decide which remaining surfaces were worth a dedicated ExecPlan. The remaining old CLI migration closure has since been implemented in `docs/plans/archives/tradingview-cli-remaining-migration-closure.md`.

## Current recommendation

`layout list`, `pine save`, `draw clear`, `layout switch`, `alert delete --all`, `pine raw-compile`, and generic `ui` compatibility commands have been implemented through dedicated ExecPlans.

That closes the clearest remaining practical Pine workflow gap: the Rust CLI can now read, set, compile, analyze, check, create, open, list, save already saved scripts, and inspect Pine scripts. `pine save` remains isolated as an explicit persistence command because it writes to TradingView cloud state. Explicit named new-save for unsaved scripts remains deferred because the TradingView naming dialog can be outside the CDP page target.

Do not implement `pine save` as a side effect of another Pine command. Keep `pine compile`, `pine check`, and `pine analyze` non-persistent.

`draw clear` is now implemented as an explicit bulk chart-local cleanup command with `--dry-run`, preflight target reporting, and post-delete verification. It should not be used in live smoke when pre-existing user drawings are present.

`layout switch` is implemented as an explicit saved chart layout mutation with `--dry-run` target reporting. It resolves layout ids and exact case-insensitive names, avoiding the old CLI's partial-match fallback. It does not automatically dismiss unsaved-change dialogs.

## Deferred surface classification

| Surface | Classification | Reason |
| --- | --- | --- |
| `layout list` | `implemented` | Restores the read-only saved layout inventory from the old CLI. |
| `layout switch` | `implemented` | Loads a saved chart layout by exact id/name and supports `--dry-run`; it avoids old partial matching and does not auto-dismiss unsaved-change dialogs. |
| `pine save` | `implemented` | Completes the Pine development loop as an explicit persistence command for the current saved script. |
| Pine named new-save | `research_only` | Current TradingView Desktop live smoke showed the naming dialog for unsaved scripts can be outside the CDP page target. |
| `pine raw-compile` | `implemented` | Preserves the old broad button behavior as a separate compatibility command while keeping safer `pine compile` unchanged. |
| `draw clear` | `implemented` | Rust preserves the old all-shapes cleanup capability but adds `--dry-run`, target reporting, and post-action verification. |
| `alert delete --all` | `implemented` | Rust implements a structured bulk-delete contract with `--dry-run`, target ids, and post-delete verification. |
| alert edit / pause / resume | `not_old_cli_backlog` | These were not found as old JavaScript CLI commands during the migration closure pass. Treat them as future feature research, not remaining migration. |
| generic UI automation | `implemented` | The old click, keyboard, hover, scroll, find, eval, type, panel, fullscreen, and mouse commands are implemented as compatibility commands. `ui eval` is default-disabled behind `TV_ALLOW_UNSAFE_UI_EVAL=1` because it runs arbitrary JavaScript in the authenticated TradingView page context. |

## Evidence from the old JavaScript CLI

The old Pine CLI exposes `raw-compile` and `save`. `raw-compile` calls the broad compile path, while `save` sends Ctrl+S and may click a visible Save button inside a dialog.

The old broad compile paths may click "Save and add to chart", a save button fallback, or keyboard shortcuts. Rust keeps `pine compile` safe and adds `pine raw-compile` for compatibility with that broader old behavior.

The old alert bulk deletion command did not actually delete all alerts through a structured API. It opened the alerts UI and a context menu, then returned a note that manual confirmation was required. Rust implements `alert delete --all` through the same internal alert endpoint family used for `alert delete --id`, with dry-run target reporting and post-action verification.

The old layout list command reads saved chart layouts through `window.TradingViewApi.getSavedCharts`. Rust implements this as `tv layout list`. Rust also implements `tv layout switch <TARGET> [--dry-run]`, but deliberately avoids the old automatic unsaved-dialog dismissal.

The old drawing clear command directly calls the chart API's all-shapes removal method. Rust implements `draw clear` as the explicit counterpart to that old capability, but it exposes `--dry-run`, reports the entities it would clear, and rejects a non-empty post-delete state as `internal_api_unavailable`.

The old UI automation surface is broad and generic. Rust now implements it as compatibility surface, but higher-level `tv` commands should remain preferred for stable workflows.

## Completed Pine save contract

`pine save` was implemented in `docs/plans/archives/tradingview-cli-pine-save.md` with these contract choices:

- `tv pine save` saves the current Pine Editor buffer through an explicit persistence command when that buffer already belongs to a saved script.
- If a naming dialog appears for an unsaved script, the command must fail rather than keyboard-type into an unverified focus target.
- The payload reports `saved`, `action`, `name`, `dialog_handled`, `source`, `editor_open_before`, `opened_editor`, `dirty_before`, and `dirty_after`.
- Live smoke created a default-named script during a rejected keyboard fallback experiment; see the Pine save ExecPlan for the exact leftover name and id.

## Completed drawing clear contract

`draw clear` has now satisfied that requirement through `docs/plans/archives/tradingview-cli-draw-clear.md`.

- `tv draw clear --dry-run` is read-only and reports `before_count`, `would_clear_count`, and `cleared_entities`.
- `tv draw clear` removes all chart-local drawings through TradingView's chart API and verifies the post-clear drawing count is zero.
- Live smoke must stop after dry-run when pre-existing drawings are present.

## Completed saved layout list contract

`layout list` has been implemented through `docs/plans/archives/tradingview-cli-layout-list.md`.

- `tv layout list` is read-only and reports `layout_count`, `source`, and `layouts`.
- Layout rows expose `id`, `name`, `symbol`, `resolution`, and `modified`.
- Read failures remain visible as `data.error` with an empty layout list.

## Current status

No remaining old JavaScript CLI command surface is known to be unmigrated in Rust after the remaining migration closure slice. MCP server implementation remains explicitly not planned for this repository. Alert edit/pause/resume and Pine named new-save are future feature research topics rather than confirmed old CLI migration backlog.
