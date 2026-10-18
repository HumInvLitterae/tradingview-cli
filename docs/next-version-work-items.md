# Development work inventory

This inventory tracks unresolved product work. Command behavior and supported
interfaces are documented in the [CLI specification](cli-spec.md),
[Rust API reference](rust-api.md) and [internal API reference](internal-tradingview-apis.md).

| Candidate | Current limitation | Required investigation |
| --- | --- | --- |
| Desktop price-alert response-ID attribution | Confirmation selects a newly listed alert by content; concurrent matching writes can be ambiguous. | Establish the authoritative response ID and saved-row identity before changing the success contract. |
| Broader indicator-input qualification | A non-default integer case does not establish every Pine input type's provider representation. | Qualify additional types when a concrete supported workflow requires them; preserve typed comparisons. |

These candidates do not change existing behavior or authorize account mutations.
Release procedures are maintained in [release packaging](release-packaging.md).
