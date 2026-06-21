# Plan — platform-evolution-and-kbd-evolve

**Phase:** platform-evolution-and-kbd-evolve-2026-06-21  
**Changes total:** 5  
**Model policy:** frontier for assess/analyze/reflect; tiered for execute

---

## Change index

| # | ID | Title | Complexity |
|---|-----|-------|------------|
| 1 | change-001-kbd-evolve-skill | Create `/kbd-evolve` skill | Medium |
| 2 | change-002-auto-update | Auto-update + delta-install mechanism | Medium |
| 3 | change-003-opencode-kimi-mmx-native | Full native platform SDK semantics (opencode / Kimi desktop / MiniMax Code) | Medium |
| 4 | change-004-entity-skills-import | Import prometheus-entity-management skills as git submodule | Medium |
| 5 | change-005-flint-skills-import | Import flint-realtime-fabric SDK skills as git submodule | Medium |

---

## Change 001 — `/kbd-evolve` skill

### Goal

A new skill at `skills/process/kbd-evolve/SKILL.md` that implements the "domain-landscape-first evolution" cycle:

1. **Assess** current project/codebase state against goals (using KBD assess phase)
2. **Research landscape** — external domain research via web search / Tavily / firecrawl: what tools, patterns, competitors, and best practices exist for the project's problem domain
3. **Analyze gaps** — compare current state against landscape to find highest-impact improvement opportunities
4. **Determine evolution** — apply configurable criteria (effort, impact, alignment, feasibility) to rank opportunities and select the next evolution target
5. **Generate evolution spec** — write an evolution brief that `/kbd-new-phase` or `/kbd-process-orchestrator` can consume

**Distinguishes from `/kbd-next-phase`**: evolve doesn't just advance the next planned phase; it does a fresh landscape survey to ask "what SHOULD we build next?" from first principles.

### Steps

1. Create `skills/process/kbd-evolve/` directory
2. Write `SKILL.md` with YAML frontmatter (`name: kbd-evolve`, `version: 1.0.0`, `license: MIT`, tags: `[process, orchestration, evolution, research]`)
3. Write `references/criteria.md` — configurable evaluation criteria (default: effort × impact matrix)
4. Write `references/landscape-research.md` — research protocol (domain taxonomy, search strategy, source weighting)
5. Write `references/evolution-brief-template.md` — output format for evolution briefs
6. Validate: `npm run validate:strict skills/process/kbd-evolve`

### Acceptance criteria

- [ ] Skill triggers on `/kbd-evolve` in Claude Code
- [ ] Produces a ranked evolution brief with at least 3 scored candidates
- [ ] Brief is consumable by kbd-process-orchestrator as a new phase seed
- [ ] `npm run validate:strict` passes

---

## Change 002 — Auto-update + delta-install

### Goal

`scripts/update-skill-pack.sh` — a single command that:
1. `git pull --ff-only` (non-destructive; fails on conflict rather than clobbering)
2. Detects changed skills since last install (using `git diff --name-only` vs a stored `.last-install-ref`)
3. Re-installs only changed skills to each platform (delta install)
4. Updates `_meta.json` for MiniMax only for changed skills
5. Prints a summary: N skills updated, M platforms refreshed

### Steps

1. Write `scripts/update-skill-pack.sh` (bash, chmod +x)
2. Add `npm run update` script to `package.json` → calls `bash scripts/update-skill-pack.sh`
3. Store `.last-install-ref` (git SHA) in `~/.prometheus/skill-pack-install-ref` after install; read it on update
4. Add `update-skill-pack.sh` to smoke-test assertions
5. Document in `README.md` under "Updating"

### Acceptance criteria

- [ ] `npm run update` works on a clean pull with no changes (prints "0 skills changed, nothing to do")
- [ ] After adding a skill, `npm run update` installs only that skill (confirmed by diff of target dirs)
- [ ] MiniMax `_meta.json` `updated_at` timestamp updates on changed skills only

---

## Change 003 — Full native platform SDK semantics

### Goal

Verify and fix skill discovery + MCP wiring for:

- **opencode**: opencode uses `plugin.ts` (TypeScript plugin API) in addition to directory-based skills. Verify `~/.opencode/skills/` is the correct path and that `plugin.json` is parsed. If opencode has a TypeScript plugin format, create `.claude-plugin/opencode-plugin.ts` shim.
- **Kimi desktop** (vs Kimi Code CLI): Kimi desktop app may use a different skill directory (`~/Library/Application Support/KimiDesktop/skills/` on macOS or `~/.kimi/skills/`). Research actual path and add to installer.
- **MiniMax Code** (vs mmx CLI): MiniMax Code (the desktop IDE) may differ from `mmx` CLI in skill discovery. Research and add separate target if needed.

### Steps

