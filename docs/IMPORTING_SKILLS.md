# Importing Skills as Submodules

This guide explains how to import external skills into the Prometheus Skill Pack using git submodules.

## When to Import as Submodule

Import a skill as a submodule when:
- ✅ The skill is maintained in a separate repository
- ✅ The skill has its own development lifecycle and versioning
- ✅ You want to track updates from the original source
- ✅ Multiple projects might use the same skill
- ✅ The skill is complex enough to warrant independent maintenance

**Don't** use submodules for:
- ❌ Skills you're developing specifically for this pack
- ❌ Simple, single-file skills
- ❌ Skills that will never be updated independently
- ❌ Temporary or experimental skills

## Quick Start

```bash
# Add a new skill as submodule
git submodule add git@github.com:USER/REPO.git skills/imported/skill-name

# Commit the addition
git add .gitmodules skills/imported/skill-name
git commit -m "Add skill-name as imported skill"

# Document in README
# Add entry to skills/imported/README.md
```

## Step-by-Step Import Process

### 1. Verify the Skill Structure

Before importing, check that the external skill follows agentskills.io standards:

```bash
# Clone temporarily to inspect
git clone git@github.com:USER/REPO.git /tmp/skill-check
cd /tmp/skill-check

# Check for required files
ls -la SKILL.md  # Must exist
head -20 SKILL.md  # Should have YAML frontmatter

# Check directory structure
ls -la scripts/ references/ assets/  # Optional directories
```

### 2. Add as Submodule

```bash
# Navigate to repository root
cd /path/to/prometheus-skill-pack

# Add submodule to skills/imported/
git submodule add git@github.com:USER/REPO.git skills/imported/skill-name
```

**Naming conventions**:
- Use the skill's canonical name (usually matches `name` in SKILL.md frontmatter)
- Use kebab-case: `artifact-refiner`, not `ArtifactRefiner`
- Be descriptive but concise

### 3. Validate the Imported Skill

```bash
# Validate against agentskills.io spec
npm run validate:skill skills/imported/skill-name

# Check for common issues
bash scripts/check-imported-skill.sh skills/imported/skill-name
```

### 4. Document the Import

Add entry to `skills/imported/README.md`:

```markdown
### skill-name
- **Repository**: git@github.com:USER/REPO.git
- **Description**: Brief description of what the skill does
- **Version**: v1.0.0
- **Type**: Complete plugin | Simple skill | With agents
```

Update main `README.md` in the "Imported Skills" section.

### 5. Pin to Specific Version (Recommended)

```bash
# Navigate to submodule
cd skills/imported/skill-name

# List available versions
git tag -l

# Checkout specific version
git checkout v1.0.0

# Return to root
cd ../../..

# Commit the pinned version
git add skills/imported/skill-name
git commit -m "Pin skill-name to v1.0.0"
```

### 6. Test the Import

```bash
# Install locally
npm run install:project

# Test in Claude Code
# Launch Claude Code
# Verify skill appears: /skills
# Test skill functionality
```

### 7. Commit and Push

```bash
# Commit the submodule addition
git add .gitmodules skills/imported/skill-name
git commit -m "Import skill-name as submodule from USER/REPO"

# Push to remote
git push origin main
```

## Importing Skills from Different Sources

### From GitHub (SSH)

```bash
git submodule add git@github.com:USER/REPO.git skills/imported/skill-name
```

### From GitHub (HTTPS)

```bash
git submodule add https://github.com/USER/REPO.git skills/imported/skill-name
```

### From GitLab

```bash
git submodule add git@gitlab.com:USER/REPO.git skills/imported/skill-name
```

### From Private Repository

```bash
# Ensure SSH keys are configured
git submodule add git@github.com:PRIVATE-ORG/private-repo.git skills/imported/skill-name

# Document access requirements in skills/imported/README.md
```

## Managing Imported Skills

### Updating to Latest Version

```bash
cd skills/imported/skill-name
git pull origin main
cd ../../..
git add skills/imported/skill-name
git commit -m "Update skill-name to latest"
```

### Updating All Imported Skills

```bash
git submodule update --remote
git add skills/imported/
git commit -m "Update all imported skills to latest versions"
```

### Switching Versions

```bash
cd skills/imported/skill-name
git fetch --tags
git checkout v1.2.0
cd ../../..
git add skills/imported/skill-name
git commit -m "Update skill-name to v1.2.0"
```

### Checking for Updates

```bash
# Check if updates are available
git submodule update --remote --dry-run

# Show current versions
git submodule foreach 'echo $name: $(git describe --tags --always)'
```

## Special Cases

### Importing a Skill with Its Own Submodules

If the skill itself has submodules:

```bash
# Add with recursive flag
git submodule add --recursive git@github.com:USER/REPO.git skills/imported/skill-name

# Or initialize after adding
git submodule update --init --recursive
```

### Importing from a Monorepo

If the skill is in a subdirectory of a larger repository, you'll need to:

1. Ask the maintainer to create a separate repository, or
2. Use `git subtree` instead (more complex), or
3. Copy the skill manually (loses update tracking)

### Importing a Skill That Requires Build Steps

If the imported skill has build requirements:

```bash
# After importing
cd skills/imported/skill-name
npm install  # or other setup
npm run build
cd ../../..

# Document in skills/imported/README.md
```

## Troubleshooting Imports

### Submodule URL Changed

```bash
# Update .gitmodules manually
vim .gitmodules

# Sync the change
git submodule sync

# Update submodule
cd skills/imported/skill-name
git fetch
cd ../../..
```

### Imported Skill Doesn't Validate

If validation fails:

```bash
# Check what's wrong
npm run validate:skill skills/imported/skill-name

# Common issues:
# - Missing or invalid frontmatter → Contact skill maintainer
# - Windows-style paths → Contact skill maintainer
# - Non-executable scripts → chmod +x scripts/*.sh
```

### Submodule Import Failed

```bash
# Remove failed submodule
git submodule deinit -f skills/imported/skill-name
git rm -f skills/imported/skill-name
rm -rf .git/modules/skills/imported/skill-name

# Try again with correct URL
git submodule add <correct-url> skills/imported/skill-name
```

## Best Practices

### ✅ DO

- **Pin to versions** for production use
- **Test after import** before committing
- **Document the source** and version clearly
- **Check compatibility** with agentskills.io standard
- **Validate regularly** after updates
- **Track upstream changes** in your CHANGELOG
- **Respect licenses** of imported skills

### ❌ DON'T

- **Don't modify** imported skill files directly
- **Don't import** without testing first
- **Don't track unstable** branches in production
- **Don't forget** to document access requirements for private repos
- **Don't blindly update** without testing compatibility

## Contributing Improvements Back

If you improve an imported skill:

```bash
# Fork the original repository
# Create a feature branch
cd skills/imported/skill-name
git checkout -b feature/improvement

# Make changes and commit
git add .
git commit -m "Improve feature X"

# Push to your fork
git remote add myfork git@github.com:YOURUSERNAME/REPO.git
git push myfork feature/improvement

# Create pull request on GitHub
# Once merged upstream, update submodule pointer
git checkout main
git pull origin main
cd ../../..
git add skills/imported/skill-name
git commit -m "Update skill-name with merged improvements"
```

## References

- [Git Submodules Documentation](https://git-scm.com/book/en/v2/Git-Tools-Submodules)
- [AgentSkills.io Specification](https://agentskills.io/specification)
- [Prometheus Skill Pack Submodule Guide](SUBMODULES.md)
- [Imported Skills README](../skills/imported/README.md)
