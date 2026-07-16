# v0.28.0 pre-release completion and architecture audit

This ExecPlan is a living document. Keep `Progress`, `Surprises & Discoveries`,
`Decision Log`, and `Outcomes & Retrospective` current while work proceeds.
Maintain it according to `.agents/PLANS.md`.

## Purpose / Big Picture

This slice freezes v0.28 feature work and audits the promoted release scope
before release readiness. The candidate includes safer current-build Pine
saved-script operations, verified current-build indicator insertion, launch
environment hardening, native three-point `parallel_channel` creation, and the
owner-approved bounded Desktop-free scanner pagination exception.

The audit determines whether implementation, contracts, help, stable docs,
packaged guidance, runtime skills, tests, executable JavaScript gates, source
and mutation boundaries, and module responsibilities agree. Small local drift
may be corrected here. A public behavior change, substantial module split, or
new compatibility investigation must become a separate plan.

No new command, option, payload field, dependency, source, fallback, live
mutation, or version bump belongs in this slice. Windows MSIX package-identity
launch, finite-`f64` selected-chart right-offset restoration, width-derived
drawing geometry, other three-point shapes, and the preserved indicator-search
prototype remain deferred.

Bounded scanner pagination is the one owner-approved exception to the earlier
feature freeze. Its separate ExecPlan was added after this audit plan was first
drafted because a downstream workflow needs bounded breadth promptly. Complete,
validate, and independently review that scanner slice before executing this
audit. The scanner implementation then becomes a fifth promoted v0.28 area and
must be included in the candidate, architecture, docs, and test inspection.

## Progress

- [x] (2026-07-16) Completed and archived reviewed native three-point drawing
  implementation.
- [x] (2026-07-16) Created this self-contained audit ExecPlan and synchronized
  the plan index, roadmap, work inventory, changelog, and continuity ledger.
- [x] (2026-07-16) Completed, validated, independently reviewed, and archived
  the bounded scanner pagination exception.
- [x] (2026-07-16) Classified the exact `v0.27.0..HEAD` candidate diff by production,
  dependency, workflow, test, documentation, and skill ownership.
- [x] (2026-07-16) Audited Pine saved-script selection, editor ownership, source and save
  verification, platform shortcuts, and public-safe diagnostics.
- [x] (2026-07-16) Audited indicator insertion resolution, exactly-once mutation, input
  preservation, immediate readback, result typing, and cleanup.
- [x] (2026-07-16) Audited launch environment removal, macOS fallback precedence, child-state
  classification, help, and failure shaping.
- [x] (2026-07-16) Audited native three-point validation, exactly-one creation, sticky
  ambiguity, native point readback, and preserved one/two-point behavior.
- [x] (2026-07-16) Audited scanner offset and aggregate pagination, request/page caps, total
  and drift semantics, shared-client ownership, and no-partial-success.
- [x] (2026-07-16) Audited all pinned executable JavaScript gates and confirmed ordinary Cargo
  tests remain Node.js-independent.
- [x] (2026-07-16) Inspected module size, production/test boundaries, helper ownership, and
  generated-JavaScript responsibilities.
- [x] (2026-07-16) Audited help, public docs, packaged guidance, runtime skills, tests, source
  taxonomy, mutation metadata, deferred boundaries, and public hygiene.
- [x] (2026-07-16) Ran focused tests, every JavaScript gate, the full Rust baseline,
  packaging, workflow, guide-parity, and diff checks.
- [x] (2026-07-16) Recorded the architecture verdict and prepared a read-only reviewer prompt.
- [x] (2026-07-16) Received independent review with no architecture blocker
  and three correction findings: sanitize Pine source-operation diagnostics,
  classify the dependency-bearing current candidate, and synchronize the
  continuity ledger.
- [x] (2026-07-16) Applied the Pine diagnostics and candidate/ledger
  corrections and reran focused and full validation.
- [ ] Obtain independent review before archiving this plan or starting release
  readiness.

## Milestones

### Milestone: complete the final promoted scanner slice

The scanner work at
`docs/plans/archives/tradingview-cli-scanner-bounded-pagination.md` is
implemented, validated, documented, independently reviewed, and archived. This
milestone is complete: bounded pagination is an explicit v0.28 candidate area
and no scanner implementation work remains pending.

