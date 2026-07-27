# Audit the frozen v0.31 candidate before release preparation

This ExecPlan is a living document. Keep `Progress`, `Surprises & Discoveries`,
`Decision Log`, and `Outcomes & Retrospective` current while work proceeds.
Maintain it according to `.agents/PLANS.md`.

## Purpose / Big Picture

This slice freezes v0.31 feature work and determines whether the final
candidate is coherent, correctly documented, fully tested, and ready for a
separate release-readiness plan. A contributor should be able to reproduce the
exact `v0.30.2..HEAD` inventory, distinguish shipped behavior from test and
planning evidence, trace the one-minute range and bars diagnostic contracts end
to end, run the complete deterministic baseline, and state whether any
release-blocking defect or architecture correction remains.

The promoted user-facing changes are bounded one-minute Desktop-free
`tv bars --from/--to` support and additive public-safe
`source_failure_stage` diagnostics for Desktop-free bars failures. The
one-minute change preserves `bars.v1`, period-start timestamps, the 5,000
returned-bar cap, and the existing coverage vocabulary. The diagnostic change
preserves existing error kinds, messages, details, and exit codes. The
retained-backlog comparison is release evidence rather than another runtime
feature.

No new feature, dependency, source, fallback, live operation, version bump, or
release operation belongs in this audit. Small contract-preserving corrections
may be made here. A larger behavior change or refactor stops the audit and
requires its own ExecPlan.

## Progress

- [x] (2026-07-28) Closed and archived the one-minute date-range and
  Desktop-free bars transport-diagnostics plans.
- [x] (2026-07-28) Compared retained product candidates and selected no
  additional promotion because none met its recorded trigger.
- [x] (2026-07-28) Created this completion and architecture audit ExecPlan and
  synchronized current planning state.
- [x] (2026-07-28) Obtained focused independent plan review. One
  reproducibility gap was corrected by adding commit-level classification and
  start/end repository-state checks; narrow re-review found no remaining
  finding and authorized audit execution.
- [x] (2026-07-28) Froze candidate `336d229`: 16 commits, 29 changed paths,
  no Cargo manifest, lockfile, workflow, or `mise.toml` change, with clean
  tracked and staged state and the ignored ledger read directly.
- [x] (2026-07-28) Audited one-minute date-range behavior and downstream
  guidance end to end without rerunning live evidence.
- [x] (2026-07-28) Audited Desktop-free bars failure-stage behavior,
  deterministic lifecycle fixtures, and non-recovery boundaries end to end.
- [x] (2026-07-28) Audited retained defers, public documentation, packaging,
  private-data hygiene, and architecture ownership.
- [x] (2026-07-28) Ran focused tests and the complete deterministic validation
  baseline; every gate was green and live tests remained ignored.
- [ ] Obtain focused independent audit review, record the outcome, and archive
  this plan before release readiness.

## Surprises & Discoveries

- Observation: the first owner-authorized one-minute live smoke stopped on a
  common-path connection failure before range classification.
  Evidence: a later bounded production-binary comparison succeeded for the
  existing five-minute path and all three intended one-minute cases, so the
  timeframe implementation was not the cause.

- Observation: the transient failure exposed insufficient Desktop-free bars
  attribution even though the feature itself was valid.
  Evidence: prior errors could not distinguish WebSocket connection, setup,
  response wait, protocol, heartbeat, pagination, or empty-result boundaries.
  The reviewed `source_failure_stage` contract now provides that distinction
  without adding recovery behavior.

- Observation: no retained product candidate currently meets its promotion
  trigger.
  Evidence: explicit bounded windows cover the current historical workload,
  and event, Pine, Screener, and alert candidates still lack their required
  consumer or ownership boundary.

- Observation: the candidate contains no dependency or release-tooling change.
  Evidence: `git diff --quiet v0.30.2..336d229 -- Cargo.toml Cargo.lock .github
  mise.toml` exited zero. The 16 commits change 29 paths across the two bars
  slices and their public, test, evidence, and planning surfaces.

