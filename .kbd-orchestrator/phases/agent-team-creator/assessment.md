# Assessment: agent-team-creator
Date: 2026-09-24
Status: assessment draft for independent review; no implementation approved or complete.

## Scope and user requirements
Create and manage coding-agent teams in a project or at UAR/BossFang scope. Targets: UAR, Codex, Claude Code, GitHub Copilot, Kimi Code, MiniMax's own mcode CLI, OpenCode, and DeepSeek Harness. Include a novice task-to-role guide, skill assignment, hard/medium/low model selection using capability and cost information, handoffs through Karpathy logs and shared memory, native agent/subagent/plugin/marketplace surfaces where supported, complete configuration preservation, documentation and both Docusaurus sites. New scripts/hooks must be Node.js and TypeScript 7. Identical new runtime in full and mini; edits only inside the two authorized repositories. User additionally requires every KBD stage and correction of observed workflow inconsistencies.

## Cross-tool progress and process correction
The phase has zero registered changes and zero completed implementation changes. Assess was entered through the typed CLI. Two early OpenSpec commits are draft work, not approved Spec/Plan artifacts. The initial type scaffold was withdrawn and adapter implementation delegation stopped after the user corrected the process. The earlier harness research is evidence available for Analyze, not proof that Analyze or its adversarial review completed.
Current phase state, rather than project-wide counters, determines this phase's progress. Other worktrees have ongoing work; their changes must not be restored or published by this phase.

## Implementation completeness
| Area | Status | Evidence / gap |
| --- | --- | --- |
| Native skill inventory and packaging | PARTIAL | Existing inventory and Claude/Codex distributions provide reusable packaging. They do not constitute native team support in eight harnesses. |
| KBD lifecycle and logging | PARTIAL | Canonical CLI, stage gates and Karpathy receipts exist. Stage documentation and runtime projections have the conflicts below. |
| Guided task-to-team creation | MISSING | No agent-team-creator skill or runtime exists in the inspected inventory. Novices need outcome-oriented questions, reasons for roles, ownership and a one-agent alternative. |
| Portable team/task contract | MISSING | Existing KBD phase/change/task state is not a general-purpose harness team manifest. Avoid adding a second owner for the same KBD work item. |
| Cross-harness handoff | PARTIAL | Existing KBD handoff schemas and boundary receipts carry process context; portable agent identity, task transfer, artifact provenance, ownership acceptance and destination-native configuration remain undefined. |
| Model routing | PARTIAL | liter-llm/KBD model roles exist. A selectable team/role/skill/task policy with discovered models, actual supported capabilities and known/unknown cost is absent. |
| Shared memory | PARTIAL | KBD event mirroring, recall and local receipts exist. A team handoff is not automatically a canonical KBD boundary event or permission grant. Other memory providers require verified contracts. |
| Eight-harness exports | MISSING | Native schemas differ. Preserving opaque options is necessary but is not proof that a native runtime supports them. No complete adapter certification exists. |
| Documentation / sites | PARTIAL | Both sites and catalogs exist; team workflows and configuration coverage documentation are missing. |
| Portable TypeScript 7 runtime | MISSING | Existing runtime and compiler baselines differ. New runtime needs a reproducible pinned build and shipped artifacts, with no dependency on full-pack-only tools. |