### Milestone: establish and inspect the frozen candidate

Classify `v0.27.0..HEAD` by owning release area, including the completed scanner
slice, and separate shipped candidate changes from planning-only work. Trace
all five promoted areas through validation, source or mutation behavior,
readback, result shaping, diagnostics, help, docs, and tests. This milestone is
complete when each responsibility has a named owner and every architecture
finding has concrete evidence rather than a file-size inference.

### Milestone: validate and close the audit

Apply only small contract-preserving corrections, run every focused and full
gate, and record one architecture verdict. This milestone is complete only
after all commands pass, current-state documents agree, and independent
read-only review reports no unresolved finding. Only then may release-readiness
planning begin.

## Surprises & Discoveries

- Observation: v0.28 has several large adapters, but line count alone cannot
  determine whether they need a release-blocking split.
  Evidence: `crates/cli/src/ops/pine/editor/scripts.rs`,
  `crates/cli/src/ops/indicator.rs`, `crates/cli/src/ops/launch.rs`, and
  `crates/cli/src/ops/drawing/create.rs` include substantial deterministic
  fixtures alongside production code. The audit must locate the real
  production boundaries before judging cohesion.

- Observation: v0.28 has multiple executable JavaScript contracts outside the
  ordinary Cargo baseline.
  Evidence: study-value identity, Pine open/save, indicator insertion, and
  three-point drawing each have a pinned Node.js gate wired through local
  tooling and CI/release workflows, while their Rust fixtures are ignored in
  ordinary workspace tests.

- Observation: the largest changed modules remain operation-cohesive once
  deterministic fixtures are separated from production code.
  Evidence: production/test boundaries begin near line 752 in Pine
  `scripts.rs`, 558 in `indicator.rs`, 697 in `launch.rs`, 597 in drawing
  `create.rs`, and 932 in scanner `scan.rs`. Each production section owns one
  public operation family and its sanitizer/readback boundary; no cross-source
  fallback or shared mutable background runtime was introduced.

- Observation: a dependency-update commit landed after the audit plan began
  and is part of the current frozen candidate.
  Evidence: current `HEAD` is `b28603a`; `Cargo.toml` updates direct `clap`
  from `4.6.1` to `4.6.2`, and `Cargo.lock` records compatible patch/minor
  updates for `bitflags`, `bstr`, `clap`, `http-body`, `regex`, `simd_cesu8`,
  `socket2`, `syn`, and related transitive packages. The complete baseline and
  metadata are green with these versions.

- Observation: Pine source operations retained an older raw Runtime error
  boundary after adjacent open/save paths had been sanitized.
  Evidence: independent review found direct evaluation-error propagation and
  raw malformed-value details in `source.rs`. The correction preserves the
  original `ErrorKind` and exposes only fixed operation, stage, and
  response-type metadata for `get`, `set`, and `new`.

## Decision Log

- Decision: freeze v0.28 feature work after native three-point drawing support.
  Rationale: Pine, indicator, launch, and drawing form a coherent mutation-
  safety and current-build compatibility candidate. More feasibility work
  would broaden risk immediately before release preparation.
  Date/Author: 2026-07-16 / Codex

- Decision: admit bounded scanner pagination as the sole exception to that
  freeze and restart the audit only after its implementation review is green.
  Rationale: the owner confirmed that an active downstream workflow needs the
  capability promptly. Making the exception explicit prevents an unfinished
  plan from entering release scope ambiguously.
  Date/Author: 2026-07-16 / Codex

- Decision: preserve reviewed live evidence and do not repeat TradingView
  mutations during the audit.
  Rationale: deterministic implementation and regression gates are the audit
  subject. Repeating account- or chart-bearing operations needs a separate
  reason and approval.
  Date/Author: 2026-07-16 / Codex

- Decision: evaluate module cohesion before treating size as a blocker.
  Rationale: a large adapter may remain coherent when it owns one bounded
  operation and colocates tests. A split is release-blocking only when current
  ownership causes contract drift, unsafe coupling, dead code, or untestable
  behavior.
  Date/Author: 2026-07-16 / Codex

