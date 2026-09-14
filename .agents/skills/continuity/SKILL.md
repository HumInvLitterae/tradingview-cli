---
name: continuity
description: Prepare or refresh a project handoff when the user explicitly requests continuity work.
disable-model-invocation: true
---

# Continuity handoff

Run only when explicitly requested. Complete the requested handoff, then return
to normal task handling; this skill does not remain active across future turns.
It assigns no PM role, delegation permission, or commit authority.

Use the existing plan or work record as the detailed source. Confirm the
relevant current files and Git state, preserving staged work, stashes, running
services, and other tasks. Keep older observations labeled as historical until
verified; a ledger does not override later user instructions.

Include the outcome, agreed constraints and approvals, protected work, current
state, next action, and unresolved questions that affect that action. Link to
supporting records rather than copying their history. Choose the length from
what the next executor needs; no minimum task count or fixed output is required.

If the user requests a saved local brief, update untracked `CONTINUITY.md` with
those facts. Otherwise provide the handoff in the response. Update the existing
work record when useful; do not add a second plan or an every-turn update rule.
