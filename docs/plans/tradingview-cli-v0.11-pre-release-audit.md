# v0.11 pre-release completion and refactor audit

This ExecPlan is a living document. Keep `Progress`, `Surprises & Discoveries`,
`Decision Log`, and `Outcomes & Retrospective` current as work proceeds.

This document follows `.agents/PLANS.md` from the repository root. It records
the final audit before `v0.11.0` release readiness.

## Purpose / Big Picture

`v0.11.0` adds downstream-safe `tv compare` contract metadata. Successful
compare payloads now include a command-local contract marker, stable requested
indexes, per-item follow-up hints, and field coverage readback while preserving
raw per-symbol `items[]` as the evidence source.

Before release readiness, stop feature work and confirm that this is enough for
`v0.11.0`, that the metadata is still additive, and that no small
release-blocking refactor or documentation mismatch remains. After this slice,
the next step should be `v0.11.0` release readiness unless a release blocker is
found.

## Progress

- [x] (2026-05-08T04:39Z) Archived the completed compare contract metadata
  plan and created this pre-release audit plan.
- [x] (2026-05-08T04:39Z) Audit compare metadata construction, typed result shape, CLI contract
  tests, live compare smoke, runtime skills, and stable docs.
- [x] (2026-05-08T04:39Z) Run focused compare tests, the full Rust baseline, docs validation,
  packaging script syntax check, grep audits, and optional read-only compare
  smoke.
- [x] (2026-05-08T04:39Z) Record whether any v0.11 release blocker or immediate refactor need was
  found.

## Surprises & Discoveries

- The `compare` metadata implementation remains localized in
  `tradingview-market`: `summary.field_coverage`, requested indexes, and
  follow-up hints are derived from finalized compare items and do not introduce
  additional network reads.
- The TODO / panic audit found only expected assertion-style `panic!` calls in
  ignored live smoke tests, one Pine template TODO string, archived validation
  examples, and this plan's validation command. No release-blocking TODO,
  FIXME, `unimplemented!`, or `todo!` marker was found.
- The hygiene grep reported existing safety policy wording, archived
  validation-command examples, and this plan's safety wording. No new
  machine-specific path, credential, raw target id, account-local metadata, or
  raw live payload was added.

## Decision Log

- Decision: Treat this slice as audit-only unless a release blocker is found.
  Rationale: The v0.11 roadmap goal is already implemented through additive
  `compare` metadata. New feature work or broad refactoring would add release
  risk and should wait for a separate plan.
  Date/Author: 2026-05-08 / Codex.

- Decision: Keep the contract marker command-local.
  Rationale: `contract_version: "compare.v1"` guards only the `compare`
  payload. A global envelope or cross-command schema version would require a
  wider compatibility policy that is not part of this release audit.
  Date/Author: 2026-05-08 / Codex.

- Decision: Treat `v0.11.0` as complete after compare contract metadata polish.
  Rationale: The roadmap goal was downstream-safe readback, not a new data
  source. The current implementation, tests, docs, and runtime skills satisfy
  that goal without broadening `compare`.
  Date/Author: 2026-05-08 / Codex.

- Decision: Do not refactor compare internals before `v0.11.0`.
  Rationale: Metadata construction is small, localized, covered by focused
  tests, and still additive. A behavior-preserving refactor would add release
  risk without addressing a blocker.
  Date/Author: 2026-05-08 / Codex.

## Outcomes & Retrospective

Audit-only slice completed. `tv compare` contract metadata remains additive:
successful payloads include `contract_version: "compare.v1"`, ordered
`requested_index` values, per-item `follow_up_hints[]`, and
`summary.field_coverage`, while the existing raw `items[]`, section-level
errors, top-level counts, source metadata, `summary.resolved_symbols[]`,
`errors[]`, and `next_action_hints` remain in place.

No Rust code was changed. No release blocker or immediate refactor need was
found. The next step is `v0.11.0 release readiness`.

## Context and Orientation

The relevant v0.11 surface is `tv compare <SYMBOL>...`, a Desktop-free command
that gathers scanner-backed quote, symbol-info, and fundamentals evidence for
multiple symbols. Desktop-free means the command does not require TradingView
Desktop, Chrome DevTools Protocol, chart switching, or screenshot capture.

The implementation lives primarily in `crates/market/src/compare.rs`, and the
typed payload structs live in `crates/market/src/types.rs`. The CLI serializes
the typed compare result into the shared JSON envelope without adding chart
reads or Desktop fallback.

