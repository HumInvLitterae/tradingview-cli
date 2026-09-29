# Post-v0.33.0 ordered work inventory

Direction approved 2026-09-29. [Roadmap](next-version-roadmap.md) ·
[Contracts and acceptance](plans/tradingview-cli-analysis-reliability.md).

| Order | Work | State / completion condition |
| --- | --- | --- |
| 1 | Close v0.33.0 | Release and successful four-target workflow verified; completed record archived. Repository notes example is corrected in b147bc1; public Release-body correction is still pending authorization. |
| 2 | MCP provider-failure diagnostics | Proposal ready for review. Preserve mcp_error.v1/code/stage/attempts; add safe textual clues only for a provider-declared failure. No automatic retry. |
| 3 | Equity meaning and zero values | Proposal ready for review. Label observed source separately from confirmed series meaning; preserve raw layout/selection and zero drawdown. |
| 4 | Follow-up effects | Proposal ready for review. Correct chart_quote non_mutating, retain its chart-only meaning, and describe file effects separately. Check public Rust struct compatibility. |
| 5 | Legacy account operations | Review candidate, not implementation scope. Prioritize actual use of uncertain-write fallback, alert condition/persistence and saved-indicator identity; prefer explicit MCP for supported workflows. |
| 6 | Output schemas and offline validation | Design candidate after contract cleanup. Start with actual chart/data consumers; no schema catalog or validation command is approved yet. |
| 7 | Qualify next release | Choose version after contracts settle; use focused local checks and candidate CI, then a separate final version/notes commit. |

## Follow-ups, not release gates

- Remaining spec annotations: diagnose quote-data, scanner hotlist, data depth,
  discover and spec. Current semantic coverage is 164/169; advance for a concrete
  workflow rather than a completion percentage.
- Official technical snapshots are deferred; current provider failure evidence
  is not permission to implement a guessed decoder or silently change source.
- Alert-history nonempty native normalization/default-deadline success, graphical
  Linux OAuth and Windows skill-manager execution remain unverified. Reuse
  fixtures and existing platform evidence without broadening their claims.
- Downstream reports v0.33.0 skill synchronization and 12 MCP-bars observation
  tests passing, with no Rust/CLI/persistence migration needed. This is a reported
  downstream result, not a new upstream runtime test.

## Working limits

The PM works alone. Use one Cargo build job and one test thread, no automatic
heavy pre-push checks or repeated release builds. Preserve unrelated work and
stashes. No live Desktop/account changes, new authentication, provider probes,
dependencies, downstream edits or push/publication are implied by this plan.
Existing applicable approvals remain valid; concrete public-contract proposals
must be agreed before implementation. Keep tracked records concise and public-safe.
