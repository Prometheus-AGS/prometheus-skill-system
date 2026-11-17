# Skill Pack Certification Plan

**Date:** 2026-11-17
**Status:** Proposal. Nothing in this plan has been run.
**Scope:** every skill in `skills/`, the native code in `substrate/` and `tools/`, and installation on nine harnesses.

## 1. Summary

The pack can prove today that its skills are well-formed. It cannot prove that a skill triggers on the right request, improves the result, or behaves the same on another harness. No skill directory has an `evals/` folder.

This plan defines five certification levels, three domain tracks (native code, Feynman learning loop, Karpathy loop), and a per-harness matrix. Each level names the tools that produce its evidence.

Three facts shape the plan:

1. **No official agentskills.io certification exists.** The spec ships a reference validator (`skills-ref`) for `SKILL.md` files. There is no published certification programme and no conformance test for clients (harnesses). "Certified" therefore means a certificate this project issues, backed by named tools and recorded evidence.
2. **All evidence is produced locally.** The repository forbids hosted CI for validation. Every tool below runs on the development machine; where a tool ships a GitHub Action, only its CLI is used.
3. **Behavioural evals count as integration evidence; unit-level tools do not.** A skill eval runs the real harness with a real model through its production entry point. Mutation testing, Miri and fuzzing are useful diagnostics, but under the repository's integration-only policy they are supporting findings, not acceptance evidence.

## 2. Certification levels

A skill holds the highest level for which it has current evidence. Levels are cumulative.

| Level | Claim | Evidence | Cost |
|---|---|---|---|
| **C0 Spec** | The skill conforms to agentskills.io | Two independent validators agree | Seconds, deterministic |
| **C1 Static** | No broken references, no cross-harness frontmatter hazards, no security findings | Linters and a security scanner | Seconds to minutes |
| **C2 Trigger** | The skill activates on requests it should handle and stays quiet on those it should not | Trigger eval with held-out queries | Model calls |
| **C3 Outcome** | Output with the skill is measurably better than output without it | With/without comparison, repeated runs | Model calls, the most expensive level |
| **C4 Portable** | C2 and C3 hold on each named harness | The same cases run through each harness | Model calls × harnesses |

C0 and C1 apply to all skills. C2 to C4 are applied by tier (section 7), because running every case for every skill on nine harnesses is not affordable before a pilot measures the real cost.

## 3. Tools per level

Items marked † were found in web research on 2026-11-17. Unmarked items are established tools named from general knowledge and were not re-verified in this research.

### C0: agentskills.io conformance

| Tool | Role |
|---|---|
| `npm run validate:strict` (this repo) | Existing gate: frontmatter, name pattern, version, licence, tags |
| `skills-ref validate` † (`agentskills/agentskills`) | Official reference validator. Also `read-properties` and `to-prompt` |
| `skill-validator` † (`agent-ecosystem/skill-validator`) | Validates and scores skill packages against the spec |
| `avocetta` † (`korya/avocetta`) | Deterministic spec, Claude Code and Codex compliance |

Pass rule: the repo validator and `skills-ref` both pass. A disagreement between them is a finding, because each can be wrong in the same way a drift check can (see the E1 residual-class lesson in `CLAUDE.md`).

### C1: static quality and security

| Tool | Role |
|---|---|
| `agnix` † (`agent-sh/agnix`) | 385 rules across six platforms; 26 for Agent Skills; also lints `CLAUDE.md`, hooks and MCP configs |
| `skills-lint` † (`pachara-n/skills-lint`) | Broken script and file references, over-long descriptions, duplicate names |
| `skillcheck` † | Description discoverability score, cross-agent issues |
| Cisco AI Defense `skill-scanner` † | Prompt injection, exfiltration, malicious code |
| SkillScanner † (Swarms) | Pattern scan plus LLM review with a verdict |
| OWASP Agentic Skills Top 10 † | Checklist to cite in the certificate |
| Tessl review and Snyk score † | Third-party quality and security score. Requires publishing to a registry, so it is optional |
| `npm run check:mobile-classification` (this repo) | Execution class per skill |

Duplicate-name and description-collision checks matter most here. With about 310 skills (per `CLAUDE.md`), collisions are the likeliest cause of mis-triggering, and Codex's fixed catalogue budget makes them worse.

### C2: trigger accuracy

