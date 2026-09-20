# Next-version ordered work inventory

Direction: [roadmap](next-version-roadmap.md). Detailed MCP decisions:
[one MCP ExecPlan](plans/tradingview-cli-official-mcp-client.md).

## Order and current state

| Order | Work | State / acceptance |
| --- | --- | --- |
| 1 | Reconcile released v0.31.3 and current checkout | Publication verified; prior record closed with historical evidence preserved. |
| 2 | Classify maintenance candidate | Ten commits classified below; no Rust source diff. Current lockfile inspected, not runtime-qualified. |
| 3 | Review MCP proposal and concrete contracts | Direction accepted on 2026-09-20; dependency and account/store effects now concretized in the same plan. |
| 4 | Prepare v0.31.4 maintenance release | [Local preparation complete](plans/tradingview-cli-v0.31.4-release-readiness.md); publication remains owner-controlled. |
| 5 | Prove and build bounded official MCP client | Concrete dependency/local-proof and bounded live scope approved, queued after v0.31.4 closeout; follow the existing MCP plan, including early OAuth/restart/refresh proof and downstream acceptance. |
| 6 | Qualify v0.32.0 candidate | Only after the complete initial user journey and accepted downstream artifact; no expansion to unrelated MCP tools. |

The owner confirmed that v0.31.4 comes first. Prepare and close out the patch
before starting MCP dependencies or live proof. The MCP approval remains valid
for its specified targets and effects; sequencing does not require reapproval.
The release-readiness record is included with the patch preparation.

## Verified baseline (2026-09-20)

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

`962010d` changes contributor/runtime guidance and release resources: six skills
per root, consolidation into `market-data`, contributor-only `continuity` and
`release-prep`, retired generic wrappers, and transitive package reference
validation in staging/CI. This is a user-facing package change. Current fixture
staging is the evidence source for the 48-file/6-skill contract; historical
v0.31.3's 46-file/8-skill record is not reused as current evidence.

The other nine commits change Cargo inputs only. The table below is generated
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
