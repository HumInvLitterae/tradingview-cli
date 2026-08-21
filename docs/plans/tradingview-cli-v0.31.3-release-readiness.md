# Prepare the v0.31.3 build-provenance patch

This ExecPlan is a living document maintained according to `.agents/PLANS.md`.
Keep `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes &
Retrospective` current while work proceeds.

## Purpose / Big Picture

Prepare `v0.31.3` from released `v0.31.2` plus reviewed build-provenance
reporting, two compatible dependency updates, and one behavior-preserving
Clippy baseline refactor. `tv --version` keeps a one-line version response but
adds commit and date provenance. `tv --version --verbose` adds a stable detailed
report with release, full commit hash, commit date, build timestamp, dirty
state, and target triple.

The patch updates workspace versions, the changelog, curated release notes,
the README archive example, the locked release binary, and the explicit release
package. It stops before tag creation, branch or tag push, workflow execution,
or GitHub Release publication.

## Progress

- [x] (2026-08-21) Confirmed `v0.31.2` is released from `03e9c24` and archived
  its completed release-readiness plan.
- [x] (2026-08-21) Froze and classified the ten commits in
  `v0.31.2..5a24cc6` and confirmed the reviewed worktree is clean.
- [x] (2026-08-21) Confirmed focused implementation review and correction
  re-review are green, including ten deterministic provenance tests.
- [x] (2026-08-21) Created this release-readiness ExecPlan and synchronized
  durable state without changing versioned artifacts.
- [ ] Bump the workspace and seven local lockfile package versions to `0.31.3`.
- [ ] Cut the changelog, add curated release notes, and update the README
  archive example.
- [ ] Run focused provenance and CLI contracts, four pinned JavaScript gates,
  and the complete Rust release baseline.
- [ ] Build `--release --locked`, stage and inspect the explicit package, and
  verify short and verbose source/staged version output.
- [ ] Record aggregate evidence and obtain focused release-readiness review.

## Surprises & Discoveries

- Observation: Cargo must watch git state beyond the currently loose branch
  ref to avoid retaining a stale dirty stamp after commit.
  Evidence: the reviewed correction watches the refs directory, index,
  packed-refs, and `logs/HEAD`; deterministic fixtures cover packed refs and
  reflog movement.

- Observation: a reproducible build timestamp cannot be tested safely by
  assuming the wall-clock day differs from a fixed commit day.
  Evidence: `Stamp::read_built_at` injects a fixed timestamp in deterministic
  tests, while production `Stamp::read` still obtains the actual build time.

- Observation: the pre-existing strict Clippy baseline failure was unrelated
  to provenance but blocked the required release gate.
  Evidence: `5a24cc6` boxes one large chart-quote timeout field without changing
  command behavior, envelopes, diagnostics, or timeout policy.

## Decision Log

- Decision: release the reviewed provenance work as `v0.31.3` rather than wait
  for a larger feature roadmap.
  Rationale: the feature is complete, independently useful for binary
  identification, reviewed, and bounded to version output and build metadata.
  Date/Author: 2026-08-21 / Codex

- Decision: expose the binary platform as `target`, not `host`.
  Rationale: Cargo's target triple describes where the produced `tv` binary
  runs; the compiler host is a distinct build concept and is not reported.
  Date/Author: 2026-08-21 / Codex

- Decision: preserve ordinary command JSON envelopes and version exit status.
  Rationale: provenance belongs only to the explicit version surface and must
  not alter normal command contracts or add common output metadata.
  Date/Author: 2026-08-21 / Codex

- Decision: require the full release baseline despite focused review being
  green.
  Rationale: build scripts, CLI root parsing, release packaging, dependency
  selections, and native target reporting cross release boundaries not proven
  by focused provenance tests alone.
  Date/Author: 2026-08-21 / Codex

## Outcomes & Retrospective

Preparation is in progress. The candidate and output contract are frozen, but
versioned artifacts and the complete release baseline are not yet recorded.
No tag, push, workflow execution, GitHub Release publication, live network
operation, Desktop mutation, or stash operation is authorized by this plan.

## Context and Orientation

`v0.31.2` is tag `03e9c24`. The ten later commits are local at `5a24cc6` and
not yet on `origin/main`. Their exact classification is:

- `bba3178`: compatible `h2` 0.4.18 lockfile update.
- `246d618`: short `tv --version` build provenance.
- `7f5414c`: detailed `tv --version --verbose` provenance.
- `823a406`: RFC 3339 local `built-at` with `SOURCE_DATE_EPOCH` support.
- `2f14bb7`: compatible `icu_provider` 2.3.1 and `zerovec-derive` 0.11.6
  lockfile updates.
- `cdcffcb`: refs-directory watching plus real-repository provenance fixtures.
- `e645cba`: reflog watching for an independent commit-change trigger.
- `5bb5024`: fixed build-time injection for deterministic provenance tests.
- `e9c0b80`: verbose vocabulary correction from `host` to `target`.
- `5a24cc6`: behavior-preserving chart-quote Clippy baseline refactor.

`crates/cli/build.rs` includes `crates/cli/build/provenance.rs`, derives git
and build metadata, and exports fixed compile-time environment values.
`crates/cli/src/build_info.rs` owns the short and verbose display shapes.
`crates/cli/src/cli.rs` disables clap's automatic version flag so root
`--version` can be combined with root `--verbose`; subcommand-specific verbose
flags remain separate.

