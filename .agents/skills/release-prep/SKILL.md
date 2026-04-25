---
name: release-prep
description: Prepare a tradingview-cli release by updating versioned release notes, changelog, packaging expectations, validation evidence, and release workflow guardrails. Use when the user asks for release prep, version bump readiness, GitHub Release notes, release archive contents, or pre-tag release checks. "リリース準備", "release prep", "CHANGELOG", "GitHub Release", "tag release", "配布", "v0.1".
allowed-tools: Read, Grep, Glob, Bash
---

# Release Prep

Use this skill for repository release preparation. It is a development-only skill and must not be packaged into release archives.

## Core Rule

The user pushes tags and branches. Do not push or create remote releases unless explicitly asked in the current turn.

Release archives must include only runtime-oriented skills. Before changing packaging, verify `scripts/stage-release-package-files.sh` still copies skills from an explicit allowlist and does not copy every folder under `.agents/skills`.

## Workflow

1. **Ground the release state.**
   - Read `Cargo.toml`, `CHANGELOG.md`, `README.md`, `.github/workflows/release.yml`, `scripts/stage-release-package-files.sh`, and any `docs/releases/<tag>.md`.
   - Check `git status --short` and recent commits.
   - If CI or GitHub Release status matters, inspect it with `gh run list` / `gh run view`; do not assume current status.

2. **Prepare versioned notes.**
   - Update `CHANGELOG.md` with a dated section for the release tag.
   - Add or update `docs/releases/<tag>.md` for the GitHub Release body.
   - Do not put a top-level `# <tag>` heading in the release body; the GitHub Release title already contains the tag.
   - Keep release notes user-facing: Added / Changed / Fixed / Security / Tests and docs are usually enough.

3. **Check package contents.**
   - Confirm release archives contain the binary, `README.md`, `CHANGELOG.md`, `LICENSE`, user-facing `AGENTS.md` and `CLAUDE.md`, and runtime skills under `.agents/skills/` and `.claude/skills/`.
   - Confirm development-only skills such as `continuity`, `conventional-commits`, `discovering-skills`, and `release-prep` are not copied.
   - If packaging changes, run the staging script locally against an existing built binary and inspect the staged tree.

4. **Validate before commit.**
   - Run the normal baseline when code or workflow behavior changes:
     - `cargo fmt --check`
     - `cargo clippy --all-targets --all-features -- -D warnings`
     - `cargo test`
     - `git diff --check`
   - For docs-only release note changes, `git diff --check` plus tracked-doc hygiene scans may be enough.
   - Always scan tracked docs and skills for machine-local paths and account-local metadata before public release.

5. **Commit intentionally.**
   - Use the conventional-commits skill for the final message.
   - Keep release prep, CI workflow fixes, and feature work in separate commits unless the change is inseparable.
   - Do not tag or push; tell the user the exact commit and remaining manual tag/push step.

## Guardrails

- Do not include raw live smoke payloads, account-local TradingView IDs, local paths, usernames, emails, or secrets in release notes.
- Do not make release notes depend on generated GitHub notes when a curated `docs/releases/<tag>.md` exists.
- Do not remove runtime skills from release archives without explicitly updating user-facing package docs.
- Do not package development-only skills by using broad copies such as `cp -R .agents/skills`.
