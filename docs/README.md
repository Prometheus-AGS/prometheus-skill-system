# Documentation

Start with the [product README](../README.md) and [guide index](guide/README.md)
for installation, workflows and support boundaries. The [documentation site](https://prometheus-ags.github.io/prometheus-skill-system/)
combines guide pages with operational references from `site/docs/`.

[Choose a pack and work through an export change](guide/28-working-with-the-packs.md) · [Upgrade from an earlier release](guide/27-release-history.md)

## Current product references

- [Agent teams](agent-teams.md): requests, native definitions and ownership.
- [Memory tiers](guide/memory-tiers.md): scoped lessons and durable delivery.
- [Deployment modes](deployment-modes.md): optional services and their requirements.
- [Extension contract](integration-contract.md): the standalone pack and optional Companion boundary.
- [Readiness report](production-readiness-report.md): evidence and its limits.
- [Contribution guide](CONTRIBUTING.md): documentation and skill maintenance.

## Design and historical material

The [survey of third-party Rust skills](research/rust-skills/report.md) compares tools for writing, reviewing and porting Rust and documents the evidence behind its selections.

The `prometheus-feynman-learning-agent-*` documents in this directory describe
an application and marketplace design, not an installed skill-pack product.
Architecture, functional specifications, implementation plans and business-model
addenda are retained for design work. They do not establish a released frontend,
mobile application, coaching marketplace, payment integration or deployment.

`future-work/`, `audits/`, `evidence/`, `releases/` and decision records retain
plans and dated evidence. Interpret their claims at the recorded source and time;
use current product references for onboarding. No public route is removed by this
index change.