## Decision Log

- Decision: freeze v0.31 after the two promoted bars slices.
  Rationale: the concrete downstream one-minute blocker is closed, failure
  attribution now reduces unnecessary diagnosis exchanges, and another feature
  without its trigger would weaken the evidence-first release boundary.
  Date/Author: 2026-07-28 / Codex

- Decision: audit `source_failure_stage` separately from the Desktop CDP
  `failure_stage` contract.
  Rationale: Desktop-free bars and Desktop-backed CDP operations have different
  transports and lifecycle boundaries. Reusing one field or vocabulary would
  obscure ownership rather than improve diagnostics.
  Date/Author: 2026-07-28 / Codex

- Decision: do not rerun the one-minute live comparison during this audit.
  Rationale: the bounded owner-approved run already proved the production
  binary scenarios. The audit verifies deterministic contracts and recorded
  aggregate evidence; another network run would not establish a new release
  property.
  Date/Author: 2026-07-28 / Codex

- Decision: keep retry, reconnect, timeout changes, fallback, shared sessions,
  and broker behavior unpromoted.
  Rationale: stage attribution identifies where a failure occurred but does not
  prove that replaying an operation is safe. Recovery requires repeated
  evidence and a separate reviewed plan.
  Date/Author: 2026-07-28 / Codex

## Outcomes & Retrospective

The candidate audit and full deterministic baseline are complete. The exact
16-commit, 29-path inventory is classified below. One-minute date ranges and
Desktop-free bars source-stage diagnostics preserve their reviewed contracts,
and no retry, fallback, shared ownership, dependency change, release operation,
or private tracked evidence was found. No release-blocking defect or required
architecture refactor was identified locally.

Focused independent audit review remains the sole completion gate. This plan
must not be archived and release readiness must not begin until that review is
green.

## Context and Orientation

The repository is a Rust workspace that builds one `tv` binary. Desktop-free
historical bars are implemented in `crates/market/src/bars.rs` and its
`bars/validation.rs`, `bars/transport.rs`, and `bars/payload.rs` modules. CLI
argument parsing and help live in `crates/cli/src/cli.rs`; public command
contracts are exercised by `crates/cli/tests/cli_contract_bars.rs`.

Date-range mode accepts inclusive calendar dates from the user and converts the
end date to an exclusive timestamp at the next day boundary. Returned bars use
period-start timestamps and are filtered to the half-open interval
`from <= time < to_exclusive`. A fetch window requests at most 500 bars, while
one command returns at most 5,000. Incomplete coverage is represented by the
existing `range_coverage_status`, `range_truncated`, and
`range_truncation_reason` fields rather than by silent success.

`source_failure_stage` is an additive string in Desktop-free bars error details.
Its closed vocabulary is `symbol_search`, `request_prepare`,
`websocket_connect`, `session_setup`, `series_setup`, `response_wait`,
`protocol`, `heartbeat_send`, `pagination`, `source_result`, and
`source_unknown`. It is distinct from the Desktop-backed CDP `failure_stage`
field. The new field classifies evidence only; it does not authorize retry.

The completed implementation plans are
`docs/plans/archives/tradingview-cli-one-minute-bars-date-range.md` and
`docs/plans/archives/tradingview-cli-bars-transport-diagnostics.md`. The
retained candidate decision is
`docs/notes/v0.31-retained-backlog-product-selection.md`. These documents are
evidence and rationale, while production source and public contract tests
remain the authority for shipped behavior.

## Plan of Work

