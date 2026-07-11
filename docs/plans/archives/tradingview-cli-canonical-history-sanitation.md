# Sanitize machine-specific paths from canonical Git history

This ExecPlan is a living document. Keep `Progress`, `Surprises & Discoveries`,
`Decision Log`, and `Outcomes & Retrospective` current while working. Maintain
this document in accordance with `.agents/PLANS.md`.

## Purpose / Big Picture

This slice removes a machine-specific local username from every canonical
`main` and release-tag history reachable through the GitHub repository. The
current tracked tree is already clean, but old commits and release tags still
expose the historical text. A normal follow-up commit cannot alter those old
objects, so this work requires rewriting commit and tag identifiers and then
force-updating the canonical refs.

The realistic success condition is that `main` and all release tags on the
canonical GitHub repository no longer reach a blob containing the reported
username. This cannot erase third-party clones, forks, quoted commit IDs, or
provider caches. Those limits must be communicated explicitly.

## Progress

- [x] (2026-07-11) Completed and independently reviewed the current-tree cleanup.
- [x] (2026-07-11) Committed the current-tree cleanup as `97949d7`.
- [x] (2026-07-11) Confirmed `git-filter-repo` is installed locally.
- [x] (2026-07-11) Inventoried one canonical branch, 28 tags, and 27 GitHub Releases.
- [x] (2026-07-11) Confirmed `v0.1.0` is the only annotated tag and is unsigned; all other tags are lightweight.
- [x] (2026-07-11) Reduced historical content to two replacement forms and 29 unique text lines.
- [x] (2026-07-11) Completed a disposable all-ref dry-run after excluding local `refs/codex` and remote-tracking refs.
- [x] (2026-07-11) Verified the dry-run keeps 29 canonical refs, 28 tags, the current HEAD tree, and zero sensitive matches.
- [x] (2026-07-11) Confirmed the rewrite prunes only empty commit `3ad570b`, reducing main from 393 to 392 commits.
- [x] (2026-07-11) Confirmed the release workflow is active and GitHub repository rulesets are empty.
- [x] (2026-07-11) Made this plan current in the plan index, roadmap, work inventory, changelog, and continuity ledger.
- [x] (2026-07-11) Used a self-contained read-only plan-review prompt, then
  removed transient review prompts from the tracked tree after their gates
  closed.
- [x] (2026-07-11) Deleted the first disposable mirror and replacement file after recording public-safe aggregate evidence.
- [x] (2026-07-11) Received an independent plan review with three mutation-safety blockers and four documentation/verification findings.
- [x] (2026-07-11) Revised the plan to use exact remote-ref backups,
  per-ref force-with-lease checks, mechanical workflow recovery, canonical
  release/tag manifests, a two-OS guard, and executable clone recovery.
- [x] (2026-07-11) Received focused re-review with three additional findings:
  a self-rejecting guard example, a pre-trap workflow-disable window, and a
  missing post-filter canonical-remote setup.
- [x] (2026-07-11) Removed the guard example, armed restoration before disable,
  added a post-disable run recheck, and fixed the rewrite remote identity flow.
- [x] (2026-07-11) Obtained focused independent re-review of all ten plan
  corrections with no remaining findings.
- [x] (2026-07-11) Added the cross-platform tracked-tree hygiene guard, its
  self-tests, and unconditional Ubuntu/Windows CI execution.
- [x] (2026-07-11) Added downstream fresh-clone and explicit realignment
  recovery guidance.
- [x] (2026-07-11) Independently reviewed and committed the guard, notice, and
  execution plan as `0f0e42e`.
- [x] (2026-07-11) Completed a full preflight rehearsal from `0f0e42e`,
  including rewrite, exact remote backup, manifests, and leased atomic push
  dry-run, without external mutation.
- [x] (2026-07-11) Established the final freeze protocol: the tracked plan uses
  state-independent invariants, while exact freeze OIDs and generated evidence
  live in private `summary.json` and the local continuity ledger.
- [x] (2026-07-11) Completed a final-freeze rehearsal of the disposable
  rewrite, manifests, rollback bundle, dry-run, and exact mutation scripts.
- [x] (2026-07-11) Aligned the documented rollback remote and bundle-manifest
  ordering with the private generator. Exact current-freeze regeneration state
  remains in ignored `summary.json` and the local continuity ledger.
