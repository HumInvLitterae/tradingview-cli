# Public docs and agent guide audience cleanup

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document follows `.agents/PLANS.md` from the repository root. It is self-contained and describes how to clean current public and agent-facing documentation before `v0.6.0` release readiness.

## Purpose / Big Picture

The current documentation still mixes human-facing overview, contributor handoff, implementation history, and runtime agent guidance. Some current docs also describe the project with old early-implementation phase labels, which now read like current version or release labels rather than historical implementation context.

This cleanup separates audiences:

- `README.md` is a human-facing public overview.
- root `AGENTS.md` and `CLAUDE.md` are contributor-facing agent guides and remain identical.
- `packaging/agent/AGENTS.md` is the runtime guide copied into release archives.

This work is docs-only. It does not change CLI behavior, JSON payloads, version numbers, source taxonomy semantics, or release packaging policy.

## Progress

- [x] (2026-05-05T02:19Z) Audited current README, root agent guides, packaged agent guide, roadmap, plan index, and stale early-implementation phase references.
- [x] (2026-05-05T02:31Z) Rewrote README as a human-facing public overview with shorter examples and a documentation index.
- [x] (2026-05-05T02:31Z) Rewrote root `AGENTS.md` and `CLAUDE.md` as matching contributor-facing guides.
- [x] (2026-05-05T02:31Z) Rewrote `packaging/agent/AGENTS.md` as a runtime archive guide.
- [x] (2026-05-05T02:35Z) Updated roadmap, plan index, and changelog.
- [x] (2026-05-05T02:45Z) Ran docs, packaging, staged-package, symlink, and hygiene validation.
- [ ] Commit the related changes.

## Surprises & Discoveries

- Observation: root `AGENTS.md` and `CLAUDE.md` were identical and should remain identical.
  Evidence: `cmp -s AGENTS.md CLAUDE.md` returned identical before edits.

- Observation: release archives copy both `AGENTS.md` and `CLAUDE.md` from `packaging/agent/AGENTS.md`.
  Evidence: `scripts/stage-release-package-files.sh` copies that single runtime guide to both package filenames.

## Decision Log

- Decision: Remove old phase-label current-state language from README and root agent guides, but do not rewrite historical notes or archived plans.
  Rationale: Historical documents may retain phase language, while current docs should not make `v1` look like the active release or product name.
  Date/Author: 2026-05-05 / Codex.

- Decision: Keep README examples representative instead of preserving a full command inventory.
  Rationale: A long command dump made the README harder for humans to scan. Full details belong in `tv --help` and stable docs.
  Date/Author: 2026-05-05 / Codex.

## Outcomes & Retrospective

Implemented. README now reads as a shorter human-facing public overview. Root `AGENTS.md` and `CLAUDE.md` are matching contributor-facing guides, with `CLAUDE.md` preserved as a symlink to `AGENTS.md`. The packaged runtime guide is now scoped to release archive users and agents. Current docs no longer present early implementation phase labels or old handoff entrypoints as current guidance.

## Context and Orientation

The v0.6.0 roadmap has already added stream observation controls, source taxonomy metadata, readiness and screenshot diagnostics, Desktop-free metadata, and root `--version` support. Before release readiness, the public docs need to reflect that current state cleanly.

The cleanup must avoid local validation-environment details, raw target ids, account-local metadata, credentials, or machine-specific paths.

## Plan of Work

Rewrite the current public docs by audience rather than by implementation history. Keep current source-of-truth links short, move historical handoff references out of the main path, and ensure release-packaged guidance is runtime-focused.

Do not edit Rust code or version numbers. Do not change release packaging behavior except through text that explains the existing behavior.

## Concrete Steps

From the repository root, run:

    git diff --check
    bash -n scripts/stage-release-package-files.sh
    rm -rf target/docs-package-smoke
    scripts/stage-release-package-files.sh target/docs-package-smoke target/debug/tv
    find target/docs-package-smoke -maxdepth 4 -print | sort

Also run targeted greps over current public docs for stale early-phase labels,
old README handoff headings, and historical handoff-note links. Run a broad
hygiene grep over public docs and packaged assets for local paths, credentials,
raw target ids, account-local metadata, and local validation-environment
wording.

## Validation and Acceptance

Acceptance is reached when current docs no longer present old phase labels as current state, README reads as a public overview, root agent guides read as contributor guidance, packaged agent guides read as runtime guidance, and packaging validation shows the staged archive receives the runtime guide rather than the contributor guide.

## Idempotence and Recovery

This is a docs-only cleanup. If a validation grep finds historical terms in archived plans or old notes, leave them unless they are linked as current guidance. If staged package validation fails because `target/debug/tv` does not exist, build or use an existing binary path before rerunning the staging command.

## Artifacts and Notes

Validation evidence:

    cmp -s AGENTS.md CLAUDE.md
    targeted grep for stale phase labels, old README handoff headings, and historical handoff-note links in current public docs
    git diff --check
    bash -n scripts/stage-release-package-files.sh
    rm -rf target/docs-package-smoke
    scripts/stage-release-package-files.sh target/docs-package-smoke target/debug/tv
    find target/docs-package-smoke -maxdepth 4 -print | sort
    cmp -s target/docs-package-smoke/AGENTS.md packaging/agent/AGENTS.md
    cmp -s target/docs-package-smoke/CLAUDE.md packaging/agent/AGENTS.md

All completed successfully. The broad hygiene grep reported existing policy text, archived validation-command examples, and secret-safety wording only; no new local environment-specific guidance, local path, raw target id, account-local metadata, credential, or raw live payload was added.

## Interfaces and Dependencies

No CLI interface, Rust API, or dependency changes are introduced.

## Open Questions

No open questions.
