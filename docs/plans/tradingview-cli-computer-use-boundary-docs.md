# Computer Use boundary docs and skills cleanup

This ExecPlan is a living document. Keep `Progress`, `Discoveries`,
`Decisions`, and `Validation` updated as work proceeds.

## Purpose

Scope Computer Use guidance to environments where it is actually available,
such as the Codex app, and keep the default runtime guidance portable for Codex
CLI, packaged agents, and other non-visual agent environments.

This is a docs and skills cleanup slice. It does not change CLI behavior,
JSON payloads, Rust code, or release packaging contents.

## Progress

- [x] Archived the completed lab-bars evidence review plan.
- [x] Audited current Computer Use references in docs and runtime skills.
- [x] Updated roadmap, internal API docs, operation boundaries, and runtime
      skills to treat Computer Use as an optional environment-specific aid.
- [x] Confirmed packaged agent guidance does not depend on Computer Use.
- [x] Ran skill, docs, packaging, and hygiene validation.
- [x] Committed the docs/skills cleanup.

## Discoveries

- Computer Use references were concentrated in `chart-analysis`,
  `market-data-interpretation`, `multi-symbol-scan`, `docs/v0.5-roadmap.md`,
  and `docs/internal-tradingview-apis.md`.
- `packaging/agent/AGENTS.md` already stays CLI-oriented and does not require
  Computer Use.
- `tv screenshot --region chart|full --output <PATH>` is the portable visual
  evidence path for release archive users and non-Codex-app agents.

## Decisions

- Default operational guidance should be CLI-only: use `tv status`, `tv tab
  list`, `tv state`, source-specific reads, structured errors, and
  `tv screenshot` before any visual/manual fallback.
- Computer Use should appear only as an optional note for environments that
  explicitly provide it. It must not be required or presented as the standard
  recovery path.
- Do not add new runtime skills for Computer Use. Keeping the note short avoids
  extra context load in agents that cannot use it.

## Work Items

- Move `docs/plans/tradingview-cli-lab-bars-evidence-review.md` to
  `docs/plans/archives/`.
- Create this plan and make it current in `docs/plans/README.md`.
- Update `docs/v0.5-roadmap.md` so the `tv` / visual boundary names
  screenshot and manual inspection as portable paths, with Computer Use scoped
  to capable Codex app environments.
- Update runtime skills:
  - `chart-analysis`
  - `market-data-interpretation`
  - `multi-symbol-scan`
  - `screener-workflow` if needed
- Update stable docs:
  - `docs/internal-tradingview-apis.md`
  - `docs/operation-adapter-boundaries.md`
  - `CHANGELOG.md`
- Leave `packaging/agent/AGENTS.md` free of Computer Use requirements.
- Update `CONTINUITY.md` as local-only ledger.

## Validation

- `python3 "${CODEX_HOME:-$HOME/.codex}/skills/.system/skill-creator/scripts/quick_validate.py" .agents/skills/chart-analysis`
- `python3 "${CODEX_HOME:-$HOME/.codex}/skills/.system/skill-creator/scripts/quick_validate.py" .agents/skills/market-data-interpretation`
- `python3 "${CODEX_HOME:-$HOME/.codex}/skills/.system/skill-creator/scripts/quick_validate.py" .agents/skills/multi-symbol-scan`
- `python3 "${CODEX_HOME:-$HOME/.codex}/skills/.system/skill-creator/scripts/quick_validate.py" .agents/skills/screener-workflow`
- `bash -n scripts/stage-release-package-files.sh`
- `git diff --check`
- `rg -n "Computer Use" README.md docs .agents/skills packaging/agent/AGENTS.md`
- `rg -n '(/Users/|C:\\|USER;|sessionid|cookie|authorization|bearer)' README.md CHANGELOG.md docs .agents/skills packaging scripts || true`

## Rollback

Revert the docs and skill edits, move this plan to archives only if the cleanup
is complete, and restore the previous plans index. No Rust state or generated
artifacts are involved.
