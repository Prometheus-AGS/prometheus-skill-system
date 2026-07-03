---
id: metaprompting-pmpo-kbd
title: Metaprompting, PMPO & KBD
sidebar_label: Metaprompting & KBD
---

# Metaprompting, PMPO & KBD

This section covers the core philosophy of the Prometheus Skill Pack.

See the full chapter:
[docs/guide/02-metaprompting-pmpo-kbd.md](https://github.com/prometheusags/prometheus-skill-pack/blob/main/docs/guide/02-metaprompting-pmpo-kbd.md)

## Summary

**PMPO** (Prometheus Meta-Prompting Orchestration) is the underlying philosophy: all meaningful
work should flow through structured phases of **Assess → Analyze → Plan → Execute → Reflect**.

**KBD** (Know-Build-Deploy) is the lifecycle that implements PMPO as a set of slash commands:

```
/kbd-assess   → inspect the codebase, identify gaps
/kbd-analyze  → research libraries, build-vs-adopt decisions
/kbd-plan     → ordered change list
/kbd-execute  → implement changes one-by-one
/kbd-reflect  → measure goal achievement, write lessons
```

Each phase writes structured artifacts (assessment.md, analysis.md, plan.md, execution.md,
reflection.md) that become the source of truth for the next phase.
