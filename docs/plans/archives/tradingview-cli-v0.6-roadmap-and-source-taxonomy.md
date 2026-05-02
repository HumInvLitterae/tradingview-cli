# v0.6 roadmap and command source taxonomy

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document follows `.agents/PLANS.md` from the repository root. It is self-contained so a new contributor can finish the roadmap and documentation work without prior chat context.

## Purpose / Big Picture

This slice prepares the project for `v0.6.0` planning after the `v0.5.0` release. The user asked whether Desktop-free and Desktop-backed functionality should become separate binaries, for example `tv` and `tvd`. The decision for now is not to split binaries. Instead, this slice introduces a command source taxonomy inside the single `tv` binary so users and downstream agents can tell which commands need TradingView Desktop, which can run without it, which may mutate state, and which are experimental.

After this change, a reader can open `docs/command-source-taxonomy.md` and understand how to choose between Desktop-free reads, Desktop-backed reads, Desktop-backed operations, hybrid source selection, and lab-gated experimental commands. The `v0.6.0` roadmap will also record the next product direction: observation-first workflows where `tv` emits structured JSON or JSONL observations and agents decide what to do next.

## Progress

- [x] (2026-05-02 08:39Z) Archived the completed `v0.5.0` release readiness ExecPlan.
- [x] (2026-05-02 08:39Z) Confirmed existing docs already mention Desktop-free and Desktop-backed behavior, but there is no single command source taxonomy document.
- [x] (2026-05-02 08:46Z) Added `docs/v0.6-roadmap.md`.
- [x] (2026-05-02 08:44Z) Added `docs/command-source-taxonomy.md`.
- [x] (2026-05-02 08:52Z) Connected README, architecture, operation boundary, internal API, plans index, changelog, packaged agent guide, root agent guide, and runtime skills to the taxonomy.
- [x] (2026-05-02 08:55Z) Validated docs, skills, packaging script syntax, and hygiene.
- [x] (2026-05-02 09:00Z) Committed as `docs(roadmap): Add v0.6 source taxonomy` (`eeb487d` before final plan-status amend).

## Surprises & Discoveries

- Observation: `tv stream quote`, `tv stream bars`, and `tv stream all` already exist as Desktop-backed current-chart JSONL polling commands.
  Evidence: `docs/notes/rust-cli-contract-migration-2026-04-24.md` and `docs/plans/archives/tradingview-cli-stream-read.md` record the implemented stream surface. The `v0.6.0` roadmap must therefore describe future stream work as improving the observation contract and adding Desktop-free/browserless observation candidates, not as inventing streaming from nothing.

## Decision Log

- Decision: Keep a single `tv` binary for `v0.6.0`.
  Rationale: Desktop-free and Desktop-backed commands are related in real workflows, especially source comparison and fallback cases such as `quote --source scanner|chart|auto`. Splitting binaries now would make downstream use and documentation harder before the command source boundary is fully stable.
  Date/Author: 2026-05-02 / Codex

- Decision: Introduce a source taxonomy before any binary split.
  Rationale: The current confusion is mostly about source, mutation, fallback, and freshness semantics. A taxonomy can clarify those without changing command names, release artifacts, or scripts.
  Date/Author: 2026-05-02 / Codex

- Decision: Treat the reported `quote --source chart` symbol/data mismatch as a patch candidate, not as part of this docs-only roadmap slice.
  Rationale: The mismatch affects trust in an existing command and should be fixed in a focused patch plan. This slice records the need and does not change Rust behavior.
  Date/Author: 2026-05-02 / Codex

## Outcomes & Retrospective

The taxonomy and roadmap were added without changing Rust code or public CLI
behavior. Skills now describe Desktop-free, Desktop-backed, hybrid, and
experimental sources consistently. The docs also record that the reported
post-`v0.5.0` `quote --source chart` mismatch is a separate patch candidate.

## Context and Orientation

The repository builds a Rust-native `tv` command. Some commands work without TradingView Desktop by calling credential-free TradingView HTTP or WebSocket surfaces. Other commands connect to the user's local TradingView Desktop through Chrome DevTools Protocol, abbreviated CDP. CDP is the debugging protocol the CLI uses to evaluate JavaScript, capture screenshots, and send input events inside the running Desktop app.

The term command source means where a command gets its data or executes its work. For example, scanner REST is a Desktop-free source, while the selected chart target is a Desktop-backed source. A command can also be mutating, meaning it changes chart, account, editor, or UI state. The source taxonomy must let a user distinguish these cases without needing to understand the Rust crate layout.

Relevant files:

- `README.md` is the user-facing overview and command examples.
- `docs/architecture.md` records stable implementation boundaries.
- `docs/operation-adapter-boundaries.md` records what belongs in `ops` and why.
- `docs/internal-tradingview-apis.md` records non-public TradingView source boundaries.
- `.agents/skills/*/SKILL.md` are runtime agent workflows included in release archives when allowlisted.
- `docs/plans/README.md` points to the active ExecPlan.
- `CHANGELOG.md` records user-facing and durable docs changes.

## Plan of Work

