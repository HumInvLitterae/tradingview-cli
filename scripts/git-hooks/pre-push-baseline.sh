#!/usr/bin/env bash
set -euo pipefail

repo_root="$(git rev-parse --show-toplevel)"
cd "$repo_root"

cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
git diff --check
