# Add cross-platform config-based hook guardrails

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document follows `.agents/PLANS.md` from the repository root.

## Purpose / Big Picture

After this change, contributors can opt into local Git 2.54 config-based hooks that catch formatting and baseline failures before commit or push. The setup works on Unix/macOS through shell scripts and on Windows through PowerShell scripts. The hooks are development guardrails only; they do not change the Rust `tv` runtime behavior and do not touch TradingView Desktop, CDP, charts, layouts, alerts, Pine scripts, or account state.

This plan responds to upstream `tradesdontlie/tradingview-mcp` PR #102, which adds CI and agent guardrails to the original JavaScript project. The Rust repository already has CI and agent instructions, so this slice adopts only the parts that fit this Rust CLI: tighter CI workflow guardrails, optional local hooks, and documentation.

## Progress

- [x] (2026-04-25T15:26Z) Created this ExecPlan and fixed the cross-platform hook strategy.
- [x] (2026-04-25T15:32Z) Added CI permission/concurrency hardening and script syntax checks.
- [x] (2026-04-25T15:34Z) Added Unix/macOS and Windows hook scripts plus installer scripts.
- [x] (2026-04-25T15:35Z) Added `mise.toml` task shortcuts without changing Rust toolchain ownership.
- [x] (2026-04-25T15:40Z) Updated docs and upstream PR triage notes.
- [x] (2026-04-25T15:48Z) Ran local validation, installed local config-based hooks, and exercised hook runs.
- [ ] Commit the completed slice.

## Surprises & Discoveries

- Observation: This macOS environment has `mise` and Git 2.54, but no local `pwsh` or Windows PowerShell executable.
  Evidence: `mise tasks validate` passed and `git --version` reported 2.54.0 during planning, while `command -v pwsh`, `powershell.exe`, and `powershell` returned no path during validation.

- Observation: Local config-based hooks installed cleanly without writing tracked files.
  Evidence: `git config --local --get-regexp '^hook\.'` showed `tv-fast` for `pre-commit` and `tv-baseline` for `pre-push`; `git hook list pre-commit` showed `tv-fast`; `git hook list pre-push` showed `tv-baseline`.

## Decision Log

- Decision: Do not add `.pre-commit-config.yaml` or require the Python `pre-commit` package.
  Rationale: Git 2.54 config-based hooks provide the hook mechanism without adding a Python tool dependency.
  Date/Author: 2026-04-25 / Codex

- Decision: Make `mise` an optional task runner, not a hook runtime dependency.
  Rationale: `mise run hooks:install` is convenient, but commit and push hooks should still work when `mise` is not available in the Git hook environment.
  Date/Author: 2026-04-25 / Codex

- Decision: Provide PowerShell scripts for Windows instead of relying on Git Bash shell scripts.
  Rationale: Git for Windows often includes Bash, but GUI clients and PowerShell-oriented environments may not resolve shell hooks consistently. PowerShell scripts make the Windows path explicit.
  Date/Author: 2026-04-25 / Codex

## Outcomes & Retrospective

- Implemented cross-platform local hook guardrails. CI now has read-only permissions, concurrency, and script syntax checks. Developers can install Git 2.54 config-based hooks with `mise run hooks:install`, `scripts/install-config-hooks.sh`, or `scripts/install-config-hooks.ps1`. The hooks remain optional local helpers and do not replace CI or the normal Rust validation baseline.

## Context and Orientation

This repository is a Rust CLI project. The existing CI workflow is `.github/workflows/ci.yml`; it runs formatting, clippy, and tests. The release workflow is `.github/workflows/release.yml` and is out of scope for this task.

Git 2.54 supports config-based hooks, where local git config entries such as `hook.tv-fast.event` and `hook.tv-fast.command` define hook behavior. These entries are local configuration, not tracked files. Therefore this repository needs tracked scripts and a tracked installer script, while the actual hook enablement remains an explicit local action by each developer.

`mise` can run tasks from `mise.toml`. This plan uses `mise.toml` only as a convenience layer for commands such as `mise run hooks:install` and `mise run check:baseline`. Rust toolchain selection remains owned by the existing `rust-toolchain.toml`.

## Plan of Work

Update `.github/workflows/ci.yml` by adding read-only permissions and workflow concurrency. Add a script syntax check job that runs both shell syntax checks and PowerShell parser checks. Keep the current fmt, clippy, and OS matrix test jobs intact.

Create `scripts/git-hooks/pre-commit-fast.sh` and `scripts/git-hooks/pre-commit-fast.ps1`. These scripts must move to the repository root, run `cargo fmt --check`, and run `git diff --check`.

Create `scripts/git-hooks/pre-push-baseline.sh` and `scripts/git-hooks/pre-push-baseline.ps1`. These scripts must move to the repository root, run `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`, and `git diff --check`.

