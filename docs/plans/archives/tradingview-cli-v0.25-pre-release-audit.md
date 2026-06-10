# v0.25.0 pre-release architecture audit

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document follows `.agents/PLANS.md`. It is self-contained so a future contributor can continue from this file without needing prior conversation history.

## Purpose / Big Picture

This slice audits the completed `v0.25.0` chart-backed workflow maturity work before release readiness.

`v0.25.0` added `tv chart compare`, `tv events compare`, `tv replay log --attach-ohlcv-summary`, and `tv screenshot --region strategy`. The goal now is to stop feature work and confirm that contracts, docs, help, runtime skills, tests, source boundaries, and module responsibilities are ready for release prep.

Small docs, help, naming, metadata, or test drift can be fixed in this audit. Larger refactors should be reported here and handled by a dedicated follow-up plan.

## Progress

- [x] (2026-06-11) Create this pre-release architecture audit plan and move the completed Strategy Tester screenshot plan to archives.
- [x] (2026-06-11) Update `docs/plans/README.md`, `docs/v0.25-roadmap.md`, and `CHANGELOG.md` for the audit slice.
- [x] (2026-06-11) Inspect v0.25 source-boundary docs and runtime skill references for chart compare, events compare, Replay OHLCV attachment, and Strategy Tester screenshot evidence.
- [x] (2026-06-11) Inspect v0.25 implementation modules for release-blocking architecture concerns.
- [x] (2026-06-11) Run docs hygiene and packaging-script validation.
- [x] (2026-06-11) Run Rust baseline and focused contract tests.
- [x] (2026-06-11) Validate updated runtime skills.

## Surprises & Discoveries

- Observation: `crates/cli/src/ops/market/chart_compare.rs` is currently a focused selected-chart compare adapter rather than a general compare engine.
  Evidence: the module is about 344 lines and keeps chart switching, quote readback, restore status, and payload shaping within a single Desktop-backed workflow boundary.

- Observation: `crates/market/src/events.rs` and `crates/cli/src/app/replay_log.rs` are medium-sized but still cohesive for this release.
  Evidence: events code still centers on scanner-backed earnings / dividends field shaping, while Replay log code owns the bounded runner and one explicit OHLCV summary attachment. Neither requires a release-blocking split before `v0.25.0`.

- Observation: `crates/cli/src/ops/screenshot.rs` remains acceptable as a shared screenshot adapter.
  Evidence: Strategy Tester clipping is implemented as a visual evidence region and does not mix with structured `tv data strategy`, `tv data trades`, or `tv data equity` reads.

- Observation: hygiene scans still match existing policy text, archived validation commands, test fixture strings, and live-smoke assertions.
  Evidence: no newly introduced v0.25 public docs or payload docs contain raw target ids, raw payloads, credentials, account-local metadata, or local absolute paths.

## Decision Log

- Decision: Treat `tv chart compare` as complete for `v0.25.0` and keep it separate from scanner-backed `tv compare` / `tv watch compare`.
  Rationale: the command is explicitly Desktop-backed, state-changing, and source-labeled with `chart_compare.v1`; there is no hidden fallback to Desktop-free compare, bars, Replay, chart export, or quote-data.
  Date/Author: 2026-06-11 / Codex

- Decision: Treat `tv events compare` as complete for `v0.25.0` while keeping full event calendar work deferred.
  Rationale: `events_compare.v1` is a multi-symbol view over scanner fundamentals earnings / dividends evidence. It does not infer calendar semantics, ranking, recommendations, or trading judgment.
  Date/Author: 2026-06-11 / Codex

- Decision: Treat `tv replay log --attach-ohlcv-summary` as complete for `v0.25.0` and do not add screenshot attachment in this release.
  Rationale: the OHLCV summary attachment is opt-in and separately labeled. Screenshot attachment still needs file naming and artifact lifecycle semantics.
  Date/Author: 2026-06-11 / Codex

- Decision: Treat `tv screenshot --region strategy` as complete for `v0.25.0`.
  Rationale: it is a non-mutating visual evidence read for the visible Strategy Tester panel, not a structured strategy metric extractor or replacement for `tv data ...`.
  Date/Author: 2026-06-11 / Codex

- Decision: No larger refactor is required before `v0.25.0` release readiness.
  Rationale: the inspected modules have clear enough responsibilities for the current release. Future splits may become worthwhile if events expands beyond earnings / dividends, Replay attachments multiply, or screenshot regions gain artifact-management behavior.
  Date/Author: 2026-06-11 / Codex

