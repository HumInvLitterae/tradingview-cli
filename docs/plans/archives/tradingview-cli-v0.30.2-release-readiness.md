# Prepare the v0.30.2 dependency and screenshot-decoding patch

This ExecPlan is a living document maintained according to `.agents/PLANS.md`.
Keep Progress, Surprises & Discoveries, Decision Log, and Outcomes current.

## Purpose / Big Picture

Prepare `v0.30.2` from released `v0.30.1` plus two committed changes:
compatible dependency refresh `50d3096` and screenshot base64 decoding
improvement `f157531`. Users retain the same commands, JSON envelopes, error
classification, screenshot files, and package layout. Screenshot payloads
returned by CDP are decoded by `base64-simd` instead of the scalar
`base64` engine.

The release updates workspace versions, changelog, release notes, README,
locked build, and the existing 46-file package. It stops before tag, push,
workflow execution, or GitHub Release publication.

## Progress

- [x] (2026-07-27) Confirmed `v0.30.1..HEAD` contains exactly dependency
  refresh `50d3096` and screenshot decoding change `f157531`.
- [x] (2026-07-27) Benchmarked the scalar decoder, `base64` 0.23 SIMD engine,
  and `base64-simd` 0.8 with equal decoded bytes on representative payloads.
- [x] (2026-07-27) Replaced the sole direct `base64` call site with
  `base64-simd`, preserving the existing screenshot decode error boundary.
- [x] (2026-07-27) Added focused success and malformed-input regression tests;
  focused tests, strict workspace Clippy, full workspace tests, metadata,
  public hygiene, package syntax, guide parity, and diff hygiene are green.
- [x] (2026-07-27) Prepared `0.30.2` version metadata, changelog, release notes,
  README, and durable planning state.
- [x] (2026-07-27) Ran four pinned JavaScript gates and the complete Rust
  release baseline successfully.
- [x] (2026-07-27) Built `--release --locked`, staged and inspected the 46-file
  package, and verified both source and staged binaries report `tv 0.30.2`.
- [x] (2026-07-27) Completed focused release-readiness review with no code,
  dependency, artifact, or packaging finding; corrected the stale local ledger
  entry and stopped before publication.

## Surprises & Discoveries

- Observation: upgrading to `base64` 0.23 alone did not accelerate the existing
  `general_purpose::STANDARD` call site.
  Evidence: the crate includes SIMD engines by default, but callers must select
  a SIMD engine explicitly.

- Observation: the dedicated `base64-simd` API is both faster in the local
  arm64 microbenchmark and simpler at the only direct call site.
  Evidence: the implementation becomes one `STANDARD.decode_to_vec` call,
  without project-owned architecture conditionals or cached engine setup.

- Observation: a full Windows cross-check from macOS cannot compile the
  existing `aws-lc-sys` dependency without Windows SDK headers.
  Evidence: the check stopped on missing `windows.h`; an isolated
  `base64-simd` check succeeded for Windows MSVC and Linux musl targets.

## Decision Log

- Decision: release the two commits as a patch without opening the next feature
  roadmap.
  Rationale: the behavior change is internal, additive in performance intent,
  and covered by the existing screenshot contract. Mixing a new product slice
  would expand an otherwise narrow candidate.
  Date/Author: 2026-07-27 / Codex

- Decision: use `base64-simd` rather than wrapping `base64` 0.23's SIMD engine.
  Rationale: `base64` is used directly only for screenshot decoding.
  `base64-simd` owns runtime feature detection and scalar fallback, removes
  project-owned conditional code, and was faster in the local microbenchmark.
  Date/Author: 2026-07-27 / Codex

- Decision: describe the release as faster screenshot payload decoding, not a
  guaranteed screenshot command speedup.
  Rationale: CDP transfer, PNG generation, rendering, file I/O, and local image
  processing remain outside the measured base64-only benchmark.
  Date/Author: 2026-07-27 / Codex

## Outcomes & Retrospective

Implementation, release artifacts, four pinned JavaScript gates, the complete
Rust workspace baseline, strict Clippy, metadata, public hygiene, locked build,
and package inspection are green. Both source and staged binaries report
`tv 0.30.2`; the staged tree contains 46 files and exactly eight runtime skills
under each skill root, with no development-only material. Focused
release-readiness review found no substantive release finding. Its sole
correction was a stale `CONTINUITY.md` `Now` entry, which is synchronized in
the release-preparation commit. The candidate is locally release-ready. No
Desktop operation, live probe, stash mutation, tag, push, workflow execution,
or GitHub Release publication has occurred.

