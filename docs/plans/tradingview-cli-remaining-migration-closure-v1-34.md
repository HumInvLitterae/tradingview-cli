# Close remaining old CLI migration surfaces

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document follows `.agents/PLANS.md`. It is self-contained so a contributor can continue from this file and the working tree alone.

## Purpose / Big Picture

The Rust `tv` CLI has implemented nearly all practical old JavaScript `tv` CLI commands, but a small set of old CLI surfaces remains deferred. After this change, operators can use Rust `tv` for saved layout switching, bulk alert cleanup, raw Pine compile compatibility, and the old generic UI automation command group. The migration inventory should then show the old CLI command migration as closed except for MCP server implementation, which remains explicitly not planned.

The user-visible proof is that `tv layout switch`, `tv alert delete --all`, `tv pine raw-compile`, and `tv ui ...` appear in help, return the Rust JSON envelope, and pass automated tests without requiring TradingView Desktop.

## Progress

- [x] (2026-04-25 01:38Z) Read repository guidance, current migration inventory, deferred surface audit, current Rust CLI modules, and the old JavaScript implementations for layout switch, alert delete-all, Pine raw compile, and UI automation.
- [x] (2026-04-25 02:05Z) Added saved layout switch command and tests.
- [x] (2026-04-25 02:05Z) Added alert delete-all command and tests.
- [x] (2026-04-25 02:05Z) Added Pine raw-compile compatibility command and tests.
- [x] (2026-04-25 02:05Z) Added generic UI automation compatibility commands and tests.
- [x] (2026-04-25 02:05Z) Updated README, AGENTS, contract/inventory notes, remaining deferred audit, handoff note, and this ExecPlan.
- [x] (2026-04-25 02:12Z) Ran automated validation; commit remains to be created.

## Surprises & Discoveries

- Observation: The old `alert delete --all` CLI did not perform structured deletion. It opened the alerts UI/context menu and returned a note that manual confirmation was required.
  Evidence: `tradingview-mcp/src/core/alerts.js` returns `note: 'Alert deletion requires manual confirmation in the context menu.'` for `delete_all`.
- Observation: The old `layout switch` dismissed unsaved-change prompts by clicking buttons matching open-anyway, don't-save, or discard.
  Evidence: `tradingview-mcp/src/core/ui.js` calls `loadChartFromServer` and then searches all buttons for those labels.
- Observation: The old `pine raw-compile` can click "Save and add to chart" or a Pine save button, unlike Rust `pine compile`, which intentionally rejects save-related compile buttons.
  Evidence: `tradingview-mcp/src/core/pine.js` `compile()` clicks save-related buttons before falling back to Ctrl+Enter.
- Observation: The current Rust CDP boundary already supports `Input.insertText` and `Input.dispatchKeyEvent`; UI mouse commands need one small trait extension for `Input.dispatchMouseEvent`.
  Evidence: `src/cdp.rs` defines `RuntimeEvaluator::insert_text` and `RuntimeEvaluator::dispatch_key_event`.
- Observation: Automated tests covered all new command modules without requiring a live TradingView Desktop session.
  Evidence: `cargo test` passed 202 unit tests and 61 CLI contract tests after the implementation.
- Observation: The Rust lint baseline passed with warnings denied.
  Evidence: `cargo clippy --all-targets --all-features -- -D warnings` completed successfully.

## Decision Log

- Decision: Implement old CLI surfaces by default, and consult only if a command proves technically unsafe or unavailable during implementation or live smoke.
  Rationale: The user clarified that original CLI commands should generally be implemented; old CLI parity is the current priority.
  Date/Author: 2026-04-25 / Codex.
- Decision: Add `--dry-run` to destructive or high-impact Rust commands where it materially reports targets, but do not add `--yes`.
  Rationale: A confirmation flag that only blocks execution is friction without much safety; target reporting and post-action verification are more useful.
  Date/Author: 2026-04-25 / Codex.
- Decision: Keep `pine compile` safe and add `pine raw-compile` as the compatibility surface for old behavior.
  Rationale: Existing users may need the old broad button-click behavior, but downstream callers should be able to choose the safer compile path.
  Date/Author: 2026-04-25 / Codex.

## Outcomes & Retrospective

Implemented the remaining known old JavaScript CLI command surfaces in Rust. The closure adds saved layout switching, structured bulk alert deletion, raw Pine compile compatibility, and generic UI automation compatibility commands while keeping the improved Rust JSON envelope. Live smoke remains optional because several commands can mutate account, layout, Pine, or UI session state.

Automated validation passed with `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`, `git diff --check`, and the tracked-doc absolute local path grep.

## Context and Orientation

The Rust CLI is a single binary named `tv`. Command-line parsing lives in `src/cli.rs`; dispatch and JSON envelope output live in `src/main.rs`; operation functions are re-exported by `src/ops.rs`; individual command implementations live under `src/ops/`. The CDP runtime boundary is `RuntimeEvaluator` in `src/cdp.rs`; tests use `FakeRuntime` in `src/ops/test_support.rs` so they do not require TradingView Desktop.

The old JavaScript CLI came from `tradingview-mcp`. It exposed CLI command groups for layout, alert, Pine, and UI automation. The Rust CLI uses a different JSON envelope: successful command payloads go under top-level `data`, and errors go under top-level `error`. Migration requires preserving practical information, not cloning the old top-level JSON wire shape.

## Plan of Work

