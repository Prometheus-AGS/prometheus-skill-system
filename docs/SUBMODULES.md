# Managing Submodules in Prometheus Skill Pack

This guide explains how to work with git submodules in the Prometheus Skill Pack, specifically for managing imported skills.

## What are Git Submodules?

Git submodules allow you to keep a Git repository as a subdirectory of another Git repository. This lets you:

- Track external dependencies at specific commits
- Maintain separate development histories
- Share code across multiple projects
- Update dependencies independently

## Why Use Submodules for Skills?

In the Prometheus Skill Pack, submodules are used for **imported skills** - skills that:

- Are maintained in separate repositories
- Have their own development lifecycle
- May be used across multiple skill packs
- Require independent versioning

## Directory Structure

```
skills/
├── imported/              # Git submodule skills
│   ├── artifact-refiner/ # Submodule: external skill
│   └── README.md         # Imported skills documentation
├── react/                # Native skills (not submodules)
├── rust/                 # Native skills
└── ui-ux/                # Native skills
```

## Common Operations

### Initial Setup (for contributors)

When you first clone the repository:

```bash
# Option 1: Clone with submodules
git clone --recurse-submodules git@github.com:GQAdonis/prometheus-skill-pack.git

# Option 2: Clone then initialize submodules
git clone git@github.com:GQAdonis/prometheus-skill-pack.git
cd prometheus-skill-pack
git submodule init
git submodule update
```

### Checking Submodule Status

```bash
# Show all submodules and their current commits
git submodule status

# Show detailed info
git submodule foreach 'echo $name @ $(git rev-parse HEAD)'

# Check for upstream updates
git submodule update --remote --dry-run
```

### Updating Submodules

#### Update to Latest

```bash
# Update all submodules to latest remote commits
git submodule update --remote

# Commit the updates
git add skills/imported/
git commit -m "Update imported skills to latest versions"
```

#### Update Specific Submodule

```bash
# Navigate to submodule
cd skills/imported/artifact-refiner

# Pull latest changes
git pull origin main

# Return to root
cd ../../..

# Commit the pointer update
git add skills/imported/artifact-refiner
git commit -m "Update artifact-refiner to latest"
```

#### Pin to Specific Version

```bash
cd skills/imported/artifact-refiner

# Checkout specific tag or commit
git checkout v1.1.0
# or
git checkout abc123def

cd ../../..

git add skills/imported/artifact-refiner
git commit -m "Pin artifact-refiner to v1.1.0"
```

### Adding New Submodules

```bash
# Add submodule to skills/imported/
git submodule add git@github.com:USER/REPO.git skills/imported/skill-name

# Commit the addition
git add .gitmodules skills/imported/skill-name
git commit -m "Add skill-name as imported skill"

# Update documentation
# Edit skills/imported/README.md to document the new skill
```

### Removing Submodules

```bash
# Deinitialize the submodule
git submodule deinit -f skills/imported/skill-name

# Remove from git tracking
git rm -f skills/imported/skill-name

# Remove the module from .git/modules/
rm -rf .git/modules/skills/imported/skill-name

# Commit the removal
git commit -m "Remove skill-name submodule"
```

### Making Changes to Submodules

If you have write access to the submodule repository:

```bash
cd skills/imported/artifact-refiner

# Create a branch
git checkout -b feature/improvement

# Make changes
# ... edit files ...

# Commit in submodule
git add .
git commit -m "Improve feature"

# Push to submodule's remote
git push origin feature/improvement

# Return to main repo
cd ../../..

# Optionally update the pointer to track your branch
git add skills/imported/artifact-refiner
git commit -m "Track feature/improvement branch in artifact-refiner"
```

## Submodule Workflows

### Development Workflow

```bash
# 1. Always start with up-to-date submodules
git pull
git submodule update --init --recursive

# 2. Make changes to native skills (skills/react, skills/rust, etc.)
# ... your development work ...

# 3. Before committing, check submodule status
git submodule status

# 4. Commit your changes
git add .
git commit -m "Your changes"

# 5. Push to remote
git push
```

### Update and Test Workflow

```bash
# 1. Update imported skills
git submodule update --remote

# 2. Test the updates
npm run validate
npm run install:project
# Test in Claude Code

# 3. If tests pass, commit the updates
git add skills/imported/
git commit -m "Update imported skills and verify compatibility"
git push

# 4. If tests fail, pin to known-good versions
cd skills/imported/artifact-refiner
git checkout v1.0.0  # or last known-good commit
cd ../../..
git add skills/imported/artifact-refiner
git commit -m "Revert artifact-refiner to v1.0.0 due to compatibility issues"
```

### Release Workflow