A clean short response is `tv <release> (<short-commit> <commit-date>)`. A
dirty executable-source build appends `-dirty` and uses the local build date.
Only `crates/`, `Cargo.toml`, `Cargo.lock`, and `rust-toolchain*` affect dirty
state; docs do not. Missing git metadata yields `UNKNOWN` rather than failing
the build.

The verbose response starts with the same short line, then reports `binary`,
`release`, `commit-hash`, `commit-date`, `built-at`, `dirty`, and `target`.
`SOURCE_DATE_EPOCH` fixes `built-at` in UTC; malformed input fails the build
instead of silently becoming non-reproducible.

Seven workspace crates inherit `[workspace.package].version`. Release
preparation may change only the root version and seven local `tradingview-*`
lock entries. The reviewed third-party versions and checksums at `5a24cc6`
must remain fixed.

The package contract remains owned by
`scripts/stage-release-package-files.sh`. It copies the binary, public docs,
packaged agent guide, and exactly eight runtime skills under both skill roots.
Plans, notes, `CONTINUITY.md`, and development-only skills remain excluded.

## Plan of Work

Freeze `v0.31.2..5a24cc6` as the reviewed implementation and dependency
candidate. Change the workspace version from `0.31.2` to `0.31.3` and
synchronize seven local lock entries without selecting another third-party
version.

Move the current Unreleased provenance, internal, and planning entries into a
dated `v0.31.3` section, leave a fresh empty Unreleased section, add prose-first
`docs/releases/v0.31.3.md`, and update the current README archive example.
Release notes must explain short and verbose provenance, reproducible build
behavior, and preserved command contracts without promising binary
authenticity or supply-chain attestation beyond the reported build inputs.

Run the focused provenance and CLI contract tests under the reviewed
`SOURCE_DATE_EPOCH`, all four pinned JavaScript contracts, formatting, strict
workspace Clippy, the complete workspace suite and doctests, metadata, a
locked release build, public hygiene, package syntax, guide parity, workflow
YAML parsing, source/staged output checks, package inspection, and diff
hygiene. Record only aggregate and public version evidence.

## Concrete Steps

Run from the repository root:

    git status --short --branch
    git log --oneline v0.31.2..5a24cc6
    git diff --name-status v0.31.2..5a24cc6
    git stash list

After version and release-note edits, run:

    SOURCE_DATE_EPOCH=1709510400 cargo test -p tradingview-cli --test build_provenance
    cargo test -p tradingview-cli --test cli_contract
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
    target/release/tv --version --verbose
    scripts/stage-release-package-files.sh target/release-package-smoke target/release/tv
    target/release-package-smoke/tv --version
    target/release-package-smoke/tv --version --verbose
    python3 scripts/check-public-hygiene.py --self-test
    python3 scripts/check-public-hygiene.py
    bash -n scripts/stage-release-package-files.sh
    cmp -s AGENTS.md CLAUDE.md
    ruby -e 'require "yaml"; Dir[".github/workflows/*.{yml,yaml}"].sort.each { |f| YAML.load_file(f); puts "parsed #{f}" }'
    git diff --check

Inspect the staged package and require the reviewed file and skill counts:

    find target/release-package-smoke -type f | sort
    find target/release-package-smoke/.agents/skills -mindepth 1 -maxdepth 1 -type d | sort
    find target/release-package-smoke/.claude/skills -mindepth 1 -maxdepth 1 -type d | sort
    find target/release-package-smoke -type f | wc -l

## Validation and Acceptance

All seven workspace packages, source and staged version output, changelog,
release notes, and current README example agree on `0.31.3`. Release prep
changes only the root/local package versions and release documents after
`5a24cc6`; all reviewed third-party versions and checksums remain fixed.

The provenance suite passes ten tests with `SOURCE_DATE_EPOCH=1709510400`.
The CLI contract suite passes ten tests. The clean source and staged binaries
report non-UNKNOWN commit metadata, `dirty: false`, and the current target
triple. The package contains the existing 46 files and exactly eight runtime
skills under each skill root with no development-only material.

All deterministic gates pass. No ordinary command envelope, source, timeout,
retry, reconnect, fallback, shared session, broker, workflow, or package
contract changes in release preparation.

## Idempotence and Recovery

The deterministic commands and package staging may be rerun. Recreate only the
named staging directory. If Cargo selects another third-party version, stop
and restore only that unintended lockfile drift. Do not weaken provenance or
CLI tests, apply or drop either stash, run live probes, or compensate for a
failed gate with unrelated production work.

## Artifacts and Notes

Keep tracked evidence repository-relative. Public version output may record
the release commit and target triple, but do not retain machine paths,
credentials, account metadata, target IDs, raw payloads, or environment values.

## Interfaces and Dependencies

The stable version surfaces are:

    tv --version
    tv -V
    tv --version --verbose

No JSON envelope or library API is added. Build metadata is internal to the
binary and uses compile-time `TV_VERSION_*` and `TV_BUILD_*` values. The final
locked graph preserves `h2 = 0.4.18`, `icu_provider = 2.3.1`, and
`zerovec-derive = 0.11.6` from the reviewed candidate.

## Open Questions

There are no unresolved critical questions. Remote release operations remain
owner-controlled after focused release-readiness review.