First, implement `layout switch`. Extend `LayoutCommand` in `src/cli.rs` with `Switch { target, dry_run }`, dispatch it from `src/main.rs`, and add `saved_layout_switch(runtime, target, dry_run)` in `src/ops/saved_layout.rs`. The command resolves target by layout id or exact case-insensitive name from `getSavedCharts`. It rejects missing, ambiguous, or id-less matches. `--dry-run` returns the matched layout without loading. Normal mode calls `window.TradingViewApi.loadChartFromServer(id)`, returns `action: "switched"`, `layout`, `layout_id`, `source`, and `dry_run: false`, and reports whether an unsaved dialog was observed or dismissed if that evidence is available. It must not write machine-specific paths.

Second, implement `alert delete --all`. Change `AlertCommand::Delete` to accept `--id`, `--all`, and `--dry-run`, requiring exactly one of `--id` or `--all`. Keep `alert delete --id` behavior compatible. Add `alert_delete_all(runtime, dry_run)` in `src/ops/alert.rs`. It lists alerts, returns target details in dry-run mode, no-ops when there are zero alerts, posts all ids to `delete_alerts` in execution mode, and verifies that the target ids are absent afterward. The payload must include counts, target ids, and source.

Third, implement `pine raw-compile`. Extend `PineCommand` and dispatch to `pine_raw_compile(runtime)`. The operation opens Pine Editor, performs the old broad compile button search including save-related buttons, falls back to Ctrl+Enter when no button is found, waits briefly, and returns `button_clicked`, `source`, `editor_open_before`, and `opened_editor`. It intentionally does not replace the safer `pine compile` diagnostics contract.

Fourth, implement `ui` command compatibility. Add a `UiCommand` enum with subcommands `click`, `keyboard`, `hover`, `scroll`, `find`, `eval`, `type`, `panel`, `fullscreen`, and `mouse`. Implement a new `src/ops/ui.rs` module. Use DOM evaluation for click/find/panel/fullscreen/eval and CDP input methods for keyboard/type/mouse/hover/scroll. Extend `RuntimeEvaluator` with `dispatch_mouse_event` and test it through `FakeRuntime`. Validate required inputs and finite coordinates before connecting where practical.

Finally, update durable docs. Move implemented items out of the deferred section in the migration inventory and remaining deferred audit. README Quick Start should include the new commands. AGENTS current status should no longer list these old CLI surfaces as remaining deferred. Handoff notes should state that old CLI migration is closed except MCP server, with any live-smoke caveats recorded.

## Concrete Steps

From the repository root, create and maintain this ExecPlan:

    pwd
    cargo fmt --check
    cargo test

After each implementation slice, run the targeted tests for the changed module. At the end, run:

    cargo fmt --check
    cargo clippy --all-targets --all-features
    cargo test
    git diff --check
    git grep -nE '(/Users/|C:\\)' -- README.md AGENTS.md docs .agents/skills || true
    git status --short

## Validation and Acceptance

Automated acceptance requires the full Rust baseline to pass. `tv layout --help` must list `list` and `switch`. `tv alert delete --help` must expose `--id`, `--all`, and `--dry-run`; invalid combinations must fail before CDP connection. `tv pine --help` must list `raw-compile`. `tv ui --help` must list the old UI automation subcommands.

Operation tests must prove layout target resolution, dry-run behavior, alert delete-all dry-run and post-verification, raw Pine compile button selection, and UI command serialization. CLI contract tests must cover the new help and validation behavior without requiring TradingView Desktop.

Live smoke is optional and separate from CI. Safe smoke commands are `tv layout switch <current-layout-id> --dry-run`, `tv alert delete --all --dry-run`, `tv ui find Chart --strategy text`, and `tv pine raw-compile` only when the current Pine Editor buffer is disposable or already intended for compile/add side effects. Destructive alert delete-all smoke should not run unless the alerts are known test alerts.

## Idempotence and Recovery

Automated tests are safe to rerun. `layout switch` can change the active saved chart layout, so live smoke should start with `--dry-run` or switch only to the current layout id. `alert delete --all` can remove account alerts, so live smoke should use `--dry-run` unless the target account state is intentionally disposable. `pine raw-compile` may save or add a study through old behavior, so smoke only with disposable Pine state. UI automation can click or type in the active page, so live smoke should prefer read-only `find` unless the target action is explicitly safe.

If a slice fails after partial edits, keep this plan and revert only the uncommitted slice edits, not unrelated user changes. If a live smoke leaves state behind, record the exact added or changed object and cleanup command in this plan before committing docs.

## Artifacts and Notes

Initial old-source evidence:

    layout switch: loadChartFromServer plus unsaved dialog button matching
    alert delete-all: context menu/manual confirmation only in old CLI
    pine raw-compile: click save/add/update buttons or Ctrl+Enter
    ui automation: click, keyboard, hover, scroll, find, eval, type, panel, fullscreen, mouse

## Interfaces and Dependencies

No new third-party Rust dependencies are required.

At completion, public operation exports should include:

    pub async fn saved_layout_switch(runtime: &mut impl RuntimeEvaluator, target: &str, dry_run: bool) -> Result<Value, AppError>
    pub async fn alert_delete_all(runtime: &mut impl RuntimeEvaluator, dry_run: bool) -> Result<Value, AppError>
    pub async fn pine_raw_compile(runtime: &mut impl RuntimeEvaluator) -> Result<Value, AppError>

The `RuntimeEvaluator` trait should additionally expose a mouse event method for UI automation:

    async fn dispatch_mouse_event(&mut self, event: MouseEvent) -> Result<(), AppError>

## Open Questions

There are no unresolved critical questions at plan creation time. If live TradingView behavior shows a command cannot be safely reproduced, pause that command and report the specific technical blocker instead of silently dropping it from migration.

## Revision Note

This ExecPlan was created to implement the user's clarified direction that original JavaScript CLI commands should generally be migrated, with consultation only when implementation reveals a technical or safety blocker.
