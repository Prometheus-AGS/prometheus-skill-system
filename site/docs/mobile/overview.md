---
id: overview
title: Mobile Skill Portability — Overview
sidebar_label: Overview
---

# Mobile Skill Portability

Most skills in this pack assume something mobile does not have: **the ability to
spawn a process**. A skill that shells out to `bash`, `python3`, or a compiled
binary works on a laptop and is inert on iOS.

This section documents what actually runs on a phone today, what does not, and
the three mechanisms that close the gap. It is deliberately specific about
limits — a portability story that overstates its reach costs more than one that
admits its edges.

## The core constraint

| Platform | Can spawn processes? | Consequence |
|---|---|---|
| macOS / Linux / Windows | Yes | Every skill in the pack works |
| iOS | **No** — the sandbox forbids `fork`/`exec` | Script-bearing skills cannot run as written |
| Android | Technically yes, practically no | No shell, no interpreters, hostile to bundled binaries |

The design goal was never "make every skill run everywhere." It was: **know
exactly which skills run where, and give the rest a path that does not lie about
its cost.**

## What the measurement found

Every skill was classified by what it actually needs at runtime — not by what its
description claims. Run the classifier yourself:

```bash
bash skills/process/adversarial-review/scripts/classify-mobile-execution.sh
```

Current output:

| Class | Count | Meaning |
|---|---|---|
| **manifest-only** | **249** | No scripts at all — **portable today, unchanged** |
| E0 | 28 | Needs a process; no mobile path without a host |
| E1 | 18 | Portable **only with capabilities** (filesystem/clock) |
| E2 | 2 | Portable to a Wasm component |
| R | 13 | Remote-execution candidates |
| *script-bearing total* | *61* | |
| **Total** | **310** | |

**The headline is that 249 of 310 skills already work on mobile**, because a
manifest-only skill is instructions for a model — nothing to execute. The
portability problem is confined to the 61 that carry scripts.

### A correction worth reading

E1 was originally defined as "pure text/JSON transformation" — the *residual*
class, whatever E0, E2, and R did not match. When a skill was later needed for
porting, all **18 of 18** E1 members turned out to touch the filesystem or the
clock. The residual had silently absorbed every skill no other rule matched.

E1 now carries an explicit `needs_capabilities` field, so "portable" states its
price. **A residual class is a guess wearing a verdict's clothes** — and a
`--check` drift test cannot catch it, because it compares the file to itself.

## The three mechanisms

Each solves a different slice. None solves all of it.

### 1. Manifest-only skills — nothing to port

A skill with no `scripts/` directory is a `SKILL.md` the model reads. It runs
anywhere a model runs. **249 skills need no work.** Treat this as the default
and the target: when writing a new skill, ask whether the script is load-bearing
or merely convenient.

### 2. WebAssembly components — for pure computation

The `prometheus:component@0.1.0` WIT family
(`wit/prometheus-component/`) defines the interface a skill implements to run
inside a Wasm host:

| File | Purpose |
|---|---|
| `types.wit` | Shared value types |
| `capabilities.wit` | What the host grants (filesystem, clock, network) |
| `skill.wit` | The skill interface itself |
| `plugin.wit` | Plugin-level composition |

**Status, stated plainly: the family is authored, parses as one package, and a
reference component builds and validates against it — but nothing has executed
it.** UAR's Wasm tier is still a stub. Well-formed is not the same as working,
and this documentation will say so until a component has actually run.

:::warning Two Wasm formats, only one of which loads
`skills/rust/librefang-wasm-skill/` generates **core-wasm** guests with an
`extern "C"` pointer ABI and no `.wit` files. UAR loads
`wasmtime::component::Component`. These are **different binary formats with no
adapter** — guests from those templates cannot load in UAR.

If you are writing a skill intended to run in UAR, target the Component Model
via `wit/prometheus-component`, not the librefang core-wasm templates.
:::

### 3. Native FFI — for embedding in an app

`substrate/skill-ffi` exposes the skill registry to iOS and Android through a
native library. Verified builds:

| Target | Artifact | Size |
|---|---|---|
| `aarch64-apple-ios` | dylib | 16,408 B |
| `aarch64-linux-android` | `.so` | 454,856 B |

Public API (`substrate/skill-ffi/src/api.rs`):

```rust
pub fn run_skill(skill_id: String, input: String) -> Result<String, SkillError>;
pub fn describe_skill(skill_id: String) -> Result<SkillDescriptor, SkillError>;
pub fn list_skills() -> Result<Vec<SkillDescriptor>, SkillError>;
pub fn world_version() -> String;
```

Build both targets:

```bash
bash substrate/skill-ffi/build-mobile.sh
```

Seven round-trip tests assert on returned values, not merely on "it linked."

## Choosing a binding pattern

An honest note on how this decision was made, because the reasoning generalises.

The original comparison was **uniffi vs cbindgen**, decided on maintenance cost.
Adversarial review returned CRITICAL: the stated consumer is Flutter. One command
against `know-me-system` showed **`flutter_rust_bridge` 2.12.0 already in
production there** — a third option in neither column.

**Best practice:** before comparing binding generators, grep the consuming
project's manifests. The incumbent is frequently in neither column of your
comparison, and adopting it costs nothing while switching costs a migration.

| Consumer | Use |
|---|---|
| Flutter app (e.g. KnowMe) | `flutter_rust_bridge` — already in production |
| Swift / Kotlin direct | `uniffi` |
| C interop required | `cbindgen` |

## Remote execution — the escape hatch for E0

Thirteen skills are classified **R**: they cannot run on the device, but they do
not have to. A phone can drive a desktop that has the full pack installed, over
the existing P2P sync layer — no intermediate server required.

This preserves full function for skills that have no mobile answer at all. The
user needs exactly one internet-connected machine acting on their behalf. See
[Sovereign Sync](../sovereign-sync/overview) for the transport.

## Best practices

1. **Prefer manifest-only.** Before adding `scripts/`, ask whether the model can
   do it directly. 249 skills already prove this is usually the answer.
2. **If you must script, declare the class.** Run the classifier and check where
   your skill lands. An unclassified script-bearing skill is a mobile bug.
3. **Never let a residual class stand as a verdict.** If a class is defined as
   "everything left over," verify a sample by hand. E1 was wrong for 18 of 18.
4. **Target the Component Model, not core-wasm**, if the skill is meant for UAR.
5. **Check the consumer's manifests before choosing an FFI pattern.**
6. **Test the built artifact, not the build.** A `.so` that links but returns
   garbage passes a build check and fails a round trip.

## Verification

Every claim on this page is reproducible:

```bash
# Classification counts
bash skills/process/adversarial-review/scripts/classify-mobile-execution.sh

# Drift check — fails if the committed classification is stale
bash skills/process/adversarial-review/scripts/classify-mobile-execution.sh --check

# Mobile FFI builds for both targets
bash substrate/skill-ffi/build-mobile.sh

# Fabric invariants (fails CI on drift)
bash skills/devops/fabric-integration/scripts/check-invariants.sh
```