Create `docs/command-source-taxonomy.md` as the source of truth for the current single-binary taxonomy. Define five categories in plain language:

- `Desktop-free read`: no TradingView Desktop or CDP required.
- `Desktop-backed read`: reads state from a running TradingView Desktop target.
- `Desktop-backed operation`: may change chart, account, editor, Replay, Screener, or visible UI state.
- `Hybrid`: has explicit source selection or fallback between Desktop-free and Desktop-backed paths.
- `Experimental`: lab-gated behavior that is not stable enough to treat as a normal read path.

For each category, document `requires_desktop`, `may_mutate`, `fallback_allowed`, `freshness_boundary`, and `recommended_agent_use`. Keep the descriptions compact and public-safe. Do not list every command in the CLI; include representative examples and rules for choosing a source.

Create `docs/v0.6-roadmap.md` with the theme `observation-first TradingView agent toolkit`. Explain that `tv` remains a single binary for now and that `tv` / `tvd` or similar binary split is deferred until the source taxonomy and observation workflows prove that separate commands would help more than hurt. Record three lanes: Desktop-free observation and market data reads, Desktop-backed readiness and recovery, and command taxonomy and user experience clarity. Record `quote --source chart` mismatch as a `v0.5.1` patch candidate.

Update README and stable docs to link to the taxonomy instead of repeating all details inline. The README should keep practical examples, but the long safety paragraph should become easier to scan by naming the taxonomy and keeping only the most important warnings.

Update runtime skills so agents use the taxonomy terms consistently. `market-data-interpretation` should be the main skill for source/freshness interpretation. `chart-analysis`, `multi-symbol-scan`, and `screener-workflow` should mention which category they primarily operate in and should avoid implying that future Desktop-free stream work already exists. Existing `tv stream ...` commands should be described as Desktop-backed current-chart JSONL polling.

## Concrete Steps

Work from the repository root.

1. Move the completed release readiness plan into `docs/plans/archives/`.
2. Add this ExecPlan at `docs/plans/tradingview-cli-v0.6-roadmap-and-source-taxonomy.md`.
3. Add `docs/command-source-taxonomy.md` and `docs/v0.6-roadmap.md`.
4. Update `README.md`, `docs/architecture.md`, `docs/operation-adapter-boundaries.md`, `docs/internal-tradingview-apis.md`, `docs/plans/README.md`, `CHANGELOG.md`, and `packaging/agent/AGENTS.md`.
5. Update the four runtime skills named in the user request: `.agents/skills/market-data-interpretation/SKILL.md`, `.agents/skills/chart-analysis/SKILL.md`, `.agents/skills/multi-symbol-scan/SKILL.md`, and `.agents/skills/screener-workflow/SKILL.md`.
6. Run validation:

        git diff --check
        bash -n scripts/stage-release-package-files.sh
        python3 "$HOME/.codex/skills/.system/skill-creator/scripts/quick_validate.py" .agents/skills/market-data-interpretation
        python3 "$HOME/.codex/skills/.system/skill-creator/scripts/quick_validate.py" .agents/skills/chart-analysis
        python3 "$HOME/.codex/skills/.system/skill-creator/scripts/quick_validate.py" .agents/skills/multi-symbol-scan
        python3 "$HOME/.codex/skills/.system/skill-creator/scripts/quick_validate.py" .agents/skills/screener-workflow
        rg -n "Desktop-free|Desktop-backed|Hybrid|Experimental|tvd|tvb|Computer Use" README.md docs .agents/skills packaging/agent/AGENTS.md
        rg -n '(/Users/|C:\\|USER;|sessionid|cookie|authorization|bearer)' README.md CHANGELOG.md docs .agents/skills packaging scripts || true

Rust code is not expected to change. If Rust code changes, run the normal Rust baseline before committing.

## Validation and Acceptance

Acceptance is met when `docs/command-source-taxonomy.md` clearly defines the categories and representative commands, `docs/v0.6-roadmap.md` records the observation-first direction and deferred binary split, and runtime skills point agents to the same source/freshness model. The taxonomy must state that `tv` remains a single binary for now, that `tvb` or `tvd` are deferred future considerations only, and that `quote --source chart` mismatch is a patch candidate outside this slice.

The validation commands above must pass. The hygiene grep may report existing policy text, archived validation-command examples, and secret-safety wording, but it must not reveal newly added local filesystem paths, account-local identifiers, cookies, tokens, or authorization values.

## Idempotence and Recovery

This work is docs and skills only. Re-running validation is safe. If a skill validation fails, edit only the failing skill and rerun the validator for that skill. If the broad taxonomy grep shows `tvb` or `tvd` outside deferred binary split wording, revise the text so those names are not presented as active commands. If accidental machine-specific paths are added, remove them before committing.

## Artifacts and Notes

No live TradingView payloads, target ids, account-local values, or local absolute paths should be recorded in these docs.

## Interfaces and Dependencies

No public CLI command, JSON payload, Cargo package, or Rust API changes in this slice. The new durable documentation interface is `docs/command-source-taxonomy.md`. Runtime skills may reference that taxonomy by name, but they should remain concise and not duplicate the whole document.

## Open Questions

None.
