# Reorganize agent guidance and runtime skills

## Outcome and scope

Make command selection immediate, runtime references usable from a release
archive, and contributor instructions proportional to the task. This approved
change covers project guides, skills, planning guidance, and packaging checks.
CLI behavior, public API shapes,
dependencies, release versions, live sessions, and remote publication are outside
this change. Work stays in the current session.

## Decisions

- Retain `continuity` as an explicit-only, one-shot handoff skill. Remove its
  automatic PM role, minimum task count, and every-turn persistence rules.
- Replace the old planning template at its existing path so current and archived
  links remain valid; link to specifications instead of duplicating them.
- Merge market-data interpretation, multi-symbol comparison, and screen-result
  analysis into `market-data`. Put purpose, first command, and follow-up condition
  together in its entrypoint; isolate detailed source semantics in references.
- Retain chart, Pine, Replay, Screener, strategy, and release workflows with
  narrow triggers. Move the commit convention into development documentation;
  retire the repository-specific skill-discovery wrapper.
- Keep runtime resources self-contained within each packaged skill root. Share
  common Desktop connection details by reference, not repeated procedures.

## Work and acceptance

Update the guides and skill references together, then update the explicit
package allowlist and its checks. Preserve frozen plans and release evidence.
Verify skill metadata, local reference resolution in both packaged skill roots,
guide parity, package exclusions, public hygiene, and the final diff. Package
checks use a disposable local staging directory and an existing binary; they do
not establish new Rust/runtime behavior. Exercise missing-reference detection
with a deliberately incomplete disposable package. Model behavior improvements
remain unmeasured until observed in normal work.

## Progress

- Approved proposal and four annotations incorporated; clean starting worktree.
- Contributor and packaged guides rewritten; six runtime skills and two
  contributor skills retained, with explicit-only continuity metadata.
- Old plan-index history moved to a historical catalog. Current direction,
  ordering, acceptance records, and the optional local handoff now have distinct
  owners. Frozen plan evidence remains unchanged.
- Validation: seven package-checker fixtures passed; actual staging produced
  48 files with six skills in each root and a byte-identical existing binary.
  Removing the bars reference from both copies made validation fail as intended.
- Changed-document reference checks passed for 106 local links. Seven skills
  passed the generic metadata validator; continuity's supported optional
  explicit-only frontmatter and Codex policy were checked separately because
  that generic validator does not accept the optional frontmatter key.
- Skill and workflow YAML parsed; agent guide parity, public hygiene,
  and diff checks passed. No live operation or model-behavior comparison ran.

## Outcome

The guidance change is complete. Future runtime changes update the relevant
command-selection entry and its owned reference, with package-reference checks
at staging. Static routing and package completeness are verified; improvements
to model behavior and execution time remain to be observed in normal use.
