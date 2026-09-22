# MCP operational improvements and two additional reads

Status: **scope accepted; contract proposal ready for review**, 2026-09-22.
Direction and priority live in the [roadmap](../next-version-roadmap.md) and
[inventory](../next-version-work-items.md). This is the single work record for
v0.33.0; the [v0.32.0 record](archives/tradingview-cli-official-mcp-client.md)
is closed historical evidence.

## Outcome, consumers and authority

Deliver alert firing history, official technical snapshots and a more usable
login/upgrade journey through the independent `tv mcp` surface. The owner
accepted the proposed direction and explicitly promoted technical snapshots
into planned scope. Connection reuse is conditional on measurement. Existing
CLI consumers keep their commands, envelopes and source semantics.

The PM remains the sole executor; no additional agent/session is authorized.
Public CLI/JSON examples below require agreement before implementation under
[AGENTS.md](../../AGENTS.md). Their values are synthetic client-output proposals,
not observed provider payloads. No new Rust dependency or persisted format is
proposed. Local documentation commits are within the established PM authority;
push/tag/workflow/release and downstream edits are not authorized here.

Reuse existing same-target verification authority. New verification needs are
listed together below; no arbitrary count/time approval cycle is introduced.
Actual rate limits, deadlines, dispatch guards and user-controlled OS consent
remain mandatory. This planning stage performs no account or credential I/O.

## Current evidence and alternatives

- v0.32.0 is published at `54dabec`; its release workflow succeeded. Downstream
  reports released-binary daily observation/readback and unchanged D/W/M intake
  contracts. Account-history and technical snapshots are not yet accepted by a
  downstream analytical consumer.
- `client::failure` already gives `next_action: tv mcp login` for missing
  credentials or OS interaction. Improve the explicit login journey rather than
  add a redundant doctor command or change the ordinary structured error.
- `transport::call` initializes and discovers tools on every call. Watchlist and
  alert changes call it once for mutation and again through `account::readback`.
  This proves repeated setup, not its latency cost. Start with private counters
  and timings in the synthetic harness; do not add public timing metadata.
- Two upstream MCP fixtures timed out in an earlier parallel run and passed
  serially; CI later passed. Investigate reproducibility, not an assumed
  production defect. Downstream test scheduling has a separate owner.
