# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a comprehensive, enterprise-grade skills package collection for AI-assisted development. The repository manages centralized Agent Skills across multiple domains (React, Rust, UI/UX, DevOps, Testing, Documentation) with full compliance to the [agentskills.io](https://agentskills.io/specification) standard and Claude Code plugin marketplace requirements.

**Key Characteristics**:

- Multi-domain skill collection with unified management
- Dual-format support: standalone agentskills.io + Claude Code plugin
- Shared utilities, scripts, and templates across skills
- Automated validation and marketplace distribution
- Portable across AI platforms (Claude Code, GitHub Copilot, Cursor, VS Code)

## Essential Commands

### Submodule Management

```bash
# Initialize submodules (for new clones)
git submodule init
git submodule update

# Update all imported skills to latest
git submodule update --remote

# Update specific imported skill
cd skills/imported/artifact-refiner && git pull origin main && cd ../../..

# Check submodule status
git submodule status
```

### Validation

```bash
# Validate all skills (including imported) against agentskills.io specification
npm run validate

# Validate a specific skill
npm run validate:skill skills/react/skill-name

# Validate an imported skill
npm run validate:skill skills/imported/artifact-refiner

# Check code formatting
npm run check-format

# Auto-fix formatting
npm run format
```

### Build & Distribution

```bash
# Build marketplace distribution (creates symlinks in .claude-plugin/)
npm run build

# Install skills to user scope (~/.claude/skills/)
npm run install:user

# Install skills to project scope (.claude/skills/)
npm run install:project
```

### Testing

```bash
# Run skill tests
npm test

# Lint all skills
npm run lint

# Watch mode for development
npm run dev
```

## Architecture

### Directory Organization

```
prometheus-skill-pack/
├── .claude-plugin/          # Claude Code plugin format
│   ├── plugin.json         # Plugin manifest
│   ├── skills/             # Symlink -> ../skills/
│   ├── agents/             # Symlink -> ../agents/
│   └── hooks/              # Symlink -> ../hooks/
│
├── skills/                 # Main skills directory (agentskills.io)
│   ├── imported/           # Git submodule skills from external repos
│   │   ├── artifact-refiner/  # Submodule: PMPO artifact refinement
│   │   └── README.md       # Imported skills documentation
│   ├── react/              # React domain skills
│   ├── rust/               # Rust domain skills
│   ├── ui-ux/              # UI/UX domain skills
│   ├── devops/             # DevOps domain skills
│   ├── testing/            # Testing domain skills
│   └── documentation/      # Documentation domain skills
│
├── shared/                 # Shared resources across all skills
│   ├── scripts/            # Reusable scripts
│   │   ├── validators/     # Validation utilities
│   │   ├── generators/     # Code generation
│   │   ├── formatters/     # Formatting utilities
│   │   └── parsers/        # File parsing
│   ├── templates/          # Reusable file templates
│   └── utils/              # Helper functions
│
├── agents/                 # Specialized subagents
├── hooks/                  # Automation hooks
├── marketplace/            # Marketplace configuration
│   └── marketplace.json    # Distribution manifest
├── scripts/                # Build and validation tools
│   ├── validate-skills.js  # AgentSkills.io validator
│   ├── build-marketplace.js # Symlink builder
│   └── install.js          # Installation script
└── docs/                   # Documentation
    ├── SKILL_TEMPLATE.md   # Template for new skills
    └── CONTRIBUTING.md     # Contribution guidelines
```

### Dual-Format Support

This repository supports two distribution formats simultaneously:

1. **AgentSkills.io Standard** (`skills/` directory):
   - Portable across all AI platforms
   - Direct directory structure
   - Standard format: `SKILL.md`, `scripts/`, `references/`, `assets/`
   - Can be copied to any `~/.claude/skills/` or `.github/skills/` location

2. **Claude Code Plugin** (`.claude-plugin/` directory):
   - Enhanced with `plugin.json` manifest
   - Supports hooks, agents, MCP servers
   - Marketplace distribution via Git
   - Uses symlinks to maintain single source of truth

**Important**: The `skills/` directory is the source of truth. The `.claude-plugin/` directory contains symlinks created by `npm run build`.

### Imported Skills (Git Submodules)

The repository includes a third category for **imported skills** - skills maintained in external repositories:

