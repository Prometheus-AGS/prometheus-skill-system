---
id: wasm-components
title: WebAssembly Components
sidebar_label: Wasm Components
---

# WebAssembly Components

`prometheus:component@0.1.0` is the WIT package a skill implements to run inside
a Wasm host — including on a phone, where spawning a process is impossible.

## Status — read this first

**The WIT family is authored and parses as a single package. A reference
component builds against it and validates. Nothing has executed it.**

UAR's Wasm tier is still a stub. This documentation will keep saying so until a
component has actually run in a host, because **well-formed is not working**, and
a portability story that blurs the two is worse than one that admits its edge.

What that means practically:

| Claim | Status |
|---|---|
| The WIT parses as one package | ✅ verified |
| A component builds against it | ✅ verified |
| A component validates | ✅ verified |
| A component **runs** in UAR | ❌ **not yet** |

## Package layout

`wit/prometheus-component/`:

| File | Contents |
|---|---|
| `types.wit` | `interface types` — shared value types |
| `capabilities.wit` | `interface log`, `interface kv-store`, `interface clock` |
| `skill.wit` | `world skill` — what a single skill implements |
| `plugin.wit` | `world plugin` — composition of multiple skills |
| `MAPPING.md` | How execution classes map onto worlds |

All four declare `package prometheus:component@0.1.0;`.

## Capabilities are explicit

A component gets **nothing** it is not granted. The three interfaces in
`capabilities.wit` are the entire surface:

```wit
interface log      { /* structured logging back to the host */ }
interface kv-store { /* durable key/value, not a filesystem */ }
interface clock    { /* current time, host-controlled */ }
```

This is why [execution class](./execution-classes) E1 carries
`needs_capabilities`. A skill that reads the clock is portable — but the host must
grant `clock`, and that grant is visible rather than implicit.

Note what is **absent**: no raw filesystem, no arbitrary network, no process
spawn. `kv-store` deliberately replaces file access, because a key/value
interface is implementable on every target while a POSIX filesystem is not.

## Two Wasm formats — only one loads

:::danger This trips people up
`skills/rust/librefang-wasm-skill/` generates **core-wasm** guests with an
`extern "C"` pointer ABI and **zero `.wit` files**.

UAR loads `wasmtime::component::Component` — the **Component Model** format.

These are different binary formats with **no adapter between them**. A guest
built from the librefang templates *cannot load in UAR*, and no amount of
configuration changes that.
:::

The pack ships wasm tooling that cannot run in the host this work targets. That
is recorded in `wit/prometheus-component/MAPPING.md` and remains unresolved by
design — the templates have their own consumer.

**If you are targeting UAR, use `wit/prometheus-component`.** If you are
targeting librefang's core-wasm host, use its templates. Do not mix them.

## Authoring a component

```bash
# 1. Add the WIT package to your guest crate
cp -r wit/prometheus-component wit/

# 2. Generate bindings (Rust example)
cargo add wit-bindgen

# 3. Implement `world skill`
# 4. Build to the component model target
cargo build --target wasm32-wasip2 --release

# 5. Validate against the WIT
wasm-tools component wit target/wasm32-wasip2/release/my_skill.wasm
```

Step 5 is not optional. A component that compiles but does not match the world is
a runtime failure moved to a place nobody looks.

## Which skills should become components

| Class | Component-suitable? |
|---|---|
| **E2** (2 skills) | Yes — pure computation, no capabilities needed |
| **E1** (18 skills) | Yes, **with** the capabilities they declare |
| **E0** (28 skills) | No — they need a process; use remote execution |
| manifest-only (249) | Unnecessary — nothing to execute |

Start with E2. Two skills is a small enough surface to prove the path end to end
before committing to eighteen more.

## Best practices

1. **Do not port a skill just because you can.** If it is manifest-only, leave it
   alone — 249 skills already run everywhere.
2. **Declare the narrowest capability set that works.** `clock` for a log
   timestamp is usually removable; dropping it moves the skill from E1 to E2.
3. **Validate the built artifact against the WIT**, every build.
4. **Never assume the two wasm formats interoperate.** Check which host you are
   targeting before choosing a template.
5. **Treat "it validates" as a checkpoint, not a finish line.** Until a component
   has executed in the host, the integration is unproven — which is exactly the
   state this page documents.
