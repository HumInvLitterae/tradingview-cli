# Stabilize Windows CDP connection-refusal fixtures

This ExecPlan is a living document. Keep `Progress`, `Surprises & Discoveries`,
`Decision Log`, and `Outcomes & Retrospective` current while working. Maintain
this document according to `.agents/PLANS.md`.

## Purpose / Big Picture

The `tradingview-cdp` crate has deterministic local HTTP tests for connection
refusal, timeout, keep-alive reuse, malformed responses, and remote status
handling. The production transport behavior is correct on Linux and macOS, but
the Windows CI job currently fails in the two refusal tests because parallel
loopback fixtures can reuse a just-released ephemeral port. The request then
reaches another fixture and reports a timeout instead of the expected
connection failure.

After this change, the same test suite will retain its existing error taxonomy
on all CI operating systems. The change is test-only: users will not see a new
CLI option, payload field, dependency, or transport policy. The observable proof
is a green Windows CI test job for both refusal tests together with the existing
full workspace baseline.

This repository currently includes two dependency-maintenance commits after the
canonical-history rewrite. They are part of the release candidate and must be
preserved; this plan does not perform another dependency update.

## Progress

- [x] (2026-07-12) Confirmed the rewritten primary clone is clean and canonical
  sanitation is closed.
- [x] (2026-07-12) Confirmed the current local `main` includes the two
  dependency-maintenance commits after the rewrite.
- [x] (2026-07-12) Reproduced the current local CDP transport and workspace
  baseline successfully.
- [x] (2026-07-12) Confirmed GitHub CI fails only in the Windows test job for
  `target_list_connection_refusal_remains_connection` and
  `version_probe_connection_refusal_remains_not_ready`; Format, Clippy,
  Ubuntu, macOS, and script checks are green.
- [x] (2026-07-12) Added the test-only async fixture lock and its Tokio
  development feature.
- [x] (2026-07-12) Updated the current roadmap, work inventory, plan index,
  changelog, recovery notice, and archived sanitation-plan references.
- [x] (2026-07-12) Ran focused repeat tests and the full local baseline after
  the change.
- [x] (2026-07-12) Prepared an independent read-only review prompt without
  pushing.
- [x] (2026-07-12) Received independent review findings that the module-local
  lock omitted `client.rs` loopback fixtures and that CHANGELOG / roadmap
  wording overstated or lagged the current state.
- [x] (2026-07-12) Moved the lock into crate-wide test support, made every CDP
  loopback-listener allocation participate, and corrected the documentation
  wording.
- [x] (2026-07-12) Stressed the complete `tradingview-cdp` test binary with
  high parallelism, reran the full baseline, and prepared the focused
  re-review handoff.
- [x] (2026-07-12) Received focused independent re-review with no remaining
  findings.
- [x] (2026-07-12) Committed the crate-wide test correction as `0fb3f1e` with
  `test(cdp): Serialize loopback test fixtures`.
- [x] (2026-07-12) Committed the roadmap, changelog, ExecPlan, and archive
  transition in the documentation commit; request explicit approval before a
  normal push.
- [ ] Verify Windows CI is green after the authorized push.

## Surprises & Discoveries

- Observation: Windows-only CI failures occur in the two tests that drop a
  loopback listener and immediately connect to its old port.
  Evidence: the Windows job reports `Timeout` where both tests assert
  `Connection`/not-ready, while the same 32 CDP tests pass on Ubuntu and macOS.

- Observation: the current local dependency state is not the cause of a local
  regression.
  Evidence: focused CDP transport tests, strict Clippy, and the full workspace
  test suite pass after the `tokio-tungstenite` and lockfile maintenance commits.

- Observation: all seven local HTTP fixture tests bind ephemeral loopback ports
  and run in one test module.
  Evidence: `transport.rs` contains seven `TcpListener::bind("127.0.0.1:0")`
  tests; the two refusal tests are the only ones that release a port before the
  client request.

- Observation: serializing the complete local HTTP fixture group is stable on
  the current platform under repeated parallel execution.
  Evidence: the focused transport suite passed 25 consecutive runs with
  `--test-threads=16`, including both refusal assertions on every run.

