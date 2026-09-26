# Prompt, skill, or native agent

Use a prompt for one-off work. Use a skill for repeated process guidance inside
an existing harness. Use a native agent when the capability needs its own runtime,
protocol surface, persistence, deployment, or independent lifecycle.

## Decision prompt

```text
Classify this capability as prompt, skill, or native agent.

Capability:
Consumers:
Lifecycle:
Persistence:
Protocols:
Packaging:
Verification:

Return the classification, evidence, rejected alternatives, required creator
prompt, and stop conditions.
```

## Native-agent proof

A native agent needs build, launch, protocol consumer, health/readiness, and
package/container proof. Do not use a case study's stale model catalog or
unsupported authentication instructions as current guidance.
