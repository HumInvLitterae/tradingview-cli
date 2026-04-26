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
- user-facing `AGENTS.md`
- user-facing `CLAUDE.md`
- runtime-oriented skills under `.agents/skills/`
- the same runtime-oriented skills under `.claude/skills/`

The user-facing `AGENTS.md` and `CLAUDE.md` are staged from
`packaging/agent/AGENTS.md`. They are intentionally different from the
repository root contributor guides.

## Runtime skill allowlist

`scripts/stage-release-package-files.sh` owns the release skill allowlist.

Runtime skills currently included:

- `chart-analysis`
- `multi-symbol-scan`
- `pine-develop`
- `replay-practice`
- `screener-workflow`
- `strategy-report`

Development-only skills must stay out of release archives. Examples:

- `continuity`
- `conventional-commits`
- `discovering-skills`
- `release-prep`

When adding a runtime skill, update the staging script, the packaged agent
guide, and README release archive description. Validate the changed skill with
the repo-local skill validator when available.

## Packaging validation

For release packaging changes, run:

```bash
bash -n scripts/stage-release-package-files.sh
cargo build --release --locked
rm -rf target/release-package-smoke
scripts/stage-release-package-files.sh target/release-package-smoke target/release/tv
find target/release-package-smoke -maxdepth 4 -print | sort
git diff --check
```

Confirm the archive staging directory includes runtime skills and excludes
development-only skills.

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
