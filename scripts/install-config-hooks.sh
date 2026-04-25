#!/usr/bin/env bash
set -euo pipefail

repo_root="$(git rev-parse --show-toplevel)"
cd "$repo_root"

git_version="$(git version | awk '{ print $3 }')"
major="${git_version%%.*}"
rest="${git_version#*.}"
minor="${rest%%.*}"

if [ "$major" -lt 2 ] || { [ "$major" -eq 2 ] && [ "$minor" -lt 54 ]; }; then
  echo "Git 2.54 or newer is required for config-based hooks. Found: $git_version" >&2
  exit 1
fi

git config --local hook.tv-fast.event pre-commit
git config --local hook.tv-fast.command 'bash scripts/git-hooks/pre-commit-fast.sh'
git config --local hook.tv-fast.enabled true

git config --local hook.tv-baseline.event pre-push
git config --local hook.tv-baseline.command 'bash scripts/git-hooks/pre-push-baseline.sh'
git config --local hook.tv-baseline.enabled true

git hook list pre-commit
git hook list pre-push