## Observed conflicts requiring reconciliation before team implementation
1. **Stage order was bypassed in this session.** Draft Spec/Plan files preceded Assess/Analyze and independent review. Corrective evidence: process-correction.md. Revisit drafts only in their proper stages; do not backdate handoffs or claim prior reviews.
2. **Runtime phase creation bypasses compatibility metadata and lifecycle hooks.** The runtime-authority branch of mini scripts/kbd-new-phase.mjs returns before flipProjectActivePhase and hooksFire; the corresponding full helper also exits early. Mini project.json named an earlier phase; full worktree had no project.json. Both now match the canonical phase via the authorized phase-creation postcondition repair (only activePhase changed in mini; missing full local identity derived from package.json). The phase:before hook was fired manually exactly once for this phase; do not repeat it during repair. The helper's success banner does not prove all documented postconditions.
3. **Phase completion display borrows run-wide evidence.** Full substrate/kbd-runtime/src/lib.rs phase projection constructs local implementation counts but reads evidence/certification/publication from state.completion. Therefore a fresh phase displays old publication and certification summaries. The canonical run's history is real and must be preserved; the incorrect attribution is in projection scope, not evidence to erase. Mini uses the installed shared runtime and is affected by the same contract. Never repair this by hand-editing generated progress.json.
4. **Stage reminder labels lifecycle as stage.** The reminder says Stage: ready while canonical activePath.stageId is assess and the assess stage is in_progress. Clarify lifecycle versus stage in the producer and consuming guidance.
5. **Legacy instructions prescribe editing generated state.** Full KBD prompts/assess.md asks for assessment_complete writes and references/cross-tool-protocol.md prescribes direct progress/waypoint edits, conflicting with the orchestrator's typed-command-only contract. Readers also conflate project-wide and phase-local completion. Reconcile source instructions and generated copies through their owners.
6. **Verification policy conflicts.** Full .kbd-orchestrator/constraints.md still describes documentation/file-count QA exemptions; current kbd-execute and adversarial-review explicitly remove them. Mini openspec/config.yaml still says tests first and hosted CI, while AGENTS.md A-9 requires testing completed behavior at boundaries. Mini constraints tests-for-new-features demands red/green module tests. The stricter current implementation-first/local-integration policy governs this work; align stale source text without weakening checks.
7. **Stage-gate documentation contradicts implementation.** Mini stage-gate.mjs creates the handoff directory and fails on a missing required predecessor; carried skill text still says missing directories warn and pass. Document actual behavior and require all six stages for this phase, including Analyze and Spec even though generic legacy gates may treat them as optional.
8. **Default repository scope conflicts with this explicit task.** Mini's generic reference-only full-pack rule is valid for ordinary mini-only work; this user explicitly authorizes both repositories. Record a scoped phase exception instead of broadening the global rule or changing unrelated workspace authorization.
9. **Configuration coverage is not yet defined.** 'Every option' cannot be certified by a small hand-written common schema. Analyze must identify pinned native schemas/config inventories, opaque preservation, conflict diagnostics, supported/unsupported features and installed-version validation for all eight. MiniMax global agent discovery and DeepSeek experimental teams need explicit treatment.
10. **Cross-platform proof remains pending.** Local machine is macOS with Node 26; the package contract targets Node LTS >=22 and Windows. Do not label local macOS checks as Windows or Node22 certification. Use actual local runtimes where available and record remaining evidence limits.

## Spec alignment
The current agent-team-management OpenSpec delta is a premature draft, not the baseline. It lacks complete native option matrices, cancellation/reassignment semantics, authority boundaries, multi-repository ownership and offline recovery acceptance cases. Existing platform and packaging specs constrain the implementation. Do not silently change protected tests or legacy behavior to make a new implementation pass. Each conflict correction needs traceable requirements and integration evidence before implementation.
Karpathy boundary receipts validate canonical KBD identity. Generic team messages must not be forged as successful KBD task boundaries. The pk CLI remains sole writer of .prometheus/knowledge; local team records must use a separate owned location or supported ingestion.

## Build health and coverage
- Build: UNKNOWN for this phase; no team production implementation exists.
- Tests: no new phase tests authored or run. This follows implementation-first policy; it is not a passing result.
- Existing test infrastructure exists but is not evidence for new behavior.
- Model preflight: existing Node helper reports gateway http://localhost:4000/v1, judge gpt-5.5, critic MiniMax-M3, status ok. Producer is GPT-6; actual independent review is still required.
- Prior source research and file inspection are static evidence, not native-harness execution tests.

## Goal progress
All delivered team capabilities are NOT MET; existing packaging, routing and logging are PARTIAL foundations. Process correction is IN PROGRESS. No final validation, publication or release is claimed.

## Questions for Analyze
Which existing modules can be adapted into an identical standalone TS7 runtime without repo-root imports? Which native features need per-version negotiation? How can KBD-linked team tasks share canonical ownership while non-KBD tasks remain usable? What real integration paths can verify each adapter without installing global artifacts or spending on live agent work? How should role guidance distinguish tools, skills, models and execution ownership for novices?