## Outcomes & Retrospective

The audit found no release-blocking architecture issue and no larger refactor requirement before `v0.25.0` release readiness.

Small docs state drift was addressed by moving the current plan to this audit, marking Strategy Tester screenshot evidence complete for `v0.25.0`, and recording the audit in the changelog.

The next step is `v0.25.0 release readiness`: freeze features, bump the version, cut the changelog section, add release notes, validate the release package, and stop before tag / push / GitHub Release creation.

## Context and Orientation

The `v0.25.0` theme is chart-backed workflow maturity.

Completed release lanes:

- Lane 1: chart-backed compare command is complete for `v0.25.0`.
- Lane 2: `tv events` multi-symbol readback is complete for `v0.25.0`.
- Lane 3: Replay OHLCV summary attachment is complete for `v0.25.0`.
- Lane 4: Strategy Tester screenshot evidence is complete for `v0.25.0`.

Deferred after `v0.25.0`:

- remaining intraday bars ranges;
- richer strategy-report evidence;
- Screener columns reset / cleanup workflow;
- Pine new-script save workflow;
- full event calendar;
- Replay screenshot attachment;
- automatic source mixing;
- daemon, dashboard, MCP server, binary split;
- ranking, scoring, recommendation, or buy / sell judgment.

## Plan of Work

First, confirm current docs and packaged agent guidance describe v0.25 surfaces as source diagnostics and workflow evidence, not ranking or recommendations.

Second, inspect the implementation modules that changed in v0.25:

- `crates/cli/src/ops/market/chart_compare.rs`
- `crates/market/src/events.rs`
- CLI events adapter code
- `crates/cli/src/app/replay_log.rs`
- `crates/cli/src/ops/screenshot.rs`

Third, run docs hygiene, architecture inspection, Rust baseline tests, focused contract tests, and runtime skill validation.

Fourth, record whether the audit found no blocker, applied small fixes, or identified a larger refactor plan needed before release readiness.

## Validation and Acceptance

This audit is acceptable when:

- docs and skills do not imply source mixing, ranking, recommendation, trade advice, or automatic fallback;
- `tv chart compare` remains separated from Desktop-free `tv compare` / `tv watch compare`;
- `tv events compare` remains scanner fundamentals evidence rather than a full calendar;
- `tv replay log --attach-ohlcv-summary` remains an explicit selected-chart attachment;
- `tv screenshot --region strategy` remains visual evidence and not structured strategy data extraction;
- public docs and packaged assets do not contain raw live outputs, raw bars, raw JSONL, raw target ids, credentials, account-local metadata, or local absolute paths;
- architecture inspection states either no release-blocking issue, small fixes applied, or a larger refactor recommended before release readiness;
- the validation commands listed in this plan pass or any failures are explained.

Validation performed:

- `git diff --check`
- `bash -n scripts/stage-release-package-files.sh`
- tracked-doc and code hygiene scans for private / raw data markers
- TODO / panic / unimplemented scan
- v0.25 source-boundary reference scan
- architecture line-count and symbol-reference inspection
- `cargo fmt --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace`
- `cargo metadata --no-deps --format-version 1`
- focused chart / events / replay / screenshot / desktop / quote contract tests
- runtime skill validation for chart analysis, market data interpretation, multi-symbol scan, replay practice, and strategy report

## Artifacts and Notes

Do not paste optional live smoke raw output, screenshots, raw JSONL, raw bars, raw DOM, raw payloads, target ids, account-local metadata, credentials, or local absolute paths into tracked docs.

Optional live smoke evidence, if run, should be summarized only with command, contract marker, source category, count / status, and file-existence level information.

## Interfaces and Dependencies

This audit adds no command, option, payload semantics, dependency, or version bump.

It updates only release planning docs and the changelog entry for audit completion.

## Open Questions

None are release-blocking for `v0.25.0`.

Future refactor candidates if adjacent lanes expand:

- split `crates/market/src/events.rs` if a full calendar source or additional event types are added;
- split Replay log attachment helpers if screenshot or multi-attachment support is added;
- split screenshot region adapters if additional evidence regions require artifact lifecycle management;
- keep chart-backed compare separate from scanner-backed compare unless a future plan defines an explicit source-separated integration.
