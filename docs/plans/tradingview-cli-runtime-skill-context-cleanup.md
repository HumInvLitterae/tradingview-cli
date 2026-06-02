# Runtime skill context cleanup

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This plan follows `.agents/PLANS.md`. It is self-contained and describes the documentation and runtime skill cleanup needed before `v0.24.0` release readiness. It does not change CLI behavior, JSON payload contracts, Rust APIs, dependencies, or version numbers.

## Purpose / Big Picture

Runtime skills are loaded by agents to decide what to do next. They should be compact enough to guide action without filling the model context with rarely needed details. Recent feature slices correctly added `tv launch`, `tv bars` symbol resolution, and `tv events` guidance, but the main `SKILL.md` bodies became too dense because source taxonomy, historical edge cases, and contract details were appended into the everyday workflow.

After this cleanup, agents should still know the important boundaries, but they should read long source notes only when the task requires them. The observable outcome is shorter `SKILL.md` files with clear reference links, valid skill packages, and no loss of critical source-boundary guidance.

## Progress

- [x] (2026-06-03 18:30Z) Created this ExecPlan and archived the completed v0.24 pre-release audit plan.
- [x] (2026-06-03 18:35Z) Reviewed the three largest runtime skills, existing references, and release packaging behavior.
- [x] (2026-06-03 18:45Z) Rewrote runtime skills so the main bodies are concise and detailed source notes live in references.
- [x] (2026-06-03 18:50Z) Ran an empirical-prompt-tuning structural review pass and applied small clarity fixes.
- [x] (2026-06-03 18:55Z) Validated skills, packaging script syntax, and diff hygiene.
- [ ] Commit the related documentation and skill cleanup changes.

## Surprises & Discoveries

- Observation: Release packaging copies whole runtime skill directories.
  Evidence: `scripts/stage-release-package-files.sh` uses `cp -R ".agents/skills/$skill" "$root/$skill"`, so new `references/` files under runtime skills are included without changing the packaging script.

- Observation: The largest runtime skills are large enough to create context pressure.
  Evidence: `wc -l` reported 348 lines for `market-data-interpretation`, 215 lines for `chart-analysis`, and 196 lines for `multi-symbol-scan`.

- Observation: After the split, the always-loaded skill bodies are materially smaller while references still carry the detailed source-boundary notes.
  Evidence: `wc -l` reported 91 lines for `market-data-interpretation/SKILL.md`, 92 lines for `chart-analysis/SKILL.md`, and 85 lines for `multi-symbol-scan/SKILL.md`.

- Observation: The empirical-prompt-tuning structural review found no large structural gap.
  Evidence: The review verdict was "small fixes suggested"; it specifically confirmed that routine requests can start from the short `SKILL.md` files and that detailed `tv bars`, quote-data, JSONL, Replay, and unsupported-feature notes live in references.

## Decision Log

- Decision: Keep `SKILL.md` as the always-read short workflow and move detailed source semantics into `references/`.
  Rationale: The skill frontmatter triggers load `SKILL.md`; putting every feature contract there makes common tasks pay for rare edge-case context. References preserve detail without forcing it into every run.
  Date/Author: 2026-06-03 / Codex

- Decision: Do not change release packaging for this cleanup.
  Rationale: Runtime skills are copied recursively, so reference files are already included in release archives. Broadening the package allowlist would add churn without benefit.
  Date/Author: 2026-06-03 / Codex

- Decision: Clarify that references are loaded only when needed.
  Rationale: The structural review noted that unconditional "read docs" wording could make agents load `docs/observation-workflows.md` on routine tasks. The final wording says to use `SKILL.md` for routine work and read references or docs only when source details or command choice are unclear.
  Date/Author: 2026-06-03 / Codex

## Outcomes & Retrospective

The cleanup reduced the always-loaded runtime skill bodies from 348 / 215 / 196 lines to 91 / 92 / 85 lines for market data interpretation, chart analysis, and multi-symbol scan respectively. Detailed `tv bars`, `tv events`, quote-data, selected-chart export, Replay, JSONL, and unsupported-feature notes now live in skill references.

Validation passed:

    uvx --with pyyaml python <skill-validator>/quick_validate.py .agents/skills/market-data-interpretation
    uvx --with pyyaml python <skill-validator>/quick_validate.py .agents/skills/chart-analysis
    uvx --with pyyaml python <skill-validator>/quick_validate.py .agents/skills/multi-symbol-scan
    git diff --check
    bash -n scripts/stage-release-package-files.sh

No CLI behavior, JSON payload contract, Rust API, dependency, or packaging-script change was made. The next step can return to `v0.24.0 release readiness`.

## Context and Orientation

The runtime skills live under `.agents/skills/`. Release archives copy selected runtime skill directories into both `.agents/skills` and `.claude/skills`. The most relevant skills for this cleanup are:

- `.agents/skills/market-data-interpretation/SKILL.md`
- `.agents/skills/chart-analysis/SKILL.md`
- `.agents/skills/multi-symbol-scan/SKILL.md`

`chart-analysis` and `multi-symbol-scan` already have `references/workflow.md`. `market-data-interpretation` does not yet have references, so this plan adds `references/source-boundaries.md`.

## Plan of Work

First, rewrite the three main `SKILL.md` files so they provide a short working recipe: when to use the skill, which command family to start with, which source metadata must be reported, when to read references, and what not to infer.

Second, move dense source details into references. Put the long market-data source taxonomy in `market-data-interpretation/references/source-boundaries.md`. Update chart and multi-symbol workflow references so they contain the detailed command mapping and edge-case notes that were previously crowding `SKILL.md`.

Third, run an empirical-prompt-tuning style structural review. Because the task is prompt / skill quality, ask a blank-slate reviewer to inspect whether the rewritten skills are concise, self-contained enough for common tasks, and clear about when to open references. Apply only small clarity fixes from that review.

Finally, validate the changed skills and packaging. This is docs / skill work only; Rust tests are not required unless implementation files are touched.

## Validation and Acceptance

Run and expect success:

    uvx --with pyyaml python <skill-validator>/quick_validate.py .agents/skills/market-data-interpretation
    uvx --with pyyaml python <skill-validator>/quick_validate.py .agents/skills/chart-analysis
    uvx --with pyyaml python <skill-validator>/quick_validate.py .agents/skills/multi-symbol-scan
    git diff --check
    bash -n scripts/stage-release-package-files.sh

Also inspect:

    wc -l .agents/skills/market-data-interpretation/SKILL.md .agents/skills/chart-analysis/SKILL.md .agents/skills/multi-symbol-scan/SKILL.md
    rg -n "tv events|events\\.v1|tv bars|symbol_resolution|quote-data|range_fetch_summary|range_alignment|source mixing|ranking|recommendation" .agents/skills

Acceptance means the main skill files are materially shorter, references preserve the detailed source-boundary knowledge, source mixing / ranking / recommendation guardrails remain present, and release packaging still includes references through recursive skill directory copying.

## Idempotence and Recovery

This cleanup is safe to rerun. If a rewritten skill loses a critical boundary, restore the missing detail from the prior version or move it into the relevant reference. If validation fails, fix the specific skill frontmatter or markdown issue and rerun the validator.

## Interfaces and Dependencies

No CLI interface changes are allowed. No new dependencies are added. The only expected new file is:

    .agents/skills/market-data-interpretation/references/source-boundaries.md

## Open Questions

No critical open question remains. If validation passes, the next step returns to `v0.24.0 release readiness`.
