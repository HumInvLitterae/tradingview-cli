# Harden Pine Editor detection and compile labels

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document follows `.agents/PLANS.md` from the repository root.

## Purpose / Big Picture

The Rust `tv pine ...` commands depend on TradingView Desktop exposing the Pine Editor's Monaco editor instance inside the page. Upstream pull requests on the original JavaScript project report that TradingView Desktop 3.1.0 can temporarily render the Pine Editor while the React fiber path used to find Monaco is not stable, and that non-English compile buttons can be missed. After this change, existing Rust Pine commands should recover more reliably from Pine Editor transition states and should recognize Korean Add/Update-on-chart buttons without changing the public CLI shape.

## Progress

- [x] (2026-04-25T09:42:24Z) Read `.agents/PLANS.md`, current `src/ops/pine/editor.rs`, and upstream PR evidence from `#97`, `#95`, and `#50`.
- [x] (2026-04-25T09:44:39Z) Implemented direct Monaco fast path, idempotent Pine panel re-open polling, and Korean compile label matching.
- [x] (2026-04-25T09:44:39Z) Added automated fake-runtime and expression-content tests.
- [x] (2026-04-25T09:44:39Z) Updated upstream triage notes.
- [x] (2026-04-25T09:48:44Z) Ran full validation and read-only live smoke.
- [ ] Commit the completed work.

## Surprises & Discoveries

- Observation: Rust already reads button labels from `textContent`, `aria-label`, and `title`, so upstream `#95` is mostly already covered.
  Evidence: `label(button)` in both compile expressions already checks those three sources.

- Observation: Rust currently recognizes English and Japanese compile labels, but not the Korean labels from upstream `#50`.
  Evidence: `isCompileAction` checks `Add to chart`, `Update on chart`, and Japanese `チャート` with `追加` or `更新`; it does not contain `차트에 넣기` or `차트 업데이트`.

- Observation: Rust currently polls Monaco for up to 50 iterations after one open trigger, but does not re-open the Pine panel mid-poll.
  Evidence: `ensure_pine_editor_open` evaluates the open expression once, then loops over readiness checks.

- Observation: Targeted Pine tests passed after implementation.
  Evidence: `cargo test pine -- --nocapture` passed 50 unit tests and 13 CLI contract tests.

- Observation: Read-only live smoke succeeded on one current chart target and failed on two others that did not expose Pine Editor/Monaco.
  Evidence: `tv launch` reported existing CDP readiness. `tv pine get` with one explicit target returned `line_count: 7`, `char_count: 175`, `editor_open_before: true`, and `opened_editor: false`; the other explicit targets returned `internal_api_unavailable` without source mutation.

## Decision Log

- Decision: Keep all existing `tv pine` command names and payloads unchanged.
  Rationale: This is a hardening slice, not a new Pine feature or contract migration.
  Date/Author: 2026-04-25 / Codex

- Decision: Add a direct `window.monaco.editor.getEditors()` path before the existing React fiber traversal.
  Rationale: The fast path avoids transient React fiber state when TradingView exposes the global Monaco API, while the existing fallback preserves older build compatibility.
  Date/Author: 2026-04-25 / Codex

- Decision: Re-run the Pine panel open trigger during polling only when Monaco is not yet found.
  Rationale: The trigger is idempotent and recovers if the panel self-closes, while successful already-open sessions remain unchanged.
  Date/Author: 2026-04-25 / Codex

## Outcomes & Retrospective

The Pine hardening slice is complete. Existing Pine command payloads remain unchanged, Monaco lookup is more resilient through a global API fast path, panel opening is retried during polling, and Korean compile button labels are recognized. Automated validation passed, and live smoke confirmed read-only Pine source retrieval on one current chart target while preserving clear failure on targets without an available Pine Editor.

## Context and Orientation

Pine Editor behavior lives in `src/ops/pine/editor.rs`. The Rust CLI uses `RuntimeEvaluator::evaluate` to run JavaScript inside the TradingView page. `FIND_MONACO` is a JavaScript expression that returns the Monaco editor object used by TradingView's Pine Editor. `ensure_pine_editor_open` checks whether Monaco is present, opens the Pine panel if needed, and polls until Monaco appears.

`pine compile` is safer than the old JavaScript command: it refuses save-related action buttons and uses Ctrl+Enter as a fallback. `pine raw-compile` intentionally preserves the old broad button behavior. This plan must keep that split intact.

## Plan of Work

First, update `FIND_MONACO` so it tries `window.monaco.editor.getEditors()` before walking React fibers. For each editor returned by the global API, call `getContainerDomNode()` and accept the editor only when the container is inside `.pine-editor-monaco`. If that direct path throws or finds nothing, fall back to the existing DOM/fiber walk.

Second, extract the existing panel-open JavaScript into a reusable constant named `OPEN_PINE_PANEL_EXPRESSION`. Use it once after the initial missing-editor check, then run it again every tenth poll attempt while Monaco is still missing. The poll interval remains 200ms and the maximum remains 50 attempts.

Third, add Korean compile labels to both safe and raw compile button matching. `pine compile` should treat `차트에 넣기` and `차트 업데이트` as compile actions, but it should still reject save-related labels. `pine raw-compile` should use the same Korean labels as broad fallback actions.

Finally, update the upstream triage note to mark the Pine robustness cluster as addressed by this bounded Rust hardening slice, while preserving future research topics outside this change.

## Concrete Steps

From the repository root:

    cargo fmt --check
    cargo clippy --all-targets --all-features -- -D warnings
    cargo test
    git diff --check
    git grep -n 'USER'';' -- README.md docs .agents/skills || true

Also inspect the diff for local absolute paths before committing.

Live smoke, if run, should not save Pine scripts:

    target/debug/tv launch
    target/debug/tv pine get

If buffer mutation smoke is needed, first save the current source into ignored `target/`, then restore it with `tv pine set --file ...` before finishing.

## Validation and Acceptance

Automated acceptance is that tests prove the new Monaco fast path string is present, the open trigger is re-evaluated during a failed poll, and Korean compile labels are present in both safe and raw compile expressions. The full Rust baseline must pass.

Behavioral acceptance is that existing commands such as `tv pine get` and `tv pine compile` keep the same JSON payload fields. The change should only increase the chance that existing Pine commands find the editor and compile buttons on current TradingView Desktop builds.

## Idempotence and Recovery

The code changes are additive. Tests do not require TradingView Desktop and do not mutate TradingView state. Live smoke can open the Pine panel and read source. If a manual smoke changes the Pine Editor buffer, restore it from the ignored `target/` backup before committing.

## Artifacts and Notes

Relevant upstream evidence:

    #97: Direct Monaco fast path and repeated panel-open polling improve resilience during TradingView Desktop 3.1.0 state transitions.
    #95: Pine button labels may live in `title`; Rust already checks title.
    #50: Korean labels `차트에 넣기` and `차트 업데이트` should be recognized as Add/Update-on-chart actions.

## Interfaces and Dependencies

No new crate dependency is required. The implementation remains inside `src/ops/pine/editor.rs`. No new CLI command, flag, or JSON field is introduced.

## Open Questions

No critical questions are open. Live compile smoke is optional because it may add or update a chart-local study; read-only Pine smoke is enough when automated tests pass.

Revision note 2026-04-25: Updated after implementation to record completed Pine hardening work and targeted test evidence before the full baseline.
