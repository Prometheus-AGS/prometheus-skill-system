---
id: quick-start
title: Quick Start
sidebar_label: Quick Start
---

# Quick Start

## Install skills

From an owner-approved source commit:

```bash
git clone https://github.com/Prometheus-AGS/prometheus-skill-system.git
cd prometheus-skill-system
./install.sh --profile skills
```

The root installer initializes pinned imports and projects the signed generation to selected detected clients. It prints the mutation plan; preserve the source while registrations refer to it. Release and minimum-active versions come from `skill-system.json`, not a fixed version in this guide.

Use Node matching the repository requirements; Node 22 is the configured development line. Full installation on macOS/Linux additionally builds binaries and configures selected services. Native service installation requires Bash 4+. See [Installation](19-installation.md) and [Services, ownership and recovery](26-service-operations.md).

## Start a workflow

In a harness that exposes the installed skill, invoke:

```text
/kbd-assess my-project
/kbd-analyze my-project
/kbd-plan my-project
/kbd-execute my-project
/kbd-reflect my-project
```

Finish the phase's production work before its consolidated local integration gate. A team can then split disjoint ownership; follow [Agent Teams](24-agent-teams.md) for actual creation and lifecycle requests.

Learning is optional: `/learn-goal "I want to understand Rust lifetimes"` begins a learning workflow when its prerequisites are available. Companion sync skills require their separate package; full installation alone does not provide them.

Read [the guide index](README.md) for the full sequence.