1. Research opencode plugin.ts API: read opencode docs / source to determine if `plugin.ts` is required alongside `plugin.json`
2. Probe Kimi desktop skill directory: `find ~/Library/Application\ Support -name "skills" -type d 2>/dev/null | grep -i kimi`
3. Probe MiniMax Code directories: `find ~/Library/Application\ Support -name "skills" -type d 2>/dev/null | grep -i minimax`
4. If opencode needs `plugin.ts`: add template to `.claude-plugin/opencode-plugin.ts` and reference from `plugin.json`
5. If Kimi desktop has separate path: add `install_to_dir "kimi-desktop"` entry to `install-skills-flat.sh` and Platform entry to `install-platforms.ts`
6. If MiniMax Code has separate path: same pattern
7. Update `plugin.json` `compatibility.platforms` if new platform variants found
8. Update smoke-test assertions

### Acceptance criteria

- [ ] opencode skill loading verified (manual test or path confirmed)
- [ ] Kimi desktop skill path documented (even if same as CLI)
- [ ] MiniMax Code skill path documented (even if same as mmx)
- [ ] No regressions in existing platforms per smoke-test

---

## Change 004 — prometheus-entity-management skills import

### Goal

Add `skills/imported/prometheus-entity-skills/` as a git submodule pointing to `https://github.com/prometheus-ags/prometheus-entity-management` (or local path) at the `prometheus-entity-skills/` subtree. Alternatively, symlink the local checkout for development.

The entity-skills package contains 7 plugins (entity-graph-setup, crud, graphql, realtime, prisma, optimize) with ~35 sub-skills. Import strategy: git submodule at the `prometheus-entity-skills/` subdirectory, not the full monorepo.

### Steps

1. Determine if a standalone git repo exists for `prometheus-entity-skills/`; if not, use `git subtree` or a local path submodule pointing to `/Users/gqadonis/Projects/prometheus/prometheus-entity-management`
2. Add submodule: `git submodule add <url-or-path> skills/imported/prometheus-entity-skills`
3. Validate all imported skills: `npm run validate:skill skills/imported/prometheus-entity-skills/entity-graph-setup` (lenient mode — imported skills may have minor format differences)
4. Update `SKILLS.md` collection index with entity-management section
5. Update `marketplace/marketplace.json` with entity-skills plugins
6. Update `install-skills-flat.sh` to skip `skills/imported/prometheus-entity-skills/_shared` (shared refs, not a skill)
7. Update smoke-test to verify entity-graph-setup is installed to `~/.claude/skills/`

### Acceptance criteria

- [ ] `git submodule status` shows `prometheus-entity-skills` entry
- [ ] `npm run validate:skill skills/imported/prometheus-entity-skills/entity-graph-setup` passes
- [ ] `entity-graph-setup` visible in `~/.claude/skills/` after install
- [ ] `SKILLS.md` updated with entity-management table

---

## Change 005 — flint-realtime-fabric SDK skills import

### Goal

Create native skills for the Flint Realtime Fabric SDKs (`sdks/ts`, `sdks/go`, `sdks/swift`, `sdks/kotlin`, `sdks/dart`, `sdks/csharp`) in `skills/process/` or a new `skills/flint/` category. Each skill teaches agents how to install and use the SDK for its target language.

These are NOT submodules of the SDK source — they are skill files that describe how to USE the SDKs (install, configure, authenticate, subscribe to channels, publish events). The SDKs live in the flint repo; the skills live here.

### Steps

1. Create `skills/flint/` category directory
2. Create 6 skill directories: `flint-sdk-ts`, `flint-sdk-go`, `flint-sdk-swift`, `flint-sdk-kotlin`, `flint-sdk-dart`, `flint-sdk-csharp`
3. For each: write `SKILL.md` with YAML frontmatter, installation steps (`cargo add`/`npm install`/`go get`/etc.), configuration (env vars, channel setup), and a minimal "subscribe + publish" example drawn from `/Users/gqadonis/Projects/prometheus/flint-realtime-fabric/sdks/<lang>/`
4. Add `install_flint_sdks()` function to `install-skills-flat.sh` that installs Flint SDK to target language toolchain if available
5. Validate: `npm run validate:strict skills/flint/flint-sdk-ts`
6. Update `SKILLS.md` with flint SDK section

### Acceptance criteria

- [ ] 6 Flint SDK skills exist and pass `npm run validate:strict`
- [ ] Each skill has working installation instructions verified against SDK source
- [ ] `SKILLS.md` updated with flint section

---

## Execution order

Changes 001–005 are largely independent. Recommended order: 004 → 005 → 001 → 002 → 003 (tackle external integrations first, then new skills, then infra).

001 (kbd-evolve) can proceed in parallel with 004/005 if subagents are available.

---

## Phase exit criteria

- [x] All 5 changes complete and passing acceptance criteria
- [x] `npm run validate:strict` clean on all new/modified skills
- [x] `npm run validate` clean (no new errors)
- [x] smoke-test: all assertions pass
- [x] Committed to `main` with conventional commit messages
