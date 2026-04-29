# Runtime market-analysis skills

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This repository uses `.agents/PLANS.md` as the ExecPlan standard. This document follows that standard and is self-contained so a future contributor can resume the work without reading the chat history.

## Purpose / Big Picture

The `v0.4.0` market-data lane added Desktop-free quotes, batch quotes, scanner field discovery, explicit quote source selection, and typed market/scanner APIs. The runtime skills now need to teach agents how to interpret those reads without mixing source types, overstating freshness, or turning screening output into investment advice.

After this change, release archives include two focused runtime skills: one for interpreting market data sources and one for analyzing scanner or Screener results. Existing operation skills stay lean and refer to these analysis skills instead of absorbing long interpretation checklists.

## Progress

- [x] (2026-04-30T00:00:00Z) Reviewed the current runtime skill layout and release staging allowlist.
- [x] (2026-04-30T00:00:00Z) Created this ExecPlan and added `market-data-interpretation` and `screener-result-analysis`.
- [x] (2026-04-30T00:00:00Z) Updated `chart-analysis`, `multi-symbol-scan`, and `screener-workflow` to reference the new focused skills.
- [x] (2026-04-30T00:00:00Z) Updated release package staging and packaged agent guidance to include the new runtime skills.
- [x] (2026-04-30T00:00:00Z) Ran skill validation, package staging smoke, whitespace check, and hygiene grep.

## Surprises & Discoveries

- Observation: The source project contained useful command patterns, but most were preset or MCP-specific.
  Evidence: The reusable parts were analysis shape, source labeling, due-diligence next steps, and non-advice framing rather than command text or tool names.

- Observation: Existing runtime skills already had some quote and Screener guidance, so the right change was extraction and reference rather than adding more prose to `screener-workflow`.
  Evidence: `screener-workflow` already had an "Interpreting Results" section, while `chart-analysis` and `multi-symbol-scan` already mentioned Desktop-free quote reads.

## Decision Log

- Decision: Add two focused runtime skills instead of one large market-analysis skill.
  Rationale: Market-data source interpretation is shared by chart, scan, and Screener workflows, while Screener result explanation is a narrower research workflow.
  Date/Author: 2026-04-30 / Codex

- Decision: Do not add market-regime, peer-comparison, due-diligence, or investment-thesis skills in this slice.
  Rationale: Those workflows are useful but broader than release-readiness guidance and should be designed with downstream analysis needs in a later plan.
  Date/Author: 2026-04-30 / Codex

- Decision: Include the new skills in release archives.
  Rationale: They are user-facing runtime guidance, not development-only contributor workflows.
  Date/Author: 2026-04-30 / Codex

## Outcomes & Retrospective

Completed. The repository now has focused runtime skills for market data interpretation and Screener result analysis. Existing skills delegate interpretation to them, and release packaging includes them in both `.agents/skills` and `.claude/skills`. No CLI behavior changed.

## Context and Orientation

Runtime skills are stored under `.agents/skills/` and copied into release archives by `scripts/stage-release-package-files.sh`. Development-only skills such as `continuity`, `conventional-commits`, `discovering-skills`, and `release-prep` must not be copied into release packages.

The new skills are based on high-level ideas from a comparable TradingView MCP project: explaining screening results as research candidates, preserving source labels, and giving next research steps. This repository does not copy that project's command files or MCP tool names. The skills are written for the Rust `tv` CLI.

## Plan of Work

Create `.agents/skills/market-data-interpretation/SKILL.md` with concise rules for reading scanner REST quotes, batch quotes, scanner scans, scanner metainfo, chart-sourced quote reads, and chart OHLCV. It should explain `time`, `update_mode`, `delay_seconds`, `extended_hours`, missing values, symbol mismatches, and source/freshness boundaries.

Create `.agents/skills/screener-result-analysis/SKILL.md` with a workflow for explaining scanner or Screener rows as research candidates. It should require the agent to name filters, columns, sort, screen name, or hotlist slug; explain why rows matched; identify concentration and missing data; and suggest next reads without giving buy/sell recommendations.

