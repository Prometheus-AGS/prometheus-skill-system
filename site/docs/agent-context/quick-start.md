---
title: Quick Start
description: Install the agent context structure into a new or existing project, and verify it.
---

# Bootstrap a project's context

Choose an existing authorized project and resolve the actual installed skill directory. From this source checkout, the full helper is:

```bash
bash skills/process/prometheus-context-bootstrap/scripts/bootstrap.sh --path "/path/to/project" --dry-run
bash skills/process/prometheus-context-bootstrap/scripts/bootstrap.sh --path "/path/to/project"
```

The default layout is legacy, with `mixed` profile. `--layout v4` selects the rules-source layout; v4 does not use the legacy profile variants. Inspect the proposed changes before applying. The broader full bootstrap requires its declared shell and tools; the direct portable UI/team installers use Node 22 or later.

Existing legacy v3 content can require migration rather than another bootstrap. The migration helper defaults to a report; `--apply` changes the project. Read the proposed disposition and preserve operator-owned sections before applying. Do not force a bootstrap to resolve malformed markers or escaping links.

After all planned phase production is complete, run the appropriate local integration and the helper's `verify.sh --path "/path/to/project"` diagnostic. A matching context layout does not prove installed harness discovery, native delegation or a rendered UI. See [UI routing commands](/docs/guide/ui-ux-routing) for the narrow portable installation and project-team selection contracts.

## Find rules that contradict the boundary rule

Before or after a bootstrap, list guidance that would have an agent run tests, builds or checks before the phase boundary. The audit is read-only:

```bash
bash skills/process/prometheus-context-bootstrap/scripts/bootstrap.sh --audit-conflicts --path "/path/to/project"
```

It reads the root and nested `AGENTS.md` and `CLAUDE.md`, their `@` imports inside the project, `.claude/rules`, and the repository's other Git worktrees. It also reports whether the installed pack generation verifies. It exits 0 with or without findings, checks recognized patterns only, and never edits a rule file.