First freeze the candidate. Record the exact starting `HEAD`, commit count,
commit list, changed paths, tracked worktree state, staged state, and ignored
local-ledger state. Classify every commit in `v0.30.2..HEAD` individually as
one-minute production, one-minute test/live evidence, bars-diagnostics
production, bars-diagnostics deterministic evidence, public/runtime guidance,
or docs/plan evidence. For each commit record its subject, changed paths, and
why it belongs to exactly one primary category; a mixed commit must name each
secondary category explicitly. Separately classify every changed path as
production, test, public documentation, runtime guidance, plan, or note.
`CONTINUITY.md` is an ignored local ledger, not a candidate path, so read its
State, Now, and Next sections directly at both boundaries instead of relying on
Git diff output.

Inspect `Cargo.toml`, `Cargo.lock`, `.github/`, and `mise.toml` separately so
dependency, feature, workflow, or toolchain drift cannot hide inside the larger
diff. At closeout, compare `git rev-parse HEAD` with the recorded starting
value, repeat tracked, staged, and ledger checks, and stop if anything moved.
If a reviewed correction creates a new commit, record the new HEAD, classify
that commit, and rerun the candidate inventory before relying on it.

Second audit one-minute date ranges. Trace normalization and validation from
`crates/cli/src/cli.rs` into `crates/market/src/bars/validation.rs`, then trace
transport filtering and payload shaping through `bars/transport.rs`,
`bars.rs`, and `bars/payload.rs`. Prove that normalized `1` is accepted only in
bounded date-range mode, that `1m` remains its alias, and that `3`, `45`, `120`,
`180`, and `240` remain rejected. Verify half-open end-date filtering,
period-start semantics, the 500-bar fetch window, 5,000 returned-bar cap,
single absolute deadline, no-progress stop, source-exhausted and timeout
classification, and the absence of synthetic closure bars. Confirm recent
count mode, weekly/monthly behavior, `tv ohlcv`, `tv range`, and selected-chart
sources did not change.

Third audit Desktop-free bars diagnostics. Follow every transport lifecycle
boundary through `crates/market/src/bars.rs` and
`crates/market/src/bars/transport.rs`. Confirm the initial five setup sends,
connection and response failures, malformed protocol data, heartbeat,
pagination, bare-symbol search, and zero-result facade mapping use the reviewed
closed vocabulary. Prove `Message::Close` remains `response_wait`, pagination
uses `pagination`, and zero bars become `source_result`. Verify existing
`ErrorKind`, message, exit code, object details, availability summaries, range
summaries, and partial-bar diagnostics are preserved. Details without an
object may be converted to an object; non-object details must use
`previous_details_omitted: true`. Unknown stage inputs must fail closed to
`source_unknown`.

Fourth audit what did not ship. Search production source for new retry,
reconnect, timeout extension, fallback, alternate source, shared session,
broker, or background work. Confirm the Desktop CDP `failure_stage` contract is
unchanged. Confirm the retained-backlog note does not describe its no-promotion
comparison as a runtime capability. Public docs and packaged runtime guidance
must explain one-minute ranges and bars source stages without exposing private
symbols, dates, target IDs, endpoints, payloads, credentials, or account-local
metadata.

Finally run focused and full validation, record concise counts, and request an
independent read-only audit review. The reviewer must check production behavior,
test realism, architecture ownership, public claims, package boundaries,
private-data hygiene, and durable-state synchronization. Archive only after
that review is green.

## Concrete Steps

Run from the repository root.

Freeze and classify the candidate:

    git rev-parse HEAD
    git rev-list --count v0.30.2..HEAD
    git log --oneline --decorate v0.30.2..HEAD
    git log --reverse --format='%H%x09%s' v0.30.2..HEAD
    git log --reverse --format='%H' v0.30.2..HEAD |
      while read commit; do
        git show --format='commit %H%nsubject %s' --name-status --no-renames "$commit"
      done
    git diff --name-status v0.30.2..HEAD
    git diff --stat v0.30.2..HEAD
    git status --short --branch
    git diff --check
    git diff --cached --check
    git diff --check v0.30.2..HEAD
    git diff --quiet v0.30.2..HEAD -- Cargo.toml Cargo.lock .github mise.toml
    git check-ignore -v CONTINUITY.md
    sed -n '/^## State:/,/^## Done:/p' CONTINUITY.md
    sed -n '/^## Now:/,/^## Working set/p' CONTINUITY.md

