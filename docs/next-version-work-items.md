# v0.33.0 ordered work inventory

Current state: 2026-09-26. Direction: [roadmap](next-version-roadmap.md).
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
| 10 | Add offline command specifications | Implemented with clap-derived syntax and explicit partial validation. A 2026-09-26 executable audit found 156 of 169 leaves annotated; the 13 remaining paths and priorities are below. Further annotations do not change runtime contracts. Schema export and request validation remain separate. |
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
- Next order: annotate `data shapes`, then chart setup observations and analysis
  support as ordered below. Reassess lower-priority gaps before freezing scope;
  100% semantic annotation is not a newly imposed release condition. Then qualify
  the dependency graph/current candidate, finish docs/package checks and prepare
  version/notes separately. Reuse evidence whose inputs remain unchanged.

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


## Remaining command-specification order

A 2026-09-26 audit queried every executable path through the cached development
binary with an invalid CDP port. All 169 leaf queries returned valid JSON without
stderr; 156 have semantic annotations and the following 13 do not. The root
contains 195 paths including 26 groups. This proves offline lookup, not runtime
behavior. Validation coverage remains partial for annotated paths too.

Chart analysis remains first. Prior downstream call sites and recorded runs are
usage evidence, although no aggregate frequency ranking exists. The ordering
below uses current source inspection and workflow relevance, not a claim that
all remaining commands are actively used downstream.

| Priority | Exact remaining paths | Acceptance focus |
| --- | --- | --- |
| 1 | `data shapes` | Pine shape/character observations: bar window, plotted values, filtering and lossy summaries. Distinguish from hand-drawn objects. |
| 2 | `status`, `ui-state` | Connection versus chart readiness; observed UI state versus permission or ability to execute later actions. |
| 3 | `watch compare`, `events compare`, `fundamentals` | Bounded repeated observations, event coverage and source-dependent financial interpretation. |
| 4 | `screener open`, `screener close` | Explicit UI/target effects and full-page versus dialog behavior; do not infer opening from a read request. |
| 5 | `diagnose quote-data` | Advance when diagnosing a concrete quote-data problem; state network/subscription effects and evidence limits. |
| 6 | `scanner hotlist`, `data depth`, `discover`, `spec` | Remaining discovery/self-description; depth is a heuristic DOM read, not authenticated exchange depth proof. Review value before treating annotation completion as a release gate. |

This inventory authorizes metadata for existing commands, not changed execution
guarantees, live operations or new commands. Schema export and offline request
validation remain separate follow-ups.

## Candidate qualification still required

- Latest observed CI success is for released `54dabec`, not the current candidate:
  [CI run](https://github.com/HumInvLitterae/tradingview-cli/actions/runs/35682348029).
  Current CI defines tests on macOS/Windows/Linux, Linux Secret Service integration,
  workspace Clippy, script/package checks and four JavaScript contract jobs.
  Obtain candidate-SHA results after the owner pushes; do not duplicate broad
  suites locally by default or trigger a workflow without authorization.
- Dependency constraints changed from v0.32.0: rmcp 3.4.0 to 3.4.1 and thiserror
  2.0.20 to 2.0.21. Lock changes also affect encoding_rs, hyper-util, platform
  verification and zerocopy. Review relevant regression evidence against this
  graph; the current audit is not a dependency security or platform qualification.
- Preserve native limits: alert history succeeded with an explicit 90-second
  budget and empty history; default-deadline/nonempty native behavior remains
  unqualified. Linux graphical OAuth and remote skill installation remain
  unverified. Decide which limitations can be documented without expanding claims;
  further native checks require concrete targets and any uncovered authority.
- Finish current-candidate distribution checks when executable/package inputs
  are final. Guidance fixture staging is not release-binary validation. Version
  remains 0.32.0 until qualification; version/notes preparation stays separate
  from features. Publication is not authorized by this inventory.

## Behavioral findings kept separate from annotations

These findings were disclosed during metadata work. They are not fixed by
passing spec tests and are not silently added to v0.33.0 implementation scope.
Any correction needs concrete before/after contracts and consumer impact.

- Legacy watchlist mutations can fall back to DOM after selected uncertain
  writes; API readback can use a different active list when the original vanishes.
- Legacy alert listing can return embedded errors or empty normalized data under
  outer success. DOM price-alert creation does not set the requested condition
  and reports a button click rather than persistence. Numeric IDs cross a
  JavaScript Number conversion without a safe-integer bound.
- Indicator-alert creation does not verify supplied source against saved version
  and uses the first name-matched chart study. Preview and readback do not prove
  complete input/version identity.


Chart-analysis follow-up finding: current `data equity` does not distinguish its
Buy & Hold, equityData and strategy-bar paths in a reliable series discriminator.
Its bars path also maps zero drawdown to null. Metadata and standalone guidance
now disclose these limits. Treat correction as a separate contract proposal with
before/after payloads and consumer impact, not an implicit behavior change inside
specification work.

Packet follow-up finding: snapshot/compare hints mark chart_quote and screenshot
non_mutating despite their possible chart/file effects. Metadata and standalone
guidance now direct agents to the hinted command's own spec. A later correction
must agree consumer-visible before/after hint semantics; no change was made to
the existing packet contract in the metadata work.