- Decision: keep Windows MSIX launch and finite-`f64` right-offset restoration
  outside implementation scope.
  Rationale: each requires separate platform or mutation evidence and approval.
  Date/Author: 2026-07-16 / Codex

- Decision: no dedicated architecture refactor is required before v0.28
  release readiness.
  Rationale: all five promoted areas retain clear operation or crate
  ownership, focused and full tests are green, public contracts agree with
  help and guidance, and no unsafe coupling, hidden fallback, dead production
  path, or release-blocking module responsibility drift was found.
  Date/Author: 2026-07-16 / Codex

## Outcomes & Retrospective

Local audit inspection and validation found no release-blocking architecture
issue or dedicated pre-release refactor. Independent review found one local
public-safety defect in Pine source diagnostics plus stale candidate and ledger
evidence after a dependency commit. Those corrections are applied. Focused
  Pine validation and the complete baseline are green; the four JavaScript
  gates were already green on the dependency-bearing `HEAD`.

The candidate remains frozen and release readiness remains blocked only on
focused independent review of this audit. Windows MSIX launch, finite-`f64`
right-offset restoration, width-derived drawing geometry, other three-point
shapes, and the preserved indicator-search prototype remain deferred.

## Context and Orientation

The latest release is `v0.27.0`, tagged at `73ee3a2`. The workspace version
remains `0.27.0` until release readiness. The v0.28 candidate has five promoted
areas.

Pine saved-script compatibility is owned by
`crates/cli/src/ops/pine/editor/`. `scripts.rs` opens and saves scripts,
`runtime.rs` selects the active Pine-owned Monaco editor, and `source.rs`
writes and verifies source. Success depends on the intended saved-script
binding, same editor/store ownership, explicit saved-and-clean readback, and
sanitized diagnostics. Line-ending normalization may equate CRLF, LF, and lone
CR only; it must not hide content differences.

Current-build indicator insertion is owned by
`crates/cli/src/ops/indicator.rs`. It resolves one exact public metainfo entry,
uses one chart-owned inserter signature, preserves JSON keys and JavaScript-
safe scalar values, awaits insertion, and validates the first post-await
snapshot through the same chart-local entity ID. It must not try alternate
signatures, dialog clicks, or legacy `createStudy` fallback.

Launch hardening is owned by `crates/cli/src/ops/launch.rs`. Direct spawn and
the normal macOS system launcher remove inherited `ELECTRON_RUN_AS_NODE` while
preserving unrelated environment entries. CDP readiness and successful macOS
fallback take precedence over the original direct child. Only an observed
exited or unavailable child may produce the structured connection failure when
no successful fallback exists.

Native three-point drawing is split between request validation in
`crates/model/src/drawing.rs`, CLI parsing in `crates/cli/src/cli.rs`, and the
Desktop adapter in `crates/cli/src/ops/drawing/create.rs`. Paired `--price3`
and `--time3` are accepted only for exact `parallel_channel`; point 3 is the
native width point at point 1's time. Success requires exactly one new entity
and verified native identity and three-point readback. One- and two-point
behavior must remain unchanged.

Bounded scanner pagination is owned by `tradingview-scanner`, with CLI option
routing only in `tradingview-cli`. One page remains capped at 100 rows. The
aggregate must reuse one configured HTTP client, stay within 10,000 rows and
100 requests, require integer provider totals, deduplicate in first-seen order,
report sequential drift, and return no partial successful aggregate after any
page failure.

Four separately managed gates execute Rust-generated production JavaScript
with Node.js `24.18.0`: study-value identity, Pine open/save, indicator
insertion, and three-point drawing. They are wired into `mise.toml`, CI, and
release workflow dependencies. Ordinary `cargo test --workspace` must remain
Rust-only and not require Node.js on `PATH`.

Current project state is recorded in `docs/v0.28-roadmap.md`,
`docs/v0.28-work-items.md`, `docs/plans/README.md`, this plan, and local
`CONTINUITY.md`. Stable guidance includes `README.md`, source taxonomy,
observation workflows, development and internal API docs, packaged guidance,
and runtime skills under `.agents/skills/`.

## Plan of Work