## Context and Orientation

`v0.30.1` is the current public tag. Commit `50d3096` updates compatible
resolved versions including `cc`, `clap`, `clap_derive`, `libc`,
`rustls-pki-types`, `syn`, and `tokio-util`. Commit `f157531` replaces the
workspace's direct `base64` dependency with `base64-simd` plus its small
`outref` and `vsimd` dependencies.

The only production call site is
`crates/cdp/src/client.rs::screenshot_bytes_from_response`. TradingView CDP
returns `Page.captureScreenshot` data as standard padded base64. The function
decodes those bytes and maps malformed input to
`ErrorKind::InternalApiUnavailable` with the existing
`Could not decode screenshot data:` message prefix. The patch changes no CDP
method, timeout, screenshot region, output file, JSON payload, retry, or
fallback behavior.

Seven workspace crates inherit `[workspace.package].version`. Release prep may
change only that root version and the seven local `tradingview-*` lock entries;
the reviewed third-party selections must remain fixed.

## Plan of Work

Freeze `v0.30.1..HEAD` as the two reviewed commits. Update workspace and local
package versions to `0.30.2`, cut a dated changelog section, add prose-first
release notes, and update the README archive example. Synchronize
`docs/v0.30-roadmap.md`, `docs/v0.30-work-items.md`, `docs/plans/README.md`, and
`CONTINUITY.md`.

Run the four pinned JavaScript contracts, strict Rust baseline, metadata,
locked release build, package staging, source and staged version readback,
public hygiene, workflow parsing, package and guide parity, stale-version
checks, and diff hygiene. Record aggregate evidence here. Do not run ignored
live tests, mutate Desktop, touch either stash, tag, push, trigger workflows,
or publish a Release.

## Concrete Steps

Run from the repository root:

    git diff --name-status v0.30.1..HEAD
    git diff v0.30.1..HEAD -- Cargo.toml Cargo.lock crates .github mise.toml scripts
    mise run check:study-values-js
    mise run check:pine-open-js
    mise run check:indicator-insertion-js
    mise run check:three-point-drawing-js
    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1
    cargo build --release --locked
    target/release/tv --version
    scripts/stage-release-package-files.sh target/release-package-smoke target/release/tv
    target/release-package-smoke/tv --version
    python3 scripts/check-public-hygiene.py --self-test
    python3 scripts/check-public-hygiene.py
    bash -n scripts/stage-release-package-files.sh
    cmp -s AGENTS.md CLAUDE.md
    ruby -e 'require "yaml"; Dir[".github/workflows/*.yml"].each { |f| YAML.load_file(f) }'
    git diff --check

## Validation and Acceptance

The exact dependency graph, all seven workspace versions, source and staged
binaries, changelog, notes, and README must agree on `0.30.2`. Focused tests
must prove valid standard base64 decoding and preserve malformed-input error
classification. All deterministic gates must pass.

The staged tree must remain 46 files with exactly eight runtime skills under
each skill root and no development-only material. Public claims must remain
limited to compatible dependency updates and SIMD-capable screenshot payload
decoding. Native GitHub Actions on Linux, macOS Intel, macOS arm64, and Windows
remain the final cross-platform release proof.

## Idempotence and Recovery

Rerun failed deterministic gates after fixing only their owning boundary. Do
not update another dependency, weaken a test, run live probes, apply or drop
either stash, or compensate with package drift. Recreate only the named staging
tree. If the dependency graph changes unexpectedly, stop and restore only that
unintended drift before continuing.

## Artifacts and Notes

Keep evidence aggregate and repository-relative. Do not retain raw screenshot
bytes, target IDs, account metadata, credentials, symbols, environment values,
machine paths, or temporary package contents in tracked files.

## Interfaces and Dependencies

The public `tv` interface is unchanged. `tradingview-cdp` depends directly on
`base64-simd = 0.8.0`, and
`crates/cdp/src/client.rs::screenshot_bytes_from_response` calls
`base64_simd::STANDARD.decode_to_vec`. The existing HTTP stack may continue to
resolve its own transitive `base64` version. No project API exposes either
decoder crate.

## Open Questions

There are no unresolved critical questions. End-to-end screenshot latency is
not established by this patch and is not a release acceptance criterion.
