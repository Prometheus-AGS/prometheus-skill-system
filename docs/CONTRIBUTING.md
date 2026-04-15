# Contributing to Prometheus Skill Pack

Thank you for your interest in contributing! This document provides guidelines for adding new skills and improving existing ones.

## Table of Contents

- [Getting Started](#getting-started)
- [Skill Creation Guidelines](#skill-creation-guidelines)
- [Directory Structure](#directory-structure)
- [Validation](#validation)
- [Pull Request Process](#pull-request-process)
- [Best Practices](#best-practices)

## Getting Started

1. Fork the repository
2. Clone your fork:
   ```bash
   git clone https://github.com/YOUR_USERNAME/prometheus-skill-pack.git
   cd prometheus-skill-pack
   ```
3. Install dependencies:
   ```bash
   npm install
   ```
4. Create a feature branch:
   ```bash
   git checkout -b feature/my-new-skill
   ```

## Skill Creation Guidelines

### 1. Choose the Right Category

Place your skill in the appropriate category directory:
- `skills/react/` - React and frontend framework skills
- `skills/rust/` - Rust programming skills
- `skills/ui-ux/` - Design and user experience skills
- `skills/devops/` - Infrastructure and deployment skills
- `skills/testing/` - Testing methodology skills
- `skills/documentation/` - Documentation and technical writing skills

If none fit, propose a new category in your PR.

### 2. Naming Conventions

- Use kebab-case for skill names: `react-entity-crud`, not `ReactEntityCRUD`
- Names should be descriptive but concise (max 64 characters)
- Only use lowercase letters, numbers, and hyphens
- No consecutive hyphens: `react--entity` ❌ `react-entity` ✅

### 3. Required Files

Every skill must have a `SKILL.md` file with:

```markdown
---
name: skill-name
description: What this skill does and when to use it
license: MIT
---

# Skill Name

[Content here]
```

### 4. Optional Directories

- `scripts/` - Executable scripts (must be self-contained)
- `references/` - Detailed documentation (loaded on demand)
- `assets/` - Templates, schemas, example files

## Directory Structure

```
skills/category/skill-name/
├── SKILL.md              # Required: Main skill file
├── scripts/              # Optional: Executable scripts
│   ├── script1.sh
│   └── script2.py
├── references/           # Optional: Detailed docs
│   ├── GUIDE.md
│   └── API.md
└── assets/               # Optional: Resources
    ├── template.json
    └── example.yaml
```

## Validation

Before submitting, run validation:

```bash
# Validate your specific skill
npm run validate:skill skills/category/skill-name

# Validate all skills
npm run validate

# Check formatting
npm run check-format

# Auto-fix formatting
npm run format
```

## Pull Request Process

1. **Create your skill** following the guidelines above

2. **Test locally**:
   ```bash
   npm run install:project
   # Then test with Claude Code
   ```

3. **Validate**:
   ```bash
   npm run validate:skill skills/category/your-skill
   ```

4. **Commit** with a clear message:
   ```bash
   git add skills/category/your-skill
   git commit -m "Add [skill-name]: brief description"
   ```

5. **Push** to your fork:
   ```bash
   git push origin feature/my-new-skill
   ```

6. **Create Pull Request** with:
   - Clear title: "Add skill: skill-name"
   - Description of what the skill does
   - Testing you've performed
   - Any special requirements or dependencies

## Best Practices

### SKILL.md Content

✅ **DO**:
- Write clear, actionable instructions
- Include concrete examples
- Use third-person voice ("Run the command", not "You should run")
- Keep the main file under 500 lines
- Move detailed content to `references/`
- Use forward slashes for paths: `scripts/run.sh`
- Include "when to use" guidance

❌ **DON'T**:
- Include Windows-style paths: `scripts\run.sh`
- Make assumptions about available tools
- Write overly generic instructions
- Duplicate content from other skills
- Include sensitive information

### Scripts

✅ **DO**:
- Make scripts executable: `chmod +x scripts/script.sh`
- Use structured output (JSON when possible)
- Include error handling
- Document script usage in SKILL.md
- Use package runners for dependencies (`npx`, `uvx`, `bunx`)

❌ **DON'T**:
- Assume specific versions are installed globally
- Use global dependencies
- Write platform-specific scripts without alternatives
- Create scripts that modify system-level configuration

### Progressive Disclosure

For complex skills:

1. **SKILL.md** - Core instructions and common use cases (≤500 lines)
2. **references/GUIDE.md** - Detailed conceptual explanation
3. **references/API.md** - Complete API reference
4. **references/EXAMPLES.md** - Extended examples

### Frontmatter Fields

Required:
- `name` - Matches directory name
- `description` - Clear, searchable description

Recommended:
- `license` - Usually MIT
- `metadata.author` - Your name/username
- `metadata.version` - Semantic version
- `metadata.category` - Skill category
- `metadata.tags` - Searchable keywords

Optional:
- `compatibility` - System requirements
- `allowed-tools` - Tool restrictions

### Testing Checklist

Before submitting, verify:

- [ ] Skill validates without errors: `npm run validate:skill`
- [ ] Frontmatter name matches directory name
- [ ] Description is clear and under 1024 characters
- [ ] All paths use forward slashes
- [ ] Scripts are executable (if present)
- [ ] No sensitive information included
- [ ] Examples are accurate and tested
- [ ] References are valid (if present)
- [ ] Formatting is consistent: `npm run format`

## Code of Conduct

- Be respectful and inclusive
- Focus on constructive feedback
- Welcome newcomers
- Give credit where due
- Keep discussions on-topic

## Questions?

- Open an issue for clarification
- Check existing skills for examples
- Review [agentskills.io specification](https://agentskills.io/specification)
- Ask in pull request comments

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
