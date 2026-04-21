---
name: conventional-commits
description: Enforce Conventional Commits with sentence-case subject style and clear scope/type mapping. Use for commit creation, commit message review, and commit message correction requests. Triggers on "コミットメッセージ", "Conventional Commits", "commit message", "コミット修正", "メッセージを考えて".
argument-hint: "[draft|review|fix-history]"
allowed-tools: Read, Grep, Glob
---

# Conventional Commits Guardrail

Use this skill whenever a commit message is requested, reviewed, or corrected.

## Output Contract

Always produce:

1. `type`
2. `scope` (or `none`)
3. `subject`
4. final one-line commit message

Example:

```text
type: feat
scope: cli
subject: Add trade-log output options
final: feat(cli): Add trade-log output options
```

## Hard Rules

1. Format must be `<type>: <Subject>` or `<type>(<scope>): <Subject>`.
2. `type` must be lowercase (e.g. `feat`, `docs`, `refactor`), even when `Subject` starts uppercase.
3. `Subject` must use sentence case:
   - first word starts uppercase
   - other words are lowercase unless proper nouns, acronyms, or IDs (e.g. `NASDAQ`, `M92`, `ExecPlan`)
4. No trailing period in `Subject`.
5. Do not use placeholders like `Step 1`, `WIP`, `tmp`, `misc`.
6. Prefer imperative verbs: `Add`, `Fix`, `Update`, `Refactor`, `Record`.
7. Avoid title case subjects like `Add Signal Cohorts Command`; use sentence case like `Add signal cohorts command`.

## Type Selection

- `feat`: new user-facing behavior or capability
- `fix`: bug fix or regression fix
- `docs`: docs-only change
- `refactor`: behavior-preserving structural change
- `test`: tests only
- `chore`: maintenance tasks
- `ci`: CI/CD pipeline change
- `build`: dependency/build system change
- `perf`: performance improvement
- `style`: formatting/style-only
- `revert`: revert commit

## Scope Selection (repo defaults)

Prefer one of:

- `cli`, `data`, `core`, `plans`, `notes`, `agents`, `scripts`, `ci`

If multiple areas changed, pick the primary impact area.  
If no meaningful scope exists, omit scope.

## Modes

### `draft` (default)

1. Inspect staged diff summary.
2. Choose `type` and `scope`.
3. Draft 1-3 candidate messages.
4. Put recommended option first.

### `review`

1. Validate candidate message against Hard Rules.
2. If invalid, return corrected message and short reason.

### `fix-history`

1. Confirm rewrite impact first (shared branch risk).
2. Provide a non-interactive rewrite plan.
3. Require explicit user confirmation before executing rewrite commands.

## Prohibitions

1. Do not generate non-Conventional style when user asked for Conventional Commits.
2. Do not uppercase the `type` token (`Feat`, `Docs`, etc. are invalid).
3. Do not lowercase the subject initial.
4. Do not hide uncertainty; if `type` is ambiguous, present 2 options with impact.
