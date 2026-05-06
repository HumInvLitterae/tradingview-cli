# v0.7 pre-release completion and refactor audit

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document follows `.agents/PLANS.md` from the repository root. It is self-contained and records the final audit before `v0.7.0` release readiness.

## Purpose / Big Picture

`v0.7.0` added an observation workflow lane around `tv observe chart`, opt-in live smoke evidence for `observe chart` and lab-gated `tv bars`, an observation workflow guide, and small scanner-backed fundamentals field enrichment.

Before release readiness, this slice stops feature work and asks two questions:

1. Is any `v0.7.0` roadmap work still required before release prep?
2. Is there any small refactor or release-blocking structure issue that should be fixed now?

The expected outcome is a clear decision to proceed to `v0.7.0` release readiness, or a narrowly scoped blocker fix if validation finds one.

## Progress

- [x] (2026-05-06T00:00Z) Created this ExecPlan and archived the completed fundamentals event field enrichment plan.
- [x] (2026-05-06T00:00Z) Reviewed current `v0.7.0` roadmap state, active plans index, and changelog entries.
- [x] (2026-05-06T00:00Z) Audited observe/stream runner shape, fundamentals field selection, scanner column allowlist, TODO/FIXME/panic markers, and large Rust files.
- [x] (2026-05-06T00:00Z) Classified remaining roadmap work as release-ready or deferred after `v0.7.0`.
- [x] (2026-05-06T00:00Z) Ran validation.
- [x] (2026-05-06T00:00Z) Committed the audit.

## Surprises & Discoveries

- `tv observe chart` and `tv stream ...` have similar bounded JSONL loops. That duplication is intentional for this release because extracting a shared runner would add behavior risk immediately before release readiness.
- The remaining `panic!` occurrences are opt-in live smoke assertion failures, and the remaining `TODO` string is a Pine template placeholder, not an unfinished implementation marker.
- The largest Rust files are older operation adapters and scanner implementation files. None are new `v0.7.0` release blockers.

## Decision Log

- Decision: Proceed to `v0.7.0` release readiness after this audit if validation passes.
  Rationale: `tv observe chart`, observation smoke evidence, lab bars smoke evidence, observation workflow docs, and fundamentals field enrichment are complete enough for `v0.7.0`; remaining lanes are intentionally deferred.
  Date/Author: 2026-05-06 / Codex.

- Decision: Do not refactor the duplicated observe/stream bounded loop before release.
  Rationale: The duplication is small and behavior-sensitive. A shared helper can be considered after `v0.7.0` if observe surfaces broaden.
  Date/Author: 2026-05-06 / Codex.

- Decision: Keep `tv diagnose`, stable browserless bars, browserless streaming, binary split, MCP server, daemon behavior, and Computer Use-specific skills deferred.
  Rationale: Current downstream value is better served by releasing the observation workflow foundation and evidence tooling already implemented.
  Date/Author: 2026-05-06 / Codex.

## Outcomes & Retrospective

Audit found no release-blocking refactor or missing `v0.7.0` implementation. The next slice should be `v0.7.0` release readiness: version bump, changelog cut, release notes, README/package examples, archive staging, and final validation.

## Context and Orientation

The current `v0.7.0` roadmap centers on agent-ready observation workflows:

- `tv observe chart` is the workflow-level Desktop-backed JSONL observation command.
- `tv stream ...` remains the lower-level Desktop-backed JSONL stream surface.
- `tv bars` remains lab-gated and browserless, with opt-in smoke evidence but no stable promise.
- `tv fundamentals` remains scanner-backed and Desktop-free, with evidence-backed earnings and dividend field bundles.
- `docs/observation-workflows.md` is the current guide for choosing between these reads.

## Plan of Work

Update durable docs so the active plan is this audit and the completed fundamentals enrichment plan is archived.

Audit the following without adding new feature surface:

- observe runner and stream bounded loop;
- `tv observe chart` tests and live smoke boundary;
- fundamentals field selection and scanner scan allowlist;
- large files, unfinished markers, and release-blocking warnings;
- roadmap, changelog, and workflow docs for mismatches with implementation.

Classify roadmap lanes:

- Pull-based observation workflows: complete for `v0.7.0`; future observe modes deferred.
- Desktop-backed diagnostics: `readiness` remains sufficient; `diagnose` deferred.
- Browserless bars: opt-in smoke evidence complete; stabilization deferred.
- Fundamentals/events: field enrichment complete; standalone `tv events` deferred.
- Workflow docs/skills: observation workflow guide complete enough for release.

## Concrete Steps

Run audit and validation commands:

    rg -n "TODO|FIXME|panic!|unimplemented!|todo!" crates docs README.md AGENTS.md CLAUDE.md packaging/agent/AGENTS.md
    find crates -path '*/src/*.rs' -o -path '*/src/**/*.rs' | xargs wc -l | sort -nr | head -40
    rg -n "observe chart|tv bars|fundamentals|diagnose|binary split|MCP|daemon|Computer Use" README.md CHANGELOG.md docs .agents/skills packaging/agent/AGENTS.md

Run baseline validation:

    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1
    git diff --check
    bash -n scripts/stage-release-package-files.sh

Run focused contract confirmation:

    cargo test -p tradingview-cli observe -- --nocapture
    cargo test -p tradingview-cli stream -- --nocapture
    cargo test -p tradingview-cli market::bars -- --nocapture
    cargo test -p tradingview-market fundamental -- --nocapture
    cargo test -p tradingview-scanner scan -- --nocapture
    cargo test -p tradingview-cli --test cli_contract observe -- --nocapture
    cargo test -p tradingview-cli --test cli_contract bars -- --nocapture
    cargo test -p tradingview-cli --test cli_contract fundamentals -- --nocapture

Run hygiene check:

    rg -n '(/Users/|C:\\|USER;|sessionid|cookie|authorization|bearer|raw live payload|account-local)' README.md CHANGELOG.md docs .agents/skills packaging scripts || true

## Validation and Acceptance

Acceptance is met when:

- validation passes;
- no release-blocking TODO/FIXME/panic/unimplemented marker is found;
- `v0.7.0` roadmap lanes are classified as complete or deferred;
- no new command, option, payload change, dependency, or large refactor is introduced;
- next step is clearly `v0.7.0` release readiness.

## Idempotence and Recovery

This audit is docs-only unless validation reveals a blocker. If a blocker is found, do not mix a broad refactor into this slice. Either make a minimal fix with focused validation or create a new ExecPlan for the blocker.

## Interfaces and Dependencies

No public interface changes. No dependency changes. No release version bump in this slice.

## Open Questions

None. Release readiness is the next step if this audit passes.
