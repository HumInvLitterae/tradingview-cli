# Snapshot workflow docs and skills alignment

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document follows `.agents/PLANS.md` from the repository root. It is self-contained and describes a docs-and-skills slice for the `v0.8.0` snapshot workflow lane.

## Purpose / Big Picture

`tv snapshot <SYMBOL>` now exists as a Desktop-free, single-symbol evidence packet. It combines scanner quote, symbol info, and scanner-backed fundamentals sections without mutating the TradingView Desktop chart.

The current gap is workflow guidance. Stable docs mention snapshot, but runtime skills still point agents toward manually stitching together `tv quote`, `tv info`, and `tv fundamentals` for many one-symbol first-pass checks. After this change, agents should use `tv snapshot` for one-symbol Desktop-free context, `tv quotes` and scanner reads for broader comparisons, and `tv observe chart` when selected-chart state over time matters.

## Progress

- [x] (2026-05-06T00:00Z) Created this ExecPlan.
- [x] (2026-05-06T00:00Z) Updated roadmap and plan index so snapshot workflow alignment is the current slice.
- [x] (2026-05-06T00:00Z) Synchronized runtime skills with the snapshot workflow.
- [x] (2026-05-06T00:00Z) Confirmed public docs already describe the snapshot / quotes / observe split clearly; no README or workflow-guide expansion was needed.
- [x] (2026-05-06T00:00Z) Validated docs, skills, packaging script syntax, and hygiene.
- [x] (2026-05-06T00:00Z) Commit the slice.

## Surprises & Discoveries

- Stable docs already introduce `tv snapshot` and the observation workflow split. The useful work was mostly runtime skill alignment rather than expanding README.

## Decision Log

- Decision: Do not add new CLI surface, options, JSON fields, or Rust code in this slice.
  Rationale: The implemented snapshot command is enough for the first v0.8 workflow step. The current risk is agents choosing lower-level reads unnecessarily.
  Date/Author: 2026-05-06 / Codex.

- Decision: Keep `snapshot` as a one-symbol Desktop-free JSON packet, not a batch, watch, JSONL, chart-backed, or screenshot-producing command.
  Rationale: The roadmap intentionally separates horizontal single-symbol evidence from Desktop-backed time-window observation.
  Date/Author: 2026-05-06 / Codex.

## Outcomes & Retrospective

Runtime skills now treat `tv snapshot <SYMBOL>` as the one-symbol Desktop-free evidence packet before chart observation. `tv quotes` and scanner reads remain the preferred multi-symbol / broad discovery path, while `tv observe chart` remains the Desktop-backed time-window observation workflow.

No Rust code, CLI behavior, JSON payload, dependency, README expansion, or release version changed.

Validation passed for the changed runtime skills, `git diff --check`, and packaging script syntax. The broad hygiene grep reported existing policy language, archived validation-command examples, and this plan's safety wording; no new local path, credential, raw target id, account-local metadata, or raw live payload was added.

## Context and Orientation

`tv snapshot <SYMBOL>` returns a `command: "snapshot"` envelope with top-level Desktop-free source metadata and section-level results for quote, info, and fundamentals. Each section may succeed or fail independently. Snapshot is useful before mutating a chart or starting a Desktop-backed observation window.

The adjacent commands keep their own roles:

- `tv quotes <SYMBOL>...` for ordered batch quote comparisons;
- `tv scanner scan` and `tv scanner hotlist` for broad discovery;
- `tv fundamentals <SYMBOL>` when only fundamentals fields are needed;
- `tv readiness` and `tv observe chart` for Desktop-backed selected-chart state;
- `tv screenshot` for visual evidence when structured fields are not enough.

## Plan of Work

Update `docs/plans/README.md` and `docs/v0.8-roadmap.md` to make this docs-and-skills alignment the current slice.

Update runtime skills:

- `market-data-interpretation`: list `tv snapshot <SYMBOL>` among Desktop-free reads and explain quote/info/fundamentals sections, section-level errors, and when lower-level reads remain preferable.
- `multi-symbol-scan`: use `tv quotes` and scanner reads for broad or multi-symbol comparison, then `tv snapshot` for one-symbol finalist context before chart mutation.
- `chart-analysis`: recommend `tv snapshot` for static one-symbol context before switching the visible chart when chart evidence is not yet required.
- `screener-result-analysis`: include `tv snapshot` as a next research step for a candidate row.

Keep README and `docs/observation-workflows.md` short. Edit them only if they contradict the source split above.

Add a concise `CHANGELOG.md` documentation entry.

## Concrete Steps

From the repository root, inspect the planned files:

    git status --short
    rg -n "snapshot|observe chart|tv quotes|scanner scan|Desktop-free|Desktop-backed" README.md docs .agents/skills

Make the docs and skill edits described above. Then validate:

    git diff --check
    bash -n scripts/stage-release-package-files.sh

Run the repository's existing skill validator for each changed runtime skill. Use the local validation method appropriate to the environment, but do not record private local validation setup in public docs.

Run hygiene checks:

    rg -n '(/Users/|C:\\|USER;|sessionid|cookie|authorization|bearer|raw live payload|account-local)' README.md CHANGELOG.md docs .agents/skills packaging scripts || true

Rust tests are not required because this slice does not touch Rust code. If Rust code is touched unexpectedly, run:

    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1

## Validation and Acceptance

Acceptance is met when:

- runtime skills treat `tv snapshot <SYMBOL>` as the default one-symbol Desktop-free evidence packet;
- runtime skills continue to use `tv quotes` / scanner reads for multi-symbol or broad discovery;
- runtime skills continue to use `tv observe chart` only when selected-chart state over time matters;
- public docs do not present `snapshot` as batch, JSONL, watch, chart-backed, screenshot-backed, or experimental bars evidence;
- docs and packaging script checks pass.

## Idempotence and Recovery

All changes are ordinary Markdown edits and can be repeated safely. If a skill validation fails, fix the affected `SKILL.md` and rerun only that validator before rerunning final docs checks.

## Artifacts and Notes

The archived snapshot implementation plan is `docs/plans/archives/tradingview-cli-symbol-snapshot.md`.

## Interfaces and Dependencies

No Rust API, CLI option, JSON payload, command behavior, dependency, or release packaging allowlist changes are part of this plan.

## Open Questions

None. Future snapshot surface expansion remains evidence-gated by the roadmap.
