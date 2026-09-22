# v0.33.0 ordered work inventory

Direction: [roadmap](next-version-roadmap.md). Decisions and acceptance:
[one active work record](plans/tradingview-cli-mcp-operational-improvements.md).

| Order | Work | Completion condition |
| --- | --- | --- |
| 1 | Close v0.32.0 and establish baseline | Done: publication/workflow verified; prior record archived with historical evidence preserved and downstream report distinguished from local checks. |
| 2 | Settle new read contracts and live verification scope | Contract proposals and scoped reads approved; actual response shapes and missing-value rules still need verification before finalizing normalization. Both are planned features. |
| 3 | Improve login/upgrade operation | Existing successful stdout/error JSON preserved; useful pre-interaction guidance for explicit login, correct credential reuse and no prompts in ordinary commands. |
| 4 | Implement alert history | Bounded read, source/evidence separation, no sensitive free text, safe empty/limited/error results, fixtures and scoped real acceptance. |
| 5 | Implement official technical snapshot | Single symbol/timeframe, provider values/ratings without local recomputation, missing/unknown conditions retained; fixtures and scoped real acceptance. |
| 6 | Measure and conditionally optimize transport | Document before/after initialization/catalog counts and timings; one command owns one connection where useful; no mutation replay or deadline/contract changes. No-go with evidence is acceptable. |
| 7 | Stabilize affected fixtures | Reproduce and correct time/readiness dependencies; normal CI parallel execution passes without production timeout changes. Work can accompany stages 3–6. |
| 8 | Integrate documentation and standalone skills | User guides, command/source mapping and account-management/market-data references reflect implemented behavior; individual skill and archive checks pass. |
| 9 | Qualify and prepare release | Applicable Rust/platform checks, scoped runtime acceptance and honest limits recorded; version/notes in a separate final preparation commit. Publication separately authorized. |

Current prerequisite: disable expensive local pre-push checks by default and
use one Cargo build job/test thread with focused local validation. The owner
approved both public-contract proposals and the scoped technical/history reads.
Next: qualify actual response shapes, then implement the agreed slices. Reuse
same-scope authority; material new effects or contract differences need review.
No new read implementation, version bump, account mutation or dependency
addition has occurred in this planning/tooling stage.