| Tool | Role |
|---|---|
| skill-creator description optimiser † (`run_loop.py`, `improve_description.py`) | About 20 should/should-not queries per skill, stratified train/test split, repeated runs |
| `claude plugin eval` † with a `tool_used: Skill` grader | Trigger check per case at plugin level |
| Promptfoo `skill-used` signal † | The same check on Claude Agent SDK, Codex and OpenCode |

Pass rule (proposed): held-out trigger accuracy at or above a threshold the pilot sets, with no should-not-trigger query firing in a majority of runs.

### C3: outcome

| Tool | Role |
|---|---|
| skill-creator test-and-compare loop † | With-skill versus baseline, grader, blind A/B comparator, mean ± stddev, HTML viewer. Cases in `evals/evals.json` |
| `claude plugin eval` † | With/without-plugin Δ, weighted graders, mock MCP servers, cost cap, JSON report. Early access |
| Promptfoo "Test Agent Skills" † | Two-version comparison with a JSON output schema |
| Inspect AI † | Used in Google's skill-evaluation codelab |
| MLflow skill tests † (`mlflow/skills`) | YAML case, setup script, Python judges |
| Harbor † (`harbor-framework/harbor`) | Task runner with verifier scripts; the basis for C4 |
| SkillsBench † (arXiv 2602.12670) | Design reference: curated skill versus self-generated versus none |

Pass rule (proposed): Δ above zero beyond run-to-run noise on at least three runs per arm. Prefer deterministic graders (file exists, regex, command exit code). LLM judges are noisy on long outputs and must be a different model family from the producer, as the adversarial-review rule already requires.

The skill-creator `evals.json` format and the `claude plugin eval` case format are not interchangeable. Section 9 asks the owner to pick one as canonical.

### C4: portability

Covered in section 4.

## 4. Harness matrix

Discovery paths below come from each vendor's documentation where found. "Unverified" means the research did not find a primary source; it does not mean unsupported.

| Harness | Skill discovery (documented) | Headless runner for evals | Eval tooling | Notes |
|---|---|---|---|---|
| **Claude Code CLI** | `~/.claude/skills`, plugins | `claude -p` | skill-creator, `claude plugin eval`, Promptfoo, Harbor `claude-code` | Reference harness. The only one with sessions exercised so far |
| **Codex** | `~/.agents/skills` as real directories (repo-verified); `[[skills.config]]` to disable | `codex exec` | Promptfoo Codex provider, Harbor `codex` | Symlinked skills are ignored silently. Catalogue budget truncates descriptions |
| **OpenCode** | `~/.config/opencode/skills`, plus `~/.claude/skills` and `~/.agents/skills`; project `.opencode/skills` | `opencode run` | Promptfoo OpenCode provider, Harbor `opencode` | Does not look for skills nested inside a skill directory. Search stops six levels down |
| **Kimi Code** | `~/.kimi/skills`, `~/.claude/skills`, `~/.codex/skills`; `~/.config/agents/skills`, `~/.agents/skills` | `kimi` print mode (unverified flag) | Harbor `kimi-code`, `kimi-cli` | `--skills-dir` replaces discovery; `extra_skill_dirs` adds to it |
| **Qwen Code** | `~/.qwen/skills` and `~/.agents/skills`; `.qwen/skills` wins on a name clash | `qwen -p` (unverified flag) | Harbor `qwen-coder` | Accepts a non-standard `priority` frontmatter field |
| **MiniMax Code** | Unverified. Vendor skills install into `~/.claude/skills` and similar via `npx skills add` | Unverified | Harbor `mcode` | MiniMax open-sourced an "MCode Stack" of `SKILL.md` skills |
| **Zed** | Native Agent Skills with a Skills Manager and `agent.skills` settings; paths unverified | None found | Manual protocol | Can also host Claude Code or Codex as external agents, which then use their own discovery |
| **Xiaomi MiMo Code** | Built on OpenCode; imports existing skills and MCP servers automatically; paths unverified | Unverified | Harbor `mimo` | Ships its own `skill-creator` |
| **DeepSeek Harness** (`dsh`) | Native `SkillProvider` with camelCase frontmatter. The community `dsh-skills-bridge` plugin mounts `~/.claude/skills` and `~/.agents/skills` | Unverified | None found | Release candidate (v0.1.0-rc.8 at the time of the bridge post). Nested YAML frontmatter is skipped with a warning |

### What C4 requires per harness

