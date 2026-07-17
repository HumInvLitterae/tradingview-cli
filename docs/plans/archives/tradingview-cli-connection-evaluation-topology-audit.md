# Audit Desktop connection and Runtime evaluation topology

This ExecPlan is a living document. The sections `Progress`, `Surprises &
Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to
date as work proceeds. This document must be maintained in accordance with
`.agents/PLANS.md` at the repository root.

## Purpose / Big Picture

Desktop-backed `tv` commands discover a TradingView Desktop target, open a CDP
WebSocket, and evaluate JavaScript in the selected page. Most commands use one
connection for one invocation, while tab, readiness, diagnostics, Desktop app,
Screener, streaming, and multi-target workflows intentionally have different
ownership. Repeated work is not automatically waste: a second discovery can
establish fresh ownership, and a second evaluation can preserve a deadline,
failure stage, mutation order, or post-condition check.

After this audit, maintainers can read one durable matrix that accounts for
every production target-discovery, WebSocket-connect, and Runtime-evaluation
call site in `tradingview-cli`. Each workflow family is classified as a normal
single-owner path, intentional repeated work, a concrete optimization
candidate, or unresolved. The audit either closes with no production change or
names a narrow candidate that requires its own ExecPlan. This plan does not
optimize code, add instrumentation to public output, create a shared session,
or change command behavior.

## Progress

- [x] (2026-07-17) Create this self-contained audit plan after the reviewed
  transport measurement slice was archived and pre-dispatch retry was deferred.
- [x] (2026-07-17) Freeze the production call-site inventory at commit `b37d510`.
- [x] (2026-07-17) Classify target-list and target-selection ownership for every call site.
- [x] (2026-07-17) Classify WebSocket connection ownership for every call site.
- [x] (2026-07-17) Classify 134 production Runtime evaluations in 44 files by
  workflow family and semantic purpose.
- [x] (2026-07-17) Deep-review each apparent duplicate against ownership, deadline, failure
  attribution, mutation ordering, and post-condition boundaries.
- [x] (2026-07-17) Write the durable audit matrix and candidate decisions in
  `docs/notes/cdp-connection-evaluation-topology-audit.md`.
- [x] (2026-07-17) Synchronize planning documents with the observed result.
- [x] (2026-07-17) Run documentation-slice validation and focused behavior
  preservation tests. Desktop CLI contracts passed 99 tests, diagnose
  contracts passed 2 tests, and metadata, public hygiene, package-script
  syntax, guide parity, production-diff, and diff hygiene checks are green.
- [x] (2026-07-17) Obtain focused independent audit review and apply the final
  helper-grouping and `status` snapshot clarifications. Review is green.
- [x] (2026-07-17) Archive this plan with outcome `candidate_deferred`; no
  optimization ExecPlan or production topology change is promoted.

## Surprises & Discoveries

- Observation: explicit-target `status` performs a second target-list request
  immediately after its initial target snapshot.
  Evidence: `status()` calls `fetch_targets()` and then `discover_target()` in
  the explicit-target branch. The second read also provides a freshness and
  target-selection failure boundary, so it is deferred rather than removed.
- Observation: all apparent repeated Runtime evaluations belong to separate
  command entry points or semantic phases such as polling, mutation,
  verification, restoration, or cleanup.
  Evidence: 134 production sites in 44 files were grouped and inspected with
  their callers and intervening non-evaluate CDP actions.

## Decision Log

- Decision: make the first topology slice a read-only source and contract audit.
  Rationale: `connect_runtime()` already performs one discovery and one
  connection, while exceptional workflows own connections directly. Changing
  topology before identifying an exact redundant round trip could weaken
  target ownership or failure attribution without measurable benefit.
  Date/Author: 2026-07-17 / planning owner.
- Decision: inventory call sites completely, then group them by ownership
  pattern rather than promising one exact count for every command branch.
  Rationale: polling, per-target loops, optional fallback, and post-condition
  reads have bounded formulas rather than one constant count. A symbolic count
  such as `one per selected target` is more accurate than a misleading scalar.
  Date/Author: 2026-07-17 / planning owner.
- Decision: do not combine JavaScript evaluations in this plan.
  Rationale: adjacent evaluations may intentionally separate preflight,
  mutation, restoration, or verification. Any coalescing candidate needs a
  dedicated implementation plan and deterministic before/after evidence.
  Date/Author: 2026-07-17 / planning owner.

## Outcomes & Retrospective

The source audit accounts for 75 normal dispatcher connection sites, three
long-running runner owners, direct readiness/status/diagnostics/Desktop/tab/
Screener/launch owners, and 134 Runtime evaluation sites in 44 files. No
production change is justified. The explicit-target `status` duplicate listing
is `candidate_deferred`; no evaluation candidate survives. Focused independent
audit review is green and confirmed the inventories independently. The plan is
complete without a production change.

## Context and Orientation

The repository is a virtual Cargo workspace. `crates/cdp` owns target listing,
target selection, WebSocket connection, CDP method/event waits, and Runtime
evaluation. `crates/cli` parses commands, selects an operation, and owns the
lifetime of each invocation.

`crates/cli/src/app/runtime.rs` defines `connect_runtime(config)`. It discovers
one target through `tradingview_cdp::transport::discover_target` and connects
one `CdpClient`. Most command arms in `crates/cli/src/app/dispatch.rs` call this
helper once and pass one mutable client through all operation helpers. The
stream, observe, and Replay-log runners connect once before their loops in
`crates/cli/src/app/stream.rs`, `crates/cli/src/app/observe.rs`, and
`crates/cli/src/app/replay_log.rs`.

Some workflows intentionally bypass `connect_runtime()`. Readiness, status,
diagnostics, Desktop app inspection, tab operations, launch, and Screener state
management can list several targets, connect to an app target and a chart
target, create a tab, or verify a newly created target. Their direct owners
include `crates/cli/src/ops/readiness.rs`, `status.rs`, `diagnostics.rs`,
`desktop.rs`, `tab.rs`, `launch.rs`, and `screener/state.rs`. These paths must
not be labeled redundant merely because they contain more than one transport
operation.

A Runtime evaluation is a `Runtime.evaluate` CDP method call, usually reached
through the `RuntimeEvaluator` trait. Evaluation count alone is not a quality
metric. This audit records the semantic purpose of an evaluation as one of:
read, preflight, mutation, verification, restoration, polling, or cleanup.
Removing or merging an evaluation is eligible for a later plan only when it
reads the same target state, has no intervening mutation, and preserves the
same deadline and public-safe failure boundary.

The completed transport measurement plan is archived at
`docs/plans/archives/tradingview-cli-cdp-transport-measurement.md`. It added
typed stage observations and public-safe `failure_stage` error details but did
not add retry or shared connection behavior. This audit may cite that stable
failure vocabulary; it must not reopen retry implicitly.

## Plan of Work

### Milestone 1: freeze a complete transport-owner inventory

Starting from repository root, enumerate production references to
`connect_runtime`, `CdpClient::connect`, `CdpHttpSession`, target listing,
target discovery, target selection, target creation, and target activation
under `crates/cli/src`. Include instance-method calls to `new_target_url` and
`activate_target`; searching only for the `CdpHttpSession::` associated-function
syntax does not find them. Exclude test modules from final counts, but list
test-only matches separately so the audit is reproducible. Record the audited
commit and exact search expressions in the audit note.

Create `docs/notes/cdp-connection-evaluation-topology-audit.md`. Its first
matrix must contain one row per production call site, with repository-relative
file and symbol, workflow family, operation kind, cardinality, selected target
owner, and classification. Cardinality uses exact or symbolic values such as
`1`, `0 or 1`, `one per target`, or `one before loop`; it must not guess a
runtime count from source text alone.

Classify each row using the closed values `single_owner`,
`intentional_multi_target`, `conditional_fallback`, `candidate`, or
`unresolved`. Every `candidate` and `unresolved` row requires prose evidence.
Acceptance for this milestone is that a fresh search produces no unaccounted
production transport-owner call site.

### Milestone 2: map Runtime evaluation purposes

Enumerate production `.evaluate(...)` calls under `crates/cli/src` and map each
file to a command or workflow family. For helpers called by several commands,
record one helper-family row and list its production callers rather than
duplicating the implementation call site. For loops and polling, record the
bound or termination owner instead of inventing a constant count.

The second matrix in the audit note must include file and symbol, caller
family, purpose, cardinality, deadline owner, mutation between adjacent reads,
and classification. Use `single_read`, `preflight`, `mutation`, `verification`,
`restoration`, `polling`, `cleanup`, `candidate`, or `unresolved`. When one
evaluation performs several semantic steps, state that explicitly and do not
call it an optimization merely because it is one CDP method call.

An intervening mutation is not limited to `Runtime.evaluate`. Input dispatch,
text insertion, mouse events, screenshot or capture operations with stateful
preconditions, and any other CDP method that can change or consume relevant
page state must be considered when judging whether two evaluations are safely
adjacent.

Acceptance for this milestone is that every production `.evaluate(...)` call
site belongs to a named family and that every apparent adjacent duplicate has
been checked against mutation order, deadline, and error shaping.

### Milestone 3: decide candidates without optimizing

Deep-review each `candidate` and `unresolved` row. A removable transport round
trip must satisfy all of these conditions: it selects the same target under the
same explicit/heuristic policy; no intervening action can invalidate the
target; removing it preserves public error kind, message, `failure_stage`, and
exit code; and the original absolute deadline does not become a resettable
per-step timeout.

A removable evaluation must be a pure read of the same target state with no
intervening mutation, event dependency, polling wait, restoration obligation,
or distinct public-safe failure stage. The audit must reject candidates whose
only support is fewer source lines or fewer calls in a synthetic happy path.

For each surviving candidate, record its affected commands, current symbolic
count, proposed count, preserved ownership and deadline, deterministic fixture
needed, and measurable acceptance threshold. Do not edit production Rust in
this plan. If no candidate survives, record a reviewed no-change result.

### Milestone 4: synchronize and review

Update `docs/architecture.md` only with stable facts discovered by the audit.
Update `docs/v0.29-roadmap.md`, `docs/v0.29-work-items.md`,
`docs/plans/README.md`, `CHANGELOG.md`, and `CONTINUITY.md` with the same result.
Do not claim a performance improvement from a source audit.

Run the validation below and obtain focused independent review of completeness,
classification, and candidate gates. After review, archive this plan. A
surviving candidate receives a new self-contained ExecPlan; the audit commit
must not contain its production implementation.

## Concrete Steps

Run all commands from repository root. Capture concise counts and file/symbol
references in the audit note, not raw command dumps.

Inventory transport owners:

    rg -n "connect_runtime\(|CdpClient::connect|CdpHttpSession::|discover_target\(|fetch_targets\(|select_target\(|new_target_url\(|activate_target\(" crates/cli/src --glob '*.rs'

Inventory evaluation sites:

    rg -n "\.evaluate\(" crates/cli/src --glob '*.rs'

Find the dispatcher and runner ownership boundaries:

    rg -n "connect_runtime\(" crates/cli/src/app --glob '*.rs'
    rg -n "CdpClient::connect|CdpHttpSession::" crates/cli/src/ops --glob '*.rs'

Confirm the audit note accounts for every production match by comparing a
fresh search against its file/symbol matrix. Test modules may be excluded only
when the note records the exclusion rule.

Run focused behavior-preservation tests:

    cargo test -p tradingview-cli --test cli_contract_desktop -- --nocapture
    cargo test -p tradingview-cli --test cli_contract_diagnose -- --nocapture

If the final slice remains documentation-only, also run:

    cargo metadata --no-deps --format-version 1
    python3 scripts/check-public-hygiene.py --self-test
    python3 scripts/check-public-hygiene.py
    bash -n scripts/stage-release-package-files.sh
    cmp -s AGENTS.md CLAUDE.md
    git diff --check

If any Rust file changes despite the audit-only decision, stop and revise this
ExecPlan before continuing. After explicit scope review, run the full baseline:

    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace

## Validation and Acceptance

The audit is complete when all production target/discovery/connection call
sites and Runtime evaluation call sites under `crates/cli/src` are represented
in the durable matrix or its documented grouping rule. A reviewer must be able
to rerun the exact searches and find no unexplained production match.

Each repeated operation must be classified with source-backed ownership and
cardinality. Intentional multi-target work, polling, preflight, mutation,
verification, restoration, and cleanup must remain distinct. No candidate may
be promoted solely from static call count.

The final outcome is exactly one of:

1. `no_change`: no removable round trip survived review, so no optimization
   ExecPlan is created;
2. `candidate_deferred`: a possible duplicate lacks measurable benefit or a
   safe contract and remains in the audit note; or
3. `candidate_promoted`: one or more named candidates have enough evidence for
   separate ExecPlans, without implementation in this slice.

Existing CLI contracts and workspace tests remain green. The diff contains no
production topology, public JSON, timeout, retry, dependency, or source/fallback
change. Public docs contain no raw target ID, endpoint, Runtime payload,
credential, account-local metadata, or machine-specific path.

## Idempotence and Recovery

This audit is read-only with respect to TradingView Desktop and production
behavior. Searches and tests may be repeated. Do not launch or mutate
TradingView, run an owner-gated live test, alter a stash, or push a branch as
part of the audit.

If inventory counts change while the audit is in progress, record the new
audited commit and rerun both inventories. Do not merge two commit snapshots
into one matrix. If a candidate requires code to prove feasibility, stop at
`candidate_promoted` and create a separate plan rather than inserting a
prototype into this audit.

## Artifacts and Notes

The durable artifact is
`docs/notes/cdp-connection-evaluation-topology-audit.md`. It contains the
audited commit, commands, grouping/exclusion rules, transport-owner matrix,
evaluation-purpose matrix, candidate decisions, validation summary, and final
outcome. Keep excerpts concise and repository-relative.

Do not paste complete search output. Do not store target identifiers, endpoint
URLs, Runtime values, local timings from an unreviewed environment, or local
absolute paths. This audit is source topology evidence, not a live performance
benchmark.

## Interfaces and Dependencies

No new Rust interface or dependency is expected. Production owners remain:

    crates/cli/src/app/runtime.rs::connect_runtime
    tradingview_cdp::transport::CdpHttpSession
    tradingview_cdp::CdpClient
    tradingview_cdp::RuntimeEvaluator

The audit note uses the closed classification vocabularies defined in this
plan. These labels are documentation-only and are not a public JSON or Rust
contract.

## Open Questions

- Which, if any, repeated transport operation is actually redundant is
  `UNCONFIRMED` until the inventory and ownership review are complete.
- Which, if any, adjacent Runtime evaluations can be combined without weakening
  deadlines or semantic failure attribution is `UNCONFIRMED`.
- Whether a surviving candidate has measurable value is `UNCONFIRMED`; static
  call count alone cannot answer it.
- Shared connection, broker, retry, recovery metadata, and a new wait surface
  remain outside this plan.

Revision note (2026-07-17): created after the transport measurement slice
closed with a zero-failure bounded observation and deferred retry. This plan
audits current topology before any structural optimization is proposed.

Revision note (2026-07-17): after focused plan review, expanded the transport
inventory to include instance-method target creation and activation, removed a
nonexistent search symbol, and required intervening non-evaluate CDP actions to
count as mutations. The source audit then completed with one deferred
explicit-target `status` listing candidate and no evaluation candidate.

Revision note (2026-07-17): focused audit review independently reproduced the
75 dispatcher sites and 134 evaluation sites in 44 files. Added the grouped
`current_new_tab_target` helper and recorded that the first explicit-target
`status` snapshot feeds readiness output. The reviewed outcome remains
`candidate_deferred` with no production change.
