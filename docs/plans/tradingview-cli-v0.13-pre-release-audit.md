# v0.13 pre-release completion and refactor audit

This ExecPlan is a living document. Keep `Progress`, `Surprises & Discoveries`,
`Decision Log`, and `Outcomes & Retrospective` current as work proceeds.

This document follows `.agents/PLANS.md` from the repository root. It is
self-contained and records the audit that must happen before `v0.13.0` release
readiness begins.

## Purpose / Big Picture

`v0.13.0` focused on helping agents keep TradingView source and session
boundaries straight. The release should not move forward if chart-source quote,
Desktop quote-session evidence tooling, snapshot metadata, or follow-up
vocabulary docs contradict the implementation. This audit checks those
contracts, records which roadmap lanes are complete for `v0.13.0`, and keeps
postmarket and premarket evidence collection explicitly deferred until the
matching market phase is available.

After this audit, the next contributor should be able to either proceed to
`v0.13.0` release readiness or see the exact blocker that must be fixed first.

## Progress

- [x] (2026-05-09T00:00Z) Created this pre-release audit ExecPlan and archived
  the completed follow-up vocabulary alignment plan.
- [x] (2026-05-09T00:00Z) Audited v0.13 source/session boundary contracts,
  roadmap lanes, and docs.
- [x] (2026-05-09T00:00Z) Ran focused contract tests, baseline validation,
  docs hygiene checks, and packaging script syntax check.
- [x] (2026-05-09T00:00Z) Recorded audit outcomes. The next step is
  `v0.13.0` release readiness.

## Surprises & Discoveries

- Observation: The public-safety hygiene scan reported existing policy text,
  archived validation-command examples, explicit placeholder examples, and
  code examples that intentionally contain fake Windows or `/Users/example`
  paths.
  Evidence: the scan did not identify a newly introduced raw live payload,
  target id, account-local metadata, credential, or downstream-private path in
  the changed v0.13 audit docs.

- Observation: The TODO / panic audit did not identify a new release blocker.
  Evidence: remaining hits are the known assertion-style `panic!` calls in
  ignored live smoke tests, the Pine template TODO string, archived validation
  examples, and this plan's validation command.

- Observation: The vocabulary scan still finds `quote_chart`, but only in
  contexts that explicitly say the alias is not shipped or should not be used.
  Evidence: current docs and skills keep `chart_quote` as the canonical
  follow-up value.

## Decision Log

- Decision: Do not run the ignored phase-specific Desktop quote-session live
  smoke in this audit.
  Rationale: Postmarket and premarket evidence only answers the research
  question during the matching market phase. Running it during regular session
  would produce a timing guard result, not release-blocking evidence.
  Date/Author: 2026-05-09 / Codex.

- Decision: Treat this audit as release preparation gating, not a feature
  slice.
  Rationale: The v0.13 feature and contract-hardening slices are already
  complete. Adding new payload fields, options, sources, or refactors here
  would mix release gating with new work.
  Date/Author: 2026-05-09 / Codex.

## Outcomes & Retrospective

Audit complete. No release blocker was found. `v0.13.0` is ready to move to
release readiness with these lanes classified as complete for the release:

- chart-source quote `session_boundary` metadata and scanner extended-hours
  separation;
- Desktop quote-session ignored live smoke and phase timing guard;
- `snapshot.v1` contract metadata, coverage summary, missing-evidence
  readback, and follow-up hints;
- stable `compare` / `snapshot` follow-up vocabulary with `chart_quote` as the
  canonical selected-chart quote kind.

The following remain deferred or phase-waiting after `v0.13.0`: postmarket and
premarket public payload support, cross-source automatic mixing, chart-backed
compare, watch/JSONL compare, realtime multi-symbol feed, stable browserless
bars, standalone `tv events`, `tv diagnose`, binary split, MCP server, daemon
behavior, and ranking, scoring, or recommendation features.

Validation passed:

    git diff --check
    bash -n scripts/stage-release-package-files.sh
    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1
    cargo test -p tradingview-cli market::quote -- --nocapture
    cargo test -p tradingview-cli --test cli_contract quote -- --nocapture
    cargo test -p tradingview-market quote -- --nocapture
    cargo test -p tradingview-market snapshot -- --nocapture
    cargo test -p tradingview-market compare -- --nocapture
    cargo test -p tradingview-cli --test cli_contract snapshot -- --nocapture
    cargo test -p tradingview-cli --test live_quote_session_extended_hours
    cargo test -p tradingview-cli --test live_snapshot
    cargo test -p tradingview-cli --test live_compare

The ignored phase-specific quote-session smoke was intentionally not run with
`TV_LIVE_QUOTE_SESSION_SMOKE=1`.

## Context and Orientation

The `tv` CLI is a Rust workspace. Desktop-free market reads live mainly under
`crates/market/`, while Desktop-backed chart reads live under `crates/cli/src/`
and use TradingView Desktop through Chrome DevTools Protocol. A source boundary
is the line between two ways of obtaining data, such as scanner REST versus the
selected Desktop chart. A session boundary is the line between regular-session
and extended-hours values such as premarket or postmarket.

The relevant v0.13 work is:

