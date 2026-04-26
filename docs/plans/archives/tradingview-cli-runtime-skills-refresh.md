# Runtime skills refresh

This ExecPlan records the runtime skill refresh after the scanner, Screener,
watchlist, and alert follow-up work. It is intentionally documentation and
packaging focused; no Rust command implementation is part of this slice.

## Purpose / Big Picture

Release archives include user-facing agent guides and runtime skills. Those
skills should teach the current Rust `tv` operating surface rather than the
older MCP or early-Rust command boundary. After the API-backed watchlist and
alert work, and after the broad Screener surface, the archive needs a dedicated
Screener workflow skill plus small updates to chart and multi-symbol workflows.

## Progress

- [x] (2026-04-27) Reviewed current runtime skills, release packaging script,
  package agent guide, Screener completion note, and internal API reference.
- [x] (2026-04-27) Added `screener-workflow` as a runtime skill.
- [x] (2026-04-27) Updated `chart-analysis` and `multi-symbol-scan` to reflect
  target selection, scanner reads, symbol-targeted quote, and API-backed
  watchlist mutation.
- [x] (2026-04-27) Added `screener-workflow` to release package allowlist and
  user-facing agent guide.
- [x] (2026-04-27) Ran skill validation, packaging smoke, and docs hygiene checks. Commit remains the final repository action.

## Decisions

- Add a new skill for Screener instead of folding it into `multi-symbol-scan`.
  Screener operation has its own target-selection, dry-run-first, test-screen,
  filter, and column safety rules.
- Keep scanner and watchlist guidance in `multi-symbol-scan`. The scanner
  commands are discovery inputs, and watchlist mutation is a follow-up action
  after user approval.
- Do not add a separate alert skill. Alert creation is now more stable, but it
  remains a small account mutation surface better covered by the general agent
  guide and explicit user approval rules.
- Keep release packaging allowlisted. Development-only skills must not be
  copied into release archives.

## Implementation

Create `.agents/skills/screener-workflow/SKILL.md` with concise instructions
for:

- selecting a full-page Screener target through `tv tab list`;
- running read-only and dry-run commands before mutations;
- using disposable screen names such as `CLI-Test` or names containing `テスト`;
- operating screen lifecycle, filters, and columns within the implemented
  surface;
- treating `columns reset` and broad multi-option or free-text filter editing
  as deferred boundaries.

Update `multi-symbol-scan` so broad discovery starts with `tv scanner hotlist`
or `tv scanner scan`, then uses symbol-targeted quote and chart reads for a
small finalist set. Watchlist write-back should use `tv watchlist add-bulk`
only after user approval.

Update `chart-analysis` to make target ambiguity handling explicit and to note
that `tv quote [SYMBOL]` temporarily switches the chart but verifies restore.

Update the release package allowlist, the user-facing package `AGENTS.md`, and
root README/CHANGELOG enough to make the new runtime skill visible.

## Validation

Run:

```bash
python <skill-creator-validator> .agents/skills/screener-workflow
python <skill-creator-validator> .agents/skills/chart-analysis
python <skill-creator-validator> .agents/skills/multi-symbol-scan
bash -n scripts/stage-release-package-files.sh
cargo build --release --locked
rm -rf target/release-package-smoke
scripts/stage-release-package-files.sh target/release-package-smoke target/release/tv
find target/release-package-smoke -maxdepth 4 -print | sort
git diff --check
git grep -nE '(/Users/|C:\\|USER;|sessionid|cookie|authorization|bearer)' -- README.md CHANGELOG.md docs .agents/skills packaging scripts || true
git status --short
```

The package smoke must show `screener-workflow` under both `.agents/skills` and
`.claude/skills`, while `continuity`, `conventional-commits`,
`discovering-skills`, and `release-prep` remain absent.

Validation result on 2026-04-27:

- skill validator returned `Skill is valid!` for `screener-workflow`,
  `chart-analysis`, and `multi-symbol-scan`
- release package staging included `screener-workflow` under both agent roots
- release package staging excluded development-only skills
- `bash -n scripts/stage-release-package-files.sh`, `cargo build --release
  --locked`, `git diff --check`, and tracked-doc hygiene grep passed
- the hygiene grep reported only existing validation-command examples and
  public-safe safety policy language, not new live account identifiers or
  credentials

## Notes

This slice should update `CONTINUITY.md` after validation. `CONTINUITY.md` is
local continuity state and must not be committed.
