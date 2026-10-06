# Importing an independently owned skill

Import a complete external repository as a submodule when it has its own maintainer, license and release lifecycle. Native pack skills belong in `skills/<category>/`; a monorepo subdirectory needs a deliberate packaging decision rather than a fabricated submodule path.

Before importing, inspect the actual source, frontmatter, helper dependencies, license and mutation behavior. Choose an HTTPS repository URL and an explicit commit or release. Document access requirements for a private source without committing credentials.

```bash
# Example shape: replace the repository and skill name with the approved source.
git submodule add https://github.com/OWNER/REPOSITORY.git skills/imported/skill-name
```

An import modifies `.gitmodules` and a gitlink; it does not validate or install the skill. Finish all planned phase production, then perform the relevant local structural checks and real packaged or installed integration. Strict skill validation and a helper-only run are narrower than native discovery and invocation. Record source identity, collaborator limits and exact local results before a push; follow owner review and merge requirements.

Fix an imported skill in its owning source repository and update the parent pointer deliberately. Do not repair an installed cache, blindly update all submodules to latest, delete `.git` metadata or remove failed imports as an automatic recovery step. Preserve uncommitted work and obtain the appropriate owner instruction for destructive recovery.

See [submodule ownership](SUBMODULES.md), [contributing](../CONTRIBUTING.md), [skill authoring](skill-authoring-guide.md) and [generated ownership](generated-output-ownership.md).
