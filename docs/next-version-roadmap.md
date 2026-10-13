# Direction after v0.37.0

Status: v0.38.0 release preparation on 2026-10-13. The implementation and final
dependency set at `e251e599` passed all 11 CI jobs across Linux, macOS, and
Windows. Version metadata and notes are ready for the normal release-preparation
commit. The [inventory](next-version-work-items.md#v0380-release-preparation-2026-10-13)
records acceptance and remaining publication checks. Live account writes,
additional agents, and publication remain outside the current authorization.

## Current baseline

v0.37.0 remains published at `1fe311be70c756beb007a58a257c07e4e2f6fd10`.
The published version and remote tag were rechecked on 2026-10-13; main now
points to `e251e599`. The four native assets plus SHA256SUMS were checked on
2026-10-10 and were not executed in this pass.
The [release notes](releases/v0.37.0.md) describe named Pine creation, opt-in
indicator verification, native input IDs, and unknown creation outcomes.
Those features are released, not remaining next-version work.

The [inventory](next-version-work-items.md#dependency-refresh-before-next-version-planning-2026-10-10)
records the new dependency set and local checks. The working tree now prepares
0.38.0; v0.37.0 remains the published release.

## Approved next slice

Make MCP read failures explain how far the operation progressed. The release
is v0.38.0 because it adds public diagnostic information. Version metadata and
notes are prepared together.

A recent bounded search timed out with `stage:tool_response` and zero tool
attempts. A standard-budget search then timed out with one attempt. The same
search later succeeded without a CLI or authentication change. This proves a
useful diagnostic gap, not a specific provider defect or a retry requirement.
The observed recovery does not establish current service health.

[client.rs](../crates/mcp/src/client.rs) sets `tool_response` before
connection setup and tool-catalog retrieval. Before this change, the public error
could not identify which of those steps was waiting.
[http.rs](../crates/mcp/src/http.rs) already records bounded protocol observations
for the private proof path.
[Existing fixtures](../crates/mcp/src/fixtures.rs) distinguish initialization and
catalog timeouts and check that tokens and request arguments are absent.
Reuse these facilities, with an explicit public allowlist, rather than exposing
the whole private report or adding another connection stack.

## Approved public behavior

Keep `mcp_error.v1`, the existing kind/code/stage/exit status, tool-attempt count,
and automatic-retry policy. For MCP read transport failures, add optional
`details.protocol` observations for `initialize`, `notifications/initialized`,
`tools/list`, `tools/call`, and `event_stream` when observed. Each observation
contains `phase` and `status`. Allowed phases are `await_headers`,
`headers_received`, `accepted`, `json_body`, `json_complete`, and `sse_stream`.
Status is the observed HTTP status or null. Omit unobserved methods. Session
cleanup and cancellation are excluded from this initial public projection.
No URLs, headers, session IDs, tokens, tool arguments, or raw responses belong
in this object.

For example, initialization succeeded but the tool catalog did not return:

```json
{
  "code": "deadline_exceeded",
  "stage": "tool_response",
  "tool_attempts": 0,
  "automatic_retry": false,
  "protocol": {
    "initialize": {"phase": "json_complete", "status": 200},
    "tools/list": {"phase": "await_headers", "status": null}
  }
}
```

This is an illustrative error-details fragment, not a replacement envelope.
It tells the caller that catalog retrieval was waiting before a tool attempt.
Multiple concurrent protocol observations are retained separately. An SSE stream
being open does not by itself identify the failed request. Attempt counts remain
local attempt evidence, not proof that the provider accepted or completed a call.

The improvement helps users decide which evidence to report and avoids treating
all timeouts as failed data-tool responses. It does not restore a stalled service,
identify a root cause without evidence, change the 30-second default deadline,
retry automatically, or switch data sources. Credential errors retain their
existing explicit-login guidance. Account mutations keep their current contract.

## Work and acceptance

One executor owns the work; no additional agents are proposed.

1. The owner approved the additive error fragment and method/phase values.
   The affected error boundary, spec builders, fixtures, and three distributed
   connection-guide copies have been inspected.
2. Implement public shaping at the existing MCP error boundary. Preserve the
   underlying per-method observations and old fields. Update offline spec, CLI
   documentation, and every distributed copy of the MCP connection guide.
3. Exercise the public error path with stalled initialization, catalog headers,
   tool headers, JSON bodies, and SSE responses. Include successful requests,
   credential failures, and overlapping streams. Assert that no extra provider
   request or retry occurs and that sensitive fields cannot reach public output.
4. Run affected MCP/CLI tests, required baseline and packaging checks, then normal
   CI on supported platforms. A bounded read-only smoke may supplement fixture
   evidence when explicitly authorized; do not induce a real outage or use live
   account writes to test errors.
5. Prepare the release only after implementation and acceptance. Keep feature
   work, necessary CI fixes, and `chore(release): Prepare v0.38.0` in that order.
   Push, tagging, and publication remain owner-controlled actions.

Every implementation stage must leave the existing commands usable. No new
command, daemon, log service, persisted format, or dependency is needed for the
approved diagnostic slice.

## Next candidate and alternatives

Complete indicator-alert creation readback is the next independent candidate.
The current [matchingNewAlert](../crates/cli/src/ops/alert/indicator.rs) checks a
new ID, condition ID, message, and symbol when supplied; it does not fully bind
the result to the saved revision, inputs, or resolution. The
[study-verification record](plans/tradingview-cli-indicator-study-verification.md)
qualifies preflight and preview, not that complete post-create relationship.

Before proposing a stronger success contract, establish which identity fields
are available from the create response and subsequent reads. The desired result
is that an unrelated concurrent alert cannot satisfy creation verification.
Missing or ambiguous evidence must remain an unknown outcome. Exact live targets,
creation limits, and cleanup authority must be agreed before an account trial.
Do not promise full identity verification until those provider capabilities are
qualified. This research need not delay the smaller MCP diagnostic release.

An alert-first release is a reasonable alternative if reliable alert creation is
the immediate priority. It needs more provider qualification and live-write scope
than the recommended diagnostics slice. Automatic timeout extension/retry and
blanket new-command expansion are not justified by the observed transient outage.
Previously deferred bars, drawing, platform, and discovery work remains deferred
unless a concrete workflow supplies new evidence or a changed priority.
