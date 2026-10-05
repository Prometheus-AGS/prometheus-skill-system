# `entity-graph-optimize` as a WASM component

Reference component for `prometheus:component@0.1.0` — the first real skill
built against the unified world (`change-msp-006`).

## Evidence boundary

This directory supplies source for the checked component. UAR and Prometheus Exec
are distinct hosts: successful instantiation in one does not certify the other.
The selected signed generation or exact component pin determines the bytes
allowed by Tier W. Refer to [the execution evidence page](../../../../../site/docs/execution/platform-and-evidence-status.md)
for dated host results and open limits; no new runtime result is asserted here.

## What it ports, and what changed

Ports `scripts/detect-orchestrators.sh` — probe for orchestrator markers, emit
JSON. Two deliberate differences:

1. **Filesystem probes go through the `kv-store` capability.** A component
   cannot call `[ -e path ]`. The guest declares what it needs and the host
   decides whether to grant it; a component importing an interface outside its
   granted set fails before any guest code runs.

2. **A denied probe is an error, not `false`.** The shell version could not
   distinguish "marker absent" from "probe failed" and reported both as absence.
   The component returns `capability-denied` for the second. Preserving the
   shell behaviour would have carried a real bug across the port.

The shell script **stays**. This does not replace it.

## Rebuild after the complete production phase

```bash
bash build.sh
bash ../../../../../scripts/check-exec-tier-w-reference.sh
```

`skill.wasm` is committed because UAR consumes this repo as a submodule and must
not need a Rust toolchain to obtain it. `build.sh` exists so the binary is never
the only copy of the truth. The Tier W reference check builds the component
twice in isolated target directories, requires byte-for-byte equality, and
compares the result with the release hash and the checked artifact.
