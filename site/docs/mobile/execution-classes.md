---
id: execution-classes
title: Execution Classes
sidebar_label: Execution Classes
---

# Execution Classes

Every skill that carries scripts is assigned an execution class describing what
it needs at runtime. The classification is **derived by a script, not asserted by
hand**, and a `--check` mode fails CI when the committed result goes stale.

```bash
bash skills/process/adversarial-review/scripts/classify-mobile-execution.sh
bash skills/process/adversarial-review/scripts/classify-mobile-execution.sh --check
```

## The classes

| Class | Name | Needs | Mobile path |
|---|---|---|---|
| — | **manifest-only** | Nothing — no `scripts/` | Runs today, unchanged |
| **E0** | Process-bound | `bash`, `python3`, a binary | None on-device; use remote execution |
| **E1** | Capability-bound | Filesystem and/or clock | Wasm component **with** granted capabilities |
| **E2** | Pure | Computation only | Wasm component, no capabilities |
| **R** | Remote | A full desktop environment | Drive a paired desktop over P2P |

## Current distribution

| Class | Count |
|---|---|
| manifest-only | **249** |
| E0 | 28 |
| E1 | 18 |
| E2 | 2 |
| R | 13 |
| **Total** | **310** |

**249 of 310 skills are already mobile-ready.** A manifest-only skill is
instructions a model reads; there is nothing to execute and therefore nothing to
port. The portability problem is confined to the 61 script-bearing skills.

## E1 was wrong for every member — a lesson

E1 was originally defined as *"pure text/JSON transformation."* In practice it was
the **residual**: whatever E0, E2, and R did not match fell into it.

When a skill was later needed for a Wasm port, an audit found **all 18 of 18 E1
members touch the filesystem or the clock**. Not one was a pure transformation.
The residual had silently absorbed every skill no other rule matched, and then
presented itself as a positive finding.

Two things make this worth remembering:

1. **A `--check` drift test cannot catch it.** Drift checks compare the committed
   file to a freshly generated one. Both come from the same wrong rule, so they
   agree perfectly and the check passes forever.
2. **The risk was written down and shipped anyway.** The script's own header said
   E1 was *"the class most likely to be wrong, which `--check` cannot detect."*

**Corrective action:** E1 now carries an explicit `needs_capabilities` field. A
skill labelled portable must state its price.

:::tip Best practice
Never let an else-branch stand as a verdict. If a class is defined as "everything
left over," hand-verify a sample before trusting it — the automated check that
would normally catch drift is structurally blind to this failure.
:::

## Reading the output

```json
{
  "skill": "some-skill",
  "class": "E1",
  "needs_capabilities": ["filesystem", "clock"],
  "evidence": "scripts/run.sh reads $PWD and calls date"
}
```

`evidence` names the file and construct that drove the decision, so a
classification can be argued with rather than merely trusted.

## What to do per class

**manifest-only** — nothing. This is the target state; prefer it when authoring.

**E2** — port to a Wasm component against
[`prometheus:component`](./wasm-components). Two skills qualify today.

**E1** — portable, but the host must grant capabilities. Decide whether the
capability is essential or incidental; a skill that calls `date` for a log line
can often drop the dependency and become E2.

**E0** — no on-device path. Either rewrite as manifest-only, or accept remote
execution.

**R** — route to a paired desktop. The user needs one internet-connected machine
acting on their behalf; the phone drives it over the existing sync layer without
an intermediate server.
