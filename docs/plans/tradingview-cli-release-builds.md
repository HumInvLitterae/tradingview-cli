# Add GitHub Release builds for tv

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This plan follows `.agents/PLANS.md`. It is self-contained so a future contributor can understand why release automation exists, what files it owns, and how to validate it.

## Purpose / Big Picture

The project has a working Rust-native `tv` binary, but users currently have to build it themselves from the repository. After this change, pushing a version tag such as `v0.1.0` creates a GitHub Release with native command-line binaries for Linux, macOS on Intel, macOS on Apple Silicon, and Windows. The release also includes `SHA256SUMS` so users can verify downloaded artifacts before placing `tv` on their `PATH`. Release archives also include user-facing agent instructions and runtime TradingView CLI skills so users can unpack the archive and ask an AI agent to operate `tv` safely.

This does not publish to crates.io, package managers, Homebrew, Snap, Winget, or installer systems. It is the first distribution step: reproducible release builds attached to GitHub Releases while keeping the binary name `tv`.

## Progress

- [x] (2026-04-25 12:42 JST) Confirmed `Cargo.lock` is present and the package binary is named `tv`.
- [x] (2026-04-25 12:42 JST) Confirmed the existing CI workflow only runs formatting, linting, and tests, not release packaging.
- [x] (2026-04-25 12:42 JST) Checked current GitHub-hosted runner labels in official GitHub documentation.
- [x] (2026-04-25 12:48 JST) Added `.github/workflows/release.yml` for tag-triggered native release builds and GitHub Release publication.
- [x] (2026-04-25 12:48 JST) Updated README and plan index so contributors can find the release workflow and artifact contract.
- [x] (2026-04-25 12:55 JST) Ran local validation: workflow YAML parse, formatting, linting, tests, release build, whitespace check, and tracked-doc absolute-path scan all passed.
- [x] (2026-04-25 13:30 JST) Planned release archive agent assets: user-facing `AGENTS.md` and `CLAUDE.md`, plus runtime skills under both `.agents/skills` and `.claude/skills`.

## Surprises & Discoveries

- Observation: The repository already had normal CI, but no release packaging workflow.
  Evidence: `.github/workflows/ci.yml` runs formatting, linting, and tests. Before this plan, there was no `.github/workflows/release.yml`.

## Decision Log

- Decision: Release builds are created from tags matching `v*`.
  Rationale: Tags are the cleanest boundary between normal continuous integration and public binary publication. This avoids creating release artifacts for every push.
  Date/Author: 2026-04-25 / Codex

- Decision: Build native targets on native GitHub-hosted runners: `ubuntu-24.04` for `x86_64-unknown-linux-gnu`, `macos-15-intel` for `x86_64-apple-darwin`, `macos-15` for `aarch64-apple-darwin`, and `windows-2025` for `x86_64-pc-windows-msvc`.
  Rationale: Native runner builds avoid early cross-compilation complexity and match the first public distribution goal: macOS, Linux, and Windows binaries with minimal toolchain assumptions.
  Date/Author: 2026-04-25 / Codex

- Decision: Include only checksum hardening for this first release workflow.
  Rationale: The project is not yet ready to promise code signing, notarization, installer packaging, or artifact attestations. `SHA256SUMS` gives users a simple integrity check without making stronger trust claims.
  Date/Author: 2026-04-25 / Codex

- Decision: Keep `publish = false` in `Cargo.toml`.
  Rationale: This step distributes binaries through GitHub Releases only. crates.io publishing is a separate release-policy decision.
  Date/Author: 2026-04-25 / Codex

## Outcomes & Retrospective

The release workflow and documentation are in place. Local validation passed, including `cargo build --release --locked`, which proves the current host can produce a release binary with the locked dependency graph. The remaining proof is GitHub-hosted end-to-end validation: after the repository owner pushes a `v*` tag, the workflow should publish the four expected archives and `SHA256SUMS` to a GitHub Release.

The first release workflow deliberately stays modest. It gives users downloadable binaries and checksum verification without claiming installer support, package-manager availability, signing, or notarization.

## Context and Orientation

This repository builds a Rust command-line application named `tv`. The package metadata lives in `Cargo.toml`; the binary target is declared as `[[bin]] name = "tv" path = "src/main.rs"`. The existing `.github/workflows/ci.yml` workflow runs the normal Rust baseline on pushes and pull requests, but it does not create release binaries or upload artifacts.

GitHub Actions workflows live under `.github/workflows/`. A workflow is a YAML file that GitHub runs when configured events happen. This plan adds `.github/workflows/release.yml`. A tag-triggered workflow means GitHub runs the workflow when a tag like `v0.1.0` is pushed.

A release artifact is a downloadable file attached to a GitHub Release. This plan creates one archive per operating-system target:

- `tv-<tag>-x86_64-unknown-linux-gnu.tar.gz`
- `tv-<tag>-x86_64-apple-darwin.tar.gz`
- `tv-<tag>-aarch64-apple-darwin.tar.gz`
- `tv-<tag>-x86_64-pc-windows-msvc.zip`

Each archive contains the executable, `README.md`, `LICENSE`, user-facing `AGENTS.md` and `CLAUDE.md`, and runtime TradingView CLI skills. The Linux and macOS archives contain an executable named `tv`. The Windows archive contains `tv.exe`.

## Plan of Work

