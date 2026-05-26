# Watch / JSONL compare contract planning

This ExecPlan is a living document. Keep `Progress`, `Surprises &
Discoveries`, `Decision Log`, and `Outcomes` updated while working.

## Purpose

`v0.21.0` made `tv bars --from/--to` a mature Desktop-free historical range
surface. The next useful area is not another automatic data source. It is a
bounded observation workflow that lets users and agents compare a known
candidate set over a short window while preserving source metadata and event
contracts.

This plan designs the first `v0.22.0` implementation candidate:
watch / JSONL compare. It is contract planning first. It does not yet add a
daemon, realtime feed, automatic source mixing, ranking, recommendation, or
stable export command.

## Progress

- [x] Create this ExecPlan.
- [x] Archive the completed `v0.21.0` release readiness plan.
- [x] Add the `v0.22.0` roadmap.
- [x] Update the plan index, `v0.21.0` roadmap, and changelog for the new
  roadmap state.
- [ ] Audit existing `snapshot`, `compare`, `observe`, `stream`, and
  `follow_up_hints` contracts.
- [ ] Decide the first bounded watch / JSONL compare event shape.
- [ ] Decide whether the first implementation should read scanner-backed quote
  evidence only, selected-chart JSONL observation only, or a strictly
  separated multi-source log.
- [ ] Write acceptance criteria for stop conditions, summary event, per-symbol
  availability, and public-safe logging.
- [ ] Update docs and runtime skills once the contract is chosen.

## Work Items

1. Inventory current evidence surfaces.
   - `tv snapshot` and `tv compare` are Desktop-free market evidence reads.
   - `tv bars` is Desktop-free historical OHLCV evidence.
   - `tv observe chart` and `tv stream ...` are Desktop-backed selected-chart
     JSONL observations.
   - `tv quote --source chart` is a selected-chart quote read.
   - `tv screenshot` is visual evidence, not structured market data.
2. Define the watch / JSONL compare contract.
   - Keep it bounded by duration and / or max events.
   - Include sample, heartbeat, and final summary events.
   - Include command-local contract marker, source metadata, per-symbol
     availability, and public-safe counters.
   - Do not expose raw live payloads, raw JSONL from dependencies, raw target
     ids, account-local metadata, credentials, or local paths.
3. Preserve source boundaries.
   - Do not automatically combine scanner, bars, chart quote, observe, stream,
     and quote-data into a single implied source.
   - If a later implementation supports multiple evidence kinds in one log,
     each event must identify its source and contract marker.
4. Keep selected-chart export and Replay extraction as later `v0.22.0`
   feasibility lanes.
   - They may be planned after this contract slice.
   - They are not hidden fallbacks for `tv bars`.

## Decision Log

- Decision: Make watch / JSONL compare the first `v0.22.0` implementation
  candidate.
  Rationale: `observe_chart.v1` and `stream.v1` already established bounded
  JSONL lessons: readiness, sample, heartbeat, summary, source metadata, and
  public-safe counters.
- Decision: Do not make this a daemon or realtime feed.
  Rationale: v0.22 should improve explicit observation workflows without
  introducing long-running service semantics.
- Decision: Keep selected-chart historical export and Replay-based extraction
  in the roadmap, but behind feasibility / contract slices.
  Rationale: both depend heavily on Desktop and UI state and should not be
  presented as browserless historical bars sources.

## Outcomes

This plan should leave the project with a decision-complete contract for the
first watch / JSONL compare implementation slice, including event shape,
source boundaries, stop conditions, logging behavior, and tests.

If the contract proves too broad, the implementation should start with a
smaller bounded scanner-backed watch log rather than mixing selected-chart and
Desktop-free sources too early.

## Validation

Docs validation for this planning slice:

- `git diff --check`
- `bash -n scripts/stage-release-package-files.sh`
- `rg -n "v0\\.22|watch|JSONL compare|follow_up_hints|snapshot|compare|observe chart|stream|selected-chart export|historical export|Replay|bars\\.v1|source mixing|quote-data auto|tv events|MCP|daemon|ranking|recommendation" README.md CHANGELOG.md docs .agents/skills packaging/agent/AGENTS.md`

Hygiene:

- `rg -n '(/Users/|C:\\|USER;|sessionid|cookie|authorization|bearer|raw live payload|raw WebSocket|raw JSONL|raw bars|account-local|target id|downstream-private)' README.md AGENTS.md CLAUDE.md CHANGELOG.md docs .agents/skills packaging scripts crates || true`

Rust tests are not required for this docs / planning slice unless Rust code is
changed.
