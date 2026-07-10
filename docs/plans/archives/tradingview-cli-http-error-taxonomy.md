# Make HTTP error classification consistent

This ExecPlan is a living document. Keep `Progress`, `Surprises & Discoveries`,
`Decision Log`, and `Outcomes & Retrospective` current while working. Maintain
this plan according to `.agents/PLANS.md`.

## Purpose / Big Picture

The CLI currently assigns different error kinds to equivalent failures from
Market, Scanner, Pine, and CDP HTTP endpoints. In particular, several received
HTTP 4xx or 5xx responses are reported as connection failures even though the
server was reached. Downstream automation therefore cannot reliably distinguish
an unavailable network, a deadline, and a remote API failure.

After this work, every HTTP-owning crate follows one policy: local validation is
`validation`; DNS, TCP, TLS, socket, and body-transport failure is `connection`;
deadline expiry is `timeout`; received non-success status, malformed JSON, or
missing required remote shape is `internal_api_unavailable`; internal client
construction or serialization failure is `internal`. Existing success payloads,
commands, public Rust signatures, and the `ErrorKind` enum remain unchanged.

## Progress

- [x] (2026-07-11) Gate 4 was independently re-reviewed, committed as `ca3db67`, and archived.
- [x] (2026-07-11) Re-read the HTTP helpers, endpoint call sites, exit-code mapping, roadmap, and ordered inventory.
- [x] (2026-07-11) Created this Gate 5 ExecPlan and made it current.
- [x] (2026-07-11) Added deterministic taxonomy helpers and tests in Market, Scanner, and Pine.
- [x] (2026-07-11) Aligned CDP HTTP status and payload failures while preserving version-probe semantics.
- [x] (2026-07-11) Removed raw reqwest strings, response bodies, URLs, and raw parsed Pine/CDP payloads from the changed public error paths.
- [x] (2026-07-11) Ran focused tests, CLI contracts, strict clippy, and the full workspace baseline; all non-ignored tests pass.
- [x] (2026-07-11) Recorded outcomes as `implemented; independent review pending`; do not start Gate 6 or commit Gate 5.
- [x] (2026-07-11) Independent review reported no findings; Gate 5 is complete and ready to archive and commit.

## Surprises & Discoveries

- Observation: scanner quote and fundamentals already classify remote status as
  `InternalApiUnavailable`, while symbol search, scanner scan/hotlist/metainfo,
  and Pine check classify the same condition as `Connection`.
  Evidence: their non-success branches differ even though all use equivalent
  configured public HTTP clients.

- Observation: CDP target list/create/activate status failures and target
  list/create/version JSON decoding currently use `Connection`.
  Evidence: `crates/cdp/src/transport.rs` passes `Connection` as the fallback
  kind for both request and response-body phases.

- Observation: Pine malformed-shape details currently retain the entire parsed
  response value.
  Evidence: `normalize_check_response` attaches `value` to the error, which is
  unnecessary for the stable taxonomy and can expose remote response content.

- Observation: CDP target creation status and unusable-target failures also
  exposed the response body or full target object.
  Evidence: Gate 5 replaced those details with stable failure class, status,
  target kind, and target-ID-presence fields only.

## Decision Log

- Decision: keep reqwest-specific helpers within each crate that owns reqwest.
  Rationale: `tradingview-core` owns stable error kinds and exit codes but must
  not gain an HTTP dependency.
  Date/Author: 2026-07-11 / Codex.

- Decision: classify a received 4xx or 5xx as
  `InternalApiUnavailable`, including CDP HTTP endpoints.
  Rationale: receiving a status proves that connection establishment succeeded;
  remote 4xx does not imply invalid local CLI input.
  Date/Author: 2026-07-11 / Codex.

- Decision: publish stable operation, failure-class, and numeric-status details
  only; do not expose reqwest strings, URLs, query values, or response bodies.
  Rationale: reqwest diagnostics may contain endpoint or request information,
  while downstream automation needs stable categories rather than raw text.
  Date/Author: 2026-07-11 / Codex.

- Decision: preserve `version_probe` best-effort behavior: ordinary connection
  failure and non-success status return `Ok(None)`, while its explicit timeout
  and a malformed successful response remain errors.
  Rationale: launch readiness intentionally treats an absent CDP endpoint as
  not ready, not as a terminal command error.
  Date/Author: 2026-07-11 / Codex.

## Outcomes & Retrospective

Implementation and validation are complete. Remote 4xx/5xx for symbol search,
scanner scan/hotlist/metainfo, Pine check, and CDP target list/create/activate
now map to `InternalApiUnavailable`/3. Malformed successful CDP target
list/create/version responses also map to 3. Connection refusal remains
`Connection`/2, timeout remains `Timeout`/4, and already-consistent scanner
quote/fundamentals failures remain 3.

Market has 91 passing tests plus one ignored live heartbeat probe, Scanner has
26 passing tests, Pine has 25 passing tests, CDP has 32 passing tests, and CLI
has 365 passing unit tests. Focused CLI contract suites, strict clippy, and the
full workspace suite are green. Independent read-only review reported no
findings, so Gate 5 is complete.