First, wait for the bounded scanner pagination ExecPlan to complete and pass
independent implementation review. Then inspect `v0.27.0..HEAD` and classify
every changed production module,
manifest, workflow, test, document, and skill. Verify that versions and third-
party dependencies have not changed prematurely and that no deferred feature
entered production.

Second, trace each promoted area end to end through validation, mutation,
readback, normalization, payload shaping, diagnostics, help, docs, and tests.
Check fail-closed behavior, exactly-once mutation, finite deadlines, ambiguity,
cleanup, preserved compatibility, and public-safe output.

Third, measure production and test boundaries in changed modules. Determine
whether Pine operations share a coherent runtime boundary, indicator insertion
owns one mutation path, launch construction and result classification remain
clear, and drawing request validation, page observation, and Rust result
validation remain separable. Inspect JavaScript gates for divergent policy. Do
not recommend a split based on size alone.

Fourth, compare CLI help, README, stable docs, packaged guidance, skills,
roadmap, inventory, plan index, and changelog with behavior. Confirm explicit
Desktop-backed mutation and Pine account-linked boundaries, and ensure no text
implies source mixing, recommendation, fallback, or unshipped feasibility work.
Keep skill Core Workflow sections concise and route detail to references.

Fifth, apply only small help, test, naming, metadata, or documentation fixes
that preserve reviewed contracts. If a larger defect appears, stop release
readiness, document the blocker, and create a dedicated follow-up plan.

Finally, record the architecture verdict and evidence here. Prepare a read-only
review prompt covering every promoted and deferred boundary. Archive this plan
only after independent review is green.

## Concrete Steps

Run from the repository root:

    git diff --stat v0.27.0..HEAD
    git diff --name-status v0.27.0..HEAD
    git diff -- Cargo.toml Cargo.lock crates .github mise.toml
    find crates/cli/src crates/model/src crates/cdp/src -type f -name '*.rs' -print0 | xargs -0 wc -l | sort -nr | head -40
    rg -n "pine_open|pine_save|createStudyInserter|ELECTRON_RUN_AS_NODE|price3|time3|parallel_channel" crates .github mise.toml
    rg -n "v0\.28|pine open|pine save|indicator add|ELECTRON_RUN_AS_NODE|price3|time3|parallel_channel|MSIX|right-offset|source mixing|ranking|recommendation" README.md CHANGELOG.md docs .agents/skills packaging/agent/AGENTS.md
    rg -n "TODO|FIXME|panic!|unimplemented!|todo!" crates docs README.md AGENTS.md CLAUDE.md packaging/agent/AGENTS.md

Run focused Rust tests:

    cargo test -p tradingview-cli pine -- --nocapture
    cargo test -p tradingview-cli indicator -- --nocapture
    cargo test -p tradingview-cli launch -- --nocapture
    cargo test -p tradingview-model drawing -- --nocapture
    cargo test -p tradingview-cli drawing -- --nocapture
    cargo test -p tradingview-cli --test cli_contract_desktop pine -- --nocapture
    cargo test -p tradingview-cli --test cli_contract_desktop indicator -- --nocapture
    cargo test -p tradingview-cli --test cli_contract_desktop launch -- --nocapture
    cargo test -p tradingview-cli --test cli_contract_desktop draw_shape -- --nocapture

Run every pinned JavaScript gate and the complete baseline:

    mise run check:study-values-js
    mise run check:pine-open-js
    mise run check:indicator-insertion-js
    mise run check:three-point-drawing-js
    mise run check:baseline
    cargo metadata --no-deps --format-version 1
    python3 scripts/check-public-hygiene.py --self-test
    python3 scripts/check-public-hygiene.py
    bash -n scripts/stage-release-package-files.sh
    cmp -s AGENTS.md CLAUDE.md
    ruby -e 'require "yaml"; YAML.load_file(".github/workflows/ci.yml"); YAML.load_file(".github/workflows/release.yml"); puts "workflow YAML parsed"'
    git diff --check

The workflow parser uses Ruby's standard-library YAML implementation and adds
no repository or production dependency. The audit environment must provide a
`ruby` executable; without it this acceptance check remains incomplete. After
parsing, inspect both workflows and confirm every release build depends on all
JavaScript gates. Do not run ignored live probes or invoke Pine, indicator-add,
launch, or drawing mutations against TradingView Desktop.