1. **Install proof.** After `cowork install`, the harness lists the pack's skills. Use the harness's own listing command or debug output, as `codex debug prompt-input` does for Codex.
2. **Discovery count.** The number of skills the harness reports equals the number installed, with no duplicates. This is the check that would have caught the Codex symlink failure.
3. **Trigger and outcome subset.** The Tier A cases (section 7) run through the harness's headless mode, driven by Harbor or Promptfoo.
4. **Recorded exceptions.** Hooks, subagents and MCP differ per harness. A skill that depends on one of them is certified only where that dependency exists.

Harbor is the recommended common runner: its pre-integrated agent list includes `claude-code`, `codex`, `opencode`, `kimi-code`, `kimi-cli`, `qwen-coder`, `mcode` and `mimo`. It uses containers by default, which is acceptable under the local-only rule when run with the local Docker environment. Zed and DeepSeek Harness need a manual protocol until a headless path is confirmed.

### Findings from the harness research

These are discrepancies between the repository's documentation and vendor documentation. None has been checked against an installed binary.

- **OpenCode global path.** `CLAUDE.md` lists `~/.opencode/skills/`. OpenCode's documentation gives `~/.config/opencode/skills`. The canonical `~/.agents/skills` root is read either way, so installs probably work, but the documented path looks wrong.
- **Kimi path.** `CLAUDE.md` lists `~/.kimi-code/skills/`. The Kimi CLI documentation lists `~/.kimi/skills` and does not mention `~/.kimi-code`.
- **OpenCode and nested skills.** OpenCode does not descend into a skill directory. Bundles such as `deep-research/skills/stage-*` would expose only the parent there, while Codex and Cursor expose every child.
- **DeepSeek Harness and nested frontmatter.** The pack requires `metadata.tags`, which is nested YAML. The bridge skips nested YAML with a warning, so tags and version may be dropped.

## 5. Domain tracks

A track adds requirements for skills whose correctness depends on something a skill eval cannot see.

### Track N: native code

Applies to `substrate/*`, `tools/*`, the 61 script-bearing skills, and Rust skills that generate code.

Acceptance evidence (integration, per repository policy):

| Surface | Evidence |
|---|---|
| Each crate | `cargo test -p <package> --test <integration-target>` against the existing `tests/` directories, one build at a time |
| `surface-bridge`, `prometheus-research` | HTTP round trip against the running server on `:7890` and `:7891` |
| `learner-model` | JSON-RPC round trip over stdin/stdout |
| `skill-ffi` | Round trip through the generated `flutter_rust_bridge` bindings from a Dart host, asserting returned values. The existing seven tests assert values but the Dart side is unconfirmed |
| `wit/prometheus-component` | `wasm-tools validate` and `wasm-tools component wit` on the built component, then a run under a real Wasmtime host. `CLAUDE.md` records that nothing has executed it yet, so this row is uncertifiable today |
| Installer | `scripts/install-binaries.sh` in a scratch `HOME`, with one submodule deliberately broken, to prove a failed build cannot abort the install |
| Generated code (Rust skills, scaffolds) | The skill's output compiles and its integration target passes. Graded by exit code |

Supporting diagnostics (reported, not gating): `cargo clippy`, `cargo deny` and `cargo audit`, `cargo semver-checks`, `cargo mutants`, `cargo fuzz`, Miri for `unsafe` in `skill-ffi`, and the installed `mutation-testing`, `property-based-testing`, `rust-review`, `sharp-edges` and `supply-chain-risk-auditor` skills. `prometheus-rust-auditor` already orchestrates several of these.

All of Track N runs at the phase boundary (B4), not during implementation.

### Track F: Feynman learning loop

Applies to the 12 `skills/learn` skills, `learner-model`, and the sycophancy gate.