## Assessment limits
This is a source-based assessment of the two isolated worktrees and a narrow read-only check of related UAR discovery code. It does not establish deployed UAR/BossFang team API compatibility or certify live harness behavior. The prior architecture report is context, not current implementation evidence.

## Full-pack evidence
Baseline e57421a86fae08ac4d1e3a625a213cf3f3eb98bc; branch codex/agent-team-creator.
- skill-system.json and scripts/lib/skill-system.js: recursive skill inventory and platform manifests.
- skills/process/.claude-plugin/plugin.json: explicit process-plugin skill roster.
- skills/process/liter-llm-bridge/SKILL.md: current REST gateway/model role integration; no liter-llm complete command.
- skills/process/karpathy-progress-memory/SKILL.md: canonical boundary recorder, local log, pk ingestion.
- openspec/specs/harness-completeness-gate/spec.md: target lifecycle and idempotency requirements; newer lifecycle distinction takes precedence over the older blanket tree-presence language.
- openspec/specs/kbd-memory-integration/spec.md: lifecycle mirroring and recall contracts.
- site/scripts/generate-skills-catalog.mjs and site/sidebars-guide.js: generated catalog and guide discovery.
- AGENTS.md: local validation, complete implementation first, integration-only evidence, protected-test integrity.

## Auditable cross-repository evidence
The review packet file tree is rooted in one repository and depth/size-limited. The following are exact excerpts from separately inspected roots; absence from that tree is not evidence of absence.

### mini: /Users/gqadonis/Projects/prometheus/prometheus-skills-mini/.worktrees/agent-team-creator/scripts/kbd-new-phase.mjs

