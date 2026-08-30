---
id: quick-start
title: Quick Start
sidebar_label: Quick Start
---

# Quick Start

## Install

```bash
git clone https://github.com/Prometheus-AGS/prometheus-skill-system
cd prometheus-skill-system
./install.sh
```

The recommended skills-only profile initializes pinned imports and installs the
signed `1.8.0` distribution into detected clients. `1.8.0` is the minimum
supported active umbrella release. Use `./install.sh --profile full` on macOS or
Linux when you also want locally built binaries, MCP configuration, services,
and doctors.

## Try a skill

In Claude Code:

```
/learn-goal "I want to understand Rust lifetimes"
```

## Check sync status

```
/sync-status
```

## Start the KBD lifecycle

```
/kbd-assess my-project
/kbd-analyze my-project
/kbd-plan my-project
/kbd-execute my-project
/kbd-reflect my-project
```

## Full documentation

Browse the full product guide from
[the guide index](README.md).
