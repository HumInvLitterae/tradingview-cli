# Prepare the v0.31.4 maintenance patch

## Outcome and scope

Status: local release preparation and applicable validation complete; not
published. v0.31.4 comes before the approved MCP client work.

Prepare v0.31.4 from published v0.31.3 plus the ten maintenance commits through
`7a7b883e0e95a62653cf0f8eb877ade57ca8a568`. The
[inventory](../next-version-work-items.md#maintenance-classification) classifies
nine dependency updates and the runtime guidance/package reorganization.
There is no production Rust source change in this patch. MCP implementation
and dependencies remain in the separate [MCP plan](tradingview-cli-official-mcp-client.md).

The release preparation commit groups workspace versions, CHANGELOG, curated
release notes, the README archive example, this work record and current release
status. Prior-release closure and future planning are separate commits.
Local commits are authorized. Push, tags, workflow dispatch, publication and
additional agents are not authorized; both existing stashes remain protected.

## Compatibility and version alignment

- Root workspace version and all seven local lock packages are 0.31.4.
- All 15 direct third-party manifest versions match their resolved lock versions,
  including clap 4.6.7 and reqwest 0.13.5. Workspace inheritance and local path
  dependencies resolve correctly.
- Every third-party lock version, checksum and dependency edge matches the
  maintenance baseline at 7a7b883. No new dependency was added.
- The archive contains 48 files and six skills under each root, with matched
  guides and transitive references. Users naming the retired market-data skills
  must switch to `market-data`; contributor-only skills remain excluded.
- CLI/JSON/source behavior has no intentional production source change.
  Dependency refreshes were covered by the release baseline below.

## Validation evidence (2026-09-20)

These results apply to the same executable sources, Cargo inputs and runtime
resources retained by this preparation. Rearranging documentation and commit
boundaries does not invalidate them.

| Check | Result |
| --- | --- |
| Cargo consistency | Full locked/offline metadata: 259 packages/nodes; all 15 direct dependencies and seven workspace packages agree. |
| Per-target dependency resolution | Linux x86_64: 209 nodes; macOS x86_64: 213; macOS arm64: 210; Windows x86_64: 210. |
| Formatting and Clippy | Formatting and strict workspace all-target/all-feature Clippy passed. |
| Rust workspace tests/doctests | 923 passed, zero failed, 27 ignored by the ordinary suite; includes ten provenance tests and ten version-contract tests. |
| Pinned JavaScript contracts | All four Node 24.18.0 gates passed, including both three-point drawing checks. |
| Local release build | `cargo build --release --locked` passed on macOS arm64. |
| Real package staging | 48 files, six skills per root, reference/guide parity and source/staged binary byte identity passed. |
| Hygiene and scripts | Public-hygiene self-test/scan, package-validator self-tests, affected document links/JSON, shell syntax, workflow YAML and diff checks passed. |

No TradingView live operation or additional reviewer ran. Native Windows, Linux
and Intel macOS builds/runtime remain unverified locally. Target metadata is
not native execution evidence; the native release jobs remain the platform gate.

## Finalization and remaining work

Keep this release-preparation commit last. Do not append another tracked commit
solely to record its own commit hash or local binary checksum. Final hash-bearing
readback belongs in the ignored local handoff and the user-facing report.

A version-only release commit does not require repeating valid functional tests.
When supplying a local binary from the final commit, build it once to align the
version/provenance, check the short/verbose display and staged package, and retain
that readback locally. Repeat broader checks only for changed executable inputs,
failures or a concrete unresolved concern.

Next is owner-controlled publication. After publication is verified, close and
archive this release record, then proceed with the already-approved MCP scope.
Do not reinterpret local preparation as remote publication.