- [x] (2026-07-11) Independently reviewed the final private preflight
  artifacts and exact mutation scripts.
- [x] (2026-07-11) Obtained explicit project-owner approval for workflow
  disable, atomic force-push of main and tags, and workflow re-enable.
- [x] (2026-07-11) Temporarily disabled the release workflow and verified its
  disabled state.
- [x] (2026-07-11) Atomically force-updated canonical `main` and all 28 tags
  from the sanitized repository.
- [x] (2026-07-11) Re-enabled the release workflow and verified its active
  state.
- [x] (2026-07-11) Verified remote refs, all tag trees, GitHub Release
  IDs/assets, and fresh-clone history.
- [x] (2026-07-11) Confirmed no unexpected release workflows were triggered by
  the tag rewrite.
- [x] (2026-07-11) Obtained focused re-review of the post-rewrite
  documentation corrections and closeout evidence.
- [x] (2026-07-11) Retained the private rollback bundles through the v0.26
  release as explicitly approved; deletion remains a separate owner decision.
- [x] (2026-07-11) Recorded downstream recovery instructions and completed
  canonical sanitation closeout before release-readiness planning.

## Surprises & Discoveries

- Observation: a local mirror clone copied Codex-internal tree refs.
  Evidence: `refs/codex/turn-diffs/...` appeared beside `main` and tags, and
  `git-filter-repo` warned that those refs pointed to trees instead of commits.
  Canonical sanitation must exclude those local refs and push only `main` and
  `refs/tags/*`.

- Observation: the historical text has only two semantic forms.
  Evidence: deduplicating all matching lines across reachable history produced
  29 unique lines: validation regexes with a redundant specific-user branch,
  and commands invoking one complete Codex skill-validator script path.

- Observation: one prior cleanup commit becomes empty after rewriting history.
  Evidence: the dry-run commit map sends `3ad570b` to the zero object. The
  current-tree cleanup commit remains meaningful, while main commit count moves
  from 393 to 392.

- Observation: force-updating existing `v*` tags can trigger the release
  workflow 28 times.
  Evidence: `.github/workflows/release.yml` runs on every `v*` tag push and is
  currently active. It must be temporarily disabled around the atomic ref
  update and restored afterward.

- Observation: a tracked symlink can point to a directory in a developer
  checkout, so reading every `git ls-files` path as a regular file fails.
  Evidence: the first guard scan encountered the tracked `.claude/skills`
  symlink. The guard now reads symlink target text without following it, skips
  deleted index entries in an in-progress worktree, and still scans regular
  tracked files and Windows checkouts where symlinks may be materialized as
  plain files.

- Observation: the final preparation commit increases source history by one,
  while the same historical cleanup commit remains the sole prune candidate.
  Evidence: the preflight rehearsal rewrote 394 source commits to 393, preserved
  the current tree, and mapped only `3ad570b` to the zero object.

- Observation: the annotated `v0.1.0` target commit is rewritten, so its tag
  object OID changes even though its metadata is preserved.
  Evidence: the object target follows the commit map while tag name, type,
  tagger, and complete message remain byte-identical.

- Observation: recording an exact freeze commit in the same tracked commit is
  self-referential and would force endless regeneration.
  Evidence: every tracked status update creates a new source commit and changes
  the rewritten tip and commit count. Stable docs now record invariants only;
  the ignored private summary and local continuity ledger record exact current
  values until post-rewrite closeout.

## Decision Log

- Decision: rewrite canonical `main` and every release tag before v0.26 release
  readiness.
  Rationale: the project owner requires historical refs, not only the current
  tree, to stop exposing the machine-specific path.
  Date/Author: 2026-07-11 / User and Codex

- Decision: perform transformation and push from a disposable sanitized bare
  repository while preserving the main working copy until remote verification.
  Rationale: this avoids processing Codex-internal refs and provides a clean,
  auditable ref set. The working repository can be realigned only after the
  canonical push is confirmed.
  Date/Author: 2026-07-11 / Codex

- Decision: generate the replacement file in a private temporary directory and
  never track the legacy username or replacement file.
  Rationale: embedding the removed value in the ExecPlan, scripts, or guard
  would recreate the disclosure.
  Date/Author: 2026-07-11 / Codex