```bash
# Before releasing, pin all submodules to specific versions

# 1. Check current versions
git submodule foreach 'git describe --tags --always'

# 2. Pin each submodule to a stable tag
cd skills/imported/artifact-refiner
git checkout v1.1.0  # Use latest stable version
cd ../../..

# 3. Commit pinned versions
git add skills/imported/
git commit -m "Pin imported skills for v1.0.0 release"

# 4. Create release tag
git tag -a v1.0.0 -m "Release v1.0.0"
git push origin v1.0.0
```

## Configuration Files

### .gitmodules

This file tracks all submodules in the repository:

```ini
[submodule "skills/imported/artifact-refiner"]
    path = skills/imported/artifact-refiner
    url = git@github.com:GQAdonis/artifact-refiner-skill.git
```

**Don't edit this manually** - use git commands instead.

### Submodule Tracking Branches

You can configure submodules to track specific branches:

```bash
# Configure submodule to track 'develop' branch
git config -f .gitmodules submodule.skills/imported/artifact-refiner.branch develop

# Update to track the configured branch
git submodule update --remote

# Commit the configuration
git add .gitmodules
git commit -m "Configure artifact-refiner to track develop branch"
```

## Troubleshooting

### Submodule directory is empty after clone

**Problem**: Cloned repo but `skills/imported/artifact-refiner/` is empty

**Solution**:

```bash
git submodule init
git submodule update
```

### Submodule shows as modified but `git diff` shows nothing

**Problem**: `git status` shows submodule as modified, but no actual changes

**Solution**: The submodule pointer is out of sync

```bash
cd skills/imported/artifact-refiner
git checkout <expected-commit>
cd ../../..
git add skills/imported/artifact-refiner
```

### Can't update submodule - detached HEAD

**Problem**: Submodule is in detached HEAD state

**Solution**:

```bash
cd skills/imported/artifact-refiner
git checkout main  # or appropriate branch
git pull
cd ../../..
git add skills/imported/artifact-refiner
git commit -m "Update submodule to track main branch"
```

### Merge conflicts in .gitmodules

**Problem**: Conflicts in `.gitmodules` after merge

**Solution**:

```bash
# Resolve conflicts in .gitmodules manually
vim .gitmodules

# Then sync submodule configuration
git submodule sync
git submodule update --init

# Commit the resolution
git add .gitmodules
git commit -m "Resolve submodule conflicts"
```

### Accidentally committed changes inside submodule

**Problem**: Made changes in submodule and committed to main repo

**Solution**:

```bash
# Uncommit from main repo
git reset HEAD^ skills/imported/artifact-refiner

# Go to submodule
cd skills/imported/artifact-refiner

# Commit properly in submodule
git add .
git commit -m "Your changes"
git push origin main  # or appropriate branch

cd ../../..

# Update pointer in main repo
git add skills/imported/artifact-refiner
git commit -m "Update artifact-refiner with recent changes"
```

### Submodule remote URL changed

**Problem**: The upstream URL for a submodule changed

**Solution**:

```bash
# Update URL in .gitmodules
vim .gitmodules  # Change the url

# Sync the change
git submodule sync

# Update submodules
git submodule update --remote

# Commit the change
git add .gitmodules
git commit -m "Update artifact-refiner remote URL"
```

## Best Practices

### ✅ DO

- **Initialize submodules** after cloning: `git submodule update --init --recursive`
- **Pin to versions** for production releases
- **Test after updates** before committing submodule changes
- **Document versions** in CHANGELOG when updating submodules
- **Use tags** rather than branch names for stable references
- **Update regularly** but carefully test compatibility

### ❌ DON'T

- **Don't modify** submodule content directly without proper git workflow
- **Don't commit** submodule changes without understanding the pointer update
- **Don't delete** `.git` directories inside submodules
- **Don't track** unstable branches in production releases
- **Don't forget** to `git submodule update` after pulling
- **Don't push** submodule changes without access to the submodule repository

## Advanced Usage

### Shallow Clones

For faster clones when you don't need full history:

```bash
git clone --recurse-submodules --shallow-submodules git@github.com:GQAdonis/prometheus-skill-pack.git
```

### Parallel Submodule Updates

Update multiple submodules faster:

```bash
git submodule update --jobs 4
```

### Execute Command in All Submodules

```bash
git submodule foreach 'git fetch'
git submodule foreach 'git status'
git submodule foreach 'npm install'
```

### Diff Including Submodules

```bash
git diff --submodule=diff
```

## References

- [Pro Git: Submodules](https://git-scm.com/book/en/v2/Git-Tools-Submodules)
- [GitHub Submodules Guide](https://github.blog/2016-02-01-working-with-submodules/)
- [Atlassian Submodules Tutorial](https://www.atlassian.com/git/tutorials/git-submodule)
