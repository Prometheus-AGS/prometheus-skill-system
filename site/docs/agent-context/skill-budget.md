---
title: Skill Budget
description: Why installed skills stop firing, how to measure the real number, and why raising the budget does not fix it.
---

# Skill context budget

A skill can be installed without being selected or invoked. Description quality, enabled scopes, duplicate payloads, harness configuration and the actual context budget all affect discovery. The full helper `skills/process/prometheus-context-bootstrap/scripts/skill-budget.sh` estimates description volume across the scopes it scans.

```bash
bash skills/process/prometheus-context-bootstrap/scripts/skill-budget.sh --path "/path/to/project" --json
```

Run diagnostics locally after completing phase production. The helper reads the configured fraction or its own default, estimates tokens from description characters and reports the scanned scopes. These are helper assumptions; they do not establish the current harness's eviction algorithm, native token accounting or whether a particular skill fired. Historical estate counts are not the current pack inventory.

Parse folded YAML descriptions correctly, preserve selected plugins and route specialized skills only when relevant. Disabling or removing user-owned skills needs the applicable instruction; do not prune a machine's skills as an automatic budget repair. Use the generated catalog for payload inventory and the harness's current official settings for context controls.

For a failed invocation, inspect the actual installed source, enabled plugin, manifest, hook trust and stderr before attributing it to budget. See [plugin source failures](/docs/operations/plugin-sources-and-hook-failures).
