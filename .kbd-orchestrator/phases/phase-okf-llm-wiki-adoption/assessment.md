ASSESSMENT: phase-okf-llm-wiki-adoption
Project: prometheus-skill-system (skill pack) — cross-repo dependency on prometheus-knowledge-rs (pk CLI)
Date: 2026-07-01
Codebase baseline: The "Karpathy LLM wiki" is implemented by the external pk CLI (prometheus-knowledge-rs v0.1.0); this repo integrates it only through hooks. No wiki-authoring skill exists here, and the current pk wiki format does not conform to OKF v0.1 on any required dimension.
Cross-tool progress: none (phase opened this session)
Sycophancy gate: detect_sycophancy score 0.018 (PASS, standard strictness); one Low S-07 length note.

IMPLEMENTATION STATUS
- Goal 1 (OKF frontmatter conformance): MISSING — pk-store/src/markdown.rs defines Frontmatter {id, title, tags, links, sources, created_at, updated_at, revision}. OKF's single REQUIRED key `type` is absent; recommended keys description/resource/timestamp are absent. Parsing is struct-strict on missing fields: an OKF-conformant document lacking `revision` or `id` fails markdown_to_entry with a frontmatter error, so pk today rejects valid OKF and emits non-OKF — non-conformant in both directions.
- Goal 2 (index.md / log.md reserved files): MISSING — pk-store store.rs loads every .md under wiki/ as an entry with no reserved-filename handling. An OKF index.md (no frontmatter) would fail to parse at load; pk never generates or maintains index.md/log.md.
- Goal 3 (body cross-links + citations): MISSING — relationships live in the frontmatter `links: Vec<ArticleId>` array, not as bundle-relative markdown links in the body (OKF §5). No Citations-section convention (§8). Note: OKF concept IDs are file paths; pk ArticleIds are opaque slugs in a flat wiki/ dir — mapping these is a design decision, not a rename.
- Goal 4 (Karpathy wiki skills in this repo): MISSING — no llm-wiki skill exists in skills/. Integration is hooks-only: pk-focus-on-prompt.sh (UserPromptSubmit), pk-health.sh (SessionStart), pk-lint.sh (weekly scheduled), and pk ingest at Stop/reflect via forge-reflect-on-stop.sh and evaluate-session.sh. The karpathy-tokenizer skill is unrelated (BPE tokenizer training; deliberately not hook-wired per 2026-06-11 decision). Karpathy's third layer — the schema document telling the agent how the wiki is structured — does not exist in either repo.
- Goal 5 (pk lint enforces OKF conformance): MISSING — lint infrastructure exists (LintReport/LintSeverity in pk-core) but checks pk-native invariants only; no OKF §9 conformance rules and no permissive-consumption semantics.

FAVORABLE BASELINE FACT
- Both knowledge bases are empty (pk stats: 0 entries in project KB; global KB also empty). There is no migration burden — adopting OKF now costs a format change, not a data migration. This window closes as soon as real ingestion starts.

CROSS-TOOL PROGRESS
- NONE — no cross-tool activity recorded (fresh phase).

SPEC GAP SUMMARY
- The OKF v0.1 spec text exists only in this conversation. Neither repo contains it. First change must vendor the spec (e.g. shared/references/okf-v0.1.md or docs/) so goals are checkable against a committed artifact.
- The Karpathy LLM Wiki pattern doc is likewise unvendored; the schema-layer document (Goal 4) should be derived from it.
- No spec defines the pk↔OKF concept-ID mapping (path-based IDs vs flat slug IDs) or how `sources` maps to OKF Citations.

BUILD HEALTH
- npm run validate: PASS — 118 skills, 1 pre-existing warning (kbd-process-orchestrator SKILL.md 548 lines).
- pk lint: PASS, but trivially — the KB is empty, so this exercises no lint rules.
- pk ingest: FAIL at runtime — "LLM error: failed to parse LLM response" on a minimal stdin ingest. The enrichment path is broken locally today, independent of OKF; it blocks end-to-end verification of any ingest skill this phase produces.
- prometheus-knowledge-rs build: UNKNOWN — no local working checkout exists (only a read-only cargo git checkout on /Volumes/my-passport). Upstream: https://github.com/Prometheus-AGS/prometheus-knowledge-rs.git. Format-layer changes require cloning it as a sibling repo.

CONSTRAINT CHECK
- CLAUDE.md violations: NONE found, but the documentation-hierarchy rule bites this phase: pk-store/pk-librarian/pk-lint changes are crate-scoped to prometheus-knowledge-rs, while the OKF adoption decision itself is cross-cutting and must be recorded in this repo's CLAUDE.md (canonical source).
- New skills must pass npm run validate:strict (version, license, metadata.tags) — gate for the Goal 4 skills.
- constraints.md: N/A (absent).

RISKS AND CONCERNS
1. Scope split risk: the user framed this as updating skills "in this repository," but three of five goals (1, 2 in part, 3, 5) are changes to prometheus-knowledge-rs. Shipping only the skill layer here would document a format the tool does not write. The plan must either declare the phase cross-repo or pin Goal 1/3/5 as upstream dependencies with explicit handoff.
2. Strict-parse regression risk: making pk emit OKF while keeping strict struct deserialization would break round-tripping of any pre-existing non-OKF entry elsewhere (e.g. ~/.prometheus/knowledge/shared/). Parser must become permissive (OKF §9) before the writer changes.
3. The broken pk ingest LLM path must be fixed or worked around before any BDD/e2e verification of ingest skills; otherwise the phase can only verify formats statically.
4. pk-mcp (knowledge_* tools at :8942) and the hook scripts consume pk output; format changes must be checked against pk-focus context injection and knowledge_get consumers, or context blocks may degrade silently.

GOAL PROGRESS
- Goal 1 (OKF frontmatter): NOT MET — required `type` key absent from writer and parser.
- Goal 2 (index.md/log.md): NOT MET — reserved filenames unhandled; index.md would break KB load.
- Goal 3 (body links + citations): NOT MET — frontmatter-array links only.
- Goal 4 (wiki skills + schema doc): NOT MET — zero wiki skills exist in this repo.
- Goal 5 (OKF-aware lint): NOT MET — lint has no OKF rules.

ASSESSMENT COMPLETE
