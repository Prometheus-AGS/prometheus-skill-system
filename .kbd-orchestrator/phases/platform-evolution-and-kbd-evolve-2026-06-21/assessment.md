# Assessment — platform-evolution-and-kbd-evolve

**Phase started:** 2026-06-21  
**Assessed by:** Claude Sonnet 4.6 / kbd-process-orchestrator v2.0.0

---

## Scope

Four independent work streams, all entering this phase together:

1. **`/kbd-evolve` skill** — New KBD command that uses `iterative-evolver` to research the current landscape of a project's problem domain and determine the most impactful logical next evolution based on configurable criteria. Distinct from `/kbd-next-phase` (which just advances a phase) — evolve does a full domain-landscape assessment first.
2. **Auto-update capability** — Mechanism to pull latest skill-pack changes and re-install across all platforms without full reinstall; smart delta installs.
3. **Full native platform SDK semantics** — Deep integration for opencode (plugin.ts), Kimi desktop vs Kimi CLI (v0.14.2) differences, MiniMax Code vs mmx CLI differences; ensure native tool definitions are wired correctly.
4. **External project skills integration** — Import `prometheus-entity-management/prometheus-entity-skills/` (7 plugins, ~35 sub-skills) and `flint-realtime-fabric/sdks/` (5 SDK languages: csharp, dart, go, kotlin, swift, ts) as native skills/submodules; include SDK installation support in `install-skills-flat.sh` and `install-platforms.ts`.

---

## Current State Inventory

### Existing assets

| Asset | State |
|-------|-------|
| `skills/process/iterative-evolver/SKILL.md` | Present, v1.0.0 |
| `skills/process/kbd-process-orchestrator/SKILL.md` | Present, v2.0.0 |
| `scripts/install-skills-flat.sh` | 16 platforms; kimi + minimax added last phase |
| `scripts/install-platforms.ts` | kimi-code + minimax Platform entries present |
| `skills/imported/` | Only `artifact-refiner` as git submodule |
| `/kb-evolve` skill | ABSENT — does not exist |
| `auto-update` mechanism | ABSENT — `git pull` only, no delta-install |
| opencode `plugin.ts` native semantics | UNVERIFIED — opencode `~/.opencode/skills/` only |
| Kimi desktop vs CLI difference | UNKNOWN — only `kimi-code` CLI covered |
| entity-management skills | NOT IMPORTED — exist at sibling repo |
| flint-realtime-fabric SDKs | NOT IMPORTED — exist at sibling repo |

### Gap analysis

| Gap | Impact | Complexity |
|-----|--------|------------|
| No `/kbd-evolve` | Cannot do landscape-first evolution cycles | Medium |
| No auto-update | Users must re-run full install after updates | Low-Medium |
| opencode native semantics | Skills may not activate correctly in opencode | Medium |
| Kimi desktop skill discovery | Skills unreachable in Kimi desktop app | Medium-High |
| entity-management skills missing | 7 entity-graph plugins not available cross-platform | High |
| flint SDK skills missing | 6 SDK language skills not available | High |

---

## Risk Register

| Risk | Likelihood | Severity | Mitigation |
|------|------------|----------|------------|
| Kimi desktop has different skill dir than CLI | Medium | High | Research via `kimi doctor`, filesystem probe |
| opencode plugin.ts API unstable | Low | Medium | Pin to specific opencode version in compatibility |
| Git submodule path assumptions | Low | Low | Use relative paths in `.gitmodules` |
| entity-skills SKILL.md format differences | Low | Low | Validate with `npm run validate:strict` after import |
| flint proto files confuse skill validator | Low | Low | Add to validator exclude list |

---

## Decision: New KBD phase (not extend)

The previous phase (`memory-write-transport`) has `status: phase_complete`, `stage: reflect_complete`. Starting a new phase is correct per KBD lifecycle. This work is a distinct scope from the last fix.

---

## Entry criteria met

- [x] Previous phase reflection complete and gated (score 0.018, PASS)
- [x] Clear scope with 4 discrete work streams
- [x] All prerequisites exist (iterative-evolver skill, install scripts, entity-skills repo)
- [x] No blockers to starting
