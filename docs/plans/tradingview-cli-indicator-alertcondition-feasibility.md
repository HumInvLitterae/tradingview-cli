# Investigate indicator alertcondition alert feasibility

This ExecPlan is a living document. The sections `Progress`, `Surprises &
Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to
date as work proceeds.

This document follows `.agents/PLANS.md` from the repository root. It is
self-contained so a reader can understand the investigation without chat
history.

## Purpose / Big Picture

Upstream PR #112 proposes creating TradingView alerts for Pine
`alertcondition()` signals by posting directly to TradingView's alert endpoint.
That workflow could save operators from repetitive alert-dialog clicking, but it
also depends on account-linked Pine script identifiers, Pine input payloads, and
plot-index details that are easy to get wrong. This investigation records
whether the Rust CLI should expose that surface now, and what evidence is still
needed before any account-mutating command is implemented.

## Progress

- [x] (2026-04-27 18:02Z) Re-read upstream PR #112 current body and compared it
  with Rust `alert`, `pine`, and internal API docs.
- [x] (2026-04-27 18:02Z) Ran read-only live checks for alert and Pine endpoint
  availability without recording raw ids.
- [x] (2026-04-27 18:02Z) Recorded the implementation boundary and updated
  durable docs.
- [x] (2026-04-27 18:05Z) Ran docs-only validation.
- [ ] Commit this docs-only investigation.

## Surprises & Discoveries

- Observation: The existing alert endpoint family is available in the current
  logged-in page session.
  Evidence: `tv alert list` returned success with source `internal_api` and an
  alert count, with no endpoint error.
- Observation: The Pine facade can list saved Pine script metadata, but that
  metadata includes account-linked saved-script identifiers.
  Evidence: `tv pine list` returned success with one saved script and metadata
  keys including id, name, title, version, and modified.
- Observation: The current visible page did not expose Pine Editor Monaco for
  `tv pine get`.
  Evidence: `tv pine get` returned `internal_api_unavailable` with
  `opened_editor: false`. This did not block the investigation because source
  inspection is optional and the risky part is alert creation, not editor reads.

## Decision Log

- Decision: Do not add a Rust `alert create-indicator` command in this slice.
  Rationale: The upstream shape is a low-level primitive that requires
  account-linked script ids, exact Pine input payloads, and plot-index
  knowledge. Publishing that directly would be easy to misuse and hard to make
  safe without discovery, dry-run, and readback design.
  Date/Author: 2026-04-27 / Codex.
- Decision: Treat PR #112 as a viable but deferred feature candidate rather
  than rejecting it outright.
  Rationale: The existing Rust alert create path already uses the same alert
  endpoint family, and `pine list` confirms saved-script metadata is reachable.
  The missing piece is a safe metadata-discovery and preview boundary.
  Date/Author: 2026-04-27 / Codex.
- Decision: Any future implementation should start with a dry-run discovery
  command before normal mutation.
  Rationale: Operators need to see the target script, condition candidate,
  symbol, resolution, expiration, and webhook/message intent before an account
  alert is created.
  Date/Author: 2026-04-27 / Codex.

## Outcomes & Retrospective

This investigation is complete as a docs-only slice and ready to commit. The
result is not a new command; it is a recorded boundary: indicator
alertcondition alerts may belong in the Rust CLI, but only after a separate
ExecPlan designs metadata discovery, dry-run output, readback verification, and
live-smoke cleanup. Raw saved-script ids, webhook URLs, alert ids, and copied
request payloads remain out of tracked docs.

## Context and Orientation

`src/ops/alert.rs` implements `tv alert list`, `tv alert create`, and alert
delete commands. These commands call TradingView alert endpoints from the
logged-in page session and require post-mutation readback before reporting
success. A "page session" means JavaScript running inside the user's logged-in
TradingView Desktop page; it can call endpoints that the page itself can call.

`src/ops/pine/` implements Pine Editor and Pine facade operations such as
`tv pine list`, `tv pine open`, and `tv pine check`. A saved Pine script id is
account-linked metadata and must be treated like private operational data.

A Pine `alertcondition()` is a Pine Script call that TradingView can expose as
an alert condition. Upstream PR #112 reports that TradingView identifies these
conditions by plot-like ids such as `plot_N`, where the count depends on
plot-emitting source calls. That makes manual ids risky because off-by-one
mistakes can create the wrong alert or fail with a low-level endpoint error.

## Plan of Work

First, document the upstream PR #112 evidence at a high level. Do not copy its
example script id, webhook URL, raw input object, alert id, or raw request body.

Second, record live read-only evidence from the current Rust CLI. Use
`tv alert list` and `tv pine list`, but summarize only counts, source names, and
metadata key presence. Do not paste alert rows or Pine script rows into tracked
docs.

Third, update the internal API reference and roadmap notes. The internal API
reference should say that indicator alertcondition creation is a
`replace_candidate` or feature candidate, not `api_backed`, and that the first
safe Rust surface should be discovery/dry-run oriented.

Fourth, leave implementation to a later plan. A future command may be named
`tv alert create-indicator`, but this plan intentionally does not define its
full CLI contract.

## Concrete Steps

Run commands from the repository root.

Read-only live checks used for this investigation:

    cargo run --quiet -- alert list | <summarize count/source without alert ids>
    cargo run --quiet -- pine list | <summarize count/source without script ids>

Observed public-safe summaries:

    alert list: success=true, source=internal_api, endpoint error absent
    pine list: success=true, source=internal_api, one saved script metadata row
    pine get: internal_api_unavailable because the Pine Editor was not opened

Validation commands:

    git diff --check
    git grep -nE '(/Users/|C:\\|USER;|sessionid|cookie|authorization|bearer|webhook|web_hook)' -- README.md CHANGELOG.md docs .agents/skills packaging scripts || true
    git status --short

## Validation and Acceptance

Acceptance is documentation-level. A future contributor should be able to read
this file plus `docs/internal-tradingview-apis.md` and understand that PR #112
is not ready for immediate raw-command implementation, but is valuable enough to
consider after metadata discovery and dry-run behavior are designed.

The grep validation may return existing policy text and validation command
examples. It must not reveal a new live saved-script id, alert id, webhook URL,
credential, or machine-specific path introduced by this slice.

Validation result: `git diff --check` passed. The safety grep returned existing
policy text, validation-command examples, and the new no-raw-webhook safety
language only; it did not reveal a webhook URL, credential, saved-script id, or
machine-specific path introduced by this slice.

## Idempotence and Recovery

This slice is read-only with respect to TradingView account state. It does not
create, delete, or edit alerts or Pine scripts. Re-running the checks may return
different counts as the user's account state changes; do not record those raw
rows in tracked files.

## Artifacts and Notes

The important upstream facts are:

- PR #112 proposes creating alertcondition alerts through the alert create
  endpoint rather than the visible alert dialog.
- The proposed shape needs saved Pine script metadata, a condition id derived
  from Pine plot-emitting calls, Pine input payloads, symbol/resolution context,
  and optional webhook/message fields.
- The PR explicitly leaves automatic discovery of script metadata and condition
  ids out of scope. That discovery is the part the Rust CLI should solve before
  exposing a user-facing command.

## Interfaces and Dependencies

No code interfaces change in this slice. Existing relevant commands remain:

    tv alert list
    tv alert create --price <NUMBER> [--condition <CONDITION>] [--message <TEXT>]
    tv pine list
    tv pine open <NAME...>
    tv pine check [--file <PATH>]

A later implementation plan may introduce a new alert subcommand. If it does,
it must use the existing Rust JSON envelope and must not report success without
alert-list readback.

## Open Questions

No critical question blocks this docs-only investigation. Open implementation
questions for a later plan are:

- Can the CLI discover alertcondition candidates from a saved script or an
  attached study without requiring raw `plot_N` input?
- Can the CLI preview the exact target script, symbol, resolution, message, and
  webhook intent without exposing account-linked ids?
- What cleanup workflow should live smoke use if a test indicator alert is
  created?
