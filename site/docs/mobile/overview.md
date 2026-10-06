---
id: overview
title: Mobile Skill Portability — Overview
sidebar_label: Overview
---

# Mobile Skill Portability

A skill can be instructions, a script, a component or a host integration. Reading instructions on a phone does not supply the filesystem, process, credentials or native services required by its workflow.

## Choose the actual execution path

- **Instructions:** a capable model/harness may read the skill, but still needs its declared tools and authorized context.
- **Process-bound scripts:** desktop interpreters are not an on-device mobile runtime.
- **Components:** an authorized component needs a matching host, capability grant and value-level execution evidence.
- **Native embedding:** pack FFI exposes index and Exec contracts; generic `run_skill` and `list_skills` currently return `Unsupported`.
- **Remote work:** an authorized host integration must deliver work and prove the destination result. Companion synchronization alone does not execute arbitrary skills remotely.

The [execution classifier](./execution-classes) is a source heuristic. Its historical counts are not current inventory or mobile acceptance. [Component format](./wasm-components), [FFI](./native-ffi) and [Exec evidence dimensions](/docs/execution/platform-and-evidence-status) explain distinct boundaries.

## Ownership

The pack owns WIT, local embedding and bounded execution contracts. UAR owns its consumers and database registration. Companion owns connected synchronization and the relocated mobile KBD-sync implementation. Neither a portable manifest nor a cross-build guarantees native discovery, successful invocation or a usable mobile app.

Keep model policy and server credentials in the authorized host boundary. A mobile catalog may contain public index metadata; it must not transport private signing keys, session tokens or another role's private lesson text.

Read the [historical UAR database record](./uar-skill-database) for earlier consumer findings; it is separate from current pack/mobile certification.
