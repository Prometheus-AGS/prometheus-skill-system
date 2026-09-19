# learn-grade eval ground truth — review sheet

Generated 2026-09-05 by change-rah-007 task 1 from `index.json` and `explanations/*.json`. Every item is `review_status: draft`. The operator reviews each row, corrects the ground truth in the item file when it disagrees, and records the verdict here and in `index.json` (`review_status: reviewed`, `reviewed_by`, `reviewed_at`).

Scores are `completeness / accuracy / clarity / misconceptions_absent` on 0 to 1. `Graded` is what learn-grade produced against the draft truth (from `results/`), shown only so a large disagreement is visible at a glance; it is not the thing being reviewed.

Verdict vocabulary: `agree` (truth stands), `corrected` (truth changed in the item file; say what), `disputed` (needs a second reviewer).

| # | Item | Domain | Tier | Corpus | Expected | Graded | Misconceptions expected | Verdict | Reviewer / date / note |
|---|---|---|---|---|---|---|---|---|---|
| 1 | `cr-001-strong-full-pathway` | cellular-respiration | strong | `cellular-respiration-corpus.json` | 0.95/1.00/0.88/1.00 | 0.75/1.00/0.95/1.00 | none |  |  |
| 2 | `cr-002-strong-fermentation` | cellular-respiration | strong | `cellular-respiration-corpus.json` | 0.85/1.00/0.90/1.00 | 0.85/1.00/0.95/1.00 | none |  |  |
| 3 | `cr-003-incomplete-vague-overview` | cellular-respiration | incomplete | `cellular-respiration-corpus.json` | 0.15/0.60/0.55/1.00 | 0.15/0.60/0.50/1.00 | none |  |  |
| 4 | `cr-004-incomplete-missing-etc-detail` | cellular-respiration | incomplete | `cellular-respiration-corpus.json` | 0.30/0.65/0.65/1.00 | 0.30/0.55/0.75/1.00 | none |  |  |
| 5 | `cr-005-flawed-mitochondria-create-energy` | cellular-respiration | factually-flawed | `cellular-respiration-corpus.json` | 0.40/0.20/0.82/0.00 | 0.15/0.15/0.85/0.00 | Common misconception: 'mitochondria make energy from scratch' |  |  |
| 6 | `cr-006-flawed-breathing-conflation` | cellular-respiration | factually-flawed | `cellular-respiration-corpus.json` | 0.35/0.15/0.85/0.00 | 0.10/0.10/0.75/0.00 | Common misconception: 'cellular respiration is the opposite of breathing' |  |  |
| 7 | `cr-007-flawed-glycolysis-location` | cellular-respiration | factually-flawed | `cellular-respiration-corpus.json` | 0.50/0.20/0.85/0.00 | 0.25/0.20/0.85/0.00 | Common misconception: 'glycolysis happens in the mitochondria' |  |  |
| 8 | `cr-008-flawed-plants-no-respiration` | cellular-respiration | factually-flawed | `cellular-respiration-corpus.json` | 0.40/0.10/0.80/0.00 | 0.10/0.10/0.60/0.00 | Common misconception: 'plants only do photosynthesis, not cellular respiration' |  |  |
| 9 | `kbd-001-strong-full-cycle` | kbd-lifecycle | strong | `kbd-lifecycle-corpus.json` | 0.95/1.00/0.90/1.00 | 0.75/0.95/0.45/1.00 | none |  |  |
| 10 | `kbd-002-strong-assess-plan-boundary` | kbd-lifecycle | strong | `kbd-lifecycle-corpus.json` | 0.85/1.00/0.92/1.00 | 0.85/1.00/0.95/1.00 | none |  |  |
| 11 | `kbd-003-incomplete-missing-openspec` | kbd-lifecycle | incomplete | `kbd-lifecycle-corpus.json` | 0.40/0.85/0.70/1.00 | 0.30/0.75/0.80/1.00 | none |  |  |
| 12 | `kbd-004-incomplete-vague-reflect` | kbd-lifecycle | incomplete | `kbd-lifecycle-corpus.json` | 0.25/0.80/0.60/1.00 | 0.25/0.75/0.70/1.00 | none |  |  |
| 13 | `kbd-005-flawed-progress-json` | kbd-lifecycle | factually-flawed | `kbd-lifecycle-corpus.json` | 0.60/0.35/0.75/0.00 | 0.35/0.30/0.70/0.00 | MISCONCEPTION: progress.json is updated automatically |  |  |
| 14 | `kbd-006-flawed-openspec-optional` | kbd-lifecycle | factually-flawed | `kbd-lifecycle-corpus.json` | 0.50/0.20/0.80/0.00 | 0.25/0.15/0.75/0.00 | MISCONCEPTION: OpenSpec is optional |  |  |
| 15 | `kbd-007-flawed-assess-writes-plan` | kbd-lifecycle | factually-flawed | `kbd-lifecycle-corpus.json` | 0.50/0.15/0.85/0.00 | 0.30/0.15/0.85/0.00 | MISCONCEPTION: kbd-assess writes the plan |  |  |
| 16 | `kbd-008-flawed-reflect-early` | kbd-lifecycle | factually-flawed | `kbd-lifecycle-corpus.json` | 0.45/0.10/0.82/0.00 | 0.20/0.10/0.85/0.00 | MISCONCEPTION: /kbd-reflect can run before all changes are DONE |  |  |
| 17 | `sp-001-strong-portability` | skill-pack | strong | `skill-pack-corpus.json` | 0.90/1.00/0.88/1.00 | 0.60/0.50/0.90/1.00 | none |  |  |
| 18 | `sp-002-strong-hooks-and-validate` | skill-pack | strong | `skill-pack-corpus.json` | 0.82/1.00/0.85/1.00 | 0.60/0.60/0.90/1.00 | none |  |  |
| 19 | `sp-003-incomplete-vague-mcp` | skill-pack | incomplete | `skill-pack-corpus.json` | 0.20/0.55/0.50/1.00 | 0.15/0.30/0.35/0.00 | none |  |  |
| 20 | `sp-004-incomplete-missing-reload` | skill-pack | incomplete | `skill-pack-corpus.json` | 0.35/0.75/0.70/1.00 | 0.30/0.80/0.85/1.00 | none |  |  |
| 21 | `sp-005-flawed-claude-only` | skill-pack | factually-flawed | `skill-pack-corpus.json` | 0.40/0.15/0.85/0.00 | 0.20/0.10/0.75/0.00 | MISCONCEPTION: skills only work on Claude Code |  |  |
| 22 | `sp-006-flawed-internet-required` | skill-pack | factually-flawed | `skill-pack-corpus.json` | 0.30/0.10/0.80/0.00 | 0.15/0.10/0.85/0.00 | MISCONCEPTION: the pack requires internet to function |  |  |
| 23 | `sp-007-flawed-restart-required` | skill-pack | factually-flawed | `skill-pack-corpus.json` | 0.35/0.10/0.85/0.00 | 0.15/0.10/0.85/0.00 | MISCONCEPTION: you have to restart Claude to use new skills |  |  |
| 24 | `sp-008-flawed-claude-plugin-source` | skill-pack | factually-flawed | `skill-pack-corpus.json` | 0.30/0.10/0.78/0.00 | 0.20/0.00/0.60/0.00 | MISCONCEPTION: .claude-plugin/ is the source of truth |  |  |