Add `.github/workflows/release.yml`. The workflow runs on pushed tags matching `v*`, builds the Rust package in release mode for each supported native target, runs `cargo test --locked` before packaging, stages the release package with the binary, README, LICENSE, user-facing agent guides, and runtime skills, uploads build artifacts, then creates one GitHub Release containing all archives and `SHA256SUMS`.

Update `README.md` with a release-build section that tells users what assets to expect, how to verify checksums, and that package-manager installers are not yet provided. Keep the Quick Start focused on using `tv`, not `cargo run`. Make clear that source-root agent files are contributor-facing while release archives contain user-facing agent files.

Update `docs/plans/README.md` so this plan is the current release-readiness plan, while the previous documentation cleanup plan remains a completed plan that can later be archived.

Add `packaging/agent/AGENTS.md` as the source for release archive user-facing agent instructions. Add `scripts/stage-release-package-files.sh` to stage package contents consistently for all operating systems. The staging script should copy only runtime-oriented skills: `chart-analysis`, `multi-symbol-scan`, `pine-develop`, `replay-practice`, and `strategy-report`. It should not copy development-only skills such as `continuity`, `conventional-commits`, or `discovering-skills`.

Do not modify Rust source code for this task. Do not modify `Cargo.toml` except in a separate release-policy task, because the user already noted they have made package metadata changes themselves.

## Concrete Steps

From the repository root, inspect the existing state:

    git status --short
    sed -n '1,220p' .github/workflows/ci.yml
    sed -n '1,220p' Cargo.toml

Create `.github/workflows/release.yml` with a `build` job and a `publish` job. The `build` job uses a matrix for the four native targets, runs tests and `cargo build --release --locked --target <target>`, stages package files through `bash scripts/stage-release-package-files.sh`, and uploads exactly one archive per target. The `publish` job downloads those archives, writes `SHA256SUMS`, and runs `gh release create "$GITHUB_REF_NAME" dist/* --title "$GITHUB_REF_NAME" --generate-notes --verify-tag`.

Update `README.md` and `docs/plans/README.md` as described above.

Run validation from the repository root:

    ruby -e 'require "yaml"; YAML.load_file(".github/workflows/release.yml"); puts "release workflow YAML OK"'
    bash -n scripts/stage-release-package-files.sh
    rm -rf target/release-package-smoke
    scripts/stage-release-package-files.sh target/release-package-smoke target/release/tv
    find target/release-package-smoke -maxdepth 4 -print | sort
    cargo fmt --check
    cargo clippy --all-targets --all-features -- -D warnings
    cargo test
    cargo build --release --locked
    git diff --check
    git grep -nE '(/[U]sers/|[C]:\\)' -- README.md AGENTS.md docs .agents/skills || true
    git status --short

Because this workflow only runs on GitHub after a tag is pushed, final end-to-end validation happens by creating and pushing a tag such as `v0.1.0` after the repository owner is ready:

    git tag v0.1.0
    git push origin v0.1.0

Do not push a tag during implementation unless the user explicitly asks for it.

## Validation and Acceptance

Local acceptance is met when the workflow YAML parses, the package staging script parses and creates the expected archive tree, the Rust baseline passes, `cargo build --release --locked` produces a local release binary, `git diff --check` reports no whitespace errors, and the tracked documentation path scan finds no machine-specific absolute paths.

GitHub acceptance is met after a pushed `v*` tag produces a GitHub Release containing four platform archives and `SHA256SUMS`. The release page should show these assets:

    tv-v0.1.0-x86_64-unknown-linux-gnu.tar.gz
    tv-v0.1.0-x86_64-apple-darwin.tar.gz
    tv-v0.1.0-aarch64-apple-darwin.tar.gz
    tv-v0.1.0-x86_64-pc-windows-msvc.zip
    SHA256SUMS

On Linux or macOS, a user can unpack the matching archive, place `tv` on `PATH`, and run:

    tv --help

On Windows, a user can unpack the zip, place `tv.exe` on `PATH`, and run:

    tv.exe --help

## Idempotence and Recovery

The workflow is safe to rerun for a tag before the release is created. If the `publish` job fails after the release already exists, delete the draft or failed release through GitHub and rerun the workflow, or create a new patch tag after fixing the workflow. Do not overwrite a public release silently.

Local validation commands are safe to repeat. `cargo build --release --locked` writes under `target/`, which is ignored build output.

## Artifacts and Notes

The release workflow owns `.github/workflows/release.yml`. Release packaging also owns `packaging/agent/AGENTS.md` and `scripts/stage-release-package-files.sh`.

The first implementation intentionally skips signing and package-manager publishing. Those should be planned separately after the first GitHub Release proves the binary archives work on each operating system.

## Interfaces and Dependencies

The workflow depends on:

- `actions/checkout@v4` to fetch the repository.
- `dtolnay/rust-toolchain@stable` to install the stable Rust toolchain.
- `actions/upload-artifact@v4` and `actions/download-artifact@v4` to move archives from build jobs to the publish job.
- GitHub CLI `gh`, available on GitHub-hosted runners, to create the GitHub Release.
- The repository `GITHUB_TOKEN`, exposed to the workflow as `github.token`, with `contents: write` permission.

The release workflow does not require repository secrets.

## Open Questions

No open question blocks the first release-build workflow. Future release work should decide whether to add signing, notarization, artifact attestations, package-manager distribution, or crates.io publication.

Revision note: initial release-build ExecPlan added with the workflow implementation so future contributors can rerun, debug, or extend the first GitHub Release path without relying on chat history.

Revision note: updated the plan to cover user-facing agent guides and runtime skill files in release archives, while keeping source-root agent files contributor-facing.
