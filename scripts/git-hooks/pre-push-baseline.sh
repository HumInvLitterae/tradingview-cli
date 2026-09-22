#!/usr/bin/env bash
set -euo pipefail

repo_root="$(git rev-parse --show-toplevel)"
cd "$repo_root"

# Keep explicit local baseline runs conservative; callers may override either.
export CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-1}"
export RUST_TEST_THREADS="${RUST_TEST_THREADS:-1}"

cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
git diff --check
