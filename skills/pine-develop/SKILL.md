---
name: pine-develop
description: Edit and validate Pine Script with tv for local Pine checks or TradingView Pine Editor workflows.
---

# Pine development

Choose the requested level of validation. A local script task does not require
chart prices, OHLCV, Desktop startup, or cloud persistence.

| Task | Command or action | Effect / next step |
| --- | --- | --- |
| Write or review a local script | Edit the requested project file | Use offline analysis when useful. |
| Offline static analysis | `tv pine analyze --file <PATH>` | No Desktop or network; static findings are not compile proof. |
| TradingView server compile check | `tv pine check --file <PATH>` | Sends source to pine-facade; requires that network/source submission be in scope. Does not mutate the editor. |
| Read current editor or saved list | `tv pine get`, `tv pine errors`, `tv pine console`, `tv pine list` | Resolve the [Desktop session](references/desktop-session.md) if needed. |
| Edit a saved script or create a buffer | `tv pine open <NAME...>`, `tv pine set --file <PATH>`, `tv pine new` | Read [editor identity and persistence](references/workflow.md) before mutation. |
| Compile in the live editor | `tv pine compile` | May add/update a chart-local study; does not save. |
| Persist an existing saved script | `tv pine save` | Explicit cloud write; requires verified script binding and intent to save. |

Use the editor reference for the guarded open/set/compile/save sequence and
its failure handling. Treat local analysis, server check, editor compile, and
cloud save as distinct results. Finish at the requested validation/persistence
level, reporting errors and unverified levels that matter to that request.

For a requested Pine `alertcondition()` alert, read
[indicator alert guidance](references/indicator-alerts.md). A live dry-run does
not verify source/version identity, study inputs or provider creation.
