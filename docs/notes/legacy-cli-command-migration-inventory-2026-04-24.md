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
- `alert list`
- `alert create`
- `alert delete --id`
- `indicator add`
- `indicator remove`
- `indicator toggle`
- `indicator set`
- `indicator get`
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

## Deferred larger surfaces

These old CLI surfaces are not first in line, but they are not automatically out of scope:

- `launch`
- `alert delete --all`
- alert editing / pause / resume commands
- Pine editor commands
- drawing commands
- replay commands
- tab commands
- stream commands
- UI automation commands

Before implementing these, write or update an ExecPlan that explains the downstream need, safety constraints, expected information contract, and recovery behavior.

## Explicitly not planned

- MCP server implementation

The old JavaScript project exposed both a CLI and an MCP server. This Rust project is a CLI-first migration. The MCP server is not just outside v1; it is not planned.

## Information compatibility requirement

When a command moves from this backlog to implementation, compare the old JavaScript CLI output with the Rust output. The Rust CLI may use the improved `{ success, command, data }` envelope, but it must preserve practical information that the old command exposed.

If preserving a field is impossible or undesirable, record the reason and require an explicit project decision before treating the command as migrated.