- [Official documentation](https://www.tradingview.com/mcp/docs), checked on
  2026-09-22, lists `get_alerts_log` and `get_technicals_rating`. Neither is in
  the current closed tool registry. Documentation is not actual wire/schema
  evidence; inspect the authenticated catalog and responses before freezing
  their decoders.

Do not implement technical snapshots by silently substituting existing column
reads. Compare overlap for usefulness and interpretation, but keep the dedicated
provider tool explicit. Existing `symbol`/`symbols` is better for arbitrary fields
or batches; the new command is a single-timeframe snapshot without field-name
assembly. No equality or chart/Pine equivalence is promised.

## Proposed CLI and JSON contracts

### Official technical snapshot

Before: v0.32.0 has arbitrary `mcp symbol` columns and chart-owned study reads,
but no dedicated official technical snapshot command.

After (proposed):

```sh
tv mcp technicals NASDAQ:EXAMPLE --timeframe 1D
```

One exchange-qualified symbol; default `1D`. Accept the provider's documented
`1m`, `5m`, `15m`, `30m`, `1h`, `2h`, `4h`, `1D`, `1W`, `1M` values after
catalog verification. In particular, `2h` support here must not widen MCP bars
or legacy date-range validation. No range, batch or local indicator calculation.

Illustrative client envelope (abbreviated indicator inventory):

```json
{
  "success": true,
  "command": "mcp",
  "data": {
    "contract_version": "mcp_technicals.v1",
    "source": "tradingview_mcp",
    "requested": {"symbol": "NASDAQ:EXAMPLE", "timeframe": "1D"},
    "reported": {"symbol": null, "timeframe": null, "data_as_of": null},
    "retrieved_at_unix_ms": 1700000000000,
    "indicators": [{"name": "RSI", "value": 52.5}],
    "ratings": {"summary": null, "moving_averages": null, "oscillators": null},
    "conditions": {"delay": null, "adjustment": null, "session": null, "finality": null}
  }
}
```

`reported` contains provider evidence only, never the requested values copied
as confirmation. A contradictory symbol/timeframe echo is an invalid response.
Indicator names and scalar value types must be fixed from inspected response
shape; do not preserve arbitrary nested payloads as an escape hatch. Known
missing values are null, never zero. Null ratings are unreported, not neutral.
Preserve provider rating values without local thresholds or trading actions.
Do not label the snapshot a historical series or point-in-time input.

An absent required container, invalid numeric value or changed type fails with
`mcp_error.v1`/`invalid_response`. Valid containers with absent optional entries
remain successful with unknown values. A response with no usable observations
must have an explicit empty state in the final reviewed schema, not fabricated
indicator values; its actual provider shape is still UNCONFIRMED.

### Alert firing history

Before: v0.32.0 can read/manage alerts but cannot read their fire history.

After (proposed):

```sh
tv mcp alert history --symbol NASDAQ:EXAMPLE --days 7 --limit 100
```

Require `--symbol` in the first CLI slice to avoid accidental account-wide
reads. Default days 7 and limit 100; limit 1..2000 per official documentation.
Days must be a positive integer; confirm any server upper bound instead of
inventing one. No pagination, account writes, monitoring or event-triggering.

```json
{
  "success": true,
  "command": "mcp",
  "data": {
    "contract_version": "mcp_alert_history.v1",
    "source": "tradingview_mcp",
    "requested": {"symbol": "NASDAQ:EXAMPLE", "days": 7, "limit": 100},
    "retrieved_at_unix_ms": 1700000000000,
    "events": [
      {
        "alert_id": 123,
        "symbol": "NASDAQ:EXAMPLE",
        "fired_at_unix_seconds": 1699999900,
        "webhook_delivery_status": null
      }
    ],
    "returned_count": 1,
    "limit_reached": false,
    "coverage": "unconfirmed"
  }
}
```

Names/values above are proposed normalized fields. Preserve only confirmed
provider fields; do not invent IDs, timestamps or delivery status. Establish
required identity/time fields and timestamp units from actual schema evidence.
Reject contradictory symbol rows rather than silently relabel/filter them.
Do not infer alert active state, receipt by a person, or trade execution.
Messages, webhook URLs, arbitrary raw conditions and raw payloads are excluded
from this first public output. Inspect private payloads only in untracked local
artifacts; fixtures use synthetic events.

Empty history is `events: []`, `returned_count: 0`, `limit_reached: false` and
`coverage: unconfirmed`. At the requested limit, `limit_reached: true` reports
only measured saturation, not proof of additional events. Fewer events than
requested is not proof of complete history. These are request caps, not a
promise to return N events. Unsupported paging/range flags fail before I/O.

### Shared failures and compatibility

Existing `mcp_error.v1`, exit codes and zero-attempt pre-I/O validation apply.
For example, an invalid timeframe returns validation/unsupported capability with
`tool_attempts: 0`; absent credentials returns `auth_required` plus the existing
login action. A dispatched 429 preserves `rate_limited` and only confirmed
Retry-After evidence; timeout uses `deadline_exceeded`; malformed responses use
`invalid_response`. No silent alternate source, zero-filled data or automatic
replay. Wire failures and optional missing values are separate outcomes.

No new success/error fields are added to existing commands. For the login
journey, propose concise human guidance on stderr before interactive waits;
success stdout and JSON failures remain unchanged. Inspect current CLI capture
conventions before choosing progress output so failure stderr stays consumable.
If that cannot be achieved without changing a consumer contract, return the
specific before/after choice rather than mixing prose into JSON stderr.

## Implementation and acceptance sequence

1. Review the proposed public contracts and authorize the new scoped reads.
   Verify actual tool names/schema/response shape. Revise material contract
   differences before implementation; do not encode guessed provider evidence.
2. Improve login reuse/interaction guidance. Test existing credentials, expired
   token, missing/locked OS store, update-related consent and interrupted login.
   Ordinary commands remain noninteractive; no extra registration on reusable
   credentials. Do not promise signing/notarization effects.
3. Add history validation/normalization to the account model and a closed tool
   adapter; add technical snapshot shaping in the I/O-free model and a closed
   service adapter. CLI owns parsing/envelopes, MCP owns auth/transport. Each
   command must be independently usable and tested before the next slice.
4. Measure mutation/readback setup. If useful, let one command own the live
   session and validated tool bindings, then close it within the same deadline.
   Preserve one mutation dispatch, readback uncertainty, cooldown and credential
   ownership. Do not cache credentials/catalogs across processes. Compare call
   counts and timings before/after; no reduction in actual tool admission checks.
5. Correct demonstrated fixture timing dependencies alongside affected changes.
   Separate intentional timeout tests from server/worker startup readiness;
   retain failures and diagnostics rather than treating serial pass as a fix.
6. Update usage/source maps and the account-management/market-data skills with
   all mandatory references inside each skill directory. Existing chart/Pine
   routes remain explicit. Do not install or modify downstream skills here.
7. Run applicable development/platform checks, qualify the two reads and report
   limitations. Release preparation is a later separate commit; no version bump
   or published capability claim in this planning change.

Fixture acceptance covers CLI-to-model integration, request defaults/bounds,
source/echo validation, null/empty/limit cases, malformed types, 401/429/timeout,
no implicit retry and sensitive-field exclusion. Connection reuse adds count
assertions for initialization/catalog calls and mutation/readback failure paths.
Run focused tests then the required Rust baseline for production changes;
unchanged JavaScript gates may reuse valid evidence. Verify normal parallel CI
on all supported OSes; Linux graphical OAuth remains a separately stated limit.

## Live verification proposal and remaining decisions

Bundle missing authority before starting live work:

- Use the existing chosen account and selected installed/trusted executable;
  no new account setup or OAuth scopes are expected. The technical tool reads
  NASDAQ:AAPL daily, weekly and monthly snapshots, compares available overlapping
  columns only when necessary, and checks `2h` if claiming that interval.
- Read NASDAQ:AAPL alert history for the previous seven days with limit 100.
  This accesses account-local event content even though output is redacted.
  No alert creation, edit, firing, deletion or webhook delivery is requested.
  If the history is empty, record empty-only native evidence; do not manufacture
  account activity to obtain a success fixture.
- Reuse existing credential-operation authority for the same target/effects.
  Announce any browser/OS interaction before starting it and wait for the user.
  An executable change alone does not imply new provider-operation authority.
- Connection optimization uses synthetic mutation/readback for measurement.
  Any additional live mutation requires a concrete disposable target proposal;
  it is not a hidden prerequisite for either new read.

Contract review must settle the two CLI shapes, source/unknown handling, limited
history output and interaction guidance. Provider response details remain
UNCONFIRMED until observation; public documentation is insufficient to assert
them. No new dependencies are expected; any demonstrated need is a separate
concrete proposal with current-version verification.

## Progress

- Scope accepted, including technical snapshots as a planned feature.
- v0.32.0 publication and release-workflow success rechecked; old plan archived.
- Current code and downstream readback reviewed; no new consumer defect claimed.
- Roadmap/inventory synchronized; synthetic contract examples proposed here.
- No implementation, live request, dependency change or version bump yet.
- Planning checks passed: 59 local Markdown file references, two synthetic
  JSON examples, public-hygiene self-test/scan (709 tracked files), and diff
  whitespace checks. Credential-language review found only policy/examples.
  No runtime resources or Rust code changed, so no rebuild or functional suite
  was run for this documentation-only change.
- Next: owner review of contracts and the bundled new read scope above.