- Observation: the first lock was not crate-wide and the focused transport
  repeat excluded competing WebSocket fixtures in `client.rs`.
  Evidence: independent review identified two additional listener allocation
  sites in the same `tradingview-cdp` test executable. The initial 25-run result
  remains historical evidence, but it is not sufficient acceptance evidence.

- Observation: placing crate-wide test support before public re-exports trips
  strict Clippy's `items_after_test_module` lint.
  Evidence: the first strict Clippy run failed only on module placement. Moving
  the `#[cfg(test)]` support after the re-exports preserved behavior and made
  the strict workspace check green.

## Decision Log

- Decision: serialize every ephemeral loopback-listener allocation in the CDP
  test executable with one crate-wide `tokio::sync::Mutex` held for the full
  fixture lifetime.
  Rationale: the refusal tests must not race with HTTP fixtures in
  `transport.rs` or WebSocket fixtures in `client.rs` that can receive a reused
  port. The shared test support is explicit, deterministic, and avoids relying
  on OS-specific port allocation behavior. The async mutex can be held across
  awaits without triggering the Clippy warning associated with a blocking
  mutex.
  Date/Author: 2026-07-12 / Codex

- Decision: preserve the existing refusal and timeout assertions.
  Rationale: a test workaround must not hide a real transport-classification
  regression by accepting `Timeout` as an alternative to `Connection`.
  Date/Author: 2026-07-12 / Codex

- Decision: enable Tokio `sync` only in the `tradingview-cdp` dev-dependency.
  Rationale: the lock is test infrastructure and must not expand the
  production feature set or public dependency surface.
  Date/Author: 2026-07-12 / Codex

## Outcomes & Retrospective

The first implementation passed local validation but did not cover competing
`client.rs` loopback fixtures. Independent review blocked approval and also
identified two status-wording drifts. The crate-wide correction, wording
updates, complete-CDP stress validation, full local baseline, focused re-review,
and test-only commit `0fb3f1e` are complete. The documentation transition is
also committed. Windows CI confirmation still requires a later authorized
push.
If Windows remains red after the shared lock, stop and investigate the fixture
or operating-system behavior in a new correction wave; do not weaken production
error classification or skip the tests.

## Context and Orientation

`crates/cdp/src/transport.rs` owns the CDP HTTP session and seven local HTTP
listener allocation sites. `crates/cdp/src/client.rs` owns CDP WebSocket tests
with two additional listener allocation sites, one of which is reused by four
tests through `connected_test_client()`. All run in the same crate test binary.
A connection refusal means that no process is listening at the target endpoint;
a timeout means the endpoint did not complete the request before the configured
deadline. They are distinct public failure categories and the assertions must
remain distinct.

`crates/cdp/Cargo.toml` has a normal dependency on Tokio with network and time
features, plus a test-only Tokio dependency. The test-only dependency will gain
the `sync` feature to provide an asynchronous mutex. No production Rust module,
CLI command, JSON contract, or runtime dependency changes.

The completed canonical-history sanitation plan is archived under
`docs/plans/archives/`. The current roadmap and work inventory must point to
this plan until the Windows CI blocker is green. After that, a separate
`v0.26.0 release readiness` ExecPlan can be created.

## Plan of Work

First update the plan index, `docs/v0.26-roadmap.md`,
`docs/v0.26-work-items.md`, `CHANGELOG.md`, and the archived sanitation plan so
the durable project state says sanitation is complete and Windows fixture
stabilization is the current pre-release blocker. Keep the recovery notice
accurate: canonical history is rewritten and the primary maintainer clone has
already been realigned, while private rollback bundles remain retained.

In `crates/cdp/Cargo.toml`, add Tokio's `sync` feature only to
`[dev-dependencies]`. In crate-wide `#[cfg(test)]` support under
`crates/cdp/src/lib.rs`, define a private async `loopback_fixture_lock()` using
a static `tokio::sync::Mutex<()>`. Acquire its guard before every local listener
allocation in `transport.rs` and `client.rs`. The shared client helper returns
the guard so callers retain it for the entire WebSocket fixture lifetime.

Do not change `CdpHttpSession`, request timeouts, error mapping, endpoint
selection, error details, or any assertion. Do not add retries, sleeps,
platform skips, or a fallback from connection failure to timeout.

