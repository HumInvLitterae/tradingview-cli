# Remaining deferred surface audit 2026-04-25

This note audits the old JavaScript CLI surfaces that remain deferred after the Rust `tv` CLI implemented the core read, chart setup, Pine, drawing, replay, tab, stream, and launch workflows.

The goal is not to implement these commands immediately. The goal is to decide which remaining surfaces are worth a dedicated ExecPlan and which should remain deferred unless downstream evidence changes.

## Current recommendation

`pine save` and `draw clear` have been implemented through dedicated ExecPlans.

That closes the clearest remaining practical Pine workflow gap: the Rust CLI can now read, set, compile, analyze, check, create, open, list, save already saved scripts, and inspect Pine scripts. `pine save` remains isolated as an explicit persistence command because it writes to TradingView cloud state. Explicit named new-save for unsaved scripts remains deferred because the TradingView naming dialog can be outside the CDP page target.

Do not implement `pine save` as a side effect of another Pine command. Keep `pine compile`, `pine check`, and `pine analyze` non-persistent.

`draw clear` is now implemented as an explicit bulk chart-local cleanup command with `--dry-run`, preflight target reporting, and post-delete verification. It should not be used in live smoke when pre-existing user drawings are present.

## Deferred surface classification

| Surface | Classification | Reason |
| --- | --- | --- |
| `pine save` | `implemented` | Completes the Pine development loop as an explicit persistence command for the current saved script. |
| Pine named new-save | `research_only` | Current TradingView Desktop live smoke showed the naming dialog for unsaved scripts can be outside the CDP page target. |
| `pine raw-compile` | `likely_no_direct_clone` | The old implementation clicks compile/add buttons without the Rust safety checks and can click save-related actions. Rust already has safer `pine compile` and `pine check`. |
| `draw clear` | `implemented` | Rust preserves the old all-shapes cleanup capability but adds `--dry-run`, target reporting, and post-action verification. |
| `alert delete --all` | `high_risk_deferred` | The old implementation only opens an alerts context menu for manual confirmation. It does not provide a reliable structured bulk-delete contract. Rust already supports scoped `alert delete --id`. |
| alert edit / pause / resume | `research_only` | These are not currently implemented in Rust, and the old CLI evidence does not yet establish a safe scoped contract. |
| generic UI automation | `research_only` | The old bridge exposes broad click, keyboard, hover, scroll, mouse, find, panel, fullscreen, and arbitrary eval tools. These are intentionally outside the core CLI unless a specific workflow proves they belong. |

## Evidence from the old JavaScript CLI

The old Pine CLI exposes `raw-compile` and `save`. `raw-compile` calls the broad compile path, while `save` sends Ctrl+S and may click a visible Save button inside a dialog.

The old broad compile paths are not a good Rust contract to clone directly. They may click "Save and add to chart", a save button fallback, or keyboard shortcuts. The Rust `pine compile` command intentionally avoids save-related buttons, reports diagnostics, and keeps persistence out of compile behavior.

The old alert bulk deletion command does not actually delete all alerts through a structured API. It opens the alerts UI and a context menu, then returns a note that manual confirmation is required. Rust already chose a safer cleanup path by implementing `alert delete --id` through the current page session.

The old drawing clear command directly calls the chart API's all-shapes removal method. Rust implements `draw clear` as the explicit counterpart to that old capability, but it exposes `--dry-run`, reports the entities it would clear, and rejects a non-empty post-delete state as `internal_api_unavailable`.

The old UI automation surface is broad and generic. It can click by label/text/class, open panels, dispatch keyboard input, type text, hover, scroll, click by coordinates, find elements, toggle fullscreen, and evaluate arbitrary JavaScript. Rust should not import this as a general CLI surface without a narrower workflow reason.

## Completed Pine save contract

`pine save` was implemented in `docs/plans/tradingview-cli-pine-save-v1-31.md` with these contract choices:

- `tv pine save` saves the current Pine Editor buffer through an explicit persistence command when that buffer already belongs to a saved script.
- If a naming dialog appears for an unsaved script, the command must fail rather than keyboard-type into an unverified focus target.
- The payload reports `saved`, `action`, `name`, `dialog_handled`, `source`, `editor_open_before`, `opened_editor`, `dirty_before`, and `dirty_after`.
- Live smoke created a default-named script during a rejected keyboard fallback experiment; see the Pine save ExecPlan for the exact leftover name and id.

## Completed drawing clear contract

`draw clear` has now satisfied that requirement through `docs/plans/tradingview-cli-draw-clear-v1-32.md`.

- `tv draw clear --dry-run` is read-only and reports `before_count`, `would_clear_count`, and `cleared_entities`.
- `tv draw clear` removes all chart-local drawings through TradingView's chart API and verifies the post-clear drawing count is zero.
- Live smoke must stop after dry-run when pre-existing drawings are present.

If `alert delete --all` is reconsidered, the ExecPlan must require explicit destructive intent, preflight counts, post-action verification, and a recovery story. Old CLI parity alone is not enough.

If generic UI automation is reconsidered, start from one concrete downstream workflow. Do not add arbitrary JavaScript evaluation or coordinate clicking as a general-purpose escape hatch.

## Current status

No remaining old JavaScript CLI lifecycle pair is half-migrated in Rust.

The remaining surfaces are deferred because they are persistent, account-wide destructive, or too generic. They are not automatically out of scope, except that MCP server implementation remains explicitly not planned for this repository.
