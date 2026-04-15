# Imported Skills

This directory contains skills imported as git submodules from external repositories. These skills are maintained in separate repositories and can be updated independently.

## Why Import as Submodules?

Imported skills via git submodules allow:
- **Independent maintenance** - Skills can be updated in their own repositories
- **Version control** - Pin specific versions or track latest changes
- **Shared development** - Multiple skill packs can reference the same skill
- **Clean separation** - Keep locally-developed vs externally-maintained skills separate

## Current Imported Skills

### artifact-refiner
- **Repository**: git@github.com:GQAdonis/artifact-refiner-skill.git
- **Description**: PMPO-driven artifact refinement engine for logos, UI components, A2UI specs, images, content, and code
- **Version**: 1.1.0
- **Type**: Complete plugin package (includes agents, hooks, scripts)

## Managing Imported Skills

### Initial Clone (for new contributors)

When cloning this repository, initialize submodules:

```bash
git clone git@github.com:GQAdonis/prometheus-skill-pack.git
cd prometheus-skill-pack
git submodule init
git submodule update
```

Or clone with submodules in one step:

```bash
git clone --recurse-submodules git@github.com:GQAdonis/prometheus-skill-pack.git
```

### Updating an Imported Skill

To pull the latest changes from an imported skill:

```bash
# Update specific skill to latest
cd skills/imported/artifact-refiner
git pull origin main
cd ../../..

# Commit the submodule pointer update
git add skills/imported/artifact-refiner
git commit -m "Update artifact-refiner to latest version"
```

### Updating All Imported Skills

```bash
# Update all submodules to latest
git submodule update --remote

# Commit all updates
git add .
git commit -m "Update all imported skills to latest versions"
```

### Pinning to Specific Version

```bash
# Navigate to the submodule
cd skills/imported/artifact-refiner

# Checkout specific version/tag
git checkout v1.1.0

# Return to main repo
cd ../../..

# Commit the pinned version
git add skills/imported/artifact-refiner
git commit -m "Pin artifact-refiner to v1.1.0"
```

### Checking Submodule Status

```bash
# Show current commit of each submodule
git submodule status

# Show if submodules have upstream changes
git submodule update --remote --dry-run
```

### Contributing Changes to Imported Skills

If you need to modify an imported skill:

```bash
# Navigate to the submodule
cd skills/imported/artifact-refiner

# Create a branch
git checkout -b feature/my-improvement

# Make changes and commit
git add .
git commit -m "Improve feature X"

# Push to the skill's repository (requires permissions)
git push origin feature/my-improvement

# Return to main repo
cd ../../..

# Update the submodule pointer (optional - if you want to track your branch)
git add skills/imported/artifact-refiner
git commit -m "Update artifact-refiner to feature branch"
```

## Adding New Imported Skills

To import another skill as a submodule:

```bash
# Add the submodule
git submodule add git@github.com:USER/skill-repo.git skills/imported/skill-name

# Commit the addition
git add .gitmodules skills/imported/skill-name
git commit -m "Add skill-name as imported skill"

# Update this README
# Document the new skill in the "Current Imported Skills" section
```

## Removing an Imported Skill

To remove a submodule:

```bash
# Remove from git
git submodule deinit skills/imported/skill-name
git rm skills/imported/skill-name
rm -rf .git/modules/skills/imported/skill-name

# Commit the removal
git commit -m "Remove skill-name imported skill"
```

## Integration with Prometheus Skill Pack

### Validation

Imported skills are validated along with native skills:

```bash
# Validate all skills including imported
npm run validate

# Validate specific imported skill
npm run validate:skill skills/imported/artifact-refiner
```

### Installation

Imported skills are included when installing the skill pack:

```bash
# Install entire pack (includes imported skills)
npm run install:user
# or
npm run install:project
```

### Plugin Format

Some imported skills (like artifact-refiner) are complete plugin packages with:
- Their own `.claude-plugin/` directory
- Agents, hooks, and MCP servers
- Independent versioning

These are treated as nested plugins and their components are discovered automatically by Claude Code.

## Best Practices

### ✅ DO
- Keep imported skills up to date regularly
- Pin to specific versions for production use
- Document the purpose and version of each imported skill
- Test after updating imported skills
- Respect the imported skill's license and contribution guidelines

### ❌ DON'T
- Make changes directly in imported skill directories without proper git workflow
- Commit unstaged submodule changes
- Remove `.git` directory from submodules
- Modify imported skill licenses or attribution

## Troubleshooting

### Submodule directory is empty

```bash
git submodule init
git submodule update
```

### Submodule shows modified but no changes

```bash
cd skills/imported/skill-name
git status
# If clean, the submodule pointer is out of sync
git checkout <expected-commit>
cd ../../..
git add skills/imported/skill-name
git commit -m "Sync submodule pointer"
```

### Pull fails in submodule

```bash
cd skills/imported/skill-name
git fetch
git reset --hard origin/main  # or appropriate branch
cd ../../..
```

### Submodule points to wrong commit

```bash
# Check current commit
git submodule status

# Update to track remote branch
cd skills/imported/skill-name
git checkout main
git pull
cd ../../..
git add skills/imported/skill-name
git commit -m "Update submodule to track main"
```

## References

- [Git Submodules Documentation](https://git-scm.com/book/en/v2/Git-Tools-Submodules)
- [AgentSkills.io Specification](https://agentskills.io/specification)
- [Claude Code Plugin Documentation](https://code.claude.com/docs/en/plugins)