The per-commit output must become an artifact with one row per commit and these
columns: abbreviated commit, subject, primary category, secondary category if
mixed, changed paths, and contract impact. The current creation-time snapshot
contains 15 commits; execution must refresh that number rather than assuming it
is permanent. The Cargo/workflow command should exit zero when no dependency,
workflow, or toolchain state changed. `git check-ignore` should identify
`CONTINUITY.md` as ignored, after which its contents are still inspected
directly. If any expectation differs, classify the difference and revise this
living plan before continuing.

Inspect the promoted contracts and exclusions:

    rg -n 'timeframe|from|to|5_000|500|period_start|range_coverage_status|range_truncated' \
      crates/cli/src/cli.rs crates/market/src/bars.rs crates/market/src/bars \
      crates/cli/tests/cli_contract_bars.rs
    rg -n 'source_failure_stage|BarsFailureStage|with_source_failure_stage|source_result|pagination' \
      crates/market/src/bars.rs crates/market/src/bars/transport.rs \
      crates/cli/tests/cli_contract_bars.rs
    rg -n 'retry|reconnect|fallback|broker|shared session|timeout extension' \
      crates/market crates/cli/src

Run focused tests:

    cargo test -p tradingview-market bars -- --nocapture
    cargo test -p tradingview-cli --test cli_contract_bars -- --nocapture
    cargo test -p tradingview-cli --test live_bars -- --nocapture

The ordinary `live_bars` command must run deterministic fixtures and leave the
network tests ignored. Do not pass `--ignored` and do not set live environment
gates during this audit.

Run the full deterministic baseline:

    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1
    python3 scripts/check-public-hygiene.py --self-test
    python3 scripts/check-public-hygiene.py
    bash -n scripts/stage-release-package-files.sh
    cmp -s AGENTS.md CLAUDE.md
    ruby -e 'require "yaml"; Dir[".github/workflows/*.{yml,yaml}"].sort.each { |f| YAML.load_file(f); puts "parsed #{f}" }'
    git diff --check

Close the audit by repeating repository-state checks:

    git rev-parse HEAD
    git status --short --branch
    git diff --check
    git diff --cached --check
    git diff --quiet v0.30.2..HEAD -- Cargo.toml Cargo.lock .github mise.toml
    git check-ignore -v CONTINUITY.md
    sed -n '/^## State:/,/^## Done:/p' CONTINUITY.md
    sed -n '/^## Now:/,/^## Working set/p' CONTINUITY.md

The closing HEAD must equal the frozen starting HEAD unless a reviewed narrow
correction was committed and added to the classification artifact. Tracked and
staged state must match the state recorded at the start. The ignored ledger
must describe the same audit/review gate as the ExecPlan, plan index, roadmap,
work inventory, and CHANGELOG.

Record exact test counts and any ignored tests in `Progress` and `Artifacts and
Notes`. A zero-test focused filter is not evidence; correct the command in this
living plan if any filter runs zero tests.

## Validation and Acceptance

The audit is accepted only when all of the following are demonstrated.

The exact candidate commit, every one of its commits, and every changed path
are classified. Each commit is assigned to the two promoted slices,
test/live-evidence support, or docs/plan evidence with mixed impact recorded
explicitly. There is no unexplained dependency, feature, workflow, toolchain,
production, test, or documentation change. Starting and closing HEAD values
match unless a reviewed correction is added and reclassified. Tracked and
staged state is checked at both boundaries. The ignored `CONTINUITY.md` ledger
is read directly at both boundaries and agrees with all tracked durable-state
sources. Any candidate movement after the freeze is explicitly refreshed.

