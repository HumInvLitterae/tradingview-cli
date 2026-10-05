# Current work

| Purpose | Record |
| --- | --- |
| Direction after v0.36.0 | [Roadmap](../next-version-roadmap.md) |
| Current priority and retained acceptance evidence | [Inventory](../next-version-work-items.md) |
| Released source-verification implementation and qualification | [Indicator source verification](tradingview-cli-indicator-source-verification.md) |
| Approved opt-in verification | [Opt-in indicator verification](tradingview-cli-indicator-study-verification.md) |
| Implemented saved-script creation | [Pine creation acceptance](archives/tradingview-cli-pine-script-creation.md) |
| Current release behavior and limits | [v0.36.0 release notes](../releases/v0.36.0.md) |
| Earlier released baseline | [v0.35.0 closeout](archives/tradingview-cli-chart-analysis-contracts.md) |
| CDP stability triggers | [Strategy](../notes/cdp-stability-and-autonomous-operation-strategy.md) |
| Completed work | [Archive catalog](archives/README.md) |

v0.36.0 is published. Release metadata and successful CI and Release runs at
5c72c081b8e0539454a8be0e001702735656585e were checked on 2026-10-05.
The inventory's current status supersedes pending-publication statements in
historical preparation records. Their fixture and live qualification limits
remain applicable.

The owner approved the first outcome correction and study/input identity
investigation. The correction is implemented locally; the inventory records
acceptance and the remaining identity evidence gaps. A bounded metadata read
and synthetic production-expression probe reproduced positional input corruption;
the owner approved the input-ID correction, now implemented locally with
acceptance in the inventory. Broader identity behavior, new live operations, and
publication remain separate decisions. Version selection follows the completed
release scope.

The authorized saved-study trial passed for one existing no-input indicator.
Saved ID/version and compiled condition matched; the new study was removed and
the original chart and saved source were preserved. The inventory distinguishes
this metadata qualification from full alert creation and input-bearing cases.

Further read-only research retrieved compiled metadata for an exact saved
revision without adding a chart study. The owner approved opt-in
verification preserving no-flag CLI/JSON and the existing Rust operation request
and function. It requires no chart insertion for no-input scripts. Implementation
and local acceptance passed, including a verified no-input preview without a
study. This preview does not live-qualify normal creation or complete
post-create identity.

Completed records preserve evidence for their recorded inputs. They are not
current runtime acceptance or authorization for new live operations.

Input-bearing live acceptance also passed automatic selection, duplicate
rejection, and explicit selection at native Length values 30 and 50. Temporary
instances were removed and the original chart restored. Named Pine creation
also passed bounded live save/readback and editor/chart preservation. It is a
required pre-release fix in the same next-version scope. Final review is complete,
and the complete v0.37.0 candidate passed local macOS arm64 archive qualification.
The [inventory](../next-version-work-items.md#complete-v0370-candidate-2026-10-06)
tracks archive acceptance, candidate CI, and publication.
