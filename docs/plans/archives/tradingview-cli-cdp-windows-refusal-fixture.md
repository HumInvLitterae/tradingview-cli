# Stabilize Windows CDP transport-failure fixtures

This ExecPlan is a living document. Keep `Progress`, `Surprises & Discoveries`,
`Decision Log`, and `Outcomes & Retrospective` current while working. Maintain
this document according to `.agents/PLANS.md`.

## Purpose / Big Picture

The `tradingview-cdp` crate has local HTTP tests for connection/transport
failure, timeout, keep-alive reuse, malformed responses, and remote status
handling. Two tests created an unused endpoint by binding an ephemeral loopback
port, dropping the listener, and connecting to the released port. Windows CI
waits for the one-second connect timeout instead of returning an immediate
connection error. Serializing every CDP listener fixture did not change that
result, proving the dropped-listener fixture itself is platform-dependent.

The corrected tests keep a local listener alive, receive the complete HTTP
request, and then close without an HTTP response. This creates a controlled,
non-timeout transport failure without depending on released-port or firewall
semantics.

After this change, the same test suite will retain its existing error taxonomy
on all CI operating systems. The change is test-only: users will not see a new
CLI option, payload field, dependency, or transport policy. The observable proof
is a green Windows CI test job for both controlled-disconnect tests together
with the existing full workspace baseline.

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
- [x] (2026-07-12) Pushed the reviewed mutex correction and confirmed Windows
  CI still failed only the same two tests with one-second timeout errors.
- [x] (2026-07-12) Replaced the released-port fixtures with controlled
  post-request disconnect fixtures and removed the disproven mutex / Tokio
  `sync` test infrastructure.
- [x] (2026-07-12) Stressed the corrected complete CDP suite and ran the full
  local baseline.
- [x] (2026-07-12) Committed the controlled-disconnect test correction as
  `32321d2` with `test(cdp): Use controlled transport disconnect fixture`.
- [x] (2026-07-12) Deferred focused review of the correction wave until after
  the first Windows CI run, as directed by the project owner.
- [x] (2026-07-12) Committed the correction-wave documentation and requested
  explicit approval before the next normal push.
- [x] (2026-07-12) Pushed the controlled-disconnect correction and confirmed
  the complete 32-test CDP suite is green on Windows. The Windows job then
  failed two CLI integration tests that still used fixed port 9 as an assumed
  connection-refusal endpoint.
- [x] (2026-07-12) Replaced the two CLI port-9 fixtures with per-test
  controlled loopback disconnect servers while preserving their JSON and exit
  code assertions.
- [x] (2026-07-12) Passed the complete five-test CLI contract target 25
  consecutive times with 16 test threads and reran the full local baseline.
- [x] (2026-07-12) Committed the initial CLI fixture correction as `1b3af72`
  and pushed it for CI-first validation.
- [x] (2026-07-12) Confirmed run `29165315156` passes all 32 CDP tests and all
  five `cli_contract` tests on Windows, then fails 17
  `cli_contract_desktop` tests that still use fixed port 9. Two later
  `cli_contract_quote` tests had the same latent assumption.
- [x] (2026-07-12) Moved the reconnect-capable controlled-disconnect fixture
  into shared CLI integration-test support and applied it to every test that
  expects a CDP connection error. Validation-only port sentinels remain
  separate because they must prove no connection is attempted.
- [x] (2026-07-12) Passed the core, Desktop, and quote CLI contract targets,
  strict Clippy, the full workspace suite, metadata, public hygiene, packaging
  syntax, guide parity, and diff checks after the shared-fixture correction.
- [x] (2026-07-12) Received focused independent review with no implementation
  finding; the only finding was stale post-CI project-state documentation.
- [x] (2026-07-12) Verified run `29173925167` is green for Windows, Ubuntu,
  macOS, Clippy, Format, and both operating-system script-check jobs.

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

- Observation: serialization does not affect the Windows failures.
  Evidence: run `29163551913` executed all 32 CDP tests with the shared mutex;
  the two released-port tests still consumed approximately one connect-timeout
  interval and returned `Timeout`. All other Windows jobs and the other 30 CDP
  tests were green.

- Observation: a listener that reads the complete request and closes without a
  response produces the required non-timeout transport failure locally.
  Evidence: both corrected focused tests pass without accepting `Timeout` or
  changing production error mapping.

- Observation: the CDP correction works on Windows, but the same
  platform-dependent assumption also existed at the CLI binary boundary.
  Evidence: run `29164449455` passed all 32 `tradingview-cdp` tests, including
  both controlled-disconnect tests, then failed only
  `connection_failure_uses_structured_json_and_exit_code_2` and
  `readiness_connection_failure_uses_structured_json_and_exit_code_2` because
  fixed port 9 timed out and produced exit code 4 instead of 2.

- Observation: fixed port 9 was a suite-wide portability assumption, not just
  a two-test problem.
  Evidence: run `29165315156` passed the corrected five-test `cli_contract`
  target, then failed 17 tests in `cli_contract_desktop` with timeout / exit
  code 4. Source inspection found two additional connection-error assertions
  in `cli_contract_quote`, while other port-9 uses are validation sentinels
  that never connect.

