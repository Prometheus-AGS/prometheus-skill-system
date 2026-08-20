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

- **Repository**: https://github.com/GQAdonis/artifact-refiner-skill.git
- **Description**: PMPO-driven artifact refinement engine for logos, UI components, A2UI specs, images, content, and code
- **Version**: 1.1.0
- **Type**: Complete plugin package (includes agents, hooks, scripts)
- **Notes**: The Tauri scaffold inside artifact-refiner is a *generic* scaffold for any Vite/React app. It supports web, PWA, Tauri desktop, and Tauri mobile as deployment targets. **Tauri-mobile support in the scaffold is not the parent project's decision** — see "Parent project mobile decision" below.

### prometheus-entity-management

- **Repository**: https://github.com/Prometheus-AGS/prometheus-entity-management.git
  (imported under the skill name `prometheus-entity-management`)
- **Description**: Normalized, globally-reactive entity graph store for React, with bindings for Tauri, Flutter/Riverpod, Svelte, Solid, and more
- **Version**: 3.0.0-alpha
- **Type**: TypeScript library + Tauri plugin + Flutter/Riverpod binding
- **Notes**: This project ships **Tauri plugin support** by design (the entity-graph has a Rust core that runs as a Tauri command). The Tauri-mobile research and device-lane docs inside the imported source (`.research/v3-tauri-mobile-plugin/`, `release/tauri-mobile-device-lane.md`) describe the *project's own* Tauri-mobile plugin development, not a recommendation that the parent prometheus-skill-pack adopt Tauri mobile. See "Parent project mobile decision" below.

## Parent project mobile decision (read before touching imported skills)

The **prometheus-skill-pack's** mobile stack is:

- **Tauri = desktop only.** Tauri 2.0 mobile is **not** in scope for the parent project.
- **Mobile = Flutter + Rust over FFI** via `flutter_rust_bridge`. Same Rust substrate crates as desktop, separate Flutter shell. 1Password and AppFlowy pattern.
- **Never Capacitor.** Web-wrappers (Capacitor, Cordova, Ionic) add a web layer and latency for no architectural benefit.

This is the parent's decision, recorded in the architecture review
`docs/audits/2026-08-20-skill-pack-architecture-review.md` §14 (resolved
2026-08-20). It does **not** retroactively change the imported skills'
internal Tauri-mobile references, because those references are about
the imported skills' own scope:

- `artifact-refiner` ships a **generic Tauri scaffold** that supports
  web, PWA, Tauri desktop, and Tauri mobile as deployment targets. The
  scaffold is a tool the user can point at any of those targets; the
  *parent project* uses the desktop variant.
- `prometheus-entity-management` is a project that ships a **Tauri
  plugin** (the entity-graph has a Rust core that runs as a Tauri
  command). Its Tauri-mobile research and device-lane docs are part
  of that plugin's own development — not a recommendation that the
  parent project adopt Tauri mobile.

If you intend to **use** one of the imported Tauri-mobile artifacts
from this skill pack, you are explicitly opting into the imported
project's scope. The parent's `Prometheus Mobile` (Pillar 6) is a
separate Flutter shell, not a Tauri-mobile build.

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
