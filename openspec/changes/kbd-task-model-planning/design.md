## Context

See proposal.md. The mini has per-change agent recommendations; the full pack also has class-based routing. Neither binds individual backend task identities to a concrete model and supported worker route.

## Goals / Non-Goals

Deliver matching portable planning instructions with minimal execute/apply consumption. No runtime schema, new launcher, automatic model switching, provider configuration, or services. The existing KBD loop owns task completion.

## Decisions

Use a Markdown assignment table in plan.md, keyed by phase path/change/backend task ID, rather than extending runtime state. This keeps existing parsers compatible. Task files link entries using non-checkbox prose; native tasks.json remains unchanged.

Choose quality-first among candidates meeting explicit policy, budget and capability constraints. Read current model/harness capabilities; unknown data stays unknown. Optional discovery tools cannot turn their cheapest-first ordering into a quality decision.

Distinguish inference through liter-llm from a tool-enabled worker. Native invocation schemas and installed versions define usable routes. Missing routes are unresolved; independent work remains possible. A task/harness/catalog change invalidates the old dispatch choice.

## Risks / Trade-offs

- Instructions are followed by the executing agent, not enforced by a new runtime → exercise the installed skill in real planning sessions and inspect handoffs.
- Provider quality and availability change → record dated evidence and recheck before dispatch.
- Five harnesses need different controls → current official docs plus installed tool schema; document untested paths.
- Existing working trees are dirty → isolated task branches, no unrelated source changes in release.

## Migration Plan

Release mini 0.2.0 and full 1.11.0/process 1.7.0. Generate packages from canonical source. Install only the full pack on this host through a clean committed release checkout, retain old generation/configuration for rollback, refresh native marketplaces, and verify payload hashes and fresh-session discovery. Existing sessions reload. Legacy plans select explicitly at execution.

## Latest OpenSpec lifecycle

Use a shared self-contained Node runner, carried in each distribution. On refresh, resolve latest stable from the official npm registry (baseline1.14.0), install an exact version into a per-user managed cache with scripts disabled, and invoke its JavaScript bin without shell shims. Subsequent KBD OpenSpec commands use that selected version, not an older global binary. Explicit operator pin/disable is recorded. Registry outages permit an already installed version with latest-unverified evidence; a missing runtime is an unresolved prerequisite.

The existing15-second context hook has a bounded refresh budget; never add network/install work to the1-second canonical-control hook. New/next phase and child scripts refresh before mutation, including their early canonical-runtime branches. Skill instructions cover resume, phase work, and direct CLI use without hooks. Raw Rust CLI commands outside the skill/wrappers are not intercepted.

Run upstream update noninteractively after backing up its generated targets. Preserve authored openspec specs/changes/config and KBD state; no universal authored-artifact version exists. Report compatibility or migration requirements rather than rewriting content or claiming semantic migration. Keep repository testing/authorization rules above generated upstream advice. Validate generated refresh, actual parsing and task ownership, offline behavior and both packaging closures at the completed boundary.