## Context and Orientation

`tradingview-core` defines `AppError`, `ErrorKind`, and the stable exit mapping:
`Connection` is 2, `InternalApiUnavailable` is 3, `Timeout` is 4, and the other
kinds are 1. It does not depend on reqwest.

`crates/market/src/http.rs`, `crates/scanner/src/http.rs`, and
`crates/pine/src/http.rs` each own a configured reqwest client and a similar
private error mapper. Endpoint modules perform status checks and JSON decoding.
`crates/cdp/src/transport.rs` owns the local CDP HTTP client and target
list/create/activate/version calls. Gate 5 changes only these HTTP boundaries;
bars WebSocket, CDP WebSocket, JSON/JSONL output, retries, and concurrency are
outside scope.

## Plan of Work

First replace each public-HTTP crate's fallback-kind mapper with private,
phase-specific helpers. Request and response-body transport failures map timeout
to `Timeout` and all other non-decode reqwest failures to `Connection`. JSON
decode failures map to `InternalApiUnavailable`. A received non-success status
uses a separate helper that returns `InternalApiUnavailable` with the numeric
status. Messages and details use stable operation names and never include raw
reqwest text, URLs, query values, or response bodies.

Update symbol search, scanner quote and fundamentals, scanner
scan/hotlist/metainfo, and Pine check to call those helpers. Remove the raw
parsed value from Pine malformed-shape details. Preserve all normalization and
success payload behavior.

Apply the same phase distinction in `crates/cdp/src/transport.rs`. Connection
refusal remains `Connection`; explicit timeout remains `Timeout`; target
list/create/activate status failures and successful-response decode failures
become `InternalApiUnavailable`. Keep `version_probe` best effort for ordinary
connection failure and non-success status, but classify a timeout as `Timeout`
and malformed JSON from a successful response as `InternalApiUnavailable`.

Add local TCP HTTP fixtures or test-only URL-taking helpers so each owner proves
connection refusal, timeout, 429, 500, malformed JSON, and missing required
shape without external network or Desktop. Assert both `ErrorKind` and
`AppError::exit_code`, and assert public error values omit raw body, URL, query,
credential, session, and target identifiers.

Finally update this plan, roadmap, work inventory, changelog, and local
continuity ledger with the observed result. Run focused and full validation.
At the implementation handoff, leave the changes uncommitted as `implemented;
independent review pending` and do not create the Gate 6 ExecPlan. After a
green review, archive and commit Gate 5 before planning Gate 6.

## Concrete Steps

Run from the repository root:

    cargo test -p tradingview-core error -- --nocapture
    cargo test -p tradingview-market http -- --nocapture
    cargo test -p tradingview-scanner http -- --nocapture
    cargo test -p tradingview-pine http -- --nocapture
    cargo test -p tradingview-cdp transport -- --nocapture
    cargo test -p tradingview-cli --test cli_contract -- --nocapture
    cargo test -p tradingview-cli --test cli_contract_quote -- --nocapture
    cargo test -p tradingview-cli --test cli_contract_desktop -- --nocapture
    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1
    git diff --check
    bash -n scripts/stage-release-package-files.sh
    cmp -s AGENTS.md CLAUDE.md

Expected results are green non-ignored tests, unchanged CLI success contracts,
and deterministic taxonomy assertions with exit codes 2, 3, and 4.

## Validation and Acceptance

Acceptance requires local fixtures across Market, Scanner, Pine, and CDP to
prove: connection refusal is `Connection`/2; stalled headers or bodies are
`Timeout`/4; 429 and 500 are `InternalApiUnavailable`/3; malformed JSON and
missing required shape are `InternalApiUnavailable`/3. Public errors must not
contain response bodies, URLs, query values, credentials, session IDs, raw
target IDs, or account-local metadata.

The CLI envelope, command names, successful payloads, public Rust signatures,
and existing `ErrorKind` variants must remain unchanged. CDP connection refusal
contract tests must continue to report `connection` and exit 2.

## Idempotence and Recovery

All required evidence is deterministic and local. Tests may bind ephemeral
loopback ports and may be rerun safely. No live TradingView data is required or
recorded. If a compatibility-sensitive command cannot adopt the policy without
changing its documented semantics, stop, record the exception in this plan,
and obtain review rather than silently preserving drift.

## Interfaces and Dependencies

Do not change public Rust function signatures. Keep HTTP classification helpers
private or crate-visible within `tradingview-market`, `tradingview-scanner`,
`tradingview-pine`, and `tradingview-cdp`. Do not add a dependency, error kind,
command, option, payload field, retry, cache, fallback, or concurrent request.

## Open Questions

None. The policy, known exit-code changes, version-probe exception, privacy
boundary, and review gate are fixed by this plan.

Revision note (2026-07-11): created the Gate 5 implementation plan after Gate 4
completed independent review and commit.

Revision note (2026-07-11): recorded the completed taxonomy implementation,
public-safe diagnostics, deterministic tests, and independent-review gate.

Revision note (2026-07-11): recorded the green independent review and Gate 5
completion before archive and commit.
