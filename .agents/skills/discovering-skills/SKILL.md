---
name: discovering-skills
description: Discover reusable workflow patterns during active tasks, evaluate them, and convert into skills. Also validates existing skills. Use when repeated procedures emerge, when quality depends on stable multi-step know-how, when a pattern should be standardized, or when checking skill quality. "スキル化して", "パターンをスキルに", "ワークフローをスキルにして", "手順を定型化して", "継続的にスキル", "スキルを発見", "スキルを発明", "スキル検証して", "スキルのチェック".
argument-hint: "[discover|validate] [name]"
allowed-tools: Read, Grep, Glob
---

# Discovering Skills

Detect reusable patterns from active work, evaluate their worth, and hand off creation to the `skill-creator` skill.

## discover (default)

1. **Detect candidates.** Extract any procedure used 2+ times, or one clearly reusable elsewhere.
2. **Evaluate 4 criteria** — skip if fewer than 3 are met:
   - **Reusability**: Applicable to other projects?
   - **Complexity**: Requires multi-step procedures or judgment?
   - **Stability**: Unlikely to change frequently?
   - **Value**: Yields time or quality improvements?
3. **If criteria not met**: Record skip reason in one line and return to normal work.
4. **If criteria met**: Invoke the `skill-creator` skill with the pattern name and description. Provide the discovered pattern as context.
5. **Validate** the created skill using the `validate` subcommand below.
6. **Report**: creation rationale, target pattern, storage location, validation result.

### Failure-Driven Extraction

When skillizing from a failure or review feedback, provide the `skill-creator` skill with:
- **What broke** and root cause
- **Guardrail rule** that must always pass next time
- **Prohibitions** to prevent recurrence
- **Verification proof** to confirm non-recurrence

## validate

Validate a skill for structural correctness, content quality, and best practices.

1. **Resolve target.** Skill name → repo-local `.agents/skills/{name}/SKILL.md`, installed-skill path under `$CODEX_HOME/skills/{name}/SKILL.md`, or a direct path.
2. **Run checks** from `references/validation-checklist.md`.
3. **Report** in Issues / Can Be Reduced / Improvements / Good format.
4. **Do NOT modify files** without user permission.

## Guidelines

- When in doubt, do not create. Prefer skipping over creating low-value skills.
- If overlap with an existing skill is found, update it instead of creating a new one.
- Do not include user-specific paths or local environment details in shared content.
