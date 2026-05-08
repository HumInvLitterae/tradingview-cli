# v0.12 pre-release completion and refactor audit

This ExecPlan is a living document. Keep `Progress`, `Surprises & Discoveries`,
`Decision Log`, and `Outcomes & Retrospective` current as work proceeds.

This document follows `.agents/PLANS.md` from the repository root. It is
self-contained and describes the audit before `v0.12.0` release readiness. The
goal is to confirm that the `v0.12.0` compare metadata work is complete and
does not need a release-blocking refactor.

## Purpose / Big Picture

`v0.12.0` focused on making `tv compare <SYMBOL>...` safer for downstream
agents to consume. The release candidate now has coverage status readback,
stable follow-up contract semantics, and item-level missing evidence routing.

This audit stops feature work and checks whether the implementation, tests,
docs, and runtime skills agree. If no blocker is found, the next step is
`v0.12.0` release readiness.

## Progress

- [x] (2026-05-08T08:36Z) Created this pre-release audit plan, archived the
  completed compare missing-evidence plan, and updated current-plan pointers.
- [x] (2026-05-08T08:45Z) Audited compare metadata contracts against code,
  tests, docs, and skills.
- [x] (2026-05-08T08:52Z) Ran focused compare contract tests and full
  workspace validation.
- [x] (2026-05-08T08:55Z) Ran docs, packaging, and public-safety hygiene
  checks.
- [x] (2026-05-08T08:58Z) Recorded v0.12 lanes as complete for release or
  deferred after release.
- [ ] Commit the completed audit as one local commit.

## Surprises & Discoveries

- Observation: no release-blocking refactor was needed.
  Evidence: the compare metadata fields remain additive and derived from
  finalized section results. Contract tests cover the success path, partial
  coverage, total-failure details, requested-order indexes, follow-up hint
  kinds, field coverage, coverage status, and item-level missing evidence.

- Observation: the `TODO` / `panic!` audit did not identify a new release
  blocker.
  Evidence: matches were limited to assertion-style panics in ignored live
  smoke tests, a Pine template TODO string, archived validation examples, and
  this audit plan's validation command.

- Observation: the public-safety hygiene grep reported existing policy text,
  archived validation-command examples, and this plan's safety wording.
  Evidence: no new machine-specific path, credential, raw target id,
  account-local metadata, or raw live payload was added by this audit slice.

## Decision Log

- Decision: Treat this slice as audit-only unless validation finds a concrete
  release blocker.
  Rationale: `summary.coverage_status` and `items[].missing_evidence[]` are
  already implemented, tested, documented, and committed. Adding new metadata
  now would blur the boundary before release readiness.
  Date/Author: 2026-05-08 / Codex.

## Outcomes & Retrospective

Completed. The v0.12 compare follow-up contract and missing-evidence work is
complete enough for `v0.12.0` release readiness.

Complete for `v0.12.0`:

- `summary.coverage_status` as evidence coverage readback;
- stable shipped `follow_up_hints[].kind` values;
- documented `summary.field_coverage` semantics;
- `requested_index` as the requested-order join authority;
- `items[].missing_evidence[]` as machine-readable evidence routing metadata;
- total-failure compare details retaining `contract_version: "compare.v1"`.

Deferred after `v0.12.0`:

- cross-command follow-up vocabulary alignment;
- snapshot-side coverage metadata;
- chart-backed compare;
- watch or JSONL compare;
- realtime multi-symbol feed;
- stable browserless bars;
- standalone `tv events`;
- `tv diagnose`;
- binary split;
- MCP server work.

The next step is `v0.12.0` release readiness.

## Context and Orientation

The repository is a Cargo workspace for the Rust-native `tv` CLI. The
Desktop-free compare implementation lives in `crates/market/src/compare.rs`,
and its typed JSON payload structs live in `crates/market/src/types.rs`.
Contract tests live under `crates/cli/tests/`, and stable workflow docs live
under `docs/`.

The v0.12 compare metadata contract currently includes:

- `data.contract_version: "compare.v1"`;
- `items[].requested_index` and
  `summary.resolved_symbols[].requested_index`;
