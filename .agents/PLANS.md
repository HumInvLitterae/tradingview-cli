# Planning substantial changes

Use an ExecPlan when a feature or refactor has multiple dependent changes,
material contract decisions, or verification that another contributor needs to
resume. Small fixes and routine reviews do not need a separate plan. Reuse an
existing goal or work record when it already serves this purpose.

## What a plan needs

The reader has the repository and can inspect its code and documentation. Link
to current sources instead of copying manuals, crate maps, or prior plans.
Make the change-specific decisions explicit:

- **Outcome and scope:** the observable result, affected consumers, and limits.
- **Decisions and authority:** agreed contracts, important tradeoffs, protected
  work, and any missing permission for concrete targets and effects. Carry
  existing approvals forward, including verification and fixes within limits.
  Identify foreseeable permission needs early; prepare reviewable results
  before publishing or irreversible actions.
- **Work:** the smallest coherent sequence that delivers the outcome. Specify
  fragile ordering or recovery where it matters; let the executor choose
  ordinary implementation details. Do not mandate a task count.
- **Acceptance:** the behavior to demonstrate, appropriate commands or
  fixtures, expected evidence, and live observations required by the task.
  Distinguish checks that were run, skipped, or only proposed.
- **Progress:** completed work, the current next action, unresolved decisions,
  and concise evidence. Update when those facts change or work is handed off,
  not after every tool call.

Use headings, prose, tables, or checklists to make these facts easy to find.
No fixed skeleton, minimum length, novice tutorial, or repeated knowledge is
required. Include before/after examples for public contracts and concrete
failure handling for operations with partial or unknown outcomes.

## Execution and closeout

A plan records scope and authority; it does not grant permission to publish,
change live state, or start additional agents. Continue through authorized
implementation, verification, and fixes without inserting new review stops.
Honor reviews or return points explicitly agreed for the task.

Use the [development validation guidance](../docs/development.md#validation-baseline)
for checks. Reuse evidence while the relevant code, dependencies, environment,
and operation remain applicable. Recheck the changed properties when inputs
change, and explain material gaps instead of treating all old evidence as void.

At completion, record the outcome and remaining limitations, move completed
plans to `docs/plans/archives`, and update their current entry links. Preserve
frozen results. Existing plans need not be reformatted to this guidance.

## State ownership and handoffs

The roadmap records direction; the work inventory records order and links to
plans; each plan owns its detailed decisions and acceptance. The plan index is
an entry point, not another copy of status history.

When explicitly asked for a handoff, summarize the goal, confirmed constraints
and approvals, protected work, current state, and next action. Link to the
existing plan. Use a local untracked `CONTINUITY.md` only when a persistent
brief is requested, and label stale or unverified observations. Do not infer a
PM role, create a second mandatory plan, or maintain an endless ledger loop.
