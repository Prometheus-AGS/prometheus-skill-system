# Submodule ownership and updates

The root `.gitmodules` declares external repositories; Git's recorded gitlinks select their source commits. Imported skills and tool repositories have independent owners and release lifecycles. A branch declaration does not authorize advancing a selected commit, and a pack version does not identify each component release.

## Clone and inspect

```bash
git clone --recurse-submodules https://github.com/Prometheus-AGS/prometheus-skill-system.git
cd prometheus-skill-system
git submodule status --recursive
git diff --submodule=diff
```

An empty submodule checkout needs initialization at the selected pin, not a blind update to upstream `main`. Preserve existing uncommitted work before initializing, changing a pin or entering recovery. Inspect `.gitmodules`, the gitlink and the component's manifest together; source hosting URLs and selected commit identity are distinct.

## Deliberate updates

Work against the component's actual source and ownership rules, with a scoped change and selected commit or release. Finish all planned production changes before the local integration boundary. Verify the real pack/component collaborator path, then record the exact new gitlink and local results. Hosted CI and unit totals cannot authorize a pin advance or serve as current release evidence. Protected dependencies and versions retain their approval requirements; never advance them during a freeze.

Fixes inside a submodule belong to that repository's source and review process. Do not edit an installed cache or a generated pack copy to repair them. Updating the parent pointer, merging the component, publishing its release and rebuilding a pack are separate operations with separate evidence.

## Recovery

Detached HEAD at a recorded pin is normal. An unexpected modified submodule can contain source edits, a different checked-out commit or untracked runtime artifacts; inspect those facts before choosing a repair. Do not reset, delete metadata, deinitialize or remove a submodule as a generic troubleshooting step. Obtain the relevant owner instruction for destructive cleanup.

Resolve authored `.gitmodules` conflicts against the intended repositories and commits. Resolve generated payload conflicts according to [generated ownership](generated-output-ownership.md), after coherent phase production. Installation and binary build commands belong in the [installation guide](guide/19-installation.md) and [service operations](guide/26-service-operations.md), where platform and process ownership are explicit.