- Decision: preserve current tree content exactly and accept pruning of empty
  commit `3ad570b`.
  Rationale: the commit only performed a correction that becomes true in its
  ancestors after rewriting; retaining an empty commit adds no information and
  is not worth custom filter behavior.
  Date/Author: 2026-07-11 / Codex

- Decision: use an atomic force-push and temporarily disable only the release
  workflow. Protect every updated ref with an explicit
  `--force-with-lease=<ref>:<captured-old-oid>` and use only exact refspecs.
  Rationale: `--atomic` prevents a partial update but does not by itself reject
  a concurrent update made after manifest capture. Per-ref leases close that
  race. Disabling the tag-triggered publisher avoids 28 duplicate
  build/release attempts. Bare `--force`, `--mirror`, `--all`, `--tags`, and
  wildcard refspecs are forbidden.
  Date/Author: 2026-07-11 / Codex

- Decision: do not promise deletion from forks, clones, external links, or
  GitHub object caches.
  Rationale: force-updating refs only changes canonical reachability. Provider
  support may be requested separately if cache removal becomes necessary.
  Date/Author: 2026-07-11 / Codex

- Decision: make fresh clone recovery the documented default and keep
  existing-clone realignment as a separately confirmed fallback.
  Rationale: old and rewritten histories must not be merged. A fresh clone
  gives the clearest canonical boundary, while the fallback preserves a path
  for users who first create a private bundle and explicitly fetch rewritten
  refs into a temporary namespace.
  Date/Author: 2026-07-11 / Codex

- Decision: keep one-off reviewer prompts outside the tracked repository.
  Rationale: acceptance criteria and outcomes belong in the ExecPlan, while a
  point-in-time handoff prompt becomes stale and adds no durable evidence
  without its review result. Active prompts live with ignored preflight
  artifacts and are removed after use.
  Date/Author: 2026-07-11 / Codex

## Outcomes & Retrospective

Planning and the first local dry-run are complete. The dry-run rewrote the
expected refs, removed all sensitive matches, preserved the current HEAD tree,
kept all 28 tags, passed `git fsck`, and pruned only the expected empty commit.

The destructive external phase completed after explicit project-owner approval.
The reviewed script disabled the release workflow, atomically moved canonical
`main` and all 28 tags with exact per-ref leases, and restored the workflow to
`active`. All 29 live refs match the rewritten manifest; the normalized 27-
Release / 135-asset manifest is unchanged; no release workflow run was created;
and rollback was not needed.

A fresh canonical clone preserves the expected tree, has no reachable legacy
path or concrete username alternation, and passes the hygiene guard, formatting,
strict clippy, workspace tests, metadata, fsck, and diff checks. The first
post-rewrite review found no mutation or validation defect, and the subsequent
documentation correction passed focused re-review. The reviewed correction was
committed as `e18f43f` and pushed normally. Canonical history sanitation is
closed; the primary clone was then realigned to the rewritten main and canonical
tags while preserving local backup refs and private rollback bundles.

## Context and Orientation

The latest public release is `v0.25.0`. The pre-rewrite cleanup tip was
`97949d7`; canonical `main` now points to the rewritten history. The canonical
remote is the GitHub `origin`; only `main` is a normal branch. There are 28 Git
tags and 27 GitHub Releases because one historical tag has no release object.
The release workflow publishes on every `v*` tag push.

History rewriting changes commit identifiers because each descendant commit
contains a different parent or tree identifier. A tag that points into rewritten
history must also move. Existing clones will see unrelated-looking rewritten
refs and must not merge old and new history.

The local Codex workspace has internal refs under `refs/codex`. They are not
canonical GitHub refs and must neither be rewritten nor pushed. A disposable
bare repository should contain only `refs/heads/main` and `refs/tags/*` before
filtering.

## Plan of Work

First, add a recurrence guard as `scripts/check-public-hygiene.py`, using only
the Python standard library. It reads `git ls-files -z`, skips binary files,
and rejects concrete macOS user homes, username-specific detector
alternations, Linux user homes, and Windows user-profile paths. Detector
fragments must be assembled so the script does not match its own source. The
only exceptions are exact path-and-value pairs for the two existing synthetic
fixtures in `crates/cdp/src/transport.rs` and
`crates/cli/src/ops/status.rs`; both use the same synthetic TradingView app
window URL. A broad path, directory, username, or regex exception is forbidden.