- Observation: a one-connection server is insufficient for subprocess
  contracts because the HTTP client may reconnect after an EOF.
  Evidence: the first shared fixture made all assertions green but left the
  93-test desktop target at 171 seconds because retries reached a released
  port. Keeping the listener alive for the command lifetime and disconnecting
  every accepted request reduced the target to 37 seconds locally.

- Observation: the reconnect-capable shared fixture closes the Windows gate
  without weakening any assertion.
  Evidence: run `29173925167` passed all workspace tests on Windows, Ubuntu,
  and macOS together with Clippy, Format, and both script-check jobs. Focused
  review found no implementation issue.

## Decision Log

- Superseded decision: serialize every ephemeral loopback-listener allocation
  in the CDP test executable with one crate-wide `tokio::sync::Mutex` held for
  the full fixture lifetime.
  Rationale: the refusal tests must not race with HTTP fixtures in
  `transport.rs` or WebSocket fixtures in `client.rs` that can receive a reused
  port. The shared test support is explicit, deterministic, and avoids relying
  on OS-specific port allocation behavior. The async mutex can be held across
  awaits without triggering the Clippy warning associated with a blocking
  mutex.
  Date/Author: 2026-07-12 / Codex

- Decision: remove the shared mutex and model the failure with a controlled
  server-side disconnect after reading the full HTTP request.
  Rationale: Windows CI disproved the port-reuse hypothesis. Keeping the
  listener bound through accept removes released-port ambiguity; reading the
  request before closing avoids retry ambiguity and deterministically exercises
  the existing non-timeout transport-error branch.
  Date/Author: 2026-07-12 / Codex

- Decision: preserve the existing connection and timeout assertions.
  Rationale: a test workaround must not hide a real transport-classification
  regression by accepting `Timeout` as an alternative to `Connection`.
  Date/Author: 2026-07-12 / Codex

- Decision: use the same controlled post-request disconnect at the CLI
  subprocess boundary instead of fixed port 9.
  Rationale: port 9 is not a portable refused endpoint and can be silently
  filtered on Windows. A listener owned by each integration test proves the
  structured connection error and exit code without firewall assumptions.
  Date/Author: 2026-07-12 / Codex

- Decision: centralize a reconnect-capable fixture in CLI integration-test
  support and use it only for assertions that expect a connection failure.
  Rationale: all such tests require the same portable transport condition,
  including subprocess commands that trigger more than one HTTP attempt.
  Validation-before-network tests retain a non-listening sentinel so an
  accidental connection remains observable rather than being satisfied by the
  fixture.
  Date/Author: 2026-07-12 / Codex

- Superseded decision: enable Tokio `sync` only in the `tradingview-cdp`
  dev-dependency.
  Rationale: this was required only by the disproven mutex approach. The
  correction removes the feature again and adds no dependency.
  Date/Author: 2026-07-12 / Codex

## Outcomes & Retrospective

The first implementation passed local validation but did not cover competing
`client.rs` loopback fixtures. After that review finding was corrected, the
crate-wide mutex passed complete local stress and focused re-review, then was
committed as `0fb3f1e` and pushed with documentation commit `a95c3e0`. Windows
CI still failed the same two released-port tests, disproving the fixture-race
hypothesis.

The second correction removes the unnecessary serialization and uses a
controlled post-request disconnect. Runs `29164449455` and `29165315156`
confirm that correction is green in all 32 CDP tests on Windows. The latter run
also confirms the first two CLI contract corrections are green, then exposes
the remaining fixed-port assumption in 17 desktop contract tests. The current
wave centralizes a reconnect-capable controlled server for every CLI test that
expects a CDP connection error, including two quote contracts that had not yet
run. Existing public JSON and exit-code assertions remain unchanged. Focused
and full local validation, independent review, and cross-platform CI are green.
The Windows fixture blocker is closed and `v0.26.0` release readiness is the
next project slice.

## Context and Orientation

`crates/cdp/src/transport.rs` owns the CDP HTTP session and its local HTTP
fixtures. A non-timeout transport failure maps to `ErrorKind::Connection`; a
request that reaches its deadline maps to `ErrorKind::Timeout`. They are
distinct public failure categories and the assertions must remain distinct.
The corrected fixture demonstrates the former by accepting a request and
closing without an HTTP response. It no longer claims that a released ephemeral
port behaves identically on every operating system.

`crates/cdp/Cargo.toml` retains its existing test-only Tokio features; the
temporary `sync` feature is removed with the mutex. No production Rust module,
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

Remove the temporary Tokio `sync` dev feature, crate-wide test support, and
fixture guards. In `crates/cdp/src/transport.rs`, replace both released-port
tests with a private `transport_disconnect_fixture()`. The helper binds and
keeps a listener, accepts one request, reads through the HTTP header terminator,
and closes without writing a response. Rename the tests to describe transport
disconnect rather than portable connection refusal.

Do not change `CdpHttpSession`, request timeouts, error mapping, endpoint
selection, error details, or any assertion. Do not add retries, sleeps,
platform skips, or a fallback from connection failure to timeout.