- chart-source quote reports `session_boundary` metadata saying the selected
  chart main-series last bar does not provide scanner-style extended-hours
  values;
- `live_quote_session_extended_hours` is an ignored test for postmarket and
  premarket evidence, but it is active only when the relevant market phase is
  available;
- `tv snapshot <SYMBOL>` now exposes additive `snapshot.v1` metadata,
  coverage summary, missing-evidence readback, and follow-up hints while
  preserving raw `sections`;
- `tv compare` and `tv snapshot` share the stable follow-up vocabulary
  `snapshot`, `chart_quote`, `observe_chart`, and `screenshot`.

The active waiting plan is
`docs/plans/tradingview-cli-desktop-quote-session-live-evidence.md`. It must
remain unarchived until postmarket or premarket evidence is collected in the
matching phase.

## Plan of Work

Move the completed follow-up vocabulary plan into `docs/plans/archives/`.
Create this audit plan as the current ExecPlan. Update `docs/plans/README.md`
and `docs/v0.13-roadmap.md` so the current slice is the pre-release audit while
the Desktop quote-session evidence plan remains active but blocked on timing.

Review the implemented contracts and docs for obvious mismatches. Confirm that
`chart_quote` remains the canonical follow-up kind and that `quote_chart` is
not introduced as an alias. Confirm that docs do not imply scanner
extended-hours, chart main-series quote, and Desktop quote-session fields are
the same source.

Run the validation commands listed below. If a release blocker appears, fix
only the smallest docs or test mismatch needed for v0.13 correctness and record
the decision. Do not introduce new command behavior, payload fields, source
mixing, or phase-specific live evidence in this audit.

## Concrete Steps

From the repository root, run:

    git diff --check
    bash -n scripts/stage-release-package-files.sh
    rg -n '(/Users/|C:\\|USER;|sessionid|cookie|authorization|bearer|raw live payload|account-local|target id|downstream-private)' README.md AGENTS.md CLAUDE.md CHANGELOG.md docs .agents/skills packaging scripts crates || true
    rg -n "TODO|FIXME|panic!|unimplemented!|todo!" crates docs README.md AGENTS.md CLAUDE.md packaging/agent/AGENTS.md
    rg -n "session_boundary|extended_hours|premarket|postmarket|quote --source chart|quote_session|snapshot.v1|missing_evidence|follow_up_hints|chart_quote|quote_chart|ranking|recommendation|realtime|diagnose|binary split|MCP|daemon" README.md CHANGELOG.md docs .agents/skills packaging/agent/AGENTS.md

Then run:

    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1

Finally run the focused contract confirmations:

    cargo test -p tradingview-cli market::quote -- --nocapture
    cargo test -p tradingview-cli --test cli_contract quote -- --nocapture
    cargo test -p tradingview-market quote -- --nocapture
    cargo test -p tradingview-market snapshot -- --nocapture
    cargo test -p tradingview-market compare -- --nocapture
    cargo test -p tradingview-cli --test cli_contract snapshot -- --nocapture
    cargo test -p tradingview-cli --test live_quote_session_extended_hours
    cargo test -p tradingview-cli --test live_snapshot
    cargo test -p tradingview-cli --test live_compare

Do not run commands with `TV_LIVE_QUOTE_SESSION_SMOKE=1` in this audit. Those
commands are for actual postmarket or premarket evidence only.

## Validation and Acceptance

Acceptance is met when all required commands pass, the hygiene searches do not
show newly introduced secrets or local-only information, and the audit records
that v0.13 can proceed to release readiness. The roadmap must classify these as
complete for `v0.13.0`: chart-source quote session-boundary metadata, quote
session live smoke and timing guard, snapshot contract metadata, and follow-up
vocabulary alignment.

The roadmap must keep these deferred or phase-waiting: postmarket and premarket
payload support, cross-source automatic mixing, chart-backed compare,
watch/JSONL compare, realtime multi-symbol feed, stable browserless bars,
standalone `tv events`, `tv diagnose`, binary split, MCP server, daemon
behavior, and any ranking, scoring, or recommendation feature.

## Idempotence and Recovery

This audit is safe to rerun. If validation fails, fix only the failing
release-blocking mismatch and rerun the affected command before rerunning the
full required set. If a command writes build artifacts under `target/`, those
artifacts are not part of the commit. Do not archive the Desktop quote-session
evidence plan until phase-specific evidence has actually been collected.

## Artifacts and Notes

Do not paste raw live payloads, target ids, account-local metadata, local
absolute paths, credentials, or downstream-private paths into tracked files.
If a validation command prints such a value, summarize it as a public-safe
category instead of copying it.

## Interfaces and Dependencies

No new public interface is introduced by this audit. The relevant existing
interfaces are the `tv quote` JSON payload, the `tv snapshot` JSON payload,
the `tv compare` JSON payload, and the ignored
`live_quote_session_extended_hours` integration test. The audit must not
change command-line options, dependency versions, release version, or JSON
payload shape unless a release blocker requires a minimal correction.

## Open Questions

None blocking this audit. Desktop quote-session postmarket and premarket
semantics remain open in
`docs/plans/tradingview-cli-desktop-quote-session-live-evidence.md`.