The compare payload must keep these existing contract pieces intact:
top-level counts, source metadata, `summary`, ordered `items[]`, section-level
errors, top-level `errors[]`, and `next_action_hints`. The v0.11 metadata is
additive: `contract_version`, `requested_index`, `follow_up_hints`, and
`summary.field_coverage` help downstream wrappers read the payload safely, but
they are not ranking, scoring, recommendations, or trading advice.

## Plan of Work

Inspect compare implementation, tests, docs, and runtime skills for contract
drift. Confirm that `data.contract_version` is `"compare.v1"`, that
`requested_index` values are zero-based and preserve validated input order,
that per-item `follow_up_hints[]` are machine-readable next evidence surfaces,
and that `summary.field_coverage` is derived from already finalized items
without extra network reads.

Confirm that docs and skills describe the metadata as readback or schema guard
helpers. They should still direct users to inspect raw `items[]` for evidence
and should not imply ranking, scoring, recommendation, realtime multi-symbol
feed behavior, or chart-backed comparison.

Update `docs/v0.11-roadmap.md` and `docs/plans/README.md` so this audit is the
current plan. Keep `CHANGELOG.md` unchanged unless the audit uncovers
user-facing docs or implementation changes that need release-note coverage.

## Validation and Acceptance

Run these commands from the repository root:

    git diff --check
    bash -n scripts/stage-release-package-files.sh
    rg -n '(/Users/|C:\\|USER;|sessionid|cookie|authorization|bearer|raw live payload|account-local|target id)' README.md AGENTS.md CLAUDE.md CHANGELOG.md docs .agents/skills packaging scripts || true
    rg -n "TODO|FIXME|panic!|unimplemented!|todo!" crates docs README.md AGENTS.md CLAUDE.md packaging/agent/AGENTS.md
    rg -n "contract_version|requested_index|follow_up_hints|field_coverage|compare|ranking|recommendation|realtime|diagnose|binary split|MCP|daemon" README.md CHANGELOG.md docs .agents/skills packaging/agent/AGENTS.md
    cargo test -p tradingview-market compare -- --nocapture
    cargo test -p tradingview-cli --test cli_contract compare -- --nocapture
    cargo test -p tradingview-cli --test live_compare
    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1
    target/debug/tv compare --help

Optional read-only smoke:

    target/debug/tv compare NASDAQ:AAPL NYSE:IONQ

Do not paste raw live output into tracked docs. Record only whether the smoke
passed and the public-safe summary of what it proved.

Completed validation:

    git diff --check
    bash -n scripts/stage-release-package-files.sh
    rg -n '(/Users/|C:\\|USER;|sessionid|cookie|authorization|bearer|raw live payload|account-local|target id)' README.md AGENTS.md CLAUDE.md CHANGELOG.md docs .agents/skills packaging scripts || true
    rg -n "TODO|FIXME|panic!|unimplemented!|todo!" crates docs README.md AGENTS.md CLAUDE.md packaging/agent/AGENTS.md
    rg -n "contract_version|requested_index|follow_up_hints|field_coverage|compare|ranking|recommendation|realtime|diagnose|binary split|MCP|daemon" README.md CHANGELOG.md docs .agents/skills packaging/agent/AGENTS.md
    cargo test -p tradingview-market compare -- --nocapture
    cargo test -p tradingview-cli --test cli_contract compare -- --nocapture
    cargo test -p tradingview-cli --test live_compare
    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1
    target/debug/tv compare --help

Optional read-only smoke also passed for a public two-symbol compare. It
confirmed `success: true`, `command: "compare"`, `contract_version:
"compare.v1"`, two requested and resolved items, zero errors, array-shaped
`items`, `summary.field_coverage`, and first `requested_index` equal to zero.

Acceptance is met when no release blocker is found, no new feature surface is
introduced, and the next step can move to `v0.11.0` release readiness.

## Idempotence and Recovery

This slice is safe to repeat. The archive move should be idempotent after the
first run: if the completed plan is already in `docs/plans/archives/`, leave it
there.

If validation finds a real contract bug, fix only that bug in a small focused
change or create a separate patch plan. Do not mix broad refactoring into
release readiness.

## Interfaces and Dependencies

No CLI behavior, JSON payload, Rust API, dependency, release package behavior,
or version changes are introduced by this audit.

`tv compare` should remain a Desktop-free read. Ranking, scoring,
recommendation, chart-backed compare, watch or JSONL compare, realtime
multi-symbol feed, stable browserless bars, standalone `tv events`,
`tv diagnose`, alternate binaries, MCP server work, daemon behavior, and a
global envelope schema version remain deferred unless a later plan changes the
boundary.

## Open Questions

None for `v0.11.0`. Deferred ideas should be handled after release readiness or
in the next roadmap.