Normalized timeframe `1` and alias `1m` work only through the reviewed bounded
date-range contract. Half-open date filtering includes the final minute before
the exclusive boundary and excludes the boundary itself. Range output
preserves `bars.v1`, `period_start`, existing requested/returned/observed
ranges, coverage and truncation fields, the 500-bar fetch window, 5,000-bar
return cap, one deadline, and no-progress stop. Closures do not produce
synthetic bars or an unjustified complete status.

Every Desktop-free bars failure boundary maps to the reviewed
`source_failure_stage` vocabulary. The zero-result production facade is covered
directly. Existing error kinds, messages, exit codes, object details, source
availability, wait summaries, range summaries, and partial-result diagnostics
remain intact. Private transport data never enters the public error.

No retry, reconnect, fallback, timeout extension, source substitution, shared
session, broker, or failure-to-success promotion exists. Desktop CDP
`failure_stage` remains unchanged. The retained selection note is not presented
as shipped functionality.

Focused tests, strict Clippy, the full workspace suite and doctests, metadata,
public hygiene, package syntax, contributor-guide parity, workflow YAML
parsing, and diff hygiene are green. Independent focused audit review finds no
release blocker or required architecture refactor. Only then may the plan be
archived and release readiness become current.

## Idempotence and Recovery

All audit commands are read-only or generate ordinary build artifacts and may
be rerun. Do not run ignored live tests, connect to TradingView, alter Desktop
state, apply or drop stashes, change versions, tag, push, execute workflows, or
publish a release.

If a deterministic test fails, preserve the failure, identify whether it is a
candidate defect or environment problem, and make only a narrow
contract-preserving correction. If a larger behavior or architecture change is
needed, stop this audit and create a separate ExecPlan. If unrelated work
appears, do not revert it; refresh the frozen inventory or wait for the owner to
resolve the overlap.

## Artifacts and Notes

Keep only public-safe aggregate evidence: commit and path counts, test counts,
fixed stage names, and validation status. Do not retain live symbols, dates,
bars, prices, endpoint strings, WebSocket frames, credentials, target IDs,
environment values, local paths, or raw error payloads.

The owner-approved one-minute production-binary comparison is already reviewed
evidence. Its three scenarios succeeded after the initial common-path
connection failure, and it must not be rerun during this audit.

The frozen start was `336d2295a0905e5f48c7ce44dba27c6c21e2edad`,
16 commits and 29 changed paths after `v0.30.2`. The commit classification is:

| Commit | Subject | Primary category | Secondary category and contract impact |
| --- | --- | --- | --- |
| `661be88` | Add v0.31 roadmap | docs/plan evidence | Creates roadmap, inventory, and one-minute ExecPlan; no runtime behavior. |
| `922614a` | Add one-minute bars date ranges | one-minute production | Updates validation, transport, CLI contract, public docs, and runtime guidance for normalized `1`/`1m`. |
| `d685faa` | Complete one-minute range evidence | one-minute test/live evidence | Adds deterministic range fixtures and bounded ignored harness; payload/transport edits are test and contract shaping for the same slice. |
| `e1958f0` | Tighten one-minute smoke validation | one-minute test/live evidence | Strengthens public-safe gates and aggregate consistency; no production path. |
| `72fe71a` | Record one-minute smoke review | docs/plan evidence | Records review gate only. |
| `d38021b` | Record one-minute live smoke outcome | docs/plan evidence | Records the first bounded aggregate failure without changing code. |
| `9ef84e6` | Record one-minute live validation | docs/plan evidence | Records successful bounded comparison and candidate interpretation. |
| `a72bcd3` | Archive one-minute bars range plan | docs/plan evidence | Moves the completed ExecPlan and synchronizes state. |
| `d832a84` | Complete one-minute bars handoff | public/runtime guidance | Fixes stable invocation, field precedence, and bounded multi-window downstream guidance. |
| `2c19e67` | Add bars transport diagnostics plan | docs/plan evidence | Creates the diagnostics investigation; no runtime behavior. |
| `781189a` | Correct bars diagnostics contract | docs/plan evidence | Corrects planned lifecycle and deterministic injection boundaries. |
| `5ec444c` | Align bars pagination stage contract | docs/plan evidence | Fixes the planned public vocabulary to `pagination`. |
| `0aaece7` | Add bars source failure stages | bars-diagnostics production | Adds the 11-value Desktop-free bars error stage mapping, deterministic transport fixtures, CLI contract, docs, and guidance. |
| `bbdd2e9` | Cover bars source-result boundary | bars-diagnostics deterministic evidence | Adds direct zero-result facade coverage and durable-state correction. |
| `b8ae52a` | Add v0.31 completion audit | docs/plan evidence | Archives diagnostics, records retained no-promotion, and creates this audit. |
| `336d229` | Make v0.31 audit freeze reproducible | docs/plan evidence | Adds commit-level and start/end repository-state requirements after plan review. |

