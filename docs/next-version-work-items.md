# Post-v0.33.0 ordered work inventory

Direction approved 2026-09-29. [Roadmap](next-version-roadmap.md) ·
[Contracts and acceptance](plans/tradingview-cli-analysis-reliability.md).

| Order | Work | State / completion condition |
| --- | --- | --- |
| 1 | Close v0.33.0 | Release and successful four-target workflow verified; completed record archived. Repository notes example is corrected in b147bc1; public Release-body correction is still pending authorization. |
| 2 | MCP provider-failure diagnostics | Implemented with focused normalization/privacy and shared-classifier fixtures. Existing mcp_error.v1/code/stage/attempts and retry policy remain unchanged. |
| 3 | Equity meaning and zero values | Implemented: extraction/meaning labels and zero drawdown preservation, with generated-JavaScript fixtures for branch selection and missing data. |
| 4 | Follow-up effects | Implemented with serialized chart/file effects and existing Rust struct construction retained. Market fixtures pass; downstream must update its stale chart_quote expectation when adopting. |
| 5 | Legacy account operations | All three approved corrections implemented with focused Rust and generated-JavaScript fixtures. Downstream duplicate matching and provider migration remain owner follow-ups. See the existing plan for contracts and acceptance. |
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