After local validation, leave a private review prompt that asks for a read-only
review of the controlled disconnect, removal of the disproven mutex, current CI
evidence, sanitation closeout docs, and absence of production-contract changes.
Commit the docs transition and test fix separately after review. Do not push
until the project owner gives explicit approval for the normal branch push.

## Concrete Steps

Work from the repository root. The expected sequence is:

    cargo test -p tradingview-cdp transport -- --nocapture
    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1
    git diff --check

After the controlled-disconnect fixture is implemented, repeat the complete
`tradingview-cdp` crate tests under normal parallel test execution at least 25
times with 16 test threads. Every run must include the transport and client
fixtures and report both disconnect tests as passing without timeout
classification.

Run the repository hygiene and packaging syntax checks:

    python3 scripts/check-public-hygiene.py --self-test
    python3 scripts/check-public-hygiene.py
    bash -n scripts/stage-release-package-files.sh
    cmp -s AGENTS.md CLAUDE.md

After an owner-approved normal push, inspect the CI run for the new commit. The
Windows test job must pass both CDP controlled-disconnect tests and both CLI
structured-connection-error tests; all other jobs must remain green.

## Validation and Acceptance

Acceptance requires all of the following:

- The complete CDP test binary passes repeatedly on the local platform with
  transport and client fixtures participating in the same stress runs.
- The complete CDP suite passes at least 25 consecutive runs with 16 test
  threads after the controlled-disconnect fixture is added.
- Strict Clippy reports no warning from the correction.
- The full workspace tests, formatting, metadata, hygiene, and script checks
  pass locally.
- The Windows CI test job passes
  `target_list_transport_disconnect_remains_connection` with `Connection` and
  exit code 2, and
  `version_probe_transport_disconnect_remains_not_ready` with `Ok(None)`.
- The same Windows job passes every CLI test that expects a CDP connection
  error, including the core, Desktop-backed, and quote contract targets, with
  the existing `kind: "connection"` and exit-code assertions.
- The Linux and macOS CI test jobs remain green.
- `git diff` shows removal of the temporary mutex/dev feature, the new
  test-only disconnect fixture, and correction-wave docs. There is no
  production transport diff, new dependency, CLI contract change, or package
  version change.
- No tracked document contains raw payloads, credentials, account-local data,
  machine-specific paths, or one-off private review prompts.

Controlled-disconnect result (2026-07-12): the complete 32-test CDP suite
passed 25 consecutive runs with 16 test threads. The five-test `cli_contract`
target also passed 25 consecutive runs before the shared-fixture expansion.
After expansion, the core 5-test, Desktop 93-test, and quote 26-test contract
targets passed, as did formatting, strict Clippy, the full workspace suite,
metadata, public hygiene self-test/check across 559 tracked files,
release-script syntax, guide parity, and `git diff --check`. Independent review
reported no implementation finding. GitHub Actions run `29173925167` passed
Windows, Ubuntu, macOS, Clippy, Format, and both script-check jobs.

If Windows still reports a timeout, the implementation is not accepted.
Preserve the failure evidence, do not skip the test or broaden the error
assertion, and investigate reqwest/Windows behavior separately.

## Idempotence and Recovery

The controlled server is test-only and can be rerun without external state. If
the fixture edit is incorrect, revert only the correction-wave hunk while
preserving dependency commits and sanitation closeout docs. Do not reset the
branch or rewrite history. The existing `main-backup` branch and private
rollback bundles are unrelated safety artifacts and must remain untouched.

## Artifacts and Notes

The final CI evidence after the shared reconnect-capable correction is:

    Run: 29173925167
    Windows tests: passed
    Ubuntu tests: passed
    macOS tests: passed
    Clippy: passed
    Format: passed
    Ubuntu and Windows script checks: passed

The recorded evidence contains only public-safe run and job outcomes. It does
not include raw CI environment paths or account-local identifiers.

## Interfaces and Dependencies

The new code-level interfaces are private test support. The CDP crate uses:

    async fn transport_disconnect_fixture() ->
        (TransportConfig, tokio::task::JoinHandle<()>)

It is defined and used only inside `#[cfg(test)]` code in `tradingview-cdp`.
CLI integration tests use a private `CdpDisconnectCommand` guard and
`tv_with_cdp_disconnect()` helper under `crates/cli/tests/support/mod.rs`. The
guard keeps the listener alive through client reconnects and joins its server
thread when the subprocess assertion completes.
Production builds, public Rust APIs, CLI output, HTTP deadlines, error kinds,
and exit codes remain unchanged.

## Open Questions

There is no unresolved implementation or release-readiness blocker in this
plan. Rollback bundle deletion remains a separate post-release owner decision.

Revision note (2026-07-12): created after canonical history sanitation closed
and the first post-rewrite CI run showed two Windows-only released-port fixture
failures. Revised after the reviewed mutex approach also failed Windows CI; the
current correction uses a controlled server disconnect and removes the
disproven serialization infrastructure. Revised again after run `29173925167`
passed every CI job and focused review found only stale project-state docs; the
plan is now complete and ready for archive.
