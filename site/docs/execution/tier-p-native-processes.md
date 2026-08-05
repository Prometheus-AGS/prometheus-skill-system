---
title: Tier P native process operations
description: When and how generated Python, Node, and Bash run under fail-closed operating-system isolation.
---

# Tier P native process operations

Tier P is for a script that is useful precisely because it can use a real installed interpreter, but should not inherit the authority of the user or agent that generated it. The runtime gives the script declared inputs, a writable output directory, bounded stdout/stderr, and a terminal deadline. Everything else is denied or explicitly granted.

## Supported programs

Tier P accepts four runtime enum values, but only three belong to the native process tier:

- `python3`
- `node`
- `bash`

`wasm-component` routes to Tier W. A compiled Rust, Go, Swift, Java, or C/C++ executable is not a Tier P input. Build persistent native tools through the normal release path, or compile bounded portable logic as a Tier W component.

## The authority model

The request declares code, named inputs, capabilities, limits, and provenance. Baseline policy rejects undeclared authority before the sandbox is started. Cedar may impose an additional denial, but a permit result cannot broaden the baseline.

At runtime the worker:

1. verifies the request and materializes code and inputs from the CAS;
2. creates a private run root;
3. writes named inputs read-only below `PROMETHEUS_INPUT_DIR`;
4. exposes a private writable `PROMETHEUS_OUTPUT_DIR`;
5. clears ambient environment variables and sets only the runtime contract;
6. launches the real interpreter under the platform sandbox;
7. kills the complete process group on timeout or stream overflow; and
8. collects only safe, bounded, non-symlink output artifacts.

Scripts should treat both directories as capabilities. A named input `records` appears as `$PROMETHEUS_INPUT_DIR/records`; an output becomes durable only when written below `$PROMETHEUS_OUTPUT_DIR`.

## macOS: Seatbelt

The release Mac backend generates a Seatbelt profile for the canonical run root, output root, actual interpreter, and required system read roots. It denies network and external writes, restricts reads, and launches through `/usr/bin/sandbox-exec`. The worker records the exact profile and interpreter hashes in the receipt.

Version-manager shims are not attested toolchains. The Node backend resolves a real executable rather than trusting a shell shim whose target depends on ambient environment. Python and Bash use the configured absolute system paths.

Use Tier P on macOS for:

- generated JSON/CSV transforms;
- repository or artifact analysis with declared inputs;
- deterministic report creation;
- bounded migration planning that emits a plan rather than applying it; and
- one-shot validation where the receipt is part of the deliverable.

Do not use it for a server, watcher, interactive process, package installation, browser automation, or a job requiring ambient credentials.

## Linux: bubblewrap and Landlock

The Linux design uses bubblewrap for namespace and mount isolation and Landlock as an additional kernel-enforced restriction. The plan drops capabilities, isolates user/PID/IPC/UTS/cgroup/network namespaces, mounts system roots read-only, and layers a private writable output tree.

This repository has source, cross-build, and portable planning evidence for Linux, but the release certification did not execute a Linux kernel runtime. Documentation and receipts must not promote that evidence to runtime-certified status. If the required sandbox binaries or enforcement level are unavailable, readiness fails; direct interpreter execution is forbidden.

## Windows: unavailable

No Windows Tier P sandbox backend is implemented. Health can still report the binary, contracts, and offline verifier, but native process readiness is unavailable. WSL is a Linux environment and would require its own real runtime certification; it is not a Windows backend claim.

## A complete Tier P request flow

```bash
prometheus-exec init --identity ./exec-identity.json

prometheus-exec daemon \
  --socket ./runtime/exec.sock \
  --state-dir ./exec-state \
  --identity ./exec-identity.json \
  --plugin-root "$HOME/.prometheus/plugins/prometheus-skill-pack"

prometheus-exec run \
  --socket ./runtime/exec.sock \
  --state-dir ./exec-state \
  --identity ./exec-identity.json \
  --runtime python3 \
  --code ./examples/prometheus-exec/tier-p/transform.py \
  --input records=./examples/prometheus-exec/tier-p/records.json \
  --timeout-ms 5000 \
  --output-mb 2 \
  --format json
```

The CLI generates and signs a new request, pins code and input bytes, submits over the Unix socket, and waits for terminal state. For caller-controlled request IDs, same-ID replay, conflict behavior, and resumable events, use the REST or MCP contract described in [Local API, CLI, and MCP](./local-api-cli-and-mcp.md).

## Failure semantics

Expected failures are evidence, not reasons to weaken policy:

| Failure | Meaning |
| --- | --- |
| `tier_unavailable` | Required platform sandbox or interpreter is unavailable |
| capability/policy denial | Request asks for authority outside the declared baseline or Cedar policy |
| timeout | Wall-clock limit expired and the process group was terminated |
| output or artifact limit | Combined streams or collected artifacts exceeded the signed limit |
| invalid output path | Traversal, absolute path, or symlink attempted to escape the output namespace |
| interrupted | The process crossed the durable spawn boundary but no valid terminal receipt existed after restart |

Never convert one of these into an unsandboxed retry that still emits an attested receipt.

Next: [Tier W portable components](./tier-w-portable-components.md).
