---
license: MIT
name: validate-skill
description: Static pre-check of an Agent Skill, covering structure, schemas, cross-references, scripts and hooks. Use for a quick structural check while building a skill. It is not certification and says nothing about whether the skill triggers or improves results; use skill-certification for that.
metadata:
  version: '1.1.0'
  tags: "process, orchestration, automation"
---

# Validate Skill

Run the static pre-check against any skill directory: frontmatter, JSON schema validity, cross-reference integrity, script quality and completeness.

**This is not certification.** A skill can pass every check here and never start on a real request. The pre-check overlaps the static levels of `skill-certification` and adds structural checks specific to skills this creator generates. For evidence about conformance to the reference validator, triggering and outcome, run `/skill-certification`.

## Usage

```
/validate-skill
```

## What You'll Be Asked

1. **Skill path** — Directory containing the skill to validate

## What Gets Checked

| Category             | Checks                                                |
| -------------------- | ----------------------------------------------------- |
| **Spec compliance**  | Frontmatter, name/description fields, line count      |
| **JSON schemas**     | Parse validity, required fields, types                |
| **Cross-references** | All referenced files exist                            |
| **Scripts**          | Executable, shebang, strict mode, syntax              |
| **Hooks**            | Valid JSON in the harness format, script paths resolve |
| **Plugin**           | Valid manifest with required fields                   |
| **Sub-skills**       | Each has valid SKILL.md with frontmatter              |

## Output

A validation report with:

- Per-check pass/fail/warn status
- Overall quality score (0.0 - 1.0)
- Specific fix instructions for any failures
- Recommendation: valid, needs fixes, or needs restructuring

A passing report means the skill is ready to be certified, not that it is certified.

## Example

```
/validate-skill
> Path: dist/my-new-skill/
```

Or validate an existing installed skill:

```
/validate-skill
> Path: .agent/skills/iterative-evolver/
```

## Automated Script

The validation can also be run directly:

```bash
bash scripts/validate-skill.sh <skill-directory>
```