## Explanation texts

Read the text before judging the expected scores; the tier is the author's intent, the text is the evidence.

### 1. `cr-001-strong-full-pathway` (strong)

> Cellular respiration converts the chemical energy stored in glucose into ATP through four stages. Glycolysis happens first, out in the cytosol rather than the mitochondria, splitting glucose into two pyruvate molecules for a net gain of 2 ATP and 2 NADH — and it doesn't need oxygen to run. If oxygen is available, pyruvate gets oxidized and enters the citric acid cycle in the mitochondrial matrix, which runs twice per glucose (once per pyruvate) and produces 3 NADH, 1 FADH2, and 1 ATP each turn, releasing CO2 along the way. Finally, oxidative phosphorylation happens across the folded inner mitochondrial membrane: NADH and FADH2 feed electrons into the electron transport chain, which pumps protons to build a gradient, and ATP synthase uses that gradient via chemiosmosis to make most of the cell's ATP, with oxygen serving as the final electron acceptor that combines with electrons and protons to form water. Total yield is usually cited around 30-32 ATP per glucose under the modern accounting, a bit lower than the older 36-38 estimate because shuttling NADH's electrons across the mitochondrial membrane costs some efficiency.

### 2. `cr-002-strong-fermentation` (strong)

> When oxygen isn't available, cells can't run the electron transport chain, so instead they rely on fermentation to keep glycolysis going. Fermentation doesn't add any extra ATP beyond glycolysis's 2 ATP — its whole job is regenerating NAD+ from the NADH glycolysis produces, so glycolysis has NAD+ to keep using. In animal muscle cells, that means converting pyruvate to lactate (lactic acid fermentation); in yeast, it means converting pyruvate to ethanol and CO2 (alcoholic fermentation). Either way, it's directly downstream of glycolysis, using the same pyruvate glycolysis just made.

