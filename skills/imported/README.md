# Imported skills

Imported skills are Git submodules with independent source and license ownership.
The full pack selects immutable revisions in [skill-system.json](../../skill-system.json);
[.gitmodules](../../.gitmodules) records repositories. Neither upstream's newest
commit nor a version printed in an old README changes those selected pins.

| Import | Distribution contract |
|---|---|
| `artifact-refiner` | Its declared child skills enter the skill inventory; the nested sycophancy copy is excluded |
| `sycophancy-correction` | The separately pinned root skill is canonical |
| `prometheus-entity-management` | The import is not distributed directly; checked source under `skills/react/prometheus-entity-skills` supplies the pack's guidance |

An uninitialized submodule is a dependency prerequisite, not an empty skill or a
stray generated file. The root installer initializes the imports required by the
selected profile at their declared commits:

```bash
./install.sh --profile skills --targets detected --dry-run
./install.sh --profile skills --targets detected
```

Run these commands from the repository root. For source work, initialize only the
imports required by the specification; preserve the selected pins and any local
changes. Do not use a blanket remote-submodule update as an ordinary pack upgrade.
Release pin changes require owner approval and the final local integration gate.

## Imported scope and mobile design

Imported templates can describe targets the pack itself does not release. In
particular, artifact-refiner's generic Tauri scaffold and the entity project's
Tauri-mobile material describe those projects' scope. They do not establish a
Prometheus mobile product.

The pack's recorded mobile design uses Flutter with Rust FFI and reserves Tauri
for its desktop shell. Mobile builds, retained size and physical-device acceptance
have separate evidence boundaries; consult [the FFI source README](../../substrate/skill-ffi/README.md).
Companion's connected controls also remain a separate optional extension.

Preserve imported licenses and attribution. Make upstream-owned changes in their
source repository; update the pack's approved pin and generated distribution only
at the release boundary. See [submodule guidance](../../docs/SUBMODULES.md).
