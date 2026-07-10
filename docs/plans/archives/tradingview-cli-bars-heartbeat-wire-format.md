# Verify and correct bars heartbeat framing

This ExecPlan is a living document. Keep `Progress`, `Surprises & Discoveries`,
`Decision Log`, and `Outcomes & Retrospective` current while working. Maintain
this plan according to `.agents/PLANS.md`.

## Purpose / Big Picture

The browserless bars WebSocket must answer TradingView heartbeat packets with a
frame whose declared payload length matches the bytes actually sent. The
current helper declares the length of `~h~{value}` but appends an extra trailing
delimiter, creating a malformed frame. This gate fixes that internal wire
invariant without changing the `tv bars` command, `bars.v1`, timeout behavior,
or source diagnostics.

## Progress

- [x] (2026-07-11) Gate 3 was independently reviewed, corrected, re-reviewed, committed, and archived.
- [x] (2026-07-11) Re-read the heartbeat parser, pong sender, live bars gate, roadmap, and work inventory.
- [x] (2026-07-11) Created this Gate 4 ExecPlan and made it current.
- [x] (2026-07-11) Added byte-accurate protocol tests and demonstrated the current mismatch before changing production code.
- [x] (2026-07-11) Emitted canonical heartbeat pongs through the common frame helper.
- [x] (2026-07-11) Verified canonical, legacy trailing-delimiter, and bare heartbeat parsing plus malformed-frame boundaries and packet order.
- [x] (2026-07-11) Verified the transport sends the canonical frame and retains Gate 3 timeout diagnostics.
- [x] (2026-07-11) Ran focused tests and the full workspace baseline; all non-ignored tests pass and the opt-in live smoke remains intentionally skipped.
- [x] (2026-07-11) Recorded outcomes as `implemented; independent review pending`; do not start Gate 5 or commit Gate 4.
- [x] (2026-07-11) Independent review approved the local framing work but rejected the production form selection because server acceptance was unproven.
- [x] (2026-07-11) Added and ran a public-safe live probe that observed a heartbeat, sent the canonical candidate, and proved post-pong series traffic on the same connection.
- [x] (2026-07-11) Selected the canonical production form from live evidence and re-ran focused validation.
- [x] (2026-07-11) Re-ran the focused checks and full workspace baseline after tightening the live probe; all non-ignored tests pass.
- [x] (2026-07-11) Tightened the probe after focused re-review so only a response received after a post-pong `request_more_data` counts as remote-acceptance evidence.
- [x] (2026-07-11) Re-ran the live probe successfully: heartbeat 1, pong 1, post-pong request 1, post-request update 1, post-request completion 0.
- [x] (2026-07-11) Re-ran focused tests, strict clippy, and the full workspace suite after the strengthened probe; all non-ignored tests pass.
- [x] (2026-07-11) Focused independent re-review reported no remaining findings; Gate 4 is complete and ready to archive and commit.

## Surprises & Discoveries

- Observation: `pong_frame(42)` currently declares payload length five but
  emits the six-byte payload `~h~42~`.
  Evidence: `pong_frame` adds three to the digit count while formatting an
  additional trailing delimiter.

- Observation: the parser already accepts both framed heartbeat payloads with
  a trailing delimiter and the canonical form without one.
  Evidence: framed heartbeat parsing uses `trim_end_matches('~')` before
  converting the value.

- Observation: the test-first invariant failed exactly on the trailing
  delimiter before production code changed.
  Evidence: `pong_frame(7)` returned `~m~4~m~~h~7~`, while the canonical common
  frame helper returned `~m~4~m~~h~7`.

- Observation: local byte invariants cannot choose between removing the
  trailing delimiter and retaining it with a corrected length.
  Evidence: independent review identified both as internally valid and noted
  that the first implementation had no server-acceptance evidence.

- Observation: TradingView accepted the canonical candidate and processed new
  work on the same connection afterward.
  Evidence: the public-safe opt-in probe observed one heartbeat, sent one
  canonical pong, sent one `request_more_data`, then received one series update
  in a WebSocket message after that request. It recorded no raw frame, bar, or
  session identifier.

## Decision Log

- Decision: treat `~h~{value}` as a candidate, not a settled production form,
  until an opt-in live probe observes heartbeat receipt, candidate pong send,
  and post-pong series traffic on the same connection.
  Rationale: local consistency does not prove the remote protocol accepts one
  of two internally valid forms.
  Date/Author: 2026-07-11 / Codex.

- Decision: retain `~h~{value}` as the canonical production payload after the
  live probe demonstrated server acceptance and a response to a new post-pong
  `request_more_data` request.
  Rationale: this closes both the byte-length invariant and the remote-
  acceptance requirement without relying on a third-party implementation.
  Date/Author: 2026-07-11 / Codex.

- Decision: synthetic heartbeat strings may appear in protocol tests, but raw
  live frames and bars must not be stored in tracked files.
  Rationale: byte-accurate deterministic fixtures are required to prove the
  invariant without recording live or account-bearing data.
  Date/Author: 2026-07-11 / Codex.

