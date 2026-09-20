# Next-version ordered work inventory

Direction: [roadmap](next-version-roadmap.md). Detailed MCP decisions:
[one MCP ExecPlan](plans/tradingview-cli-official-mcp-client.md).

## Order and current state

| Order | Work | State / acceptance |
| --- | --- | --- |
| 1 | Reconcile released v0.31.3 and current checkout | Publication verified; prior record closed with historical evidence preserved. |
| 2 | Classify maintenance candidate | Ten commits classified below; no Rust source diff. Current lockfile inspected, not runtime-qualified. |
| 3 | Review MCP proposal and concrete contracts | Direction accepted on 2026-09-20; dependency and account/store effects now concretized in the same plan. |
| 4 | Prepare v0.31.4 maintenance release | [Released and archived](plans/archives/tradingview-cli-v0.31.4-release-readiness.md); publication verified on 2026-09-20. |
| 5 | Prove and build bounded official MCP client | Concrete dependency/local-proof and bounded live scope approved, v0.31.4 prerequisite satisfied; follow the existing MCP plan, including early OAuth/restart/refresh proof and downstream acceptance. |
| 6 | Qualify v0.32.0 candidate | Only after the complete initial user journey and accepted downstream artifact; no expansion to unrelated MCP tools. |

The owner confirmed that v0.31.4 comes first; its publication is now verified. The MCP approval remains valid
for its specified targets and effects; sequencing does not require reapproval.
The release-readiness record is included with the patch preparation.

## Historical pre-release baseline (2026-09-20)

- `main` and locally recorded `origin/main`: `7a7b883e0e95a62653cf0f8eb877ade57ca8a568`;
  starting worktree clean; workspace version `0.31.3`.
- Local and remote `v0.31.3` tag: `fb23eb852efb27de15e310fe9f27024ee59b8a40`.
- [GitHub Release](https://github.com/HumInvLitterae/tradingview-cli/releases/tag/v0.31.3)
  published `2026-08-21T02:17:22Z`, neither draft nor prerelease, and reported as
  latest by the release API. Four platform archives and SHA256SUMS are listed;
  their contents and binary behavior were not revalidated here.
- [CI for the inspected HEAD](https://github.com/HumInvLitterae/tradingview-cli/actions/runs/35507975166)
  is completed/success. This is remote CI evidence for that commit, not MCP or
  release-candidate acceptance.
- Both named stashes (`fable-plan`,
  `recovered-indicator-search-prototype-2026-07-12`) remain protected.

## Maintenance classification

The released candidate's nine dependency changes and runtime-guidance/package
reorganization are preserved in the
[archived release record](plans/archives/tradingview-cli-v0.31.4-release-readiness.md#maintenance-classification).

## Final graph and release checks

At the inspected HEAD the lockfile has 259 package entries, including seven
workspace packages. Every dependency reference resolves to one lock entry.
Direct constraints changed only `clap` 4.6.6 -> 4.6.7 and `reqwest` 0.13.4 ->
0.13.5; no dependency additions were made by this planning task.

Relevant lock paths include CLI -> clap; CDP -> reqwest -> rustls -> aws-lc-rs
-> aws-lc-sys; CDP -> reqwest -> encoding_rs; CLI -> image -> png -> flate2 ->
zlib-rs; and CDP -> reqwest -> hyper-util -> windows-registry. These are lock
edges, not proof that all target-specific/optional features execute on every
platform. Keep both base64 and miniz_oxide versions where selected; do not
flatten the graph into a single version per name.

The initial planning-only full metadata attempt was blocked while unpacking
windows-registry into the global cache. Subsequent authorized candidate checks
resolved it. The release record contains the complete candidate validation;
static graph inspection alone is not native platform proof.

Release preparation owns version alignment, validation evidence and packaging.
Keep post-commit hash readback in the ignored local handoff and final report;
there is no need for an extra tracked evidence commit that advances the release
candidate. Commit/tag/push/workflow/publication authority remains explicit.

## Accepted direction and sequence

The owner accepted the recommended direction in the follow-up on 2026-09-20.
The MCP record now lists five exact direct dependencies with target features,
SDK replay/body-limit/redirect constraints, native-store behavior and a concrete
registration/credential/read budget. Two unauthenticated public discovery GETs
confirmed advertised issuer/endpoints/grants/scopes; no OAuth or tool call ran.
Public source archives were inspected only in a disposable directory. The owner
subsequently approved the concrete MCP scope while explicitly placing v0.31.4
first; the MCP plan carries that approval forward.

## Planning validation

Passed on 2026-09-20: public-hygiene self-test and tracked scan, separate
changed/new-document hygiene scan, 34 local Markdown references and eight JSON
examples, historical archived-body preservation, guide parity, shell syntax,
seven package-validator tests, placeholder staging (48 files / six skills per
root), workspace-only locked/offline metadata, lock-edge checks, and diff hygiene.
The package contains a placeholder, not a newly built/qualified executable.
Rust/JS suites and live/platform tests were not rerun for this documentation-only
change. Current HEAD CI success is separately linked above; it does not qualify
future dependencies, a bumped release, or the proposed MCP implementation.
Those initial planning checks did not include MCP/OAuth, live TradingView,
Desktop mutation, downstream writes, commits or publication. MCP approval was
subsequently granted for the later stage; release preparation is now recorded
separately above.