| Claim | Evidence | Outside reference |
|---|---|---|
| The scheduler is correct | Compare `learner-model` intervals with the reference FSRS implementation on shared review logs | `open-spaced-repetition` reference implementations and `srs-benchmark` |
| The mastery update is correct | Replay fixed observation sequences through the JSON-RPC interface and check the documented formula | None needed; deterministic |
| The grader does not flatter | Seeded explanations with known misconceptions; `learn-grade` must report each one. Explanations with none must not be failed | SycEval †, ELEPHANT †, syco-bench †, SycoBench-600 †, SPINE † (multi-turn pressure) |
| The loop cannot be closed by self-report | Scenario: the learner claims mastery and asks to move on. The loop must still require the grade, two transfer problems and the 24-hour retention check | Petri † (Anthropic's open-source auditing tool) can drive simulated-user pressure |
| Tutoring withholds answers | Rubric-scored dialogues: scaffolding rather than handing over the solution | MathTutorBench † (scaffolding reward model), MRBench † taxonomy |
| Learning happens over time | Optional and expensive: a simulated learner with a knowledge-tracing state, pre- and post-probes | EduClaw-Bench † (30-day horizon; found that tutoring quality depends on the model and harness together) |
| Surface tiers degrade correctly | The same `UiIntent` rendered at tiers 0, 1 and 2 collects an equivalent response | None needed |
| KB content stays private | Run `content-grounding-kb.sh` in KB mode with external API variables set and assert no outbound request | Network capture |

EduClaw-Bench's finding supports the C4 design: a learning skill certified on one harness is not certified on another.

Simulated learners are known to be unreliable stand-ins for real students. Track F certifies that the loop enforces its own rules. It does not certify that people learn more.

### Track K: Karpathy loop and Karpathy-named skills

"Karpathy correctness" covers three different things in this repository. Each needs its own evidence.

| Surface | Claim | Evidence |
|---|---|---|
| **LLM wiki** (`llm-wiki`, `pk`) | Entries conform to OKF v0.1; `index.md` and `log.md` stay consistent; ingest is idempotent | `bash shared/scripts/pk-lint.sh` on a scratch knowledge base; ingest the same session twice and diff; `pk context` returns the ingested entry |
| **Learning loop** (hooks, `learning_write.py`, outbox worker) | A lesson written in one session is delivered to the right role in the next, and to no other role | The existing `test-subagent-delivery.sh` per harness; a scope-isolation case where a private lesson must not appear in another role's view |
| **`karpathy-progress-memory`** | Progress events are idempotent and survive a memory-service outage | Record the same boundary twice; stop surreal-memory and record again; replay from the log |
| **`karpathy-tokenizer`** | A tokenizer trained with `rustbpe` and exported to tiktoken encodes and decodes identically on both sides | Round-trip and parity test on a fixed corpus; the skill pins `rustbpe==0.1.0` |
| **Reflector gate** | Sycophantic reflections are rejected; the two-rejection cap holds | Existing adversarial-review fixture suites, plus the sycophancy benchmarks in Track F |

The repository currently has 17 untracked `karpathy-session-*.md` wiki files and a modified `index.md`. An index-consistency check belongs in Track K because that state is exactly what it would test.

## 6. Evidence and the certificate

Each certified skill gets one record. Proposed location: `docs/evidence/certification/<skill>.json`.

```json
{
  "skill": "feynman-loop",
  "content_hash": "<sha256 of the skill directory>",
  "level": "C3",
  "tracks": ["F"],
  "harnesses": {
    "claude-code": { "version": "…", "model": "…", "result": "PASS" },
    "codex":       { "version": "…", "model": "…", "result": "NOT_EXERCISED" }
  },
  "commands": ["…exact local commands…"],
  "runs_per_arm": 3,
  "delta": { "mean": 0.0, "stddev": 0.0 },
  "judge_model": "…",
  "date": "2026-11-17"
}
```

Rules:

- A certificate is bound to the content hash. Any edit to the skill voids it.
- A certificate is bound to the harness version and the model. A new model release voids C2 to C4 until re-run.
- `NOT_EXERCISED` is a recorded result, as the current phase already uses. It is never rounded up to a pass.
- Preserved failures stay in the record.

## 7. Tiering

| Tier | Skills | Target |
|---|---|---|
| **A** | KBD lifecycle, learn domain, adversarial-review, agent-team, installer-facing skills | C4 on Claude Code, Codex and OpenCode; C3 elsewhere as harness access allows |
| **B** | Script-bearing skills (61) and code-generating skills | C3 on Claude Code plus Track N |
| **C** | Remaining manifest-only skills | C1, plus C2 for any skill flagged by a collision check |
| **Imported** | Submodules under `skills/imported/` | C0 and C1 only; behaviour is the upstream's responsibility |

## 8. Phases

| Phase | Work | Exit condition |
|---|---|---|
| **1. Static baseline** | Run `skills-ref`, `agnix`, `skills-lint` and one security scanner across the pack. Triage, do not fix | A findings report with counts per rule |
| **2. Pilot** | Write eval cases for five Tier A skills. Run C2 and C3 on Claude Code. Measure cost and noise | Thresholds for C2 and C3 set from data; cost per skill known |
| **3. Harness install proof** | Scratch-home install and discovery count on each of the nine harnesses. Resolve the path discrepancies in section 4 | A discovery table with a real count per harness |
| **4. Tier A to C4** | Harbor or Promptfoo cases across Claude Code, Codex and OpenCode | Certificates for Tier A |
| **5. Tracks** | Track N at the phase boundary; Track F and Track K scenarios | Track evidence recorded |
| **6. Tiers B and C** | Extend coverage; add the certificate check to the local release gate | `npm run check:certification` fails on a stale hash |

Phases 1 and 3 need no eval authoring and can run first.

## 9. Decisions for the owner

1. **Canonical case format.** skill-creator `evals/evals.json` (works today, per skill) or `claude plugin eval` cases (per plugin, built-in baseline and cost cap, but early access). Recommendation: skill-creator format for authoring now, with Harbor tasks for C4, and adopt `claude plugin eval` when access is confirmed.
2. **Where eval cases live.** Inside each skill (`evals/` is excluded from packaging by skill-creator) or in a separate tree. Recommendation: inside each skill, so the case travels with the content hash.
3. **Protected status.** Whether eval cases join the immutable-tests rule and need a signed approval manifest to change.
4. **Harness scope for C4.** Nine harnesses is a large surface. Recommendation: three for Tier A (Claude Code, Codex, OpenCode), install proof only for the other six until the pilot gives a cost.
5. **Model spend.** C3 and C4 have no budget yet. The pilot should produce one.
6. **Third-party registries.** Whether to publish to Tessl for an external score, which sends skill content to an outside service.

## 10. Limits of this plan

- Harness details for MiniMax Code, Zed, MiMo Code and DeepSeek Harness rest on documentation and announcements, not on an installed copy. Headless flags for Kimi and Qwen are unverified.
- Tools marked † were identified by search. None was installed or run, so fitness for this pack is unproven until Phase 1.
- Tools without † were not re-checked in this research.
- Skill counts (310 total, 249 manifest-only, 61 script-bearing) are taken from `CLAUDE.md`, not recounted.
- Two planned searches (FSRS reference vectors, Codex skill documentation) were rate-limited and not repeated. The Codex row relies on the repository's own verified notes.

## Sources

- Agent Skills specification and `skills-ref`: https://agentskills.io/specification, https://github.com/agentskills/agentskills
- No client conformance test: https://www.skillsboard.sh/agent-skills
- `claude plugin eval`: https://code.claude.com/docs/en/plugin-evals
- skill-creator evals: https://claude.com/resources/articles/improving-skill-creator-test-measure-and-refine-agent-skills
- Promptfoo: https://www.promptfoo.dev/docs/guides/test-agent-skills/
- Harbor agents: https://docs.harborframework.com/agents/pre-integrated-agents
- MLflow: https://mlflow.org/blog/evaluating-skills-mlflow/
- Inspect codelab: https://codelabs.developers.google.com/codelabs/evaluate-agent-skills-using-open-source-frameworks
- SkillsBench: https://arxiv.org/html/2602.12670v1
- agnix: https://github.com/agent-sh/agnix
- skill-validator: https://github.com/agent-ecosystem/skill-validator
- Cisco skill-scanner: https://github.com/cisco-ai-defense/skill-scanner
- OWASP Agentic Skills Top 10: https://owasp.org/www-project-agentic-skills-top-10/
- OpenCode skills: https://opencode.ai/docs/skills/
- Qwen Code skills: https://qwenlm.github.io/qwen-code-docs/en/users/features/skills/
- Kimi CLI skills: https://moonshotai.github.io/kimi-cli/en/customization/skills.html
- Zed skills: https://zed.dev/docs/ai/skills
- MiMo Code: https://github.com/XiaomiMiMo/MiMo-Code
- DeepSeek Harness bridge: https://github.com/deepseek-ai/deepseek-harness/discussions/5213
- MiniMax skills: https://github.com/MiniMax-AI/skills
- EduClaw-Bench: https://arxiv.org/html/2608.03206v1
- MathTutorBench: https://aclanthology.org/2025.emnlp-main.11.pdf
- SycEval: https://www.alphaxiv.org/abs/2502.08177
- Petri: https://alignment.anthropic.com/2025/petri/