## Outcomes & Retrospective

The local implementation and first full baseline are green. Independent review
correctly found that self-contained tests did not prove TradingView accepted
the canonical candidate. A dedicated public-safe probe now supplies that
evidence: heartbeat count 1, pong count 1, post-pong request-more count 1,
post-request series-update count 1, post-request completion count 0, and
connection usable after pong. The update was received in a WebSocket message
after the new request, so an in-flight response to the initial series request
cannot satisfy the probe.

Focused protocol, transport, bars, and CLI contract tests are green. Strict
clippy and the full workspace suite are green after the strengthened probe,
including 88 market tests and 365 CLI tests. The dedicated heartbeat probe was
executed successfully; the broader opt-in live bars smoke remains ignored in
the ordinary suite. Focused independent re-review reported no remaining
findings, so the gate is complete.

## Context and Orientation

`crates/market/src/bars/protocol.rs` owns the `~m~{length}~m~{payload}` framing
helper, heartbeat parsing, and `pong_frame`. `crates/market/src/bars/transport.rs`
uses `pong_frame` when `parse_packets` returns `WsPacket::Ping`. Gate 3 already
bounded that send and preserved bars diagnostics on timeout; Gate 4 must not
alter those behaviors.

The existing live bars test is opt-in and only proves end-to-end command
success. Gate 4 therefore uses a separate ignored probe that records only
heartbeat count, pong count, post-pong request count, post-request series
packet counts, and connection status. It must not print or persist raw frames,
bars, session identifiers, or credentials.

## Plan of Work

First add a test-only frame decoder that separates the declared length and
payload from one synthetic frame. Add one-digit, two-digit, and large positive
heartbeat cases and assert that every declared length equals the payload byte
length. Add exact equality with `frame("~h~...")`; this must fail against the
current implementation before production code changes.

Then implement `pong_frame` by creating `~h~{value}` and passing it to `frame`.
Keep the receive parser compatible with canonical framed heartbeat, framed
heartbeat with a trailing delimiter, and bare heartbeat. Expand the mixed
packet test so ordinary JSON frames and both relevant heartbeat inputs preserve
arrival order.

Add a recording sink to the transport tests and pass it through
`send_heartbeat_pong`. Assert that exactly one text message is sent and that its
bytes equal the canonical `pong_frame`. Keep the existing pending-send test to
prove timeout diagnostics remain intact.

Add a dedicated ignored live probe. It must wait for a real heartbeat, send the
canonical candidate without reading `pong_frame` as its expected value, request
additional series data when needed, and accept evidence only from a later
WebSocket message containing `timescale_update`, `du`, or `series_completed`.
Print aggregate counters and connection status only.

Update `CHANGELOG.md`, the roadmap, work inventory, plan index, and local
continuity ledger with implementation evidence. No public docs or runtime
skills require changes because the command surface and payload semantics do not
change.

## Validation and Acceptance

Run from the repository root:

    cargo test -p tradingview-market bars::protocol -- --nocapture
    cargo test -p tradingview-market bars::transport -- --nocapture
    cargo test -p tradingview-market bars -- --nocapture
    cargo test -p tradingview-cli --test cli_contract_bars -- --nocapture
    cargo test -p tradingview-cli --test live_bars
    TV_LIVE_BARS_HEARTBEAT_SMOKE=1 cargo test -p tradingview-market canonical_heartbeat_pong_live_probe -- --ignored --nocapture
    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1
    git diff --check
    bash -n scripts/stage-release-package-files.sh

Acceptance requires exact byte-length agreement for all synthetic pong frames,
canonical and legacy heartbeat receive compatibility, ordered mixed packet
parsing, canonical transport output, unchanged timeout diagnostics, green
workspace validation, and a successful public-safe live probe. The probe must
observe a real heartbeat, send exactly one canonical pong, send exactly one
post-pong `request_more_data`, and receive an update or completion in a later
WebSocket message. It remains ignored in ordinary test runs, but Gate 4 cannot
complete unless it is explicitly enabled and succeeds.

## Idempotence and Recovery

Frame lengths, parser compatibility, sent bytes, and timeout diagnostics are
covered by deterministic local tests. Canonical remote acceptance requires the
opt-in public-safe live probe. The probe stores only aggregate counters and
must not retain raw frames, bars, or session identifiers. If the probe cannot
run or does not receive a response to its post-pong request, leave Gate 4
incomplete. Do not create Gate 5 or commit Gate 4 before independent review
completes.

## Interfaces and Dependencies

Keep `frame`, `parse_packets`, `fetch_bars_ws`, all public market functions,
and CLI payloads compatible. Only the private bytes returned by `pong_frame`
change. Add no dependency, command, option, payload field, retry, timeout,
source fallback, or concurrency.

Revision note: created on 2026-07-11 after Gate 3 completed independent review
and was committed as `7daaf77`.