## Validation and Acceptance

Acceptance requires a classified candidate diff and end-to-end audit of every
promoted area. There must be no hidden fallback, source mixing, raw private
data, stale current-state documentation, or unreviewed feasibility work
represented as shipped.

Every focused command must execute at least one relevant test. The Rust
baseline must pass without Node.js. All four JavaScript gates must pass with
pinned Node.js `24.18.0`, and CI/release builds must depend on them. Hygiene,
workflow parsing, package syntax, guide parity, metadata, and diff checks must
pass.

The final plan must state one result: no release-blocking architecture issue;
small fixes applied with no dedicated refactor required; or release readiness
blocked by a named refactor. Size alone is not a blocker. Independent review
must report no unresolved finding before release readiness begins.

## Idempotence and Recovery

Inspection and validation are non-mutating and repeatable. Do not reset, stash,
apply, or drop the existing `fable-plan` or
`recovered-indicator-search-prototype-2026-07-12` stash. Do not repeat live
mutations, launch or kill TradingView, create drawings, save Pine scripts, or
insert studies.

If a deterministic gate fails, preserve evidence, fix only the owning contract,
and rerun focused checks before the baseline. If live evidence appears
contradictory, mark the claim `UNCONFIRMED` and create a separate approved
investigation instead of improvising a live operation.

## Artifacts and Notes

Record concise counts, contract markers, module boundaries, and pass/fail
summaries. Never record raw JSON, Runtime payloads, source text, target IDs,
account-local IDs, machine-specific paths, or private live values.

Audit evidence:

- Candidate range: `v0.27.0..b28603a` before the audit correction worktree.
- Candidate size at `b28603a`: 58 changed paths, approximately 10,871
  insertions and 367 deletions across 29 commits.
- Dependency classification: direct `clap` update `4.6.1` to `4.6.2` plus
  compatible lockfile updates; no new production dependency or feature flag.
- Focused results: Pine 45 passed and 1 ignored; indicator 22 passed and 1
  ignored; launch 21 passed; model drawing 7 passed; CLI drawing 26 passed;
  scanner 20 passed.
- JavaScript gates: study values, Pine open/save, indicator insertion, and
  three-point drawing all passed with pinned Node.js `24.18.0`.
- Full baseline after corrections: CLI unit tests reported 442 passed and 3
  ignored; Desktop CLI
  contracts reported 99 passed; scanner reported 36 passed; all other
  workspace and doc tests passed.
- Public hygiene inspected 595 tracked files. Metadata, package-script syntax,
  workflow YAML parsing, contributor-guide parity, and diff checks passed.
- No live TradingView operation or ignored mutation probe was executed.

The reviewer prompt must identify the commit range and validation commands,
request findings in severity order, and require an explicit release-readiness
go/no-go. Do not retain a one-off reviewer prompt after review unless it has
durable reusable value.

## Interfaces and Dependencies

This audit introduces no production interface or dependency. Existing JSON
envelopes, explicit Desktop-backed metadata, Pine binding fields, indicator
result typing, launch warning/error semantics, and drawing payload fields stay
authoritative. Any new command, option, payload semantic, fallback, source,
production dependency, or live mutation requires a separate plan and approval.

## Open Questions

No critical planning question is open. Local inspection and independent review
found no inspected module that needs a release-blocking refactor. Focused
independent re-review of the corrections is the remaining audit gate. Windows
MSIX behavior and finite-`f64` right-offset restoration remain `UNCONFIRMED`
and are not release prerequisites unless existing shipped behavior is found to
depend on them.

Revision note (2026-07-16): created this audit after reviewed native three-
point drawing support completed the promoted feature scope. It freezes features,
makes every executable JavaScript gate explicit, and separates small audit
corrections from dedicated refactors and deferred feasibility work.

Revision note (2026-07-16): completed local architecture inspection and all
deterministic validation. No release-blocking architecture issue was found;
focused independent review is pending.

Revision note (2026-07-16): independent review found Pine source-operation
diagnostic leakage and stale candidate/ledger evidence after dependency commit
`b28603a`. The local sanitizer and evidence corrections are applied; focused
re-review is pending.
