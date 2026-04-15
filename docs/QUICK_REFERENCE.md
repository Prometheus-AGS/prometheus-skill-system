# Quick Reference Guide

Quick reference for common operations in the Prometheus Skill Pack repository.

## Commands Cheat Sheet

### Validation
```bash
npm run validate                           # Validate all skills
npm run validate:skill skills/react/name   # Validate specific skill
npm run check-format                       # Check code formatting
npm run format                             # Auto-fix formatting
```

### Build & Distribution
```bash
npm run build              # Build marketplace (create symlinks)
npm run install:user       # Install to ~/.claude/skills/
npm run install:project    # Install to .claude/skills/
```

### Development
```bash
npm test                   # Run tests
npm run lint               # Lint skills
npm run dev                # Watch mode
```

## Directory Quick Reference

```
Root directories:
├── skills/          Main skills (source of truth)
├── .claude-plugin/  Plugin format (symlinks to skills/)
├── shared/          Reusable resources
├── scripts/         Build and validation tools
├── examples/        Example implementations
└── docs/            Documentation

Skill structure:
skill-name/
├── SKILL.md         Required: instructions + frontmatter
├── scripts/         Optional: executable code
├── references/      Optional: detailed docs
└── assets/          Optional: templates, resources
```

## Skill Frontmatter Template

```yaml
---
name: skill-name
description: What it does and when to use it (max 1024 chars)
license: MIT
metadata:
  author: your-name
  version: "1.0.0"
  category: react|rust|ui-ux|devops|testing|documentation
  tags: [tag1, tag2]
---
```

## Naming Rules

- **Skills**: `kebab-case` only (e.g., `react-entity-crud`)
- **Pattern**: `^[a-z0-9]+(-[a-z0-9]+)*$`
- **Max length**: 64 characters
- **No**: uppercase, underscores, consecutive hyphens, leading/trailing hyphens

## Path Rules

✅ **Use**: `scripts/file.sh` (forward slashes)
❌ **Avoid**: `scripts\file.sh` (backslashes)

## Script Best Practices

```bash
#!/bin/bash
set -euo pipefail

# Help
[[ "$1" == "--help" ]] && { echo "Usage..."; exit 0; }

# JSON output
cat << EOF
{
  "status": "success",
  "result": "value"
}
EOF
```

## Environment Variables

- `$CLAUDE_PLUGIN_ROOT` - Plugin root directory
- `$REPO_ROOT` - Repository root
- `$HOME` - User home directory

## Validation Errors

| Error | Fix |
|-------|-----|
| No YAML frontmatter | Add `---` delimiters and required fields |
| Name mismatch | Match `name:` field to directory name |
| Backslashes found | Replace `\` with `/` in all paths |
| Script not executable | Run `chmod +x scripts/*.sh` |

## Creating a New Skill

```bash
# 1. Create directory
mkdir -p skills/category/skill-name
cd skills/category/skill-name

# 2. Copy template
cp ../../../docs/SKILL_TEMPLATE.md SKILL.md

# 3. Edit SKILL.md
# - Update frontmatter
# - Write instructions
# - Add examples

# 4. Add optional directories
mkdir -p scripts references assets

# 5. Validate
npm run validate:skill skills/category/skill-name

# 6. Test
npm run install:project
# Then test in Claude Code
```

## Testing Checklist

- [ ] `npm run validate:skill` passes
- [ ] Frontmatter name matches directory
- [ ] Description clear, under 1024 chars
- [ ] All paths use forward slashes
- [ ] Scripts executable (`chmod +x`)
- [ ] Examples work as documented
- [ ] No sensitive information
- [ ] `npm run format` applied

## Git Workflow

```bash
# 1. Create branch
git checkout -b feature/new-skill

# 2. Create skill
# ... develop skill ...

# 3. Validate and test
npm run validate
npm run install:project

# 4. Commit
git add skills/category/new-skill
git commit -m "Add new-skill: description"

# 5. Push and create PR
git push origin feature/new-skill
```

## Marketplace Installation

```bash
# Users install with:
/plugin marketplace add gqadonis/prometheus-skill-pack
/plugin install prometheus-skill-pack

# Or specific domain:
/plugin install prometheus-react-skills
```

## Troubleshooting

**Skills not loading?**
- Restart Claude Code or run `/reload-plugins`
- Check skill has valid `SKILL.md` in correct location

**Validation fails?**
- Read error message carefully
- Check frontmatter syntax and required fields
- Ensure name matches directory exactly

**Scripts fail?**
- Check executable permissions: `ls -la scripts/`
- Verify script has proper shebang: `#!/bin/bash`
- Test script independently: `bash scripts/name.sh --help`

## Resources

- [Full Documentation](../README.md)
- [CLAUDE.md](../CLAUDE.md) - AI assistant guide
- [Contributing](CONTRIBUTING.md)
- [Skill Template](SKILL_TEMPLATE.md)
- [AgentSkills.io Spec](https://agentskills.io/specification)
