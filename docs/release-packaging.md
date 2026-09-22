# Release packaging

This document records the stable release and archive packaging contract for the
Rust-native `tv` CLI.

## Release channel

GitHub Releases are the first supported binary distribution path.

Pushing a version tag matching `v*` runs `.github/workflows/release.yml`, builds
native release archives, generates `SHA256SUMS`, and publishes a GitHub Release.

The release workflow currently builds:

- `x86_64-unknown-linux-gnu`
- `x86_64-apple-darwin`
- `aarch64-apple-darwin`
- `x86_64-pc-windows-msvc`

Package-manager installers, code signing, notarization, and crates.io
publication are not part of the first release workflow.

## Release notes

Keep `CHANGELOG.md` as the project-level changelog.

For tag-specific GitHub Release notes, add:

```text
docs/releases/<tag>.md
```

The workflow strips a leading top-level heading from that file because the
GitHub Release title already contains the tag. If the file does not exist, the
workflow falls back to generated notes.

## Archive contents

Each release archive includes:

- `tv` or `tv.exe`
- `README.md`
- `CHANGELOG.md`
- `LICENSE`
- user-facing `docs/getting-started.md`
- user-facing `docs/ja/getting-started.md`
- user-facing `AGENTS.md`
- user-facing `CLAUDE.md`
- runtime-oriented skills under `.agents/skills/`
- the same runtime-oriented skills under `.claude/skills/`

The user-facing `AGENTS.md` and `CLAUDE.md` are staged from
`packaging/agent/AGENTS.md`. They are intentionally different from the
repository root contributor guides.

Only the two getting-started docs are copied from `docs/` into release
archives. The full repository documentation tree remains a contributor and
project reference and is not broadly packaged.

## Runtime skill allowlist

`scripts/stage-release-package-files.sh` owns the release skill allowlist.

Runtime skills currently included:

- `account-management`
- `market-data`
- `chart-analysis`
- `pine-develop`
- `replay-practice`
- `screener-workflow`
- `strategy-report`

`market-data` replaces `market-data-interpretation`, `multi-symbol-scan`, and
`screener-result-analysis`; update prompts that name those former skills.
Development-only `continuity` and `release-prep` stay out of archives. The
former `conventional-commits` and `discovering-skills` wrappers are retired;
commit rules live in development guidance.

When changing runtime guidance, update affected entrypoints and references,
the staging allowlist, the packaged guide, and user-facing package docs together.
Keep links relative to their containing document, except the packaged guide
source, whose links are authored relative to the archive root. Shared references
must be included inside each skill that needs them and under both skill roots.
A skill must work when only its directory is attached: no required sibling-skill
or repository references. Optional handoffs name another skill without making
its files part of the current workflow.

## Packaging validation

Staging runs `scripts/check-runtime-package.py` with the same allowlist. It
checks guide/resource parity, exact skill membership, and local Markdown links
transitively from the guides and skills, including document paths written as
inline code in skills. Each skill is also checked in isolation, rejecting sibling
references even when the sibling is present in the archive. A missing or escaping reference fails
staging. Online links are not fetched. CI also exercises the checker and stages
a disposable placeholder binary, proving guidance packaging without claiming a
working CLI build.

For guidance or staging-only changes, use an existing binary and a fresh
disposable output directory:

```bash
bash -n scripts/stage-release-package-files.sh
python scripts/check-runtime-package.py --self-test
python scripts/check-runtime-package.py --skill .agents/skills/account-management
scripts/stage-release-package-files.sh target/release-package-smoke target/release/tv
git diff --check
```

The staging script replaces its output directory; use only disposable staging
paths. For a real release, first build `cargo build --release --locked` and use
that candidate's binary. Check binary provenance separately: resource checks do
not prove Rust behavior, platform execution, or remote publication. A guidance
edit alone does not require rebuilding an unchanged executable.

For release workflow changes, also inspect `.github/workflows/release.yml` and
ensure the tag-triggered asset names remain stable:

- `tv-<tag>-x86_64-unknown-linux-gnu.tar.gz`
- `tv-<tag>-x86_64-apple-darwin.tar.gz`
- `tv-<tag>-aarch64-apple-darwin.tar.gz`
- `tv-<tag>-x86_64-pc-windows-msvc.zip`
- `SHA256SUMS`

## Public release hygiene

Before a public release, check:

- `LICENSE` exists and matches the intended license
- README states the TradingView affiliation and terms boundaries
- docs do not contain local absolute paths, account-local identifiers,
  credentials, cookies, tokens, or raw live payloads
- GitHub Actions CI is green for the target commit
- release notes exist for the tag if curated notes are desired
- release archives contain only user-facing runtime guidance, not development
  or continuity skills