Give the guard a deterministic `--self-test` mode that proves one allowed
fixture at each exact path, rejects the same value at another path, and rejects
representative macOS, Linux, and Windows user homes. Run self-test and the real
tracked-tree scan in the existing CI matrix without an OS condition, so both
Ubuntu and Windows execute the same commands with `python`. Document those
commands in `docs/development.md`.

Add a downstream migration notice under `docs/` explaining invalidated commit
IDs, preservation of release assets, and recovery. Recommend a fresh clone in
a new directory while preserving the old clone until unpushed work is audited.
Before any destructive local command, show checks for a clean worktree,
unpushed commits, local-only branches and tags, linked worktrees, and submodule
state, plus commands that create a local bundle or patches outside the clone.
State explicitly that users must never merge or pull old and rewritten history.
Unpushed work may be moved only after inspection, preferably as patches or by
cherry-picking selected old commits into the fresh clone. Explain that old
SHAs, PR links, local branches, worktrees, submodule pins, and bookmarks may no
longer name canonical objects.

Provide an optional existing-clone realignment procedure separately. Require a
clean worktree and a verified local backup first; fetch rewritten `main` and
tags with explicit refspecs; inspect the fetched state; then reset the local
branch and replace local canonical tags only after a second human confirmation.
Show the expected output or invariant before each destructive command. Do not
suggest a broad fetch, pull, merge, or automatic deletion of local-only refs.

After independent review and commit of those preparatory changes, create a
private temporary directory with mode 700. Derive the legacy home prefix from
historical blobs rather than writing it in tracked files. The derivation must
find exactly one concrete Codex home. Generate two literal `git-filter-repo`
replacement rules: remove the redundant specific-user regex branch while
keeping `/Users/`, and replace the complete skill-validator path with the
portable quoted `CODEX_HOME` form. Stop if discovery returns zero or multiple
legacy homes.

Create two separate bare repositories. The rewrite repository starts from the
final local pre-rewrite commit and contains only `refs/heads/main` and the 28
exact tag refs. The rollback repository starts empty and fetches the canonical
GitHub remote directly into canonical ref names: one explicit main refspec and
one explicit refspec for each tag captured from the remote manifest. It must
not inherit local refs. Create the mode-600 rollback bundle from those exact 29
refs, run `git bundle verify`, and require `git bundle list-heads` to match the
captured direct-ref names and OIDs byte-for-byte after canonical sorting.

Run `git-filter-repo` only in the rewrite repository. Verify ref count, tag
count and types, main tree equality, expected commit pruning, zero matches
across all rewritten refs, and `git fsck --full`.

Normal `git-filter-repo` execution removes `origin`. After filtering and all
local rewrite checks are green, require the rewrite repository to have no
remotes. Add the canonical GitHub URL under the dedicated name `canonical`,
using the repository identity captured before filtering. Require both
`git remote get-url canonical` and `git remote get-url --push canonical` to
equal that exact canonical URL, require `git remote` to print only
`canonical`, and require `git ls-remote canonical` to match the expected
canonical repository identity and 29-ref preflight manifest. No local working
repository, rollback repository, filesystem URL, or alternate GitHub
repository may remain configured as a fetch or push target. The dry-run and
real push must both use this same verified `canonical` remote name.

Capture `git ls-remote` for canonical main and tags immediately before push.
The tag manifest records each direct tag-object or commit OID, its peeled commit
OID, and its tree OID. For the sole annotated tag, additionally preserve and
verify its object type, tag name, tagger metadata, complete message, and peeled
target. Capture GitHub Releases as canonical sorted JSON containing each
release's `id`, `tag_name`, `target_commitish`, `draft`, and `prerelease`, and
each asset's `id`, `name`, `size`, and `digest`, with assets sorted inside each
release. The current expected inventory is 27 Releases and 135 assets; compare
the pre- and post-rewrite manifest byte-for-byte.

Capture the release workflow state and require `active`. Require zero queued or
in-progress runs for that workflow before disabling it. An atomic push dry-run
is supporting evidence only. The real push uses one exact refspec and one
explicit lease for each of the 29 captured refs. If the remote changes after
capture, a lease fails and the whole atomic push must fail without moving any
ref; rebuild all manifests and the rollback bundle instead of overriding it.

