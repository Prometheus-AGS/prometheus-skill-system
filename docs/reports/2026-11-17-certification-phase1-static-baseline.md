# Certification Phase 1: Static Baseline

**Date:** 2026-11-17
**Plan:** [`docs/plans/2026-11-17-skill-pack-certification-plan.md`](../plans/2026-11-17-skill-pack-certification-plan.md)
**Scope:** read-only. Four tools were run across `skills/`. Nothing was fixed and no skill was edited.

## Result

**The pack passes its own validator and fails the official one.** `npm run validate:strict` exits 0. The agentskills.io reference validator passes 16 of 198 native skills. The cause is the pack's own frontmatter convention, not individual mistakes: the skill template and the strict validator both steer authors towards two constructs the specification does not allow.

No skill reaches level C0 under the plan's rule (both validators must pass) until that convention is settled.

## What was run

| Tool | Version | Command | Exit |
|---|---|---|---|
| Repo strict validator | in-repo | `npm run validate:strict` | 0 |
| Repo standard validator | in-repo | `npm run validate` | 0 |
| `skills-ref` (official) | 0.1.1 | `uvx --from skills-ref agentskills validate <dir>` per skill directory | 228 non-zero of 408 |
| `agnix` | 0.61.0 | `npx agnix@0.61.0 --format json skills` | 1 |
| Cisco `skill-scanner` | 2.3.2 | `skill-scanner scan-all skills --recursive --format json` | 0 |

The scanner ran with its local static analysers only. The LLM, VirusTotal, AI Defense and OSV analysers were left off, so no skill content left the machine.

Tools named in the plan and **not** run: `skills-lint`, `skill-validator`, `avocetta`, `skillcheck`. The `skills-ref` binary already on this machine (`~/.pyenv/shims/skills-ref`) is broken (`ModuleNotFoundError`); the run used a fresh copy through `uvx`.

## Inventory

| Count | Value |
|---|---|
| `SKILL.md` files under `skills/` | 408 |
| Native (outside `skills/imported/`) | 198 |
| Imported | 210 |
| Skills the repo validator reports checking | 121 "top-level" |

Two things to reconcile:

- `CLAUDE.md` states 310 skills (249 manifest-only, 61 script-bearing). This run found 198 native `SKILL.md` files. The difference was not investigated.
- The repo validator reports 121 top-level skills against 198 native files. Whether it also checks the 77 nested ones was not verified. If it does not, they have no gate at all.

Most imported files are harness mirrors: `prometheus-entity-management` carries the same skills under `.claude/`, `.codex/` and `.cursor/`.

## Finding 1: frontmatter convention conflicts with the specification

| Check | Native pass | Native fail |
|---|---|---|
| `skills-ref validate` | 16 | 182 |

Two causes account for the native failures.

**a. `tags` written as a list.** 147 native skills write `tags: [a, b, c]` under `metadata`. `skills-ref` rejects this at parse time (175 failures across the whole tree, all with the same message about a flow mapping). `agnix` reports the underlying rule separately as AS-016 on 33 native skills: *metadata must be a map from string keys to string values*. A list is not a string, so this is a real specification conflict and not only a strict parser.

**b. Top-level `version`.** 180 native skills declare `version:` at the top level. The specification allows six top-level fields and `version` is not one of them. `skills-ref` reports it on the skills that get past parsing (31, plus 10 more that also carry `authors`, `triggers`, `tools` or `disable-model-invocation`). `agnix` reports it on 170 native skills (CC-SK-017) and 54 (XP-SK-001).

Failure (a) hides failure (b): `skills-ref` stops at the parse error, so most of the 147 skills would report the `version` problem next.

**Where the convention comes from:**

- `docs/SKILL_TEMPLATE.md` line 9 has `tags: [tag1, tag2, tag3]`.
- `scripts/validate-skills.js` line 144 accepts a top-level `version`.
- `CLAUDE.md` lists `version` and `metadata.tags` (a non-empty array) as required elements under "AgentSkills.io Compliance".

So the repository's definition of compliance and the specification's disagree. This is a decision for the owner, not a clean-up task: either move both values into string-valued `metadata` keys (for example `metadata.version: '1.0.0'` and `metadata.tags: "a, b, c"`), or keep the convention and record it as a declared deviation. The first option also removes the DeepSeek Harness hazard noted in the plan, where nested YAML is skipped.

