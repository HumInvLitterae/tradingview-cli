# Screener surface completion and stabilization

This note records the current boundary for the Rust-native `tv screener` surface.
It is the starting point for stabilization work after the first broad Screener
implementation pass.

## Current implemented surface

The main Screener surface is implemented and should be treated as feature
coverage complete for the currently planned upstream follow-up pass:

- `tv screener status`
- `tv screener open`
- `tv screener get [--limit <N>]`
- `tv screener close`
- `tv screener screens active`
- `tv screener screens actions`
- `tv screener screens list [--catalog]`
- `tv screener screens switch --name <NAME> [--catalog] [--dry-run]`
- `tv screener screens save [--dry-run]`
- `tv screener screens create --name <NAME> [--dry-run]`
- `tv screener screens rename --name <CURRENT> --to <NEW> [--dry-run]`
- `tv screener screens save-as --name <NAME> [--dry-run]`
- `tv screener screens delete --name <NAME> [--dry-run] --confirm-delete`
- `tv screener filters list`
- `tv screener filters actions`
- `tv screener filters add --name <TEXT> --min <N>|--max <N> [--dry-run]`
- `tv screener filters modify --index <N>|--text <TEXT> --min <N>|--max <N> [--dry-run]`
- `tv screener filters modify --index <N>|--text <TEXT> --option <TEXT> [--dry-run]`
- `tv screener filters remove --index <N>|--text <TEXT> [--dry-run]`
- `tv screener filters clear [--dry-run] --confirm-clear`
- `tv screener columns list`
- `tv screener columns actions`
- `tv screener columns config`
- `tv screener columns add --id <COLUMN_ID> [--params-json <JSON>] [--after-index <N>] [--dry-run]`
- `tv screener columns remove --index <N>|--name <TEXT> [--dry-run]`
- `tv screener columns reorder --from-index <N> --to-index <N> [--dry-run]`

This does not mean every possible TradingView Screener UI workflow is
implemented. It means the project should now shift from adding new Screener
commands to hardening the implemented contract.

## Deferred boundaries

The following remain evidence-gated rather than accidentally unfinished:

- `tv screener columns reset`: deferred because the current full-page Screener
  evidence does not expose a reliable default column source or visible reset
  action that can be post-checked.
- Display-name catalog lookup for `columns add`: deferred. The implemented
  command is intentionally low-level and accepts a known storage column id plus
  JSON-object params.
- Broader multi-option filter workflow semantics: deferred. The implemented
  `--option` path covers a single visible option selection on one existing
  option-style filter.
- Free-text filter editors and arbitrary non-numeric editors: deferred until a
  separate plan proves a stable UI, input, and post-check path.
- Downstream scanner workflow packs: not part of the core CLI unless future
  evidence shows a command belongs in this repository rather than in a
  downstream adapter or skill.

## Stabilization priorities

Stabilization should focus on the implemented surface, not on adding the
deferred commands above. The highest-value checks are:

- Prefer a full-page Screener target discovered through `tv tab list` and
  `screener_targets[].target_env.TV_CDP_TARGET_ID` for live smoke.
- Run read-only and `--dry-run` commands before normal mutations.
- Keep normal screen lifecycle and column storage mutations limited to prepared
  test or disposable screens whose names contain `CLI-Test` or `テスト`.
- Keep filter mutations guarded by visible filter-pill post-checks.
- Keep column storage mutations guarded by storage re-fetch and exact id/params
  order post-checks.
- Close stale transient popups before opening filter edit popovers, because
  TradingView may leave previous menus visible after manual or failed smoke.
- Treat normal filter option mutation as still UI-fragile: current smoke can
  time out, but the command must not report success unless the visible filter
  text post-check passes.
- Record any leftover test screen, filter, or column state in the relevant
  ExecPlan and `CONTINUITY.md`.

## Next documentation and skill work

After the stabilization pass, review repo-local runtime skills for Screener
coverage. A dedicated Screener workflow skill may be useful if existing skills
do not clearly tell users how to pick the full-page target, use dry-run first,
operate on test screens, and clean up disposable filters or columns.
