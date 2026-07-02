PLAN: phase-okf-llm-wiki-adoption
Project: prometheus-skill-system + prometheus-knowledge-rs (cross-repo)
Date: 2026-07-01
OpenSpec available: YES (openspec/ + CLI exist repo-wide) — CORRECTED 2026-07-01:
  this was wrongly stated as NO during planning; I had not checked for the
  openspec/ directory. It exists with 91 pre-existing changes. However,
  os_verify (kbd-apply's openspec adapter) shells out to `openspec validate`,
  which fails even on an existing done change (change-elicit-001) because
  none of the 91 changes have a real specs/ delta directory (## ADDED/MODIFIED
  Requirements + #### Scenario: blocks) — they are PMPO-shaped proposal.md/
  tasks.md, not true OpenSpec deltas. Native KBD changes remain the correct
  choice for this phase's infra/vendoring/cross-repo work; project.json now
  pins specBackend: native-kbd explicitly (see execution.md) rather than
  relying on auto-detection, which would otherwise silently resolve to
  openspec repo-wide. Changes still live in .kbd-orchestrator/changes/.
Changes to implement: 8
Sycophancy gate: detect_sycophancy score 0.0 (PASS, standard strictness, pmpo_plan_phase)

SCOPE CONFLICT (surfaced per S-02): the request framed this as updating skills
"in this repository," but 4 of 8 changes (003–006) modify prometheus-knowledge-rs,
because that is where the wiki format is implemented (pk-store, pk-librarian,
pk lint). A skills-only plan would document a format the tool does not write.
This plan is explicitly cross-repo; the prometheus-knowledge-rs working checkout
does not exist yet and is created by change-okf-002.

Spec inputs: phases/phase-okf-llm-wiki-adoption/inputs/okf-v0.1.md and
llm-wiki-karpathy.md (vendored to shared/references/ by change-okf-001).

CHANGE LIST (ordered)

1. change-okf-001-vendor-specs: Vendor OKF + Karpathy docs; record adoption decision
   - Scope: docs (shared/references/, CLAUDE.md)
   - Depends on: NONE
   - Recommended agent: Claude Code
   - Est. complexity: S
   - Complexity score: Low
   - Model class: small
   - Customer value: MEDIUM
   - Details: Copy phase inputs to shared/references/okf-v0.1.md and
     shared/references/llm-wiki-pattern.md. Add a CLAUDE.md section recording
     the OKF v0.1 adoption decision and the cross-repo ownership split
     (format = prometheus-knowledge-rs; skills/schema/hooks = this repo).

2. change-okf-002-pk-workspace-baseline: Clone pk repo; build/test baseline; diagnose ingest failure
   - Scope: cross-repo bootstrap (sibling checkout)
   - Depends on: NONE
   - Recommended agent: Claude Code
   - Est. complexity: M
   - Complexity score: Medium
   - Model class: medium
   - Customer value: HIGH (unblocks 003–006 and all e2e verification)
   - Details: Clone github.com/Prometheus-AGS/prometheus-knowledge-rs to
     ~/Projects/prometheus/prometheus-knowledge-rs; cargo build + cargo test
     baseline. Diagnose pk ingest "LLM error: failed to parse LLM response"
     (timeboxed: fix if shallow, else document workaround and file follow-up).

3. change-okf-003-permissive-okf-parser: OKF §9 permissive parser + reserved filenames
   - Scope: pk-store, pk-core (prometheus-knowledge-rs)
   - Depends on: change-okf-002
   - Recommended agent: Claude Code
   - Est. complexity: M
   - Complexity score: High
   - Model class: frontier
   - Customer value: HIGH
   - Details: markdown_to_entry requires only `type`; unknown keys preserved on
     round-trip; missing pk fields derived (id from wiki-relative path,
     timestamps defaulted, revision=1). Store load skips reserved filenames
     index.md and log.md. Parser lands BEFORE writer (see trade-offs).

4. change-okf-004-okf-writer-and-id-mapping: OKF emitter + path-based concept IDs
   - Scope: pk-store, pk-core (prometheus-knowledge-rs)
   - Depends on: change-okf-003
   - Recommended agent: Claude Code
   - Est. complexity: M
   - Complexity score: High
   - Model class: frontier
   - Customer value: HIGH
   - Details: entry_to_markdown emits OKF frontmatter — type (required), title,
     description, tags, timestamp — with pk fields (revision, sources) kept as
     producer extension keys. ArticleId ↔ concept-ID mapping: concept ID =
     wiki-relative path minus .md; subdirectories supported.

5. change-okf-005-index-log-and-body-links: index.md/log.md maintenance + body links/citations
   - Scope: pk-librarian prompts, pk-store (prometheus-knowledge-rs)
   - Depends on: change-okf-004
   - Recommended agent: Claude Code
   - Est. complexity: L
   - Complexity score: High
   - Model class: frontier
   - Customer value: HIGH
   - Details: Every ingest updates index.md (OKF §6 catalog with descriptions)
     and appends to log.md (§7, `## YYYY-MM-DD` groups, newest first).
     Librarian prompts rewritten to produce bundle-relative markdown body links
     (§5) and a Citations section (§8) mapped from `sources`.

6. change-okf-006-okf-lint: OKF conformance rules in pk lint
   - Scope: pk lint (prometheus-knowledge-rs)
   - Depends on: change-okf-003 (reserved-file checks need change-okf-005)
   - Recommended agent: Claude Code
   - Est. complexity: M
   - Complexity score: Medium
   - Model class: medium
   - Customer value: MEDIUM
   - Details: Lint rules per §9 — frontmatter parseable, non-empty type,
     reserved-file structure when present. Permissive consumption: missing
     optional fields / unknown types / broken links are warnings, never
     rejections. Auto-fixables wired into --fix.

7. change-okf-007-llm-wiki-skills: llm-wiki skill (ingest/query/lint) + wiki schema doc
   - Scope: skills/documentation/llm-wiki (this repo)
   - Depends on: change-okf-001 (final examples verified against change-okf-004)
   - Recommended agent: Claude Code
   - Est. complexity: L
   - Complexity score: Medium
   - Model class: frontier
   - Customer value: HIGH (the user-facing capability)
   - Details: New skill exposing the three Karpathy operations through pk:
     ingest (source → wiki integration + index/log update), query (answer from
     wiki with citations; file good answers back), lint (health check —
     contradictions, orphans, stale claims). references/ carries the layer-3
     wiki schema document. Gate: npm run validate:strict.

8. change-okf-008-integration-verification: Hooks/MCP/e2e verification
   - Scope: shared/scripts hooks, pk-mcp consumers, BDD drafts (this repo)
   - Depends on: change-okf-005, change-okf-006, change-okf-007
   - Recommended agent: Claude Code
   - Est. complexity: M
   - Complexity score: Medium
   - Model class: medium
   - Customer value: HIGH
   - Details: Verify pk-focus-on-prompt.sh / pk-health.sh / pk-lint.sh and
     knowledge_* MCP tools against the OKF format; run e2e ingest → index/log →
     query round-trip; add draft BDD feature under tests/features/drafts/.

EXECUTION ROUND ORDER
Round 1 (parallel): change-okf-001, change-okf-002
Round 2 (parallel): change-okf-003, change-okf-007 (drafting only; examples blocked on 004)
Round 3: change-okf-004
Round 4 (parallel): change-okf-005, change-okf-006
Round 5: change-okf-008

DEFERRED / EXPLICIT SCOPE CUTS
- No migration tooling: both KBs are empty (assessment) — deliberately skipped.
  If real ingestion starts before change-okf-003 lands, this plan gains a
  migration change.
- qmd/search engine, Obsidian tooling, Marp/Dataview outputs from the Karpathy
  doc: out of scope this phase.
- KB sync via sovereign-sync/CRDT: out of scope.
- If the pk ingest LLM bug (002) is deep (model-routing layer), the fix moves
  to a follow-up phase and 008 degrades from e2e to static format verification
  — this is the plan's main schedule risk.

TRADE-OFFS
- Parser-before-writer ordering (003 → 004) is deliberate: emitting OKF while
  the parser is strict would break round-tripping of anything already on disk
  elsewhere (e.g. ~/.prometheus/knowledge/shared/).
- Keeping pk fields (id, revision, sources) as OKF extension keys preserves pk
  semantics at the cost of noisier frontmatter; dropping them would lose
  revision tracking.
- 4 of 8 changes land in a repo whose CI/merge process is outside this plan's
  control; upstream stalls would block Rounds 2–4.

COMMANDS TO RUN
mkdir -p .kbd-orchestrator/changes/change-okf-001-vendor-specs        # created by this plan
mkdir -p .kbd-orchestrator/changes/change-okf-002-pk-workspace-baseline
mkdir -p .kbd-orchestrator/changes/change-okf-003-permissive-okf-parser
mkdir -p .kbd-orchestrator/changes/change-okf-004-okf-writer-and-id-mapping
mkdir -p .kbd-orchestrator/changes/change-okf-005-index-log-and-body-links
mkdir -p .kbd-orchestrator/changes/change-okf-006-okf-lint
mkdir -p .kbd-orchestrator/changes/change-okf-007-llm-wiki-skills
mkdir -p .kbd-orchestrator/changes/change-okf-008-integration-verification

PLAN COMPLETE