Create `scripts/install-config-hooks.sh` and `scripts/install-config-hooks.ps1`. These installers must verify Git is version 2.54 or newer, then set local config-based hooks for pre-commit and pre-push. The Unix installer should configure shell commands. The Windows installer should configure PowerShell commands, preferring `pwsh` and falling back to `powershell.exe`. The hook commands must avoid machine-specific absolute paths and call tracked scripts by repository-relative path after changing to the repository root.

Create `mise.toml` with task shortcuts only. Do not add `[tools]`. Add tasks for `check:fast`, `check:baseline`, `hooks:install`, `hooks:list`, `hooks:disable`, and `hooks:enable`. The install task should dispatch to the PowerShell installer on Windows and the shell installer elsewhere.

Update docs to explain that hooks are optional local guardrails. The authoritative baseline remains the normal Rust validation commands and CI.

## Concrete Steps

Work from the repository root.

Create and edit the files described above. Keep scripts small and idempotent. Use `set -euo pipefail` in shell scripts and `$ErrorActionPreference = "Stop"` in PowerShell scripts.

Run syntax checks:

    ruby -e 'require "yaml"; YAML.load_file(".github/workflows/ci.yml"); puts "CI YAML OK"'
    bash -n scripts/git-hooks/pre-commit-fast.sh
    bash -n scripts/git-hooks/pre-push-baseline.sh
    bash -n scripts/install-config-hooks.sh
    pwsh -NoProfile -Command "[System.Management.Automation.Language.Parser]::ParseFile('scripts/git-hooks/pre-commit-fast.ps1', [ref]$null, [ref]$null) | Out-Null"
    pwsh -NoProfile -Command "[System.Management.Automation.Language.Parser]::ParseFile('scripts/git-hooks/pre-push-baseline.ps1', [ref]$null, [ref]$null) | Out-Null"
    pwsh -NoProfile -Command "[System.Management.Automation.Language.Parser]::ParseFile('scripts/install-config-hooks.ps1', [ref]$null, [ref]$null) | Out-Null"

Run task and hook checks:

    mise tasks
    mise run check:fast
    mise run check:baseline
    mise run hooks:install
    mise run hooks:list
    mise run hooks:disable
    mise run hooks:enable
    git hook list pre-commit
    git hook list pre-push
    git hook run pre-commit
    git hook run pre-push

Run the Rust baseline:

    cargo fmt --check
    cargo clippy --all-targets --all-features -- -D warnings
    cargo test
    git diff --check

## Validation and Acceptance

Acceptance is met when CI YAML parses, shell and PowerShell scripts parse, `mise` tasks are listed and runnable on the current host, config-based hooks are installed into local git config, `git hook list` shows pre-commit and pre-push entries, `git hook run pre-commit` and `git hook run pre-push` succeed, and the Rust baseline passes.

The docs must remain public-safe: no machine-specific absolute paths, secrets, account-local ids, or private TradingView operational metadata are added.

## Idempotence and Recovery

The hook installers are safe to run repeatedly. They overwrite only the named local config entries `hook.tv-fast.*` and `hook.tv-baseline.*`. They do not modify tracked files after installation. The disable and enable tasks only flip `hook.tv-fast.enabled` and `hook.tv-baseline.enabled`.

If a developer uses Git older than 2.54, the installer must fail with a clear message and leave normal manual validation as the fallback.

## Artifacts and Notes

Validation completed:

    ruby -e 'require "yaml"; YAML.load_file(".github/workflows/ci.yml"); puts "CI YAML OK"'
    bash -n scripts/git-hooks/pre-commit-fast.sh
    bash -n scripts/git-hooks/pre-push-baseline.sh
    bash -n scripts/install-config-hooks.sh
    mise tasks validate
    mise tasks
    mise run check:fast
    mise run check:baseline
    mise run hooks:install
    mise run hooks:list
    mise run hooks:disable
    mise run hooks:enable
    git hook list pre-commit
    git hook list pre-push
    git hook run pre-commit
    git hook run pre-push
    git diff --check
    git grep -nE '(/Users/|C:\\)' -- README.md AGENTS.md CLAUDE.md docs .agents/skills scripts mise.toml .github || true

`mise run check:baseline` and `git hook run pre-push` both ran the full Rust baseline through `scripts/git-hooks/pre-push-baseline.sh`: `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`, and `git diff --check`. `cargo test` passed 236 unit tests and 69 CLI contract tests. Local PowerShell parser validation was skipped because this host has no PowerShell executable; `.github/workflows/ci.yml` checks the PowerShell scripts on the Windows runner.

## Interfaces and Dependencies

No Rust dependency is added. `mise.toml` adds task names only and must not contain a `[tools]` section.

The public developer commands added by this slice are:

    mise run hooks:install
    mise run hooks:list
    mise run hooks:disable
    mise run hooks:enable
    mise run check:fast
    mise run check:baseline
    scripts/install-config-hooks.sh
    scripts/install-config-hooks.ps1

## Open Questions

None. The hook mechanism, Windows support strategy, `mise` role, validation commands, and docs scope are decided in this plan.