Also from `skills-ref`: 12 directory-name mismatches, all in the imported `pmpo-skill-creator` mirrors (`clone` versus `clone-skill` and three similar).

## Finding 2: `agnix` lint

1,063 diagnostics over 2,663 files: 173 errors, 444 warnings, 446 info. Native share: 75 errors, 242 warnings, 222 info.

Native error-level rules:

| Rule | Count | Meaning | Assessment |
|---|---|---|---|
| AS-016 | 33 | Metadata is not string-to-string | Finding 1 |
| CC-SK-008 | 28 | Unknown tool in `allowed-tools` (`file_system`, `code_interpreter`, `sequential_thinking`, `web_search`, `memory`, `tavily`, `browser`, `firecrawl`) | Likely real: these are not Claude Code tool names, so the restriction does nothing there |
| REF-002 | 11 | Broken markdown link | Mixed, see below |
| CC-HK-012 | 3 | `hooks/hooks.json` is a list where a map is expected | Needs a look: `kbd-process-orchestrator`, `pmpo-skill-creator`, `entity-graph-setup` |

Broken links (REF-002):

- `kbd-goal-check/SKILL.md:159` → `../../agents/kbd-goal-evaluator.md`
- `kbd-apply/SKILL.md:167,169` → `config/spec-engines.json`, `docs/guide/spec-engines.md`
- `prometheus-entity-skills/SKILL.md:100,101` → absolute `/_shared/references/...`
- `librefang-wasm-skill/SKILL.md:176,177` → `templates/lib.rs.tera`, `templates/host.rs.tera`
- `vercel-react-best-practices/AGENTS.md` → three missing rule files
- `adversarial-review/tests/fixtures/flawed-skill` → intentional; it is a negative fixture

The `kbd-apply` links climb out of the skill directory into the repository. They may resolve in this checkout and still break once the skill is installed elsewhere. Not verified.

Notable warnings and info:

| Rule | Count | Meaning |
|---|---|---|
| AS-020 | 105 | Description says what the skill does but not when to use it |
| AS-023 | 25 | Bundled file never mentioned in `SKILL.md` |
| AS-013 | 17 | Reference files chained more than one level deep |
| CC-HK-030 | 16 | `${CLAUDE_PLUGIN_ROOT}` unquoted in a shell-form hook, so a path with spaces splits |
| CC-AG-019 | 14 | Agent file uses `allowed_tools`, which Claude Code ignores |
| CC-SK-031 | 4 | Description contains an XML-like tag (`<phase>`, `<parent>`, `<name>`, `<module>`); claude.ai and the API reject such skills on upload |
| CC-SK-021 | 8 | Hardcoded `/Users/...` path. These are `/Users/operator` and `/Users/jane` in examples, so probably false positives |
| CDX-AG-009 | 2 | An `AGENTS.md` of 108 KB; Codex reads the first 32 KB by default |
| AS-012 / AS-021 | 1 / 2 | `SKILL.md` over 500 lines (one at 1,215) or over about 5,000 tokens |

AS-020 agrees with the repo validator's own warnings: about 150 "no trigger clause" and 186 "no exclusion clause" (the validator prints each twice; these are halved counts). Both tools point at the same weakness, which is the one level C2 will measure.

## Finding 3: security scan

778 findings over 408 skills; 376 skills reported safe. Native: 57 critical, 13 high, 76 medium, 181 low.

**No finding examined so far is malicious.** The critical and high findings are pattern matches on legitimate code. Only the ones listed below were opened; the 257 native medium and low findings were not reviewed.