- **Location**: `skills/imported/`
- **Management**: Git submodules
- **Purpose**: Skills with independent development lifecycles
- **Updates**: Can be updated from their source repositories
- **Versioning**: Can be pinned to specific versions or track latest

Current imported skills:

- `skills/imported/artifact-refiner/` - PMPO-driven artifact refinement engine (v1.1.0)

See `docs/SUBMODULES.md` for complete submodule management guide.

### Shared Resources Pattern

Skills can reference shared utilities via environment variables:

```markdown
## In SKILL.md

Run validation:
\`\`\`bash
bash ${CLAUDE_PLUGIN_ROOT}/shared/scripts/validators/validate-config.sh config.json
\`\`\`
```

Available variables:

- `$CLAUDE_PLUGIN_ROOT` - Root of plugin directory
- `$REPO_ROOT` - Repository root
- `$HOME` - User home directory

## Skill Development Workflow

### Creating a New Skill

1. **Choose category**: Place in appropriate `skills/{category}/` directory

2. **Create directory** with kebab-case naming:

   ```bash
   mkdir -p skills/react/react-entity-crud
   cd skills/react/react-entity-crud
   ```

3. **Create `SKILL.md`** using template:

   ```bash
   cp ../../docs/SKILL_TEMPLATE.md SKILL.md
   ```

4. **Edit frontmatter** (required fields):

   ```yaml
   ---
   name: react-entity-crud
   description: Complete CRUD operations for React entity management with hooks and TypeScript
   license: MIT
   metadata:
     author: your-name
     version: '1.0.0'
     category: react
     tags: [react, crud, entity, typescript]
   ---
   ```

5. **Write instructions** following these principles:
   - Keep main file under 500 lines
   - Use third-person voice ("Run the command", not "You should run")
   - Include concrete examples
   - Move detailed content to `references/` directory
   - Use forward slashes for all paths

6. **Add optional directories**:

   ```bash
   mkdir -p scripts references assets
   # scripts/    - Executable code
   # references/ - Detailed documentation
   # assets/     - Templates, schemas, examples
   ```

7. **Validate**:

   ```bash
   npm run validate:skill skills/react/react-entity-crud
   ```

8. **Test locally**:
   ```bash
   npm run install:project
   # Then test in Claude Code with /skill-name
   ```

### Modifying Existing Skills

When updating skills:

1. **Read current state**: Always read `SKILL.md` before modifying
2. **Preserve structure**: Maintain existing section organization
3. **Validate changes**: Run `npm run validate:skill` after edits
4. **Check references**: Update `references/` files if structure changes
5. **Version bump**: Update `metadata.version` for significant changes

## AgentSkills.io Compliance