Shorten `screener-workflow` so it remains an operation skill and points to `screener-result-analysis` for detailed result interpretation. Add brief references from `chart-analysis` and `multi-symbol-scan` to `market-data-interpretation`, and from `multi-symbol-scan` to `screener-result-analysis`.

Update release packaging docs and the staging script so both new runtime skills are bundled. Update the packaged agent guide and changelog.

## Concrete Steps

Run commands from the repository root.

Validate skills:

    python3 <skill-creator>/scripts/quick_validate.py .agents/skills/market-data-interpretation
    python3 <skill-creator>/scripts/quick_validate.py .agents/skills/screener-result-analysis
    python3 <skill-creator>/scripts/quick_validate.py .agents/skills/chart-analysis
    python3 <skill-creator>/scripts/quick_validate.py .agents/skills/multi-symbol-scan
    python3 <skill-creator>/scripts/quick_validate.py .agents/skills/screener-workflow

Validate packaging:

    bash -n scripts/stage-release-package-files.sh
    rm -rf target/skill-package-smoke
    scripts/stage-release-package-files.sh target/skill-package-smoke target/debug/tv
    find target/skill-package-smoke/.agents/skills -maxdepth 2 -name SKILL.md | sort
    find target/skill-package-smoke/.claude/skills -maxdepth 2 -name SKILL.md | sort

Validate hygiene:

    git diff --check
    rg -n '(/Users/|C:\\|USER;|sessionid|cookie|authorization|bearer)' README.md CHANGELOG.md docs .agents/skills packaging scripts || true

Completed validation:

    python3 <skill-creator>/scripts/quick_validate.py .agents/skills/market-data-interpretation
    python3 <skill-creator>/scripts/quick_validate.py .agents/skills/screener-result-analysis
    python3 <skill-creator>/scripts/quick_validate.py .agents/skills/chart-analysis
    python3 <skill-creator>/scripts/quick_validate.py .agents/skills/multi-symbol-scan
    python3 <skill-creator>/scripts/quick_validate.py .agents/skills/screener-workflow
    bash -n scripts/stage-release-package-files.sh
    rm -rf target/skill-package-smoke
    scripts/stage-release-package-files.sh target/skill-package-smoke target/debug/tv
    find target/skill-package-smoke/.agents/skills -maxdepth 2 -name SKILL.md | sort
    find target/skill-package-smoke/.claude/skills -maxdepth 2 -name SKILL.md | sort
    git diff --check
    rg -n '(/Users/|C:\\|USER;|sessionid|cookie|authorization|bearer)' README.md CHANGELOG.md docs .agents/skills packaging scripts || true

## Validation and Acceptance

Acceptance is met when both new skills validate, existing changed skills validate, release staging includes the two new runtime skills in both agent skill roots, and development-only skills remain excluded. No command behavior, Rust code, or public JSON payload should change.

## Idempotence and Recovery

This work is documentation and packaging guidance only. It is safe to rerun validation and staging. If the staging smoke fails because `target/debug/tv` is missing, run a normal debug build or use an existing release binary; do not change the staging script just for the smoke.

## Artifacts and Notes

Do not copy local paths, raw command text, MCP tool identifiers, account-local values, cookies, tokens, or raw market payloads from comparable projects into this repository. Record only reusable workflow ideas and `tv` CLI-specific instructions.

## Interfaces and Dependencies

The release skill allowlist must include:

    chart-analysis
    market-data-interpretation
    multi-symbol-scan
    pine-develop
    replay-practice
    screener-result-analysis
    screener-workflow
    strategy-report

Development-only skills remain excluded.

## Open Questions

- UNCONFIRMED: Whether future releases should add separate `market-regime-snapshot`, `peer-comparison`, or `due-diligence` skills. They are intentionally deferred from this slice.