- `items[].follow_up_hints[]`;
- `summary.field_coverage`;
- `summary.coverage_status`;
- `items[].missing_evidence[]`;
- structured total-failure details that retain the compare payload.

All of those fields are readback helpers. They route evidence and make joins
deterministic, but they do not rank, score, recommend, or infer trading action.

## Plan of Work

First inspect the compare code and live contract smoke to confirm the shipped
payload still preserves existing fields: `items[]`, `sections`, `errors[]`,
`missing_summary`, `summary.resolved_symbols[]`, `next_action_hints`, source
metadata, and top-level counts. Confirm the new fields are additive and derived
from existing section results, not from new network reads.

Then inspect stable docs and runtime skills to confirm they describe
`coverage_status` and `missing_evidence` as evidence readback and routing
metadata, not as recommendations or rankings.

Run the focused compare tests and full Rust baseline. Also run docs and
packaging checks, plus a public-safety grep for local paths, credentials, raw
target ids, account-local metadata, and raw live payloads. Existing safety
policy text and archived validation examples are acceptable; newly introduced
private data is not.

If validation finds no blocker, update this plan and `docs/v0.12-roadmap.md`
to record that `v0.12.0` is complete enough for release readiness. If a small
release-blocking issue is found, fix only that issue, record it in the Decision
Log, and rerun validation.

## Concrete Steps

From the repository root, run:

    rg -n "coverage_status|missing_evidence|follow_up_hints|field_coverage|requested_index|contract_version" crates/market crates/cli/tests docs .agents/skills

Then run the validation commands in the next section. Keep this plan current
as each result is known.

## Validation and Acceptance

Run:

    git diff --check
    bash -n scripts/stage-release-package-files.sh
    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1
    cargo test -p tradingview-market compare -- --nocapture
    cargo test -p tradingview-cli --test cli_contract compare -- --nocapture
    cargo test -p tradingview-cli --test live_compare
    target/debug/tv compare --help

Also run:

    rg -n '(/Users/|C:\\|USER;|sessionid|cookie|authorization|bearer|raw live payload|account-local|target id)' README.md AGENTS.md CLAUDE.md CHANGELOG.md docs .agents/skills packaging scripts || true
    rg -n "TODO|FIXME|panic!|unimplemented!|todo!" crates docs README.md AGENTS.md CLAUDE.md packaging/agent/AGENTS.md || true
    rg -n "coverage_status|missing_evidence|follow_up_hints|field_coverage|requested_index|compare|ranking|recommendation|realtime|diagnose|binary split|MCP|daemon" README.md CHANGELOG.md docs .agents/skills packaging/agent/AGENTS.md

Acceptance requires that all validation commands pass or only report known,
non-blocking policy text. The audit must record whether the next step is
`v0.12.0` release readiness.

## Idempotence and Recovery

The audit is safe to rerun. If a command writes only build artifacts under
`target/`, no cleanup is required. Do not change CLI behavior, add options,
add dependencies, or bump versions in this slice.

If validation discovers a release blocker, make the smallest possible fix,
rerun the relevant focused tests and baseline, and record the reason in this
plan. If the issue is not release-blocking, defer it after `v0.12.0`.

## Artifacts and Notes

Validation passed:

- `git diff --check`;
- `bash -n scripts/stage-release-package-files.sh`;
- `cargo fmt --check`;
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`;
- `cargo test --workspace`;
- `cargo metadata --no-deps --format-version 1`;
- `cargo test -p tradingview-market compare -- --nocapture`;
- `cargo test -p tradingview-cli --test cli_contract compare -- --nocapture`;
- `cargo test -p tradingview-cli --test live_compare`;
- `target/debug/tv compare --help`.

The broad docs and skill searches confirmed that `compare` metadata is
documented as readback / evidence routing, not ranking, scoring,
recommendation, or trading action.

## Interfaces and Dependencies

No new interface or dependency should be added in this audit-only slice.

## Open Questions

None. If validation finds no release blocker, move to `v0.12.0` release
readiness.
