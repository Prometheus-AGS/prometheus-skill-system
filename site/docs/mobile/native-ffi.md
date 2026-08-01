---
id: native-ffi
title: Native Mobile FFI
sidebar_label: Native FFI
---

# Native Mobile FFI

`substrate/skill-ffi` exposes the skill registry to iOS and Android as a native
library, for apps that embed the runtime rather than talking to a server.

## Verified builds

Unlike the [Wasm path](./wasm-components), this one is executed and asserted —
both targets build and seven round-trip tests check **returned values**, not
merely that linking succeeded.

| Target | Artifact | Size |
|---|---|---|
| `aarch64-apple-ios` | dylib | 16,408 B |
| `aarch64-linux-android` | `.so` | 454,856 B |

```bash
bash substrate/skill-ffi/build-mobile.sh
```

The Android build additionally requires `aarch64-linux-android-clang` from the
NDK on `PATH`.

## Public API

From `substrate/skill-ffi/src/api.rs`:

```rust
/// Execute a skill by id with a JSON input payload.
pub fn run_skill(skill_id: String, input: String) -> Result<String, SkillError>;

/// Metadata for one skill.
pub fn describe_skill(skill_id: String) -> Result<SkillDescriptor, SkillError>;

/// Everything the embedded registry knows about.
pub fn list_skills() -> Result<Vec<SkillDescriptor>, SkillError>;

/// The `prometheus:component` world version this build targets.
pub fn world_version() -> String;
```

`world_version()` exists so a host can detect a mismatch between the library it
linked and the WIT world it expects — a version skew that would otherwise surface
as confusing runtime failures.

## Crate configuration

```toml
[lib]
crate-type = ["cdylib", "staticlib", "rlib"]

[dependencies]
flutter_rust_bridge = "=2.12.0"
```

Three crate types because three consumers need different things: `cdylib` for
Android's `.so`, `staticlib` for iOS static linking, `rlib` so Rust tests can
exercise the same code without going through FFI.

The `=2.12.0` is an **exact** pin, not a caret range — matching the version
already in production in the consuming app.

## Choosing a binding pattern — how this was decided

Worth recording, because the reasoning generalises past this repo.

The original decision compared **uniffi vs cbindgen** on maintenance cost, and
chose uniffi. Adversarial review returned **CRITICAL**: the stated consumer is
Flutter, and neither option was what Flutter uses.

One command settled it:

```bash
grep -rn 'flutter_rust_bridge' know-me-system/**/pubspec.yaml
# flutter_rust_bridge: 2.12.0   ← already in production
```

**A third pattern, in neither column of the comparison, already shipping in the
consuming app.** Adopting it cost nothing; either alternative would have imposed
a migration on a working system.

:::tip Best practice
Before comparing binding generators — or any integration library — grep the
consuming project's manifests. The incumbent is frequently in neither column, and
"what they already use" beats "what scores best in the abstract" whenever it is
adequate.
:::

### When to use which

| Consumer | Pattern |
|---|---|
| Flutter app (KnowMe) | `flutter_rust_bridge` — the incumbent |
| Swift / Kotlin, no Flutter | `uniffi` |
| C interop required | `cbindgen` |

## Integration examples

### Flutter / Dart

```dart
import 'package:my_app/src/rust/api.dart';

final skills = await listSkills();
for (final s in skills) {
  print('${s.skillId}: ${s.title}');
}

final result = await runSkill(
  skillId: 'some-skill',
  input: jsonEncode({'query': 'example'}),
);
```

### Swift

```swift
let version = world_version()
guard version == expectedWorldVersion else {
    throw SkillError.worldVersionMismatch(found: version)
}

let skills = try list_skills()
```

Check `world_version()` at startup. A silent mismatch produces failures that look
like logic bugs.

## Testing guidance

Seven tests assert on returned values. That distinction matters:

```rust
// WEAK — passes if the function returns garbage
#[test]
fn it_links() {
    let _ = list_skills();
}

// STRONG — asserts on what came back
#[test]
fn list_skills_returns_the_registered_set() {
    let skills = list_skills().expect("list");
    assert!(!skills.is_empty(), "an empty registry means discovery failed");
    assert!(skills.iter().all(|s| !s.skill_id.is_empty()));
}
```

A `.so` that links but returns empty results passes a build check and fails a
round trip. **Test the artifact, not the build.**

## Best practices

1. **Pin the bridge version exactly** (`=2.12.0`). FFI codegen and runtime must
   agree; a caret range lets them drift apart between builds.
2. **Call `world_version()` on startup** and fail loudly on mismatch.
3. **Keep the FFI surface small.** Four functions is deliberate — every exported
   symbol is a compatibility obligation across two toolchains.
4. **Pass JSON across the boundary, not rich types.** It keeps the generated
   bindings trivial and the versioning story simple.
5. **Run the round-trip tests on every target you ship**, not just the host you
   develop on.
