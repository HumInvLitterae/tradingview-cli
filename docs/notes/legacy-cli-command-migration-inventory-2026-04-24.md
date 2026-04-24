# Legacy CLI command migration inventory 2026-04-24

This inventory classifies the old JavaScript `tv` CLI surface for staged Rust migration. It intentionally separates CLI migration backlog from MCP server implementation. MCP server implementation is not planned for this project.

## Classification terms

- `implemented`: present in Rust v1.
- `planned`: not implemented yet, but part of the CLI migration backlog.
- `deferred`: not implemented yet; priority or ownership needs later evidence, but it is not rejected.
- `explicitly_not_planned`: excluded by project decision.

Do not use `explicitly_not_planned` for ordinary missing old CLI commands unless a project decision explicitly excludes them.

## Implemented in Rust v1

- `status`
- `launch`
- `state`
- `info`
- `search`
- `quote`
- `values`
- `discover`
- `ui-state`
- `ohlcv --summary`
- `ohlcv --count`
- `range`
- `scroll`
- `watchlist get`
- `watchlist add`
- `watchlist remove`
- `pane list`
- `pane layout`
- `pane focus`
- `pane symbol`
- `layout list`
- `alert list`
- `alert create`
- `alert delete --id`
- `indicator add`
- `indicator remove`
- `indicator toggle`
- `indicator set`
- `indicator get`
- `draw shape`
- `draw list`
- `draw get`
- `draw remove`
- `draw clear`
- `pine get`
- `pine set`
- `pine compile`
- `pine analyze`
- `pine check`
- `pine new`
- `pine open`
- `pine save`
- `pine errors`
- `pine console`
- `pine list`
- `tab list`
- `tab switch`
- `tab new`
- `tab close`
- `replay start`
- `replay step`
- `replay stop`
- `replay status`
- `replay autoplay`
- `replay trade`
- `stream quote`
- `stream bars`
- `stream values`
- `stream lines`
- `stream labels`
- `stream tables`
- `stream all`
- `data indicator`
- `data strategy`
- `data trades`
- `data equity`
- `data lines`
- `data labels`
- `data tables`
- `data boxes`
- `data depth`
- `symbol [SYMBOL]`
- `timeframe [RESOLUTION]`
- `type [CHART_TYPE]`
- `screenshot --region full --output <PATH>`
- `screenshot --region chart --output <PATH>`

These commands still have known contract differences from the JavaScript CLI. See `docs/notes/rust-cli-contract-migration-2026-04-24.md`.

## Planned migration backlog

No high-priority planned read-only backlog remains after the diagnostic read commands slice.

## Rust-specific lifecycle considerations

`watchlist remove <SYMBOL>` is implemented as a Rust-specific lifecycle command, not direct old CLI parity. The old JavaScript CLI exposed `watchlist get` and `watchlist add`, but no remove command. Rust includes remove because `watchlist add` can leave account watchlist state behind after live smoke or downstream operator workflows.

`tab close <INDEX>` intentionally differs from the old JavaScript CLI's current-tab close command. Rust requires an explicit TradingView app-tab index and refuses to close the final TradingView app tab. `tab list` preserves the old practical chart-target list while also exposing `app_tabs` so newly opened blank app tabs can be identified and cleaned up. This preserves practical tab lifecycle behavior while reducing accidental destructive session changes.

`layout list` is implemented as a read-only saved chart layout command. It is intentionally separate from `pane layout`, which mutates the current chart grid. `layout switch` remains deferred because it loads a saved chart layout and can trigger unsaved-changes UI.

`draw clear` is implemented as the bulk cleanup pair for chart-local drawings. It preserves the old practical all-shapes removal capability, but Rust adds `--dry-run`, target reporting, and post-action verification. Live smoke should run the read-only dry-run first and must not clear pre-existing user drawings.

`pine get`, `pine set`, `pine new`, `pine open`, `pine save`, `pine compile`, `pine errors`, and `pine console` may open the Pine Editor panel to make Monaco available. `pine set` changes only the local Pine Editor buffer from stdin or `--file`. `pine new` replaces the local Pine Editor buffer with a known indicator, strategy, or library template. `pine open` loads a saved Pine script by exact name or unique partial name into the local Pine Editor buffer. Neither `pine new` nor `pine open` saves, compiles, or adds a study. `pine save` persists the current Pine Editor buffer to TradingView cloud state when the current buffer already belongs to a saved script. If a naming dialog appears for an unsaved script, Rust fails rather than typing into an unverified focus target. Explicit named new-save remains deferred because current TradingView Desktop live smoke showed that dialog can be outside the CDP target. `pine save` does not overwrite an existing script by explicit name and does not compile or add a study. `pine compile` compiles the current editor buffer and may add or update a chart-local study, but it intentionally refuses save-related action buttons and does not save or open scripts. `pine analyze` runs local static analysis without TradingView Desktop or network access. `pine check` posts source to TradingView's pine-facade compile endpoint without CDP or editor mutation. `pine list` reads saved script metadata through TradingView's pine-facade endpoint from the current page session.

`stream quote`, `stream bars`, `stream values`, `stream lines`, `stream labels`, `stream tables`, and `stream all` are implemented as read-only polling commands. They print newline-delimited JSON envelopes and emit only changed samples. They are intended for shell and external monitoring workflows rather than request-response adapters.

`launch` is implemented as a bounded local process-control command. It is intentionally safer than the old JavaScript CLI: Rust defaults to no-kill behavior, treats an already responding CDP endpoint as success, and requires explicit `--kill-existing` before attempting to terminate existing TradingView processes. It does not mutate account, chart, Pine, drawing, alert, replay, tab, or watchlist state.

## Deferred larger surfaces

These old CLI surfaces are not first in line, but they are not automatically out of scope:

- `alert delete --all`
- alert editing / pause / resume commands
- `layout switch`
- Pine editor raw compile command: `pine raw-compile`
- Pine explicit named new-save flow for unsaved scripts
- UI automation commands

See `docs/notes/remaining-deferred-surface-audit-2026-04-25.md` for the current classification of these remaining surfaces. That audit now treats `layout list`, `pine save` for already saved scripts, and `draw clear` as implemented, treats `pine raw-compile` as a likely no-direct-clone surface, and keeps `layout switch`, explicit named new-save, alert bulk destructive commands, alert edit/pause/resume, and generic UI automation deferred.

Before implementing these, write or update an ExecPlan that explains the downstream need, safety constraints, expected information contract, and recovery behavior.

## Explicitly not planned

- MCP server implementation

The old JavaScript project exposed both a CLI and an MCP server. This Rust project is a CLI-first migration. The MCP server is not just outside v1; it is not planned.

## Information compatibility requirement

When a command moves from this backlog to implementation, compare the old JavaScript CLI output with the Rust output. The Rust CLI may use the improved `{ success, command, data }` envelope, but it must preserve practical information that the old command exposed.

If preserving a field is impossible or undesirable, record the reason and require an explicit project decision before treating the command as migrated.
