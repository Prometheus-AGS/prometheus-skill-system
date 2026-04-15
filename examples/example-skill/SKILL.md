---
name: example-skill
description: Example skill demonstrating best practices and agentskills.io compliance
license: MIT
metadata:
  author: gqadonis
  version: '1.0.0'
  category: documentation
  tags: [example, template, best-practices]
---

# Example Skill

This is an example skill demonstrating the structure and best practices for creating Agent Skills compatible with the agentskills.io standard and Claude Code.

## When to Use

Use this skill when:

- Creating a new skill as a reference
- Learning the skill structure and format
- Testing skill validation and installation

## Instructions

### Step 1: Examine the Structure

This skill follows the standard directory structure:

```
example-skill/
├── SKILL.md              # This file (required)
├── scripts/              # Executable scripts
│   └── example.sh
├── references/           # Detailed documentation
│   └── DETAILED_GUIDE.md
└── assets/               # Templates and resources
    └── template.json
```

### Step 2: Review the Frontmatter

The YAML frontmatter at the top of this file contains required and optional fields:

**Required**:

- `name`: Matches directory name, kebab-case, max 64 chars
- `description`: Clear description, 1-1024 chars

**Optional but recommended**:

- `license`: License identifier
- `metadata`: Additional context (author, version, category, tags)

### Step 3: Write Clear Instructions

Instructions should be:

- Actionable and specific
- Written in third person
- Under 500 lines total (use references/ for details)
- Include concrete examples

### Step 4: Add Scripts (Optional)

If your skill needs executable code:

```bash
# Make script executable
chmod +x scripts/example.sh

# Run the script
bash scripts/example.sh --input "test"
```

### Step 5: Reference Detailed Docs (Optional)

For complex topics, create separate reference files:

See [Detailed Guide](references/DETAILED_GUIDE.md) for comprehensive information.

## Examples

### Example 1: Basic Usage

```bash
# Simple example
echo "Hello from example skill"
```

**Expected Output**:

```
Hello from example skill
```

### Example 2: Using Scripts

```bash
# Run the example script
bash scripts/example.sh --name "World"
```

**Expected Output**:

```json
{
  "status": "success",
  "message": "Hello, World!"
}
```

### Example 3: Using Templates

```bash
# Copy template to project
cp assets/template.json ./my-config.json
```

## Available Scripts

- `scripts/example.sh` - Demonstrates script structure with arguments and JSON output

Usage:

```bash
bash scripts/example.sh --name <name> [--greeting <greeting>]
```

## Common Issues

### Issue: Script not executable

**Problem**: Permission denied when running scripts
**Solution**: Make scripts executable with `chmod +x scripts/*.sh`

### Issue: Windows-style paths

**Problem**: Backslashes in paths cause errors on Unix systems
**Solution**: Always use forward slashes: `scripts/file.sh` not `scripts\file.sh`

## Best Practices

- ✅ Keep SKILL.md under 500 lines
- ✅ Use forward slashes in all paths
- ✅ Include concrete, tested examples
- ✅ Write in third person ("Run the command", not "You should run")
- ✅ Make scripts self-contained with minimal dependencies
- ✅ Output structured data (JSON) from scripts
- ✅ Document all prerequisites clearly
- ✅ Use progressive disclosure (main instructions + references)

## Notes

- This skill serves as a template and reference
- All paths use forward slashes for cross-platform compatibility
- Scripts demonstrate self-contained design with structured output
- The structure follows agentskills.io specification exactly
