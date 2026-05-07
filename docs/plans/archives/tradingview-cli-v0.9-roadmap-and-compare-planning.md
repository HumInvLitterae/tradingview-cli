# v0.9.0 Roadmap And Desktop-Free Compare Planning

This ExecPlan prepares the next roadmap after the `v0.8.0` snapshot release
and the chart-source quote concurrency audit.

## Purpose

Create `docs/v0.9-roadmap.md` and decide the first `v0.9.0` implementation
slice. The default direction is Desktop-free comparison built on `tv quotes`,
scanner reads, fundamentals, and `tv snapshot`, rather than selected-chart
switching loops.

## Context

The chart-source quote concurrency smoke passed with near-concurrent widths 2
and 3 in one live Desktop session. That does not prove chart-source quote can
serve as a multi-symbol realtime batch source. It does support moving `v0.9`
planning forward with a conservative boundary: broad comparison should use
Desktop-free reads, while chart-source quote remains a single-symbol
selected-chart feed check.

## Planned Work

- Add `docs/v0.9-roadmap.md` with `compare` as the first candidate lane.
- Define `tv compare <SYMBOL>...` as Desktop-free first-pass comparison unless
  a later plan proves a separate realtime source is appropriate.
- Keep chart-source quote, `observe chart`, screenshots, and lab `bars` as
  follow-up evidence, not default compare inputs.
- Record deferred items: chart-backed realtime batch, stable browserless bars,
  daemon/watch behavior, `tv diagnose`, binary split, MCP server, and trading
  automation.
- Update `docs/plans/README.md`, `docs/v0.8-roadmap.md`, and `CHANGELOG.md`.

## Validation

This is expected to be docs-only:

```bash
git diff --check
bash -n scripts/stage-release-package-files.sh
```

If Rust code changes unexpectedly, run the normal baseline before commit.

## Outcomes

Completed on 2026-05-07.

- Added `docs/v0.9-roadmap.md`.
- Created the first implementation ExecPlan for `tv compare <SYMBOL>...`.
- Fixed the default source boundary: compare starts Desktop-free and does not
  use chart-source quote loops.
- Archived this planning ExecPlan after updating `docs/plans/README.md`.