The 29 changed paths divide into seven production/test paths under
`crates/`, four runtime skill references, `packaging/agent/AGENTS.md`, public
README and stable docs, two archived implementation plans, this active audit
plan, the retained-selection note, roadmap/inventory state, and CHANGELOG.
There are no unclassified paths.

Focused validation produced:

    tradingview-market bars: 38 passed, 1 ignored
    cli_contract_bars: 4 passed
    live_bars: 4 deterministic passed, 2 live ignored
    tradingview-cdp: 47 passed, 1 ignored
    tradingview-cli unit: 465 passed, 5 ignored
    desktop CLI contracts: 100 passed
    tradingview-market full: 103 passed, 2 ignored
    tradingview-core: 1 passed
    tradingview-model: 54 passed
    tradingview-pine: 25 passed
    tradingview-scanner: 36 passed
    doctests: 4 passed

Formatting, strict workspace Clippy, all workspace tests, metadata, public
hygiene over 635 tracked files, package-script syntax, contributor-guide
parity, both workflow YAML files, staged diff checks, and diff hygiene were
green. No ignored live test was run.

## Interfaces and Dependencies

This audit adds no interface or dependency. It verifies these existing
contracts:

- `crates/market/src/bars/validation.rs` accepts normalized `1` in bounded
  date-range mode while retaining other guards.
- `crates/market/src/bars/transport.rs` owns one WebSocket series, setup,
  pagination, response processing, and the existing absolute deadline.
- `crates/market/src/bars/payload.rs` shapes `bars.v1` success output and range
  coverage diagnostics.
- `crates/market/src/bars.rs` adds the Desktop-free
  `source_failure_stage` error detail without replacing existing errors.
- `crates/cli/tests/cli_contract_bars.rs` protects the public CLI contract.

No new crate, feature flag, test-only production API, public command, output
envelope, or source provider may be introduced.

## Open Questions

There are no unresolved questions that block audit execution. Recovery behavior
remains conditional on future repeated stage evidence. Retained product
candidates keep their documented triggers and are not reconsidered inside this
audit.

Revision note (2026-07-28): Created the frozen-candidate audit after both
promoted bars slices completed focused review and retained product selection
closed with no additional promotion. The plan makes the two shipped contracts,
non-recovery boundary, private-data rules, and independent review gate
reproducible before release readiness.

Revision note (2026-07-28): After focused plan review, added an explicit
per-commit classification artifact and start/end checks for HEAD, tracked
worktree state, staged state, and the ignored `CONTINUITY.md` ledger. This
closes the review's reproducibility gap without changing audit scope.

Revision note (2026-07-28): After green narrow re-review, executed the audit at
frozen production/docs candidate `336d229`, classified all 16 commits and 29
paths, traced both promoted contracts and exclusions, and recorded a green full
deterministic baseline. Focused independent audit review is now the only gate.
