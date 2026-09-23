# v0.33.0 ordered work inventory

Current state: 2026-09-23. Direction: [roadmap](next-version-roadmap.md).
Contracts, approvals and dated evidence:
[one active work record](plans/tradingview-cli-mcp-operational-improvements.md).

| Order | Work | Current state and remaining acceptance |
| --- | --- | --- |
| 1 | Close v0.32.0 and establish baseline | Complete: publication/workflow verified, old record archived. Historical platform evidence is not current-candidate CI proof. |
| 2 | Settle new read contracts and verification scope | Approved. History fields are qualified; successful technical fields are still missing. The separately approved timeout option uses `--timeout`, with no `--timeout-secs` alias. |
| 3 | Improve login/upgrade operation | Implemented. Focused tests cover terminal-only guidance, captured output and credential reuse. A fresh interactive native login was not repeated for the wording change. |
| 4 | Implement alert history | Implemented and fixture-verified. Nonempty provider structure was observed; the public service succeeded with explicit 90 seconds and empty history. Default-30-second native success and nonempty native normalization remain unqualified; coverage is always unconfirmed. |
| 5 | Implement official technical snapshot | Blocked on provider response evidence. The daily request and a same-symbol basic-column control both returned HTTP 200 with application errors mentioning a rate limit and screener endpoint; neither returned values. Keep planned scope. Resume on provider availability evidence or a new diagnostic hypothesis, then implement and qualify actual fields. |
| 6 | Measure and optimize transport | Implemented and measured. A single-page mutation/readback flow uses one initialization/catalog instead of two. Lazy pagination, admission, deadlines and separate readback failures are fixture-verified. No live latency improvement is claimed. |
| 7 | Stabilize affected fixtures | Scoped serial regression passed; no new timing defect was established. Current-candidate default-concurrency and cross-platform CI remain pending. Correct concrete failures if observed; do not weaken production deadlines or assertions. |
| 8 | Integrate documentation and standalone skills | Complete for implemented behavior: help, usage, source taxonomy and standalone MCP references updated and checked. Add technical snapshot guidance only after its implementation. |
| 9 | Qualify and prepare release | Not ready. Resolve technical response/implementation, review remaining native limits, obtain current-candidate platform/CI evidence, then prepare version and notes separately. Publishing remains separately authorized. |

## Completed operating prerequisite

Heavy local pre-push checks are opt-in. Local Cargo uses one build job and one
test thread, with existing artifacts and focused checks. The current workspace
version remains 0.32.0; no release bump or new production dependency was added.

## Next actions and boundaries

- Technical data: retain the approved AAPL D/W/M/2h scope. A public-safe provider
  inquiry draft exists in the active record but has not been sent. Do not retry
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
- Release scope: technical snapshots remain a required planned feature. If their
  provider cannot be qualified, present a concrete scope/schedule decision before
  changing the release content. Do not silently drop them or start release prep.

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