```text
48:   const createResult = spawnExecutable('prometheus', [
49:     'kbd', '--path', '.', 'phase', 'create',
50:     '--command-id', `phase-create:${name}`,
51:     '--id', name, '--title', name,
52:   ]);
53:   if (createResult?.status !== 0) die(`phase create failed: ${createResult?.stderr ?? ''}`);
54: 
55:   if (guardEnabled) {
56:     const pre = evaluateBottleneck('phase', 'before', name, true, { root: '.' });
57:     if (pre.status !== 0) die('phase start precommit evaluation blocked');
58:   }
59:   const activateResult = spawnExecutable('prometheus', [
60:     'kbd', '--path', '.', 'phase', 'activate',
61:     '--command-id', `phase-activate:${name}`,
62:     '--id', name, '--exact-next-work', `/kbd-assess ${name}`,
63:   ]);
64:   if (activateResult?.status !== 0) die(`phase activate failed: ${activateResult?.stderr ?? ''}`);
65:   const transitionResult = spawnExecutable('prometheus', [
66:     'kbd', '--path', '.', 'phase', 'transition',
67:     '--command-id', `phase-start:${name}`,
68:     '--id', name, '--status', 'in-progress',
69:   ]);
70:   if (transitionResult?.status !== 0) die(`phase transition failed: ${transitionResult?.stderr ?? ''}`);
71: 
72:   if (guardEnabled) {
73:     const post = evaluateBottleneck('phase', 'before', name, false, { root: '.' });
74:     if (post.status !== 0) die('phase start postcommit evaluation blocked');
75:     process.stdout.write(`${bottleneckSignalText(post.stdout)}\n`);
76:   }
77: 
78:   const phaseDir = path.join('.kbd-orchestrator', 'phases', name);
79:   process.stdout.write(`\nCompleted kbd-new-phase — ${name} ready for /kbd-assess\n`);
80:   process.stdout.write(`  phase:  ${name}\n`);
81:   process.stdout.write(`  goals:  ${path.join(phaseDir, 'goals.md')}\n`);
82:   process.stdout.write(`  Next:   /kbd-assess ${name}\n`);
83: }
84: 
85: function writeGoals(phaseDir, goals) {
86:   const lines =
87:     goals.length > 0
88:       ? goals.map((g) => `- ${g}`)
89:       : ['<!-- TBD: enumerate goals before /kbd-assess -->'];
90:   atomicWrite(path.join(phaseDir, 'goals.md'), `# Goals\n\n${lines.join('\n')}\n`);
91: }
92: 
93: function writeProgress(phaseDir, name, sourceTool, now) {
94:   const progress = {
95:     schemaVersion: '2',
96:     phase: name,
```

### mini: /Users/gqadonis/Projects/prometheus/prometheus-skills-mini/.worktrees/agent-team-creator/scripts/kbd-new-phase.mjs

```text
237:   flipProjectActivePhase(name, now);
238: 
239:   const orchestratorRoot = process.env.KBD_ORCHESTRATOR_ROOT ?? '.';
240:   try {
241:     await hooksFire('phase', 'before', name, 1, 1, {
242:       orchestratorRoot,
243:       cwd: '.',
244:       runCommand: runHookCommand,
245:       phasePath: name,
246:       sourceTool: 'kbd-new-phase',
247:     });
248:   } catch {
249:     warn('phase:before hook fire failed (phase still created)');
250:   }
251: 
252:   process.stdout.write(`\nCompleted kbd-new-phase — ${name} ready for /kbd-assess\n`);
253:   process.stdout.write(`  phase:  ${name}\n`);
254:   process.stdout.write(`  goals:  ${path.join(phaseDir, 'goals.md')}\n`);
255:   process.stdout.write(`  Next:   /kbd-assess ${name}\n`);
256: }
257: 
258: main(process.argv.slice(2)).catch((error) => die(error?.message ?? String(error)));
259: 
```

### full: /Users/gqadonis/Projects/prometheus/prometheus-skill-pack/.worktrees/agent-team-creator/skills/process/kbd-process-orchestrator/skills/kbd-new-phase/kbd-new-phase.sh

```text
104:     kbd_bottleneck_evaluate phase before "$name" 1 >/dev/null \
105:       || die "phase start precommit evaluation blocked"
106:   fi
107:   prometheus kbd --path . phase activate \
108:     --command-id "phase-activate:${name}" \
109:     --id "$name" --exact-next-work "/kbd-assess $name" >/dev/null
110:   prometheus kbd --path . phase transition \
111:     --command-id "phase-start:${name}" \
112:     --id "$name" --status in-progress >/dev/null
113:   if [[ "$guard_enabled" == true ]]; then
114:     guard_output="$(kbd_bottleneck_evaluate phase before "$name" 0)" \
115:       || die "phase start postcommit evaluation blocked"
116:     kbd_bottleneck_print_signal "$guard_output"
117:   fi
118:   printf '\nCompleted kbd-new-phase — %s ready for /kbd-assess\n' "$name"
119:   printf '  phase:  %s\n' "$name"
120:   printf '  goals:  %s\n' "$phase_dir/goals.md"
121:   printf '  Next:   /kbd-assess %s\n' "$name"
122:   exit 0
123: fi
124: 
125: # ---------- 2. progress.json ----------
126: source_tool=""
127: if [[ -f "$wp" ]]; then
128:   source_tool="$(jq -r '.sourceTool // ""' "$wp" 2>/dev/null || true)"
129: fi
130: [[ -n "$source_tool" ]] || source_tool="unknown"
```

### full: /Users/gqadonis/Projects/prometheus/prometheus-skill-pack/.worktrees/agent-team-creator/skills/process/kbd-process-orchestrator/prompts/assess.md

```text
150: Write output to `.kbd-orchestrator/phases/<phase-name>/assessment.md`.
151: 
152: After writing, update `.kbd-orchestrator/phases/<phase>/progress.json`:
153: 
154: - Set `assessment_complete: true`
155: - Set `last_updated_by` to the executing tool name
156: - Set `last_updated` to current ISO timestamp
157: 
```

### full: /Users/gqadonis/Projects/prometheus/prometheus-skill-pack/.worktrees/agent-team-creator/skills/process/kbd-process-orchestrator/references/cross-tool-protocol.md

```text
18: KBD change, it MUST follow this protocol:
19: 
20: ### On Start of a Change
21: 
22: 1. Read `.kbd-orchestrator/current-waypoint.json`
23: 2. Read the change spec (OpenSpec or `.kbd-orchestrator/changes/<id>/change.md`)
24: 3. Update `progress.json`: set status → `IN_PROGRESS`, `started_by` → `<tool-name>`
25: 4. Update waypoint: `last_updated_by` → `<tool-name>`
26: 
27: ### During Execution (on each task completion)
28: 
29: 1. Update `progress.json`: increment `tasks_done`, update `last_task_completed` and `next_task_pending`
30: 2. Commit the progress file to git: `git add .kbd-orchestrator && git commit -m "kbd: progress update [<tool>] <change-id> task N/M"`
31: 
```

### full: /Users/gqadonis/Projects/prometheus/prometheus-skill-pack/.worktrees/agent-team-creator/substrate/kbd-runtime/src/lib.rs

```text
6124:         blockers: Vec::new(),
6125:     };
6126:     let dimension = |name: CompletionDimension| {
6127:         state
6128:             .completion
6129:             .get(&name)
6130:             .cloned()
6131:             .unwrap_or_else(Completion::not_tracked)
6132:     };
```

### full: /Users/gqadonis/Projects/prometheus/prometheus-skill-pack/.worktrees/agent-team-creator/substrate/kbd-runtime/src/lib.rs

```text
6170:         "implementation_completed": completed,
6171:         "completion": {
6172:             "primaryCounter": "implementation",
6173:             "implementation": count_completion_projection(&implementation),
6174:             "evidence": status_completion_projection(&dimension(CompletionDimension::Evidence)),
6175:             "certification": status_completion_projection(&dimension(CompletionDimension::Certification)),
6176:             "publication": status_completion_projection(&dimension(CompletionDimension::Publication))
6177:         },
6178:         "changes": changes,
6179:         "children": children,
6180:         "migrationStatus": if phase.legacy_read_only {
```

### mini: /Users/gqadonis/Projects/prometheus/prometheus-skills-mini/.worktrees/agent-team-creator/openspec/config.yaml

```text
108:   partition by responsibility, never by line count), immutable data
109:   patterns, explicit error handling, node:test with AAA structure, tests
110:   written first, CI on windows-latest + ubuntu-latest + macos-latest,
111:   conventional commits.
112: 
```

### mini: /Users/gqadonis/Projects/prometheus/prometheus-skills-mini/.worktrees/agent-team-creator/.kbd-orchestrator/constraints.md

```text
158:   - id: tests-for-new-features
159:     severity: warning
160:     source: 'openspec/config.yaml conventions — tests written first'
161:     description: 'Every new module under lib/ has a node:test file that was seen to fail before it passed'
162: 
163:   - id: no-forbidden-tools-in-instructions
```

### full: /Users/gqadonis/Projects/prometheus/prometheus-skill-pack/.worktrees/agent-team-creator/.kbd-orchestrator/constraints.md

```text
56: ## C-05 — Scripts under launchd are bash 3.2 compatible
57: 
58: Any script that can be invoked by a launchd agent (macOS `/bin/bash` is 3.2) must
59: avoid `mapfile` / `declare -A`. Test with `/bin/bash script.sh`, not just `bash`.
60: 
61: ## When QA is skipped
62: 
63: Per `/kbd-execute`: changes with fewer than 3 files modified, documentation-only
64: changes, or `--skip-qa`. Skips are logged, not silent. When the
65: `sycophancy-correction`/artifact-refiner binary is absent, the gate logs the skip
```

- full documentation site: `/Users/gqadonis/Projects/prometheus/prometheus-skill-pack/.worktrees/agent-team-creator/site`; package `prometheus-skill-pack-docs`; scripts: `{"start":"docusaurus start","build":"docusaurus build","build:deploy":"npm run generate:catalog && docusaurus build","swizzle":"docusaurus swizzle","deploy":"docusaurus deploy","clear":"docusaurus clear","serve":"docusaurus serve","write-translations":"docusaurus write-translations","write-heading-ids":"docusaurus write-heading-ids","generate:catalog":"node scripts/generate-skills-catalog.mjs","generate:api-examples":"node scripts/generate-api-examples.mjs","check:public-docs":"node scripts/check-public-docs.mjs","check:openapi":"node scripts/check-openapi.mjs","check:sovereign-openapi":"node scripts/check-sovereign-openapi.mjs","check:exec-openapi":"node scripts/check-exec-openapi.mjs","check:exec-examples":"npm --prefix .. run check:docs-exec-examples","check:api-examples":"node scripts/generate-api-examples.mjs --check","check:mermaid":"node scripts/check-mermaid.mjs","check:docs-contracts":"node scripts/check-doc-contracts.mjs","docs:check":"npm --prefix .. run test:docs-sync && npm --prefix .. run check:workflow-policy && npm --prefix .. run test:workflow-policy && npm run check:public-docs && npm run check:openapi && npm run check:sovereign-openapi && npm run check:exec-openapi && npm run check:exec-examples && npm run check:api-examples && npm run check:mermaid && npm run check:docs-contracts && npm run build","prebuild":"node scripts/check-public-docs.mjs && node scripts/check-openapi.mjs && node scripts/check-exec-openapi.mjs && node scripts/generate-api-examples.mjs --check && node scripts/check-doc-contracts.mjs && node scripts/generate-skills-catalog.mjs","prestart":"node scripts/check-public-docs.mjs && node scripts/generate-skills-catalog.mjs"}`.

- mini documentation site: `/Users/gqadonis/Projects/prometheus/prometheus-skills-mini/.worktrees/agent-team-creator/site`; package `prometheus-skills-mini-docs`; scripts: `{"start":"docusaurus start","build":"docusaurus build","build:deploy":"npm run generate:catalog && docusaurus build","generate:catalog":"node scripts/generate-skills-catalog.mjs","clear":"docusaurus clear","serve":"docusaurus serve"}`.

## Publication and explicit authorization evidence
The operator explicitly requested writes to both named skill-package directories and, after completion, commit and push. See operator-authorization.md for the exact scoped instruction and continued exclusion of personal/team log pushes. Existing generic reference-only defaults do not supersede this instruction.

Current origin: `git@github.com:Prometheus-AGS/prometheus-skill-system.git`; current committed HEAD: `428591586b006d7a3bd465d0297501cb4c6edbb3`. Both repositories have commits and an origin remote. Historic no-commits/no-remote prose is stale and must be corrected. Publication is NOT MET until applicable KBD/local gates pass; no push has occurred for this phase. Remote authentication for publication has not yet been exercised by pushing this branch.

## Unresolved review findings
Artifact review reached its two-round limit. The following findings are carried verbatim into Analyze; this is not a PASS verdict. Subsequent clarification was added after the second packet snapshot and is not claimed as reviewed.

### CRITICAL
The assessment contradicts the packet's authoritative constraints by treating QA skip exemptions as stale text that current policy removed.

Evidence: Assessment conflict #6 says: "Full .kbd-orchestrator/constraints.md still describes documentation/file-count QA exemptions; current kbd-execute and adversarial-review explicitly remove them." The packet constraints explicitly include those exemptions: "## When QA is skipped ... changes with fewer than 3 files modified, documentation-only changes, or `--skip-qa`. Skips are logged, not silent."

Suggested fix: Revise the conflict to match the supplied constraints, or include concrete packet evidence that supersedes them; do not direct downstream work to remove or weaken the documented QA-skip behavior based on an unsupported claim.

### WARNING
The assessment misses a goal-level gap analysis for committing and pushing isolated branches.

Evidence: Goals require: "Validate locally, commit and push isolated branches; modify only the two skill repositories." The assessment covers validation as unknown and repository scope in conflict #8, but it contains no corresponding finding or gap for commit/push readiness, branch isolation, publication sequencing, or evidence required before push.

Suggested fix: Add an explicit assessment item for branch isolation and commit/push readiness, including what evidence is missing before isolated branches may be committed and pushed.

## Disposition evidence for Analyze
- operator-authorization.md quotes the operator scope and push instructions; it supersedes conflicting generic defaults for this phase. Publication evidence now identifies actual remotes and commits.
- Full current skills/process/kbd-process-orchestrator/skills/kbd-execute/SKILL.md, Local review coverage: “File-count and documentation-only skips do not exist.” Its adversarial-review integration requires review before archive. Full AGENTS.md requires local final certification from committed state. The older constraints paragraph disagrees; the operator explicitly directed clearing these conflicts.
- Mini rules/src/constitution.md A-9 requires complete behavior before integration testing; openspec/config.yaml and the red/green warning disagree. Resolve by aligning stale derived context to the governing constitution, not by lowering required evidence.