After local validation, leave a private review prompt that asks for a read-only
review of the test-only lock, the current CI evidence, the sanitation closeout
docs, and the absence of production-contract changes. Commit the docs transition
and test fix separately after review. Do not push until the project owner gives
explicit approval for the normal branch push.

## Concrete Steps

Work from the repository root. The expected sequence is:

    cargo test -p tradingview-cdp transport -- --nocapture
    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1
    git diff --check

After the fixture lock is implemented, repeat the complete `tradingview-cdp`
crate tests under normal parallel test execution at least 25 times with 16 test
threads. Every run must include the transport and client fixtures, report both
refusal tests as passing, and show no timeout classification in their output.

Run the repository hygiene and packaging syntax checks:

    python3 scripts/check-public-hygiene.py --self-test
    python3 scripts/check-public-hygiene.py
    bash -n scripts/stage-release-package-files.sh
    cmp -s AGENTS.md CLAUDE.md

After an owner-approved normal push, inspect the CI run for the new commit. The
Windows test job must pass both refusal tests; all other jobs must remain green.

## Validation and Acceptance

Acceptance requires all of the following:

- The complete CDP test binary passes repeatedly on the local platform with
  transport and client fixtures participating in the same stress runs.
- The complete CDP suite passes at least 25 consecutive runs with 16 test
  threads after the crate-wide fixture lock is added.
- Strict Clippy reports no warning from holding the async mutex guard across
  fixture awaits.
- The full workspace tests, formatting, metadata, hygiene, and script checks
  pass locally.
- The Windows CI test job passes
  `target_list_connection_refusal_remains_connection` with `Connection` and
  exit code 2, and
  `version_probe_connection_refusal_remains_not_ready` with `Ok(None)`.
- The Linux and macOS CI test jobs remain green.
- `git diff` shows only the test-only mutex, its dev feature, the plan/docs
  transition, and the changelog entry. There is no production transport diff,
  new runtime dependency, CLI contract change, or package version change.
- No tracked document contains raw payloads, credentials, account-local data,
  machine-specific paths, or one-off private review prompts.

Corrected local result (2026-07-12): the complete 32-test CDP suite passed 25
consecutive runs with 16 test threads, including all transport and client
fixtures. Formatting, strict Clippy, the full workspace suite, metadata, public
hygiene self-test/check, release-script syntax, guide parity, and
`git diff --check` also passed. The dedicated Windows CI confirmation is not yet
available in this unpushed worktree.

If Windows still reports a timeout, the implementation is not accepted. Preserve
the failure evidence, do not skip the test or broaden the error assertion, and
create a focused follow-up after determining whether another local fixture or
the OS socket behavior is responsible.

## Idempotence and Recovery

The async lock is additive test infrastructure and can be applied repeatedly
without changing runtime behavior. If a test edit is incorrect, revert only the
test-lock hunk and dev-feature hunk while preserving the current dependency
commits and sanitation closeout docs. Do not reset the branch or rewrite any
history. The existing `main-backup` branch and private rollback bundles are
unrelated safety artifacts and must remain untouched.

## Artifacts and Notes

The pre-fix CI evidence is:

    Windows: two refusal tests failed with Timeout
    Ubuntu:  all workspace tests passed
    macOS:   all workspace tests passed
    Clippy:  passed
    Format:  passed
    Script checks: passed

The post-fix evidence must add a successful Windows run and must not include raw
CI environment paths or account-local identifiers in tracked documents.

## Interfaces and Dependencies

The only code-level interface added is crate-private test support:

    async fn loopback_fixture_lock() -> tokio::sync::MutexGuard<'static, ()>

It is defined and used only inside `#[cfg(test)]` code in `tradingview-cdp`.
Tokio's `sync` feature is enabled only for that crate's dev-dependency.
Production builds, public Rust APIs, CLI output, HTTP deadlines, error kinds,
and exit codes remain unchanged.

## Open Questions

There are no implementation choices left for this slice. The only external
gate is owner approval for the normal push needed to obtain Windows CI evidence.
Rollback bundle deletion remains a separate post-release owner decision.

Revision note (2026-07-12): created after canonical history sanitation closed
and the first post-rewrite CI run showed two Windows-only refusal-fixture
failures. The current local dependency state passes the full baseline; the plan
isolates the remaining cross-platform test reliability issue before release
readiness.