Present the final ref manifest, dry-run evidence, backup location, expected
commit-map effects, workflow-disable/re-enable operations, and push command to
the project owner. Do not disable workflows or push until explicit approval is
given in that turn.

After approval, run one reviewed temporary mutation script from the private
directory. It performs three external mutation phases: disable one workflow,
execute one atomic push containing all 29 exact ref updates, and enable the
workflow. Define and install the `EXIT`, `INT`, `TERM`, and `HUP` restoration
handlers before issuing the disable request, and arm recovery-required state
immediately before that request. The handler queries current workflow state and
idempotently enables and verifies it whenever state is not `active`. This also
covers a disable request that reached GitHub but returned a transport error to
the caller. Poll until the disabled state is observed, then require queued and
in-progress release-run lists to be empty a second time immediately before the
push. Keep an independently displayed emergency enable command available before
disabling. If automatic re-enable cannot be verified, stop all other work and
require the project owner to run and verify that command manually.

While the workflow is disabled, perform only the leased atomic push and the
minimal remote-ref check needed to know whether rollback is required. Enable
and verify the workflow before longer Release, clone, and test validation. Do
not clear the recovery trap until `active` has been observed. Verify remote refs
against local rewritten refs, compare the canonical Release manifest, ensure no
duplicate release workflow runs started, and clone the canonical repository
fresh to repeat history scans and baseline validation.

Keep the chmod-600 rollback bundle until remote and fresh-clone validation are
green. Ask the project owner before deleting it because deletion removes the
last immediate rollback route. Do not proceed to release readiness until a
post-rewrite independent review is green.

## Concrete Steps

Run preparatory and dry-run work from the repository root. Store sensitive
temporary values and repositories under a mode-700 temporary directory outside
tracked paths. Never echo the derived legacy home into tracked output.

The recurrence guard must pass locally and in CI. The final disposable rewrite
must satisfy:

    refs/heads/main plus 28 refs/tags entries
    exactly one fewer main commit than the frozen source after expected
    empty-commit pruning
    unchanged current HEAD tree
    zero legacy username matches across rewritten main and tags
    zero concrete username alternations across rewritten main and tags
    git fsck --full succeeds

Before any external mutation, show the project owner the exact atomic push
refspecs and leases and obtain approval. Generate arguments from the captured
manifest, then require exactly 29 lease arguments and 29 exact refspecs. The
command shape is:

    git push --atomic canonical \
      --force-with-lease=<canonical-ref>:<captured-old-oid> ... \
      <rewritten-local-ref>:<same-canonical-ref> ...

It must include only rewritten main and tags, never `refs/codex`, backup refs,
replace refs, remote-tracking refs, wildcards, or broad force flags.

The temporary mutation script must query the workflow by stable file/name or
ID, assert it is active, assert zero queued and in-progress runs, install and
arm the restoration handler, disable the workflow, poll for
`disabled_manually`, reassert zero queued and in-progress runs, perform the
leased push through the verified `canonical` remote, verify the 29 remote OIDs,
enable the workflow, and poll for `active`. Every failure and interrupt path
from immediately before the disable request onward must enter the same
restoration handler. The script exits nonzero if the original operation or
restoration fails and prints only public-safe state.

The final preflight records `REPOSITORY` from
`gh repo view --json nameWithOwner --jq .nameWithOwner` and addresses the
release workflow as `.github/workflows/release.yml`. These read-only API calls
are the required state probes:

    gh api "repos/${REPOSITORY}/actions/workflows/release.yml" --jq .state
    gh api --paginate \
      "repos/${REPOSITORY}/actions/workflows/release.yml/runs?status=queued&per_page=100"
    gh api --paginate \
      "repos/${REPOSITORY}/actions/workflows/release.yml/runs?status=in_progress&per_page=100"

Both run queries must normalize `.workflow_runs[].id`, sort numerically, and
produce an empty list. Execute them once before arming restoration and again
after `disabled_manually` is observed but before the push. The exact mutation
endpoints are:

    gh api --method PUT \
      "repos/${REPOSITORY}/actions/workflows/release.yml/disable"
    gh api --method PUT \
      "repos/${REPOSITORY}/actions/workflows/release.yml/enable"

