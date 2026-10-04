# Current work

| Purpose | Record |
| --- | --- |
| Direction after v0.35.0 | [Roadmap](../next-version-roadmap.md) |
| Approved slice, acceptance, and remaining candidates | [Inventory](../next-version-work-items.md) |
| Current source-verification acceptance | [Indicator source verification](tradingview-cli-indicator-source-verification.md) |
| Released baseline and acceptance | [v0.35.0 closeout](archives/tradingview-cli-chart-analysis-contracts.md) |
| CDP stability triggers | [Strategy](../notes/cdp-stability-and-autonomous-operation-strategy.md) |
| Completed work | [Archive catalog](archives/README.md) |

v0.35.0 is published. Release metadata and workflow success were rechecked on
2026-10-05. Before this documentation closeout, the only post-tag commit was
9cba16e, a test-fixture repair with successful CI. On 2026-10-05, the owner approved
bounded weekly/monthly technicals qualification and the diagnostic spec contract.
The reads succeeded, and the diagnostic spec is implemented with focused local
acceptance. It is unreleased; upstream CI has not run for this slice. The
inventory records the contract and evidence. No release version or broader
feature scope beyond these slices is selected. The approved Pine source-verification
change is implemented with focused local acceptance and bounded live dry-run
qualification: exact source and CRLF passed; a textual mismatch was rejected.
Normal alert creation was not exercised. The subsequent Pine Editor opening
repair also passed focused tests and live verification from a closed panel;
its evidence is in the inventory.

Completed records preserve evidence for their recorded inputs. They are not
current runtime acceptance or authorization for new live operations.
