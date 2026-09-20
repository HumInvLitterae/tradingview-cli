# Prepare the v0.31.4 maintenance patch

## Outcome and scope

Status: **complete / released**. Publication verified on 2026-09-20.
The prior release prerequisite for the approved MCP client work is satisfied.

Prepare v0.31.4 from published v0.31.3 plus the ten maintenance commits through
`7a7b883e0e95a62653cf0f8eb877ade57ca8a568`. The
[maintenance classification](#maintenance-classification) classifies
nine dependency updates and the runtime guidance/package reorganization.
There is no production Rust source change in this patch. MCP implementation
and dependencies remain in the separate [MCP plan](../tradingview-cli-official-mcp-client.md).

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

## Publication closeout

The owner reported successful publication. Read-only verification confirmed:

- Local and remote `v0.31.4` tags resolve to
  `48e500b208e67e6d13825efb7de6615a490ae4eb`.
- [GitHub Release](https://github.com/HumInvLitterae/tradingview-cli/releases/tag/v0.31.4)
  is neither draft nor prerelease, published at `2026-09-20T13:54:58Z`, with
  four platform archives and SHA256SUMS.
- [Release run 35514375266](https://github.com/HumInvLitterae/tradingview-cli/actions/runs/35514375266)
  completed successfully at that commit: four native build jobs, four pinned
  JavaScript gates and the publication job. Released artifacts were not
  downloaded or executed again during this documentation closeout.
- Starting main/recorded origin matched the released tag with a clean worktree.

This closes the release record. Historical local checks above remain evidence
for their declared inputs; no rebuild or functional retest was needed to record
publication. Resume the existing MCP plan with its previous approval intact.


## Maintenance classification

`962010d` changes contributor/runtime guidance and release resources: six skills
per root, consolidation into `market-data`, contributor-only `continuity` and
`release-prep`, retired generic wrappers, and transitive package reference
validation in staging/CI. This is a user-facing package change. Release-candidate fixture
staging was the evidence source for the 48-file/6-skill contract; historical
v0.31.3's 46-file/8-skill record is not reused as current evidence.

For `v0.31.3..7a7b883`, the other nine commits change Cargo inputs only. The table below is generated
from each commit's actual lockfile predecessor, including additions/removals.
Compatible selection does not mean every transitive change is a patch version,
nor establish absence of dependency behavior changes.

| Commit | Area | Exact package versions (before -> after) |
| --- | --- | --- |
| `9f92906` | Build, HTTP/2, TLS verification, logging, macro support | `cc` 1.4.3 -> 1.4.4; `crc32fast` 1.5.0 -> 1.5.1; `h2` 0.4.18 -> 0.4.19; `log` 0.4.33 -> 0.4.34; `rustls-webpki` 0.103.14 -> 0.103.15; `syn` 2.0.119, 3.0.3 -> 2.0.119, 3.0.4 |
| `750af2a` | Crypto/CPU support, compression, HTTP, collections | `chacha20` 0.10.1 -> 0.10.2; `combine` 4.6.7 -> 4.6.8; `cpufeatures` 0.3.0 -> 0.3.1; `flate2` 1.1.9 -> 1.1.10; `hyper` 1.11.0 -> 1.11.1; `indexmap` 2.14.0 -> 2.14.1; `miniz_oxide` 0.8.9 -> 0.8.9, 0.9.1; `zlib-rs` absent -> 0.6.7 |
| `593dc7c` | Native TLS crypto, event I/O, collections | `aws-lc-rs` 1.18.0 -> 1.18.1; `aws-lc-sys` 0.44.0 -> 0.45.0; `mio` 1.2.2 -> 1.2.3; `smallvec` 1.15.2 -> 1.16.0 |
| `43fd6ca` | Native build, URL/IP, TLS adapter, WASM, macros | `cc` 1.4.4 -> 1.4.5; `find-msvc-tools` 0.1.11 -> 0.1.12; `indexmap` 2.14.1 -> 2.14.2; `ipnet` 2.12.1 -> 2.12.2; `js-sys` 0.3.104 -> 0.3.105; `syn` 2.0.119, 3.0.4 -> 2.0.119, 3.0.5; `tinyvec` 1.12.0 -> 1.13.2; `tokio-rustls` 0.26.4 -> 0.26.5; `wasm-bindgen` 0.2.127 -> 0.2.128; `wasm-bindgen-futures` 0.4.77 -> 0.4.78; `wasm-bindgen-macro` 0.2.127 -> 0.2.128; `wasm-bindgen-macro-support` 0.2.127 -> 0.2.128; `wasm-bindgen-shared` 0.2.127 -> 0.2.128; `web-sys` 0.3.104 -> 0.3.105 |
| `1c8c071` | Encoding/CPU dispatch and TLS | `core_detect` absent -> 1.0.0; `encoding_rs` 0.8.35 -> 0.8.40; `multiversion` absent -> 0.8.0; `multiversion-macros` absent -> 0.8.0; `multiversion_no_op` absent -> 1.0.0; `rustls` 0.23.43 -> 0.23.44; `scopeguard` absent -> 1.2.0; `target-features` absent -> 0.1.6 |
| `448a1ab` | Direct HTTP client constraint plus encoding/base64/CPU graph | `base64` 0.22.1 -> 0.22.1, 0.23.1; `encoding_rs` 0.8.40 -> 0.8.41; `hybrid-array` 0.4.14 -> 0.4.15; `multiversion` 0.8.0 -> 0.9.0; `multiversion-macros` 0.8.0 -> 0.9.0; `reqwest` 0.13.4 -> 0.13.5; `target-features` 0.1.6 -> removed |
| `c98343e` | TLS/QUIC, compression, collections and native build | `bitflags` 2.13.1 -> 2.13.2; `cc` 1.4.5 -> 1.4.6; `crc32fast` 1.5.1 -> 1.5.2; `lru-slab` 0.1.2 -> 0.1.3; `quinn` 0.11.11 -> 0.11.12; `quinn-proto` 0.11.17 -> 0.11.18; `rustls` 0.23.44 -> 0.23.45; `smallvec` 1.16.0 -> 1.16.1; `tinyvec` 1.13.2 -> 1.13.3; `tinyvec_macros` 0.1.1 -> removed |
| `cbf3060` | Direct CLI parser constraint and parser/derive stack | `clap` 4.6.6 -> 4.6.7; `clap_builder` 4.6.6 -> 4.6.7; `clap_derive` 4.6.4 -> 4.6.7; `clap_lex` 1.1.0 -> 1.1.1 |
| `7a7b883` | Native build/OS, random, compression and Unicode derive stack | `cc` 1.4.6 -> 1.4.7; `cfg-if` 1.0.4 -> 1.0.5; `find-msvc-tools` 0.1.12 -> 0.1.13; `rand` 0.10.2 -> 0.10.3; `rustix` 1.1.4 -> 1.1.5; `syn` 2.0.119, 3.0.5 -> 2.0.119, 3.0.6; `synstructure` 0.13.2 -> 0.14.0; `unicode-ident` 1.0.24 -> 1.0.26; `yoke-derive` 0.8.2 -> 0.8.3; `zerofrom-derive` 0.1.7 -> 0.1.8; `zlib-rs` 0.6.7 -> 0.6.8 |