### 3. `cr-003-incomplete-vague-overview` (incomplete)

> Cellular respiration is how cells break down glucose to get energy. It happens in a few steps and needs oxygen. The mitochondria are involved and ATP comes out at the end.

### 4. `cr-004-incomplete-missing-etc-detail` (incomplete)

> Glucose goes through glycolysis, then the Krebs cycle, then the electron transport chain, and you end up with a bunch of ATP. Oxygen is needed somewhere near the end of the process.

### 5. `cr-005-flawed-mitochondria-create-energy` (factually-flawed)

> Mitochondria are the powerhouse of the cell because they generate the energy the cell needs from scratch — they take in glucose and oxygen and essentially manufacture new energy inside the mitochondrial matrix, which is then packaged as ATP for the rest of the cell to use.

### 6. `cr-006-flawed-breathing-conflation` (factually-flawed)

> Cellular respiration is basically just another word for breathing — when you inhale, that's respiration bringing oxygen into your body, and cellular respiration is the same process happening at a smaller scale inside each cell.

### 7. `cr-007-flawed-glycolysis-location` (factually-flawed)

> The first stage of cellular respiration, glycolysis, takes place inside the mitochondria alongside the Krebs cycle, where the glucose molecule is split and starts getting oxidized before moving on to the electron transport chain in the inner membrane.

### 8. `cr-008-flawed-plants-no-respiration` (factually-flawed)

> Plants get their energy through photosynthesis, converting sunlight directly into usable energy for the cell, so they don't really need cellular respiration the way animals do — that's more of an animal-specific pathway for breaking down food.

### 9. `kbd-001-strong-full-cycle` (strong)

> The KBD lifecycle moves through six stages in order: kbd-new-phase creates the phase directory and goals, kbd-assess inspects the codebase and writes assessment.md only — no code or plans — kbd-analyze is optional and researches libraries when needed, kbd-plan reads the assessment and writes plan.md plus OpenSpec change proposals but never touches actual skill files or code, kbd-execute (via kbd-apply) drives one task at a time through the chosen backend and updates progress.json explicitly after every change since nothing updates it automatically, and finally kbd-reflect writes the closing reflection but only once every single change shows status DONE in progress.json. On every turn of every kbd-* skill, the very first action is reading position-reminder.txt so the position signal stays accurate.

### 10. `kbd-002-strong-assess-plan-boundary` (strong)

> kbd-assess and kbd-plan have a hard boundary between them. kbd-assess only produces assessment.md — a gap report — and explicitly does not write plan.md or any skill/implementation files. It stops at that boundary for review. kbd-plan is the one that reads the assessment and produces the ordered change list (plan.md) plus OpenSpec proposals, but even kbd-plan doesn't write actual code — that's kbd-execute's job, driven task-by-task via kbd-apply.

### 11. `kbd-003-incomplete-missing-openspec` (incomplete)

