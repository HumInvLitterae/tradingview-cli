# README and plan archive cleanup

## Summary

This plan prepares the repository documentation for public release readiness after the known old JavaScript CLI migration closed. It changes the README from a development-first command list into a user-first `tv` CLI guide, archives historical ExecPlans, and adds MIT license metadata.

## Progress

- [x] Move completed historical ExecPlans under `docs/plans/archives/` and remove version-like suffixes from their filenames.
- [x] Add `docs/plans/README.md` as the plan index and explain that archived version-like labels were execution-slice labels, not package versions.
- [x] Update README and AGENTS to point at the plan index instead of listing every historical ExecPlan.
- [x] Add MIT license file and Cargo metadata.
- [x] Run automated validation and documentation hygiene checks.

## Validation

Run the normal baseline before committing:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
git diff --check
```

Also verify tracked documentation does not contain machine-specific absolute paths and that top-level docs no longer reference archived version-like plan filenames.
