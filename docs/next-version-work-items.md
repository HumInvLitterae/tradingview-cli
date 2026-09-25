# v0.33.0 ordered work inventory

Current state: 2026-09-25. Direction: [roadmap](next-version-roadmap.md).
Contracts, approvals and dated evidence:
[one active work record](plans/tradingview-cli-mcp-operational-improvements.md).

| Order | Work | Current state and remaining acceptance |
| --- | --- | --- |
| 1 | Close v0.32.0 and establish baseline | Complete: publication/workflow verified, old record archived. Historical platform evidence is not current-candidate CI proof. |
| 2 | Settle new read contracts and verification scope | Approved. History fields are qualified; technical fields remain unqualified and their implementation is deferred. The separately approved timeout option uses `--timeout`, with no `--timeout-secs` alias. |
| 3 | Improve login/upgrade operation | Implemented. Focused tests cover terminal-only guidance, captured output and credential reuse. A fresh interactive native login was not repeated for the wording change. |
| 4 | Implement alert history | Implemented and fixture-verified. Nonempty provider structure was observed; the public service succeeded with explicit 90 seconds and empty history. Default-30-second native success and nonempty native normalization remain unqualified; coverage is always unconfirmed. |
| 5 | Defer official technical snapshot | Owner deferred implementation on 2026-09-25. Not a v0.33.0 release prerequisite. Preserve the design and provider failure evidence; resume only when the owner resumes work with usable evidence or a concrete provider explanation. |
| 6 | Measure and optimize transport | Implemented and measured. A single-page mutation/readback flow uses one initialization/catalog instead of two. Lazy pagination, admission, deadlines and separate readback failures are fixture-verified. No live latency improvement is claimed. |
| 7 | Stabilize affected fixtures | Scoped serial regression passed; no new timing defect was established. Current-candidate default-concurrency and cross-platform CI remain pending. Correct concrete failures if observed; do not weaken production deadlines or assertions. |
| 8 | Integrate documentation and standalone skills | Complete for implemented behavior: help, usage, source taxonomy and standalone MCP references updated and checked. Add technical snapshot guidance only after its implementation. |
| 9 | Distribute standalone skills from one source | Implemented: root `skills/` owns seven runtime skills; local agent links reuse them and archives copy real files. gh local install and npm discovery confirmed seven skills. Remote installation of this unpublished layout and Windows execution remain unverified. |
| 10 | Add offline command specifications | Implemented: offline `tv spec` index and clap-derived command detail, with primary MCP read/history and watchlist/alert mutation semantics, shared validation constants and conditional Desktop chart-control variants plus launch, tab changes, chart comparison and Replay. Unannotated paths and partial validation remain explicit. Schema export and offline request validation follow separately. See the active work record. |
| 11 | Qualify and prepare release | Pending. Qualify the updated dependency graph, review remaining native limits, obtain current-candidate platform/CI evidence, then prepare version and notes separately. Publishing remains separately authorized. |

## Completed operating prerequisite

Heavy local pre-push checks are opt-in. Local Cargo uses one build job and one
test thread, with existing artifacts and focused checks. The current workspace
version remains 0.32.0. The owner updated rmcp to 3.4.1 and thiserror to
2.0.21; affected regression checks must use the updated lockfile.

## Next actions and boundaries

- Technical data: implementation is deferred; retain the AAPL D/W/M/2h
  design. A public-safe provider inquiry draft exists in the active record but has not been sent. Do not retry
  the same failure on each continuation, invent response fields or substitute
  `mcp symbol`/legacy data as the dedicated tool's output.
- Read deadlines: `--timeout` accepts 1..180 seconds only for provider reads;
  omitted reads retain 30 seconds. Rejected bounds/operations fail before
  credential/provider access. The explicit-90-second native history result is
  separate from the earlier default timeout; longer waiting does not cure the
  technical application's received failure.
- CI: once the owner makes the candidate available to CI, inspect its results.
  Existing macOS focused checks and prior-release Windows/Linux evidence do not
  establish current-candidate platform success. No push is implied by this list.
- Release scope: the owner explicitly deferred technical snapshots on 2026-09-25.
  Finish qualification of implemented changes; do not add replacement features.
- Next order: review remaining dependency regression coverage, current-candidate
  CI and
  review of native evidence limits, final docs/standalone-package checks, then
  separate release version/notes preparation. Avoid repeating unchanged checks.

No credentials, account-local payloads or machine paths belong in tracked
records. Keep private downstream collection policy and analytical admission
outside this repository. The roadmap retains the existing CDP/deferred-feature
triggers; dependency maintenance does not promote those features automatically.

The owner requested a final outer Retry-After check and then waiting. That one
technical read returned HTTP 200 without Retry-After and the same application
failure. Internal screener headers remain unknown. Additional live checks are
stopped; no automatic polling or inquiry has been initiated.

After the owner resumed the paused investigation more than 20 hours later, one
AAPL daily technical call again returned outer HTTP 200 with an application
failure mentioning a rate limit and the screener endpoint. The outer response
had no Retry-After; no indicator values were returned. Further identical reads
are stopped, and the technical implementation still awaits usable response
evidence or a provider explanation.