> kbd-plan looks at the assessment and figures out what changes need to happen, then writes them down in plan.md in order. Then kbd-execute goes through and does each change one at a time, updating the waypoint as it goes.

### 12. `kbd-004-incomplete-vague-reflect` (incomplete)

> kbd-new-phase starts things off with a goals file. Then you assess, plan, and execute the changes. At the end you reflect on how it went and figure out what to do next.

### 13. `kbd-005-flawed-progress-json` (factually-flawed)

> The KBD lifecycle is assess, plan, execute, reflect. As each change in the plan gets implemented, the system automatically detects the completed files and updates progress.json in the background, so the agent doesn't need to touch it directly. Once all changes are marked done this way, kbd-reflect runs and closes out the phase.

### 14. `kbd-006-flawed-openspec-optional` (factually-flawed)

> OpenSpec is one option among several for tracking changes in a KBD phase — you can use it if you like the structured proposal format, or just skip it and use plain native KBD change files, it's really up to the developer's preference on a given project regardless of what's already in the repo.

### 15. `kbd-007-flawed-assess-writes-plan` (factually-flawed)

> When you run kbd-assess, it looks at the codebase, figures out the gaps, and then goes ahead and writes out the ordered plan of changes needed to close those gaps — that becomes plan.md, ready for kbd-execute to pick up.

### 16. `kbd-008-flawed-reflect-early` (factually-flawed)

> You can run kbd-reflect any time you want a snapshot of how the phase is going so far — it's a good habit to reflect partway through a long phase, even with several changes still pending, so you can course-correct early instead of waiting until everything is done.

### 17. `sp-001-strong-portability` (strong)

> Skills in this pack follow the agentskills.io spec, which makes them portable across every supported platform — Claude Code, OpenCode, Codex, Kimi, Zed, Cursor, and MiniMax — not just Claude Code. The pack's core skills run entirely offline; the only pieces needing internet are the two local MCP servers, surreal-memory and sycophancy-correction, and even those only reach out when a skill's instructions explicitly call for external search. skills/ is the single source of truth on disk — .claude-plugin/ only holds symlinks into it, so you always edit under skills/, never through the plugin directory.

### 18. `sp-002-strong-hooks-and-validate` (strong)

> hooks/hooks.json at the project root is the canonical, physical source for hook definitions — .claude-plugin/hooks is only a directory symlink pointing at ../hooks, so editing it directly is a no-op trap; always edit hooks/hooks.json instead. For validation, plain npm run validate is lenient and skips the imported/ submodule directory, but any newly authored skill must pass the stricter npm run validate:strict, which enforces version, license, and metadata.tags fields before it's considered ready.

### 19. `sp-003-incomplete-vague-mcp` (incomplete)

> The skill pack has some MCP servers that help with memory and things. There's surreal-memory for one purpose and Cortex for something similar. You install everything with a script and then the skills show up.

### 20. `sp-004-incomplete-missing-reload` (incomplete)

> Skills are files under skills/ organized by category. Each has a SKILL.md with instructions. The pack can be installed to different AI tools and each one picks up the skills from its own directory.

### 21. `sp-005-flawed-claude-only` (factually-flawed)

> These skills are built specifically for Claude Code's plugin system, so they only really function there — other AI coding tools like Codex or Cursor would need their own separately-written skill files even if the underlying logic is the same, since the format itself is Claude-specific.

### 22. `sp-006-flawed-internet-required` (factually-flawed)

> Since the pack relies on MCP servers and the underlying model calls, you basically need an internet connection at all times for any of the core skills to function properly — there's no meaningful offline mode.

### 23. `sp-007-flawed-restart-required` (factually-flawed)

> After you drop a new skill into the directory, you have to fully quit and restart Claude Code before it will show up and be usable — there's no lighter-weight way to refresh the skill list.

### 24. `sp-008-flawed-claude-plugin-source` (factually-flawed)

> If you need to update a skill's hooks or its symlinked structure, the right place to make that edit is directly inside .claude-plugin/ — that's the canonical plugin layout Claude Code actually reads from at runtime, and skills/ is more of a staging area.