This repository strictly adheres to the [agentskills.io specification](https://agentskills.io/specification):

### Required Elements

- ✅ `SKILL.md` with YAML frontmatter
- ✅ `name` field: lowercase, hyphens, max 64 chars, pattern `^[a-z0-9]+(-[a-z0-9]+)*$`
- ✅ `description` field: 1-1024 characters, searchable

### Standard Directories

- ✅ `scripts/` - Executable code (optional)
- ✅ `references/` - Documentation loaded on demand (optional)
- ✅ `assets/` - Templates, resources (optional)

### Best Practices Enforced

- ✅ Forward slashes only (never backslashes)
- ✅ Self-contained scripts with package runners (`npx`, `uvx`, `bunx`)
- ✅ Progressive disclosure (main file + references)
- ✅ Structured output from scripts (JSON preferred)
- ✅ Executable permissions on scripts (`chmod +x`)

### Validation

The validator (`scripts/validate-skills.js`) checks:

- YAML frontmatter syntax and schema
- Required fields presence and format
- Name/directory consistency
- Path separator style (forward slashes)
- Script executability
- File structure compliance

## Important Conventions

### Naming

- **Skills**: `kebab-case` only, e.g., `react-entity-crud`
- **Files**: Forward slashes in all paths
- **Scripts**: Executable with `.sh`, `.py`, `.js` extensions

### Skill Size

- **Main SKILL.md**: Keep under 500 lines
- **Progressive disclosure**: Split large content to `references/`
- **Context efficiency**: Skills use lazy-loading architecture

### Script Requirements

- **Self-contained**: Use inline dependency declarations or package runners
- **Cross-platform**: Avoid platform-specific commands
- **Structured output**: JSON when possible for programmatic parsing
- **Error handling**: Non-zero exit codes on failure

### Documentation

- **Third person**: "Run the command" not "You should run"
- **Concrete examples**: Always include working examples
- **When to use**: Describe triggering scenarios clearly
- **No assumptions**: Document all prerequisites

### Shared Resources

- **Location**: `shared/{scripts,templates,utils}/`
- **Reference**: Use `${CLAUDE_PLUGIN_ROOT}/shared/...`
- **Documentation**: Maintain README in each shared directory
- **Reusability**: Prefer shared utilities over duplication

## Marketplace Distribution

The marketplace is configured for Git-based distribution:

1. **Source**: `marketplace/marketplace.json` with frontmatter
2. **Plugins**: Defined as Git repository references
3. **Granularity**: Full package or individual domain packages
4. **Installation**: Users run `/plugin marketplace add gqadonis/prometheus-skill-pack`

### Publishing Checklist

Before releasing:

- [ ] All skills validate: `npm run validate`
- [ ] Marketplace builds: `npm run build`
- [ ] Version bumped in `package.json` and `plugin.json`
- [ ] CHANGELOG updated
- [ ] README reflects new skills
- [ ] Git tag created: `git tag v1.x.x`

## Testing Strategy

### Validation Testing

```bash
# Schema validation
npm run validate

# Specific skill
npm run validate:skill skills/category/name
```

### Integration Testing

```bash
# Install to test environment
npm run install:project

# Test in Claude Code
# 1. Launch Claude Code
# 2. Run /reload-plugins
# 3. Try /skill-name or let AI auto-trigger
```

### Manual Testing Checklist

- [ ] Skill triggers on appropriate prompts
- [ ] Instructions are clear and actionable
- [ ] Examples work as documented
- [ ] Scripts execute successfully
- [ ] References load correctly
- [ ] No Windows-style paths present

## Common Patterns

### Skill with Scripts

```markdown
---
name: my-skill
description: Does something useful
---

# My Skill

## Instructions

1. Validate input:
   \`\`\`bash
   bash scripts/validate.sh input.json
   \`\`\`

2. Process data:
   \`\`\`bash
   python3 scripts/process.py --input input.json --output output.json
   \`\`\`
```

### Skill with References

```markdown
---
name: complex-skill
description: Complex workflow with detailed docs
---

# Complex Skill

## Quick Start

[Basic instructions here - keep under 500 lines]

## Detailed Documentation

For in-depth information:

- [Conceptual Guide](references/CONCEPTS.md)
- [API Reference](references/API.md)
- [Extended Examples](references/EXAMPLES.md)
```

### Skill Using Shared Scripts

```markdown
---
name: validated-skill
description: Skill with validation
---

# Validated Skill

## Instructions

1. Validate configuration:
   \`\`\`bash
   bash ${CLAUDE_PLUGIN_ROOT}/shared/scripts/validators/validate-json.sh config.json
   \`\`\`
```

## Troubleshooting

### Validation Errors

**Error**: "SKILL.md must have YAML frontmatter"

- **Fix**: Add frontmatter with `---` delimiters and required fields

**Error**: "Frontmatter name doesn't match directory"

- **Fix**: Ensure `name:` field matches directory name exactly

**Error**: "Found backslashes in SKILL.md"

- **Fix**: Replace all `\` with `/` in paths

### Build Issues

**Symlinks not created**:

- **Check**: Permissions on `.claude-plugin/` directory
- **Fix**: Run `npm run build` to recreate symlinks

**Skills not loading**:

- **Check**: Restart Claude Code or run `/reload-plugins`
- **Verify**: Skill is in correct location with valid `SKILL.md`

### Installation Issues

**Permission denied on scripts**:

- **Fix**: Run `chmod +x scripts/*.sh` in skill directory

**Module not found in script**:

- **Fix**: Use package runners (`npx`, `uvx`) or inline dependencies

## References

- [AgentSkills.io Specification](https://agentskills.io/specification)
- [Claude Code Plugin Documentation](https://code.claude.com/docs/en/plugins)
- [Contributing Guidelines](docs/CONTRIBUTING.md)
- [Skill Template](docs/SKILL_TEMPLATE.md)
