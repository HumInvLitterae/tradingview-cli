# Agent Operating Guide

Build and maintain `tv`, one Rust-native CLI for TradingView Desktop automation
and explicit Desktop-free data and account operations. This is contributor
guidance; release users get
[packaging/agent/AGENTS.md](packaging/agent/AGENTS.md).

## Project boundaries

- Keep data sources and side effects explicit. Desktop-free requests must not
  silently become Desktop operations. Preserve the shared JSON envelope,
  documented source semantics, and practical information used by consumers.
- Assess compatibility against the current public CLI, JSON, and Rust APIs and
  actual downstream use. The old JavaScript command migration is closed;
  newly found legacy commands need a concrete workflow benefit before becoming
  work items. Historical migration notes explain decisions, not automatic scope.
- Keep private downstream orchestration outside the core CLI unless a concrete
  consumer demonstrates that it belongs here. MCP server work, cookie/session
  import/export, trading bots, and broad generic UI expansion are not planned.
- Keep account-local IDs, credentials, raw live payloads, and machine-specific
  absolute paths out of tracked files. Use synthetic examples and public-safe
  evidence; see [development guidance](docs/development.md#public-hygiene-guard).

## Read for the task

Start with the affected code and its consumers. Use this table when context is
needed; it is not a mandatory reading sequence.

| Task | Reference |
| --- | --- |
| Product purpose and usage | [README](README.md) |
| Select or resume planned work | [Current work](docs/plans/README.md) |
| Data sources, JSON meaning, side effects | [Source taxonomy](docs/command-source-taxonomy.md) |
| Crate ownership or dependency boundaries | [Architecture](docs/architecture.md) |
| Public library contracts | [Rust API](docs/rust-api.md) |
| Choose checks or investigate implementation conventions | [Development](docs/development.md) |
| TradingView internal behavior | [Internal APIs](docs/internal-tradingview-apis.md) and the affected adapter |
| Prepare distribution | [Release packaging](docs/release-packaging.md) |
| Plan a complex feature or significant refactor | [Planning guidance](.agents/PLANS.md) |
| Operate the binary through an agent | [Runtime guide](packaging/agent/AGENTS.md) and the relevant runtime skill |

Code, current contracts, and explicit decisions take precedence over historical
plans. When they disagree, investigate and correct the affected current source
within scope. Do not turn a documentation disagreement into permission to edit
unrelated work. Mark unresolved consequential facts `UNCONFIRMED`.

## Execution and verification

Complete the authorized outcome, including affected callers, documentation,
checks, and related fixes. Use [the validation guidance](docs/development.md#validation-baseline)
to select checks. Ordinary deterministic checks use fixtures; live smokes are
separate, opt-in operations with their own targets, limits, and authority.
Reuse valid evidence when its inputs have not changed. Distinguish source
inspection, fixture execution, and actual Desktop/provider observations.

Keep the detailed state of a substantial change in its existing work record.
A roadmap owns direction, an inventory owns priority, and an ExecPlan owns the
change's decisions and acceptance evidence. The optional `continuity` skill
prepares a handoff only when explicitly requested; it is not an every-turn task
or a source of new PM, commit, or delegation authority.

## Repository conventions

- `crates/cli` owns the binary and operation adapters; `core` owns envelopes and
  errors; `model` owns I/O-free interpretation and shaping. `market`, `scanner`,
  and `pine` own credential-free Desktop-free services; `mcp` owns the internal
  authenticated official-MCP client; `cdp` owns Desktop transport.
- Prefer English for repository docs. Use descriptive work names rather than
  ordinal aliases. Public docs explain usage, reproduction, and maintenance.
- Commit related authorized changes in coherent batches using the
  [commit convention](docs/development.md#commit-messages). Preserve staged and
  concurrent work; follow the assigned PM/implementer commit boundary.
- Never push unless explicitly requested in the current turn. Release work
  keeps versioning, notes, packaging, and CI fixes separate from feature work.
- `CLAUDE.md` shares this guide; `.claude/skills` shares `.agents/skills`.
  Runtime guides and skills have a separate, explicit package allowlist.
- Current work lives in `docs/plans`; completed plans in `docs/plans/archives`;
  research and historical rationale in `docs/notes`. Do not rewrite frozen
  evidence merely to match a new documentation style.
