---
id: wasm-components
title: WebAssembly Components
sidebar_label: Wasm Components
---

# WebAssembly Components

The pack's `wit/prometheus-component` package defines `prometheus:component@0.1.0`. Component execution is host-specific: Prometheus Exec Tier W supplies a real bounded operation path; that does not activate a generic UAR or mobile skill host.

## Current interfaces

The package contains shared types, capability interfaces, the skill world and plugin composition. Current capability interfaces are `log`, `kv-store`, `input`, `output`, `clock` and `random`. Named input/output replaces ambient filesystem access; clock/random grants allow deterministic replay. The host must authorize exact bytes and link only the granted imports.

The component exports `run(input: string) -> result<string, error>` and may describe itself. Its exact world/imports must match the consumer. See [Tier W portable components](/docs/execution/tier-w-portable-components) for authorization, limits, profiles and the supported build/run path.

## Two formats

| Path | Format and consumer |
|---|---|
| Prometheus WIT / Exec Tier W | Component Model, normally `wasm32-wasip2` |
| LibreFang native-agent guest | Core Wasm, `wasm32-unknown-unknown`, pointer ABI |

These formats are not interchangeable. Packaging a core module in a zip does not convert it into a component. Share domain logic and build an explicit adapter for each consumer.

Mobile profiles need their own size and physical-device acceptance. The [FFI page](./native-ffi) identifies generic operations that still return `Unsupported`; instruction-only skills are not automatically executable phone applications.

Source: [current WIT](https://github.com/Prometheus-AGS/prometheus-skill-system/tree/main/wit/prometheus-component).
