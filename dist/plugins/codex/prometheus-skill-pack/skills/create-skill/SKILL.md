---
license: MIT
name: create-skill
description: Create a new Agent Skill from scratch using PMPO orchestration. Generates a complete skill package with all required files.
metadata:
  version: '1.1.0'
  tags: "process, orchestration, automation"
---

# Create Skill

Generate a complete Agent Skill from a description of what it should do. The work runs as a KBD phase and ends with certification by `skill-certification`; if a level fails, the operator chooses between fixing, accepting with a recorded exception, and abandoning.

## Usage

```
/create-skill
```

## What You'll Be Asked

1. **Skill name** — What to call the skill (kebab-case)
2. **Intent** — What the skill should do
3. **Complexity** — Simple (3-5 files), Standard (15-25), or Full (30-50+)
4. **Platforms** — Which platforms to support (agentskills-io, claude-code, opencode)
5. **Tools directory** — Custom path for OpenCode tools (optional)

## What Gets Generated

A complete skill directory at the skill's real destination (in the skill pack, `skills/<domain>/<name>/`; elsewhere `dist/<skill-name>/` unless you name a directory) with:

- Core files (SKILL.md, CLAUDE.md, AGENTS.md)
- PMPO phase controllers
- Agent definitions
- Reference materials
- JSON schemas
- State management scripts (if standard/full)
- Lifecycle hooks (if standard/full)
- Platform-specific outputs
- Eval cases (`evals/`), drafted with Anthropic's `skill-creator` when it is installed

Then, at the phase boundary: the static pre-check, an adversarial review, and a certificate (`certificate.json`, `report.md`). Trigger and outcome certification makes billed model runs and asks first.

## Example

```
/create-skill
> Name: code-reviewer
> Intent: Review code changes for quality, security, and best practices
> Complexity: standard
> Platforms: agentskills-io, claude-code
```
