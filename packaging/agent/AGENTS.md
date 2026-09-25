# TradingView CLI Agent Guide

Use `tv` to read TradingView data and operate the user's Desktop session.
This guide and its relative links are authored for the release archive root.
The project is not affiliated with TradingView and does not bypass account,
subscription, exchange-data, or script-ownership requirements.

## Find the binary and choose the task

Use `tv` on PATH, or `./tv` (macOS/Linux) / `.\tv.exe` (Windows) from the unpacked
archive. Run `tv --version` when first identifying a binary. Use
`tv --version --verbose` when commit, build time, dirty state, or platform
matters. On binaries supporting `spec`, use `tv spec <command path>` for JSON
argument metadata; read its coverage limits and unknown semantics. Use
`tv <family> --help` on older binaries or for additional prose. Specification
lookup is offline and does not establish provider or account availability.

| Task | Start here |
| --- | --- |
| Watchlists and simple price alerts, including account changes | [account-management](.agents/skills/account-management/SKILL.md) |
| Price, symbol discovery, comparison, historical bars, freshness | [market-data](.agents/skills/market-data/SKILL.md): purpose-to-command table |
| Selected Desktop chart, studies, viewport, images | [chart-analysis](.agents/skills/chart-analysis/SKILL.md) |
| Pine source and validation | [pine-develop](.agents/skills/pine-develop/SKILL.md) |
| Replay practice and step records | [replay-practice](.agents/skills/replay-practice/SKILL.md) |
| Visible/saved Screener screens, filters, columns | [screener-workflow](.agents/skills/screener-workflow/SKILL.md) |
| Strategy metrics, trades, equity | [strategy-report](.agents/skills/strategy-report/SKILL.md) |

The same skills are supplied under `.claude/skills`. Read the matching entrypoint
and only the references needed for the current question. Setup walkthroughs are
available in [English](docs/getting-started.md) and
[Japanese](docs/ja/getting-started.md).

## Choose MCP or Desktop by capability

Prefer official MCP when it meets the task and credentials are available.
Watchlist management, simple price alerts and data-only screening do not need
Desktop. Honor an explicit source choice and never change sources silently after
failure. Existing command defaults are unchanged. Use the task skill for the
required authentication, source conditions and operation effects:

- [market-data](.agents/skills/market-data/SKILL.md) for independent data reads.
- [account-management](.agents/skills/account-management/SKILL.md) for watchlists
  and price alerts, including readback and unknown mutation outcomes.

Selected chart/Pine evidence, Pine-condition alerts and saved Desktop screen
state still require their Desktop workflows. Recent-count MCP bars do not satisfy
historical date-range requests. Login is explicit; explain browser/OS consent
before starting it and wait for completion. Do not ask for renewed source approval
on every authenticated read within the requested task.

Each runtime skill can also be attached on its own: its required instructions
and references are contained in its directory. Other skill names are optional
handoffs, not dependencies. Copy the complete skill directory when attaching it.

## Authority and evidence

Prefer Desktop-free reads when they answer the question. For Desktop work,
resolve and reuse the intended target using
[Desktop session guidance](.agents/skills/chart-analysis/references/desktop-session.md).
Do not run readiness before each read once the target is confirmed.

A concrete request to change a named target authorizes that effect. Reuse
approval for the same target, scope, and effects, including necessary readback
and fixes within approved limits. A read/analysis request alone does not
authorize changing chart, Pine, Replay, screen, alert, watchlist, layout, tab,
drawing, or other saved state. Explain uncovered effects and obtain approval
before performing them. Use dry-run modes where supported. `--kill-existing`
requires explicit approval; it can terminate the user's Desktop session.

Inspect readback rather than assuming a successful dispatch proves the intended
state. A transport failure can leave a mutation outcome unknown; do not repeat
it until the outcome and remaining permission are resolved. Preserve the
original error and source diagnostics. No diagnostic or follow-up hint by itself
authorizes retry, timeout changes, source substitution, or another mutation.

Keep scanner, historical bars, chart, Replay, quote-data, and screenshots
identified by their actual sources. Null is unknown, not zero. Summaries and
follow-up hints are coverage/routing metadata, not automatic rankings, trades,
or recommendations. Report the observations and their limits without inventing
missing values. Screenshots write local files; they are visual evidence, not
proof of persistence or historical completeness.

Never print secrets, cookies, session data, or private credentials. Keep real
account-local identifiers and raw live payloads out of shared artifacts.

## Skill names

`market-data` replaces `market-data-interpretation`, `multi-symbol-scan`, and
`screener-result-analysis`. Update prompts that explicitly invoked those names.
The binary's commands and JSON contracts are unchanged by this skill reorganization.
