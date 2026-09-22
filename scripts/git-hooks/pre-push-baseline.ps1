$ErrorActionPreference = "Stop"

$repoRoot = git rev-parse --show-toplevel
Set-Location $repoRoot

# Keep explicit local baseline runs conservative; callers may override either.
if (-not $env:CARGO_BUILD_JOBS) { $env:CARGO_BUILD_JOBS = "1" }
if (-not $env:RUST_TEST_THREADS) { $env:RUST_TEST_THREADS = "1" }

cargo fmt --check
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

cargo clippy --workspace --all-targets --all-features -- -D warnings
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

cargo test --workspace
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

git diff --check
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
