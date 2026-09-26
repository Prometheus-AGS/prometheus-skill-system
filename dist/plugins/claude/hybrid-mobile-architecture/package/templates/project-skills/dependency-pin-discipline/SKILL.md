---
name: dependency-pin-discipline
description: ALWAYS invoke before adding, bumping, or unpinning a dependency in ANY manifest (pubspec.yaml, package.json, Cargo.toml, build.gradle) or before writing a version literal into a script or template — pins are EXACT, the rationale comment is the real artifact, and versions.toml is the only source. Also invoke when a codegen/analyzer version conflict appears, or when a dependency resolves differently than expected. Triggers on dependency, version, pin, bump, upgrade, caret range, pubspec, package.json, Cargo.toml, versions.toml, lockfile, analyzer conflict, build_runner, freezed, riverpod_generator, json_serializable, peer dependency, resolution failure, version solving failed, transitive dependency.
---
<!-- TJ-ARCH-MOB-001 compliant -->

> **Binding:** Prefer open standards and exact, independently verified
> dependency versions. When installed in a project, also obey that project's
> dependency policy; the rules below remain self-contained.

# Dependency Pin Discipline

## The rule

**Pin exactly. Say why. Keep it in one place.**

A floor (`^1.2.0`, `1.96+`, `"2"`) is not a pin — it is an instruction to resolve
to whatever published most recently. Two developers who scaffold a week apart get
different builds, and only one of them reproduces a given bug.

## Exact pins, not floors

```yaml
# NO — resolves to whatever shipped this morning
flutter_riverpod: ^3.3.2

# YES — this exact combination is known to resolve
flutter_riverpod: 3.3.1
```

```toml
# NO — a bare major
tauri = { version = "2", features = ["devtools"] }

# YES
tauri = { version = "2.11.5", features = ["devtools"] }
```

Caret ranges are acceptable for leaf dependencies with no codegen or peer
coupling. They are **not** acceptable for anything in a generator/analyzer graph
(below), for FFI crate/package pairs that must match exactly, or for a toolchain.

## The rationale comment is the real artifact

A pin without a reason is a number nobody dares change. Someone will "tidy" it
back to a caret in six months and re-derive the same failure.

```yaml
# 4.0.4 requires analyzer 12 and forces Freezed onto the invalid 3.2.6-dev.1
# generator; 4.0.3 is the latest Riverpod generator that resolves with
# Freezed 3.2.5.
riverpod_generator: 4.0.3
```

State **what breaks** and **what you tried** — not just "pinned for
compatibility". When a pin is a free choice rather than a forced one, say that
too, so the next person knows they may raise it.

## The analyzer-conflict class

Dart codegen packages share one `analyzer` dependency, and they do not move
together. The symptom is a resolution failure or a generator emitting invalid
code, and the cause is always the same shape:

```
freezed 4.0.0-dev.x        → analyzer 13
riverpod_generator 4.0.4   → analyzer 12
json_serializable 6.13.2+  → analyzer 10
build_runner 2.15.2        → analyzer 13.3
```

You cannot satisfy all four at their latest. The resolution is a **coherent set**
— the newest combination that actually resolves — pinned exactly, each with the
conflict noted. Never bump one member of the set alone; the set moves together or
not at all.

The same shape appears elsewhere: ESLint plugins against an ESLint major, Tauri
plugins against the Tauri core, an FFI crate against its language package.

## FFI pairs must match exactly

`flutter_rust_bridge` is a crate **and** a Dart package. Mismatched versions
produce codegen that compiles and then fails at the boundary — the worst failure
mode available, because it passes every build check.

Bump both together, re-run codegen, and never let `versions.toml` disagree with
the manifests it governs.

## One source: versions.toml

Every pin lives in `versions.toml`. Scripts read it through
`scripts/portable/versions.mjs`; authority docs quote it; `audit.mjs doc-consistency`
fails CI on drift.

**Never inline a version literal in a generator.** A hardcoded copy is invisible
to the audit, so bumping `versions.toml` changes the docs and changes nothing
about the emitted code — the generator and the projects it generated drift apart
silently. That failure is not hypothetical: it is exactly how this pack fell
behind the app it produced.

```javascript
// In a maintainer scaffolder authored under runtime/src.
import { requireVersion } from './portable/versions.mjs';
console.log(`flutter_rust_bridge: ${requireVersion('frameworks', 'flutter_rust_bridge')}`);
```

Inside a quoted heredoc, use an `@NAME@` placeholder and substitute after — see
`references/generator-placeholders.md`.

## Raising a baseline is allowed — and has a cost

Legacy-toolchain compatibility is not a product constraint. Raise Dart, Flutter,
Android, or iOS minimums whenever a modern dependency, security fix, native
capability, or materially better UX needs it.

The cost is fixed and non-negotiable: update the manifests, `versions.toml`
`[platform]`, `docs/platform-support.md`, and record a fresh **physical-device**
verification. A baseline bump verified only on a simulator is not verified.

## Red flags

| You're writing | Why it's wrong |
|---|---|
| `^x.y.z` on a codegen/analyzer package | The set moves together; a caret breaks it on the next `pub get`. |
| A bare major (`"2"`, `"1"`) in Cargo.toml | Resolves to whatever is newest; not reproducible. |
| A version literal inside a scaffolder | Invisible to `audit.mjs`; drifts from `versions.toml` silently. |
| A pin with no comment | Nobody can tell if it is forced or arbitrary, so nobody can safely bump it. |
| `--lts` / "latest" in a toolchain install | The meaning changes under you when the next LTS lands. |
| Bumping one member of a codegen set | Breaks the other three. Move the set or nothing. |
| `versions.toml` disagreeing with a manifest | One of them is lying; find out which before touching anything else. |

## Verification

```bash
node scripts/audit.mjs doc-consistency   # authority docs vs versions.toml
flutter pub get --dry-run               # does the set actually resolve?
cargo tree -d                           # duplicate transitive versions
```

## Scope note

This skill owns **how versions are chosen, pinned, and recorded**. It does not
own which engine a platform uses (`local-inference-lanes`) or whether a build is
shippable (`hybrid-runtime-verification`).