The script defines `enable_and_verify` before mutation. Before the disable
request, it installs the traps and then arms restoration:

    trap 'enable_and_verify' EXIT
    trap 'exit 130' INT
    trap 'exit 143' TERM
    trap 'exit 129' HUP
    RESTORE_WORKFLOW=1
    gh api --method PUT \
      "repos/${REPOSITORY}/actions/workflows/release.yml/disable"

`enable_and_verify` first queries workflow state whenever
`RESTORE_WORKFLOW=1`. If state is not `active`, including an unknown state after
a failed disable response, it calls the enable endpoint idempotently and polls
with a finite deadline until state reads `active`. It preserves a nonzero
original exit status. After the normal enable call has been verified, it clears
restoration-required state and then removes the traps. The emergency command is
the same enable endpoint followed by the state probe; it must be printed for
the project owner before disable and saved in the private runbook. If state
does not become `active` by the deadline, report an incident and perform no
further GitHub mutation.

Generate the remote-ref manifest from two explicit read-only queries, normalize
peeled tag lines separately, and require one head, 28 direct tags, and one
peeled annotated-tag line:

    git ls-remote --heads origin refs/heads/main
    git ls-remote --tags origin 'refs/tags/*'

Initialize the rollback repository with `git init --bare`, add the canonical
URL as `canonical`, remove its default `remote.canonical.fetch` refspec, and
construct one `git fetch --no-tags canonical` invocation with 29 exact
`+<remote-ref>:<same-local-ref>` arguments generated from that manifest. Assert
that the bare repository contains exactly those 29 refs and no remote-tracking
or other refs. A count or OID mismatch aborts before bundling. Bundle only the
explicitly enumerated canonical refs, not a glob. Normalize
`git bundle list-heads` as `<oid><TAB><ref>` sorted by ref and compare it
byte-for-byte with the 29 direct-ref manifest.

Capture Releases with `gh api --paginate
"repos/${REPOSITORY}/releases?per_page=100"`. A private standard-library Python
normalizer emits UTF-8 canonical JSON with sorted keys and compact separators,
sorts releases by `(tag_name, id)`, sorts each asset by `(name, id)`, and retains
only the required release and asset fields. Require 27 releases and 135 assets
before and after, then use a byte comparison of the normalized files. Missing
or null asset `digest` is a preflight stop, not a reason to weaken the manifest.

The forward and rollback mutation scripts are generated into the mode-700
private directory, pass `bash -n`, and are reviewed as artifacts before the
owner approval request. They print the 29 ref names and old/new abbreviated
OIDs for confirmation but never print replacement rules or historical private
values. The final approval prompt states that the normal path issues one
workflow-disable request, one Git push request containing 29 leased updates,
and one workflow-enable request. API state polls are read-only. Rollback, if
needed, repeats those three mutation phases only after separate approval.

## Validation and Acceptance

Preparation is acceptable when the guard catches synthetic forbidden fixtures,
allows only the two exact synthetic fixture path-and-value pairs, and passes on
the current tree; CI invokes it; documentation explains downstream recovery;
and focused plus baseline validation is green.

Rewrite acceptance requires unchanged current tree content, all canonical refs
present, only the documented empty commit pruned, no sensitive text reachable
from rewritten main or tags, and a valid rollback bundle. Remote acceptance
requires a per-ref-leased atomic ref update, restored active workflow state,
exact remote/local ref agreement, byte-identical canonical GitHub Release and
asset manifests, and tag evidence that matches the rewrite commit map. Tag names
and types remain unchanged; lightweight-tag direct and peeled OIDs must equal
their mapped commits and trees; the annotated tag must preserve its tagger,
message, and target semantics. Remote acceptance also requires no duplicate
release publishing runs and a clean fresh clone that passes the guard and
normal workspace baseline.

Canonical reachability is the boundary. Forks, third-party clones, historical
SHA links, provider caches, and search-engine caches are residual external
risk. Record them without claiming global erasure.

## Idempotence and Recovery

Guard and dry-run operations are repeatable. Every disposable rewrite starts
from the final pre-rewrite repository and a newly generated replacement file.
Never reuse a partially filtered repository.

