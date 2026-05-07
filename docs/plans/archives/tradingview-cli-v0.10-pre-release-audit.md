# v0.10 pre-release completion and refactor audit

This ExecPlan is a living document. Keep `Progress`, `Surprises & Discoveries`,
`Decision Log`, and `Outcomes & Retrospective` current as work proceeds.

This document follows `.agents/PLANS.md` from the repository root. It records
the final audit before `v0.10.0` release readiness.

## Purpose / Big Picture

`v0.10.0` adds downstream-oriented `tv compare` summary readback and workflow
decision-table polish. Before release readiness, stop feature work and confirm
that this is enough for `v0.10.0`, that compare contracts are still additive,
and that no small release-blocking refactor remains.

After this slice, the next step should be `v0.10.0` release readiness unless a
release blocker is found.

## Progress

- [x] (2026-05-08T00:00Z) Archived the completed compare workflow decision-table
  plan and created this pre-release audit plan.
- [x] (2026-05-08T00:00Z) Audited compare summary construction, typed result
  shape, CLI dispatch, focused contract tests, runtime skills, and stable docs.
- [x] (2026-05-08T00:00Z) Ran focused compare tests, full Rust baseline,
  docs validation, packaging script syntax check, grep audits, and read-only
  compare smoke.
- [x] (2026-05-08T00:00Z) Recorded that no v0.10 release blocker or immediate
  refactor need was found.

## Surprises & Discoveries

- The `compare` summary implementation is already localized in
  `tradingview-market`: summary fields are derived from finalized `items` and
  do not introduce a second symbol-resolution source.
- The remaining TODO/panic audit hits are the known assertion-style `panic!`
  calls in ignored live smoke tests, one Pine template TODO string, and
  archived validation examples. No release-blocking TODO, FIXME,
  `unimplemented!`, or `todo!` marker was found.
- The largest Rust files remain older operation adapters and scanner modules.
  `crates/market/src/compare.rs` is under 500 lines and does not need release
  blocking decomposition.

## Decision Log

- Decision: Treat `v0.10.0` as complete after compare summary readback and
  workflow decision-table polish.
  Rationale: The roadmap goal was downstream scanability, not a new data source.
  The current implementation and docs satisfy that without broadening compare.
  Date/Author: 2026-05-08 / Codex.

- Decision: Do not refactor compare internals before `v0.10.0`.
  Rationale: Summary construction is small, covered by focused tests, and
  behavior-preserving refactor would add release risk without a blocker.
  Date/Author: 2026-05-08 / Codex.

- Decision: Keep ranking, scoring, recommendation, chart-backed compare,
  watch/JSONL compare, realtime multi-symbol feed, stable browserless bars,
  `tv events`, `tv diagnose`, binary split, and MCP server work deferred.
  Rationale: None are required for the v0.10 downstream readback goal, and
  several would blur source boundaries.
  Date/Author: 2026-05-08 / Codex.

## Outcomes & Retrospective

Audit-only slice completed. `tv compare` summary readback remains additive:
existing top-level counts, `items[]`, section errors, source metadata, and
next-action hints remain in place, while `summary` provides machine-readable
readback for resolution, section success, missing counts, and ordered resolved
symbol mapping.

The workflow docs and runtime skills consistently separate quote-only reads,
multi-symbol Desktop-free evidence, one-symbol Desktop-free detail,
selected-chart observation, single-symbol chart-feed follow-up, and screenshot
evidence.

No Rust code was changed. No release blocker was found. The next step is
`v0.10.0 release readiness`.

## Context and Orientation

The relevant v0.10 surfaces are:

- `tv compare <SYMBOL>...` for Desktop-free multi-symbol evidence;
- `compare.summary` for readback and scanability;
- `compare.items[]` for raw per-symbol evidence;
- `tv snapshot <SYMBOL>` for one-symbol Desktop-free detail;
- `tv observe chart` and `tv quote --source chart` for explicit selected-chart
  follow-up.

`compare` must not become a ranking, recommendation, chart-switching, or
realtime batch command by accident.

## Plan of Work

Inspect compare implementation, tests, docs, and runtime skills for contract
drift. Run focused compare tests, the full Rust baseline, docs validation,
packaging script syntax check, grep audits, and an optional read-only compare
smoke that records only public-safe summary values.

Update `docs/v0.10-roadmap.md` and `docs/plans/README.md` so this audit is the
current plan. Archive the completed decision-table plan. Record user-facing
docs-only release prep context in `CHANGELOG.md` only if needed.

## Validation and Acceptance

Completed validation:

    git diff --check
    bash -n scripts/stage-release-package-files.sh
    rg -n "TODO|FIXME|panic!|unimplemented!|todo!" crates docs README.md AGENTS.md CLAUDE.md packaging/agent/AGENTS.md
    rg -n "compare|summary|resolved_symbols|snapshot|observe chart|ranking|recommendation|realtime|diagnose|binary split|MCP|daemon" README.md CHANGELOG.md docs .agents/skills packaging/agent/AGENTS.md
    cargo test -p tradingview-market compare -- --nocapture
    cargo test -p tradingview-cli --test cli_contract compare -- --nocapture
    cargo test -p tradingview-cli --test live_compare
    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1
    target/debug/tv compare --help

Optional read-only smoke also passed for a public two-symbol compare. The
tracked docs record only the success summary, not raw live output.

Acceptance is met when no release blocker is found, no Rust code is changed,
and the next step can move to `v0.10.0` release readiness.

## Idempotence and Recovery

This slice is audit-only. If a later check finds a release blocker, create a
small focused fix plan rather than mixing the fix into release readiness.

## Interfaces and Dependencies

No CLI behavior, JSON payload, Rust API, dependency, release package behavior,
or version changes are introduced.

## Open Questions

None for `v0.10.0`. Deferred ideas should be handled after release readiness or
in the next roadmap.
