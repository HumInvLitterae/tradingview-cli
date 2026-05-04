# Skill Validation Checklist

Read this when validating a newly created or updated skill.

## Structural Checks

| Target | Check |
|--------|-------|
| **Frontmatter** | Valid YAML, `name` and `description` present |
| **Directory** | Name matches `name` field in frontmatter |
| **Description** | Includes what it does AND when to trigger; lists trigger phrases |
| **Size** | SKILL.md under 500 lines; large content split to `references/` |
| **Conciseness** | No information the LLM already knows; challenge each paragraph's token cost |
| **allowed-tools** | Present and scoped appropriately (not overly broad) |
| **References** | If `references/` exists, files are linked from SKILL.md with "when to read" guidance |

## Content Checks

| Aspect | Check |
|--------|-------|
| **Missing** | Critical safety measures or error handling absent? |
| **Verbose** | Content that LLM can infer being over-specified? |
| **Duplicate** | Overlap with `AGENTS.md`, other active agent instruction files, or within the same file? |
| **Clarity** | Intent conveyed accurately? |
| **Arguments** | Clear specifications with examples? |
| **Workflow** | Steps logical and unambiguous? |

## Safety Checks (required when applicable)

- Warnings for destructive operations
- Operations requiring user confirmation
- Prohibition of safety bypasses (`--no-verify`, etc.)

## Script Validation (optional)

If a validation script is available:

1. Locate the validator under the installed `skill-creator` skill, typically `**/skill-creator/scripts/quick_validate.py`
2. Run: `python <script> <skill-dir>`
3. On Windows encoding issues: set `PYTHONUTF8=1` and retry

## Output Format

```markdown
## Validation Results: [skill-name]

### Issues
- [Specific problem and reason]

### Can Be Reduced
- [What can be removed/simplified and why]

### Improvements
- [Important items to add]

### Good
- [What to keep]
```