The atomic push prevents partial canonical ref movement, while explicit leases
prevent overwriting a ref changed after capture. If workflow disable succeeds
or may have reached GitHub but later preflight or push fails, the pre-armed trap
queries state and re-enables it before the script returns. A failed enable is an
incident: stop, preserve logs and backup, run the pre-displayed emergency
enable command, and verify `active` before any other mutation.

Rollback is a separate reviewed and explicitly approved mutation, not an
automatic branch of the forward script. It starts by verifying that the bundle
contains the exact pre-rewrite canonical refs, requires the workflow to be
active with no queued or in-progress release runs, disables it with the same
restoration trap, and executes one atomic push with 29 exact rollback refspecs
and 29 leases whose expected values are the sanitized remote OIDs. It verifies
the restored remote OIDs, re-enables the workflow, and polls for `active`.
Never rollback tags while the release workflow is active.

Do not delete the backup bundle automatically. Do not run garbage collection
on the primary working repository as part of this slice. Do not reset the
primary worktree until canonical remote verification is complete.

## Artifacts and Notes

The first disposable dry-run used two generated literal rules and produced:

    canonical refs: 29
    tags: 28
    main commits before: 393
    main commits after: 392
    sensitive matches after: 0
    concrete username alternations after: 0
    current tree preserved: yes
    pruned commit: 3ad570b only
    fsck: green

The preparatory guard implementation produced:

    self-test: passed
    current tracked files inspected: 556
    current-tree violations: 0
    clean fixture repository exit: 0
    forbidden-path fixture exit: 1
    missing allowlisted fixture exit: 1
    non-repository read failure exit: 2
    matched private value echoed in diagnostics: no
    exact synthetic fixture occurrences: 2 designated source files only

The guard and recovery notice have no Rust or Cargo diff from `97949d7`. Their
independent review was green and they were committed as `0f0e42e`.

The first complete preflight rehearsal from `0f0e42e` produced:

    current tree preserved: yes
    main commits before: 394
    main commits after: 393
    pruned commits: 1, expected cleanup commit only
    canonical refs: 29
    tags: 28
    annotated tag metadata preserved: yes
    legacy path matches: 0
    concrete username alternations: 0
    releases: 27
    assets: 135
    workflow state: active
    queued/in-progress release runs: 0/0
    rollback bundle mode: 0600
    replacement file mode: 0600
    leased atomic push dry-run: passed
    external mutation performed: no

These are historical rehearsal values. Exact current freeze OIDs, commit counts,
script hashes, and bundle hash live only in the ignored private summary and the
local continuity ledger until post-rewrite closeout.

The canonical repository has 27 GitHub Release objects and 135 assets. The
release workflow is active and idle. The reviewed leased atomic push confirmed
force-update permission and moved all 29 refs without partial movement. The
primary clone now tracks rewritten canonical `main`; `main-backup` retains the
pre-rewrite local tip without an upstream. The rollback evidence was fetched
from the pre-rewrite remote and remains available as private mode-0600 bundles.

## Interfaces and Dependencies

Use installed `git-filter-repo` for rewriting and `gh` for workflow and release
inspection. The recurrence guard uses Python 3 standard library only. No Rust
dependency, CLI surface, payload, source boundary, or package version changes.

The completed forward mutation was limited to temporarily disabling/re-enabling
one GitHub workflow and atomically force-updating canonical main and tag refs in
one reviewed script. Rollback remains exceptional and requires a second explicit
approval; it was not performed.

## Open Questions

The remaining owner decision is whether to delete the private rollback bundles
after the v0.26 release. GitHub cache purge is not part of canonical sanitation
and should be considered only if the project owner requires provider-level
removal beyond canonical ref reachability.

Revision note (2026-07-11): created after current-tree cleanup commit `97949d7`
and a successful disposable rewrite proof. Revised after two plan-review rounds,
updated when the recurrence guard and migration notice were implemented, and
updated again after their review/commit and the first full preflight rehearsal.
The final artifact review then identified a documentation/generator drift in
the rollback remote name and manifest sort key; this revision aligned the
procedure before regenerating the freeze. The reviewed forward mutation and
post-rewrite documentation closeout then completed successfully. The primary
clone was realigned only after remote verification, and backup bundles remain
retained by owner decision.