| Rule | Native count | What it matched | Assessment |
|---|---|---|---|
| `COMMAND_INJECTION_JS_CHILD_PROCESS` | 42 | `child_process` use in `agent-team-creator` (19), `delivery-cadence` (9), `kbd-process-orchestrator` (8), `kbd-apply` (4), `impeccable` (2) | Expected: these skills run git and Node. Samples seen pass argument arrays with `shell: false` |
| `COMMAND_INJECTION_EVAL` | 8 | A function named `compile` and calls to an `exec` wrapper | False positive on the samples seen |
| `ALLOWED_TOOLS_BASH_VIOLATION` | 9 | Skill ships shell scripts but `allowed-tools` omits Bash | Probably real, low risk. Overlaps with CC-SK-008 |
| `YARA_sql_injection_generic` | 4 | `adversarial-review/tests/fixtures/clean-skill/scripts/count-rows.sh` | Uses `format()` with `%I`/`%L` quoting. Worth a second look because it sits in the fixture labelled clean |
| `COMPOUND_FETCH_EXECUTE` | 1 | `dispatch-judge.sh` | `curl` calls to the configured model gateway; no downloaded code is executed |
| `COMPOUND_FIND_EXEC` | 2 | `find -exec grep` in `idea-gate-1.sh` and `generate-report.sh` | Benign. `idea-gate-1.sh` interpolates words from user-supplied idea text into the `grep` pattern, which is worth quoting review |
| `PROMPT_INJECTION_CONCEALMENT` | 1 | `impeccable/reference/craft.md`: "Do not tell users they need to invoke `craft`" | Benign UX guidance |
| `YARA_jailbreak_generic` | 2 | Rows in a UI stack CSV | False positive |
| `CORRELATED_MANIFEST_CAPABILITY_MISMATCH` | 7 | Scripts do more than the manifest declares | Not reviewed |

Consequence for the plan: this scanner cannot be a pass/fail gate as it stands. The pack's orchestration skills legitimately spawn processes. It needs a policy file with reviewed allowances, which the tool supports (`generate-policy`).

## What this changes in the plan

1. **Decide the frontmatter convention first.** It blocks C0 for nearly every native skill, and the fix is mechanical once decided.
2. **C0 pass rule needs a third state.** "Passes with a declared deviation" should be distinct from a pass, if the owner keeps the convention.
3. **Add description-overlap detection to Phase 1.** `skill-scanner` has `--check-overlap`, which was not run. With roughly 150 descriptions lacking a trigger clause, collisions are the next thing to measure.
4. **Scanner needs a reviewed policy** before it gates anything.
5. **Inventory counts need reconciling** before tiering in section 7 of the plan means anything.

## Not done

- No fixes.
- `skills-lint`, `skill-validator`, `avocetta`, `skillcheck` and the overlap check were not run.
- Medium and low scanner findings, and `agnix` warnings beyond the table above, were counted but not read.
- Imported skills were scanned and counted but not triaged.
- Raw tool output is in the session scratchpad and is not committed; it will not survive a reboot. Re-running the commands in the table reproduces it.

## Addendum: frontmatter migrated to the specification (same day)

The owner chose to match the specification. Applied to native skills only; imported submodules and the two `adversarial-review` test fixtures were left alone.

| Change | Count |
|---|---|
| Top-level `version` moved to `metadata.version` | 180 skills |
| `metadata.tags` list converted to a comma-separated string | 145 skills |
| Version conflicts resolved in favour of the former top-level value | 2 (`kreuzberg` 1.0 → 1.0.0, `deep-research` 1.0.0 → 1.1.0) |

Tooling updated to enforce the new convention: `scripts/validate-skills.js` (strict mode now rejects a top-level `version` and list-valued tags), `scripts/backfill-strict-fields.js`, `scripts/install-minimax-skills.js` (reads `metadata.version`), `shared/scripts/cedar-skill-gate.sh`, `docs/SKILL_TEMPLATE.md` and `CLAUDE.md`. `SKILLS.md` was regenerated; its only change is the commit pointer, which was already stale.

| Check | Before | After |
|---|---|---|
| `npm run validate:strict` | exit 0 | exit 0, 0 errors |
| `skills-ref validate`, native skills passing | 16 of 198 | 156 of 198 |

The 42 remaining failures are other top-level fields outside the specification, which this change did not cover:

| Field | Skills | Note |
|---|---|---|
| `language` | 16 | Pack-specific |
| `triggers`, `authors`, `model_routing` | 15 (overlapping) | Pack-specific; consumers not yet traced |
| `argument-hint` | 7 | Claude Code extension; functional, cannot simply move |
| `disable-model-invocation` | 4 | Claude Code extension; functional |
| Flow-style YAML elsewhere in frontmatter | 4 | Two are test fixtures; `karpathy-tokenizer` and `start-business-build` use `authors: [...]` |

Not run after the change: `agnix`, the security scanner, `npm test`, `npm run build`, and any install. Installed copies on this machine still carry the old frontmatter until the next pack update.
