---
id: native-ffi
title: Native Mobile FFI
sidebar_label: Native FFI
---

# Native Mobile FFI

`substrate/skill-ffi` is the pack's embedding boundary. Native linking, catalog lookup, bounded execution and a usable phone workflow need separate evidence.

## Current source surface

```rust
pub fn run_skill(skill_id: String, input: String) -> Result<String, SkillError>;
pub fn describe_skill(skill_id: String) -> Result<SkillDescriptor, SkillError>;
pub fn list_skills() -> Result<Vec<SkillDescriptor>, SkillError>;
pub fn list_indexed_skills(index_json: String) -> Result<Vec<SkillDescriptor>, SkillError>;
pub fn search_indexed_skills(index_json: String, query: String, limit: u32)
    -> Result<Vec<SkillDescriptor>, SkillError>;
pub fn world_version() -> String;
```

`run_skill` validates input and returns `Unsupported`: no generic skill host is bound. `list_skills` also returns `Unsupported`; it does not enumerate a working runtime. `describe_skill` constructs a descriptor and is not execution proof.

Indexed APIs parse the exact index JSON supplied by the host and reuse `prometheus-skill-index` ranking. The caller must verify generation/index provenance before exposing it; parsing a JSON string does not verify its signature. Returned descriptors contain `id`, `exports` and `capabilities`.

The separate `exec_*` async surface delegates to the configured embedded Prometheus Exec adapter. It has its own request, authorization, event, artifact and receipt contract. It does not make `run_skill` a universal executor or accept private key bytes from UI callers.

## Consumer and evidence boundary

The crate pins `flutter_rust_bridge` and declares native library outputs in its Cargo manifest. Generated bindings must agree with that pin. Swift/Kotlin bindings shown in older pages were illustrative, not shipped wrappers.

Historical cross-build sizes and host round trips are not current physical-device acceptance. Mobile execution still needs an actual consumer, supported profile, size evidence and real device operation. See [execution platforms](/docs/execution/platform-and-evidence-status) and [components](./wasm-components).

The mobile KBD-sync surface moved to Companion; the retained empty compatibility feature is not an implementation. Source: [skill-ffi API](https://github.com/Prometheus-AGS/prometheus-skill-system/blob/main/substrate/skill-ffi/src/api.rs).
