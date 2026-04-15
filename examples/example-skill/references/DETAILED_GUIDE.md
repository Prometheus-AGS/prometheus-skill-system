# Example Skill - Detailed Guide

This guide provides comprehensive information about the example skill structure and design patterns.

## Table of Contents

- [Architecture](#architecture)
- [Design Decisions](#design-decisions)
- [Advanced Usage](#advanced-usage)
- [Extension Points](#extension-points)

## Architecture

### Progressive Disclosure

The skill uses a three-tier progressive disclosure model:

1. **Tier 1: Metadata** (~30-50 tokens)
   - Name and description in frontmatter
   - Loaded at startup for all skills
   - Used for skill discovery and matching

2. **Tier 2: Main Instructions** (SKILL.md)
   - Core workflow and common use cases
   - Loaded when skill is triggered
   - Kept under 500 lines for efficiency

3. **Tier 3: Detailed References** (this file)
   - Comprehensive documentation
   - Loaded only when explicitly referenced
   - No token limit, can be extensive

### File Organization

```
example-skill/
├── SKILL.md              # Entry point with core instructions
├── scripts/              # Executable automation
│   └── example.sh       # Self-contained script
├── references/          # Deep documentation (this file)
│   └── DETAILED_GUIDE.md
└── assets/              # Static resources
    └── template.json    # Configuration template
```

## Design Decisions

### Why Bash for Scripts?

The example uses bash because:
- Available on all Unix-like systems (macOS, Linux)
- No external dependencies required
- Fast execution for simple operations
- Easy to understand and modify

For more complex operations, Python with inline dependencies is preferred:

```python
# /// script
# dependencies = [
#   "package-name",
# ]
# ///

import package_name
# Script logic...
```

### Why JSON Output?

Structured JSON output enables:
- Programmatic parsing by AI agents
- Easy error detection and handling
- Consistent interface across scripts
- Machine-readable status reporting

### Why Forward Slashes?

Forward slashes work on:
- ✅ macOS
- ✅ Linux
- ✅ Windows (modern versions)

Backslashes only work on:
- ✅ Windows (legacy)
- ❌ macOS (interpreted as escape character)
- ❌ Linux (interpreted as escape character)

## Advanced Usage

### Chaining Scripts

Scripts can be chained together:

```bash
# Validate, process, and format
bash scripts/validate.sh input.json | \
bash scripts/process.sh | \
bash scripts/format.sh > output.json
```

### Error Handling

Scripts use standard error codes:
- `0` - Success
- `1` - General error
- `2` - Usage error (missing/invalid arguments)

Example error handling:

```bash
if ! bash scripts/validate.sh config.json; then
    echo "Validation failed" >&2
    exit 1
fi
```

### Environment Variables

Skills can access:
- `$CLAUDE_PLUGIN_ROOT` - Plugin root directory
- `$REPO_ROOT` - Repository root
- `$HOME` - User home directory

Example:

```bash
# Reference shared script
bash ${CLAUDE_PLUGIN_ROOT}/shared/scripts/common/util.sh
```

## Extension Points

### Adding New Scripts

1. Create script in `scripts/` directory
2. Make it executable: `chmod +x scripts/new-script.sh`
3. Follow the template structure:
   - Help message with `--help`
   - Argument validation
   - Structured JSON output
   - Error handling with exit codes

4. Document in main SKILL.md

### Adding References

For new conceptual documentation:

1. Create markdown file in `references/`
2. Link from main SKILL.md
3. Keep focused on a single topic
4. Use clear section headers

### Adding Assets

For templates or example files:

1. Place in `assets/` directory
2. Document usage in SKILL.md
3. Include comments explaining placeholders
4. Keep files small and focused

## Best Practices Summary

### Script Best Practices

✅ **DO**:
- Use `set -euo pipefail` in bash
- Validate all input arguments
- Output structured JSON
- Include `--help` flag
- Handle errors gracefully
- Make scripts executable

❌ **DON'T**:
- Assume global dependencies
- Use platform-specific commands
- Output unstructured text
- Ignore error cases
- Hardcode file paths

### Documentation Best Practices

✅ **DO**:
- Write in third person
- Include concrete examples
- Keep SKILL.md under 500 lines
- Use progressive disclosure
- Link to detailed references

❌ **DON'T**:
- Write overly generic instructions
- Duplicate information
- Include obvious statements
- Use first/second person
- Make assumptions about environment

### Path Best Practices

✅ **DO**:
- Use forward slashes: `scripts/file.sh`
- Use relative paths: `./scripts/file.sh`
- Use environment variables: `${REPO_ROOT}/file.sh`

❌ **DON'T**:
- Use backslashes: `scripts\file.sh`
- Use absolute paths: `/Users/name/project/file.sh`
- Assume working directory

## Testing Checklist

Before releasing a skill:

- [ ] SKILL.md validates: `npm run validate:skill`
- [ ] Frontmatter name matches directory
- [ ] Description is clear and under 1024 chars
- [ ] All paths use forward slashes
- [ ] Scripts are executable
- [ ] Scripts output valid JSON
- [ ] Examples are accurate and tested
- [ ] References are valid
- [ ] No sensitive information included
- [ ] Formatting is consistent

## Further Reading

- [AgentSkills.io Specification](https://agentskills.io/specification)
- [Claude Code Skills Documentation](https://code.claude.com/docs/en/skills)
- [Skill Authoring Best Practices](https://platform.claude.com/docs/en/agents-and-tools/agent-skills/best-practices)
