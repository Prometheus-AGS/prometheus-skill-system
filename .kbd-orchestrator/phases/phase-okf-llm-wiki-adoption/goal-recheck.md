# Goal re-check — phase-okf-llm-wiki-adoption

Date: 2026-07-02
Verifier: claude-code (change-okf-008-integration-verification, task 5)
Baseline (assessment): all 5 goals NOT MET.

Cross-repo note: the format-layer goals (1, 2, 3, 5) were delivered in
prometheus-knowledge-rs (local commits 91aee74, 965aea9, 673c8e4, 231a5be —
NOT pushed; pushing is approval-gated). The skill/schema layer (goal 4) and the
hook/MCP integration were delivered in prometheus-skill-system.

## Goal 1 — OKF v0.1 frontmatter conformance (required `type` + recommended fields, unknown-key tolerance)
**MET.**
- Writer (`pk-store/markdown.rs::entry_to_markdown`) emits `type` (required),
  plus `title`/`description`/`tags`/`timestamp`; empty optionals are omitted.
- Parser is permissive (OKF §9): only `type` required; unknown keys preserved
  round-trip in `WikiEntry.extra`; legacy pre-OKF pk docs still parse.
- Evidence: pk-store unit tests (parses_minimal_okf_document,
  unknown_frontmatter_keys_round_trip, parses_legacy_pk_document); live MCP
  ingest produced a conformant page (`type: Reference`, ISO timestamp) at
  ~/.prometheus/knowledge/wiki/okf-v0-1-integration-verification-after-change-okf-008.md.

## Goal 2 — Reserved index.md/log.md maintained per OKF §6/§7 on every ingest
**MET.**
- `MarkdownStore::regenerate_index` (§6 catalog grouped by type) and
  `append_log` (§7 dated groups, newest-first) run in `Librarian::compile`
  after every ingest; store load skips reserved files at every level.
- Evidence: pk-store integration test index_and_log_are_written_and_survive_reopen;
  live MCP ingest maintained both wiki/index.md and wiki/log.md (verified on disk).

## Goal 3 — Cross-links as bundle-relative body links + Citations (OKF §5/§8)
**MET.**
- Compile prompt now emits inline `[Title](/id.md)` body links and a
  `# Citations` section from sources; the link graph is derived from body
  links via pulldown-cmark (frontmatter `links` array kept read-only for
  back-compat).
- Evidence: bundle unit tests (extracts_bundle_relative_links_only,
  body_links_are_deduplicated); the live MCP-ingested page contains inline
  body links AND a `## Citations` section, with `links:` derived from the body.

## Goal 4 — Karpathy LLM Wiki operations (ingest/query/lint) as first-class skills + schema doc
**MET.**
- `skills/documentation/llm-wiki/` (SKILL.md + references/wiki-schema.md +
  references/okf-conformance.md) exposes ingest/query/lint through pk and the
  knowledge_* MCP tools; the layer-3 schema document is present.
- (Completed by a parallel session; verified here: passes
  `npm run validate:strict`, and its references match the real pk commands
  and the OKF format shipped in changes 003–005.)

## Goal 5 — pk lint enforces OKF v0.1 conformance with permissive-consumption semantics
**MET.**
- Deterministic OKF §9 conformance in `Librarian::lint`: unparseable
  frontmatter and empty/missing `type` are errors; missing recommended
  fields, broken cross-links, orphans, and reserved-file shape are warnings
  (never rejections). The LLM content-lint is best-effort (a model failure
  no longer suppresses conformance results). Missing-type auto-fix is
  deterministic (no LLM).
- Evidence: 9 bundle unit tests + store-level okf_conformance_lint_and_autofix
  integration test; live `pk lint` on a fixture bundle classified a type-less
  page as an auto-fixable error and broken-link/missing-description/orphan as
  warnings, and `pk lint --fix` repaired the missing type without an LLM call.

## Summary
5 of 5 goals MET (baseline 0/5).

## Deployment actions taken during verification (for reflect)
- Installed the new OKF-capable `pk` (25 MB) to ~/.local/bin/pk and
  `pk-cherry` (29 MB) to /usr/local/bin/pk-cherry; both old binaries backed up
  to ~/.prometheus/backups/.
- Added the missing LLM env (CLOUD_LLM_URL/LOCAL_LLM_URL + PK_*_MODEL) to
  ~/Library/LaunchAgents/ai.prometheus.pk-cherry.plist (absent since Jun 28 —
  a pre-existing deploy gap that broke MCP ingest independent of OKF; plist
  backed up) and reloaded the launchd service.

## Open items / caveats (for reflect)
- prometheus-knowledge-rs commits are LOCAL ONLY (not pushed / no PR) — the
  approval gate for that remote was never lifted. A push/PR is the outstanding
  deployment step to make this durable beyond this machine.
- The `pk ingest` LLM path has a known intermittent (~1/15) failure from the
  local openai-proxy returning a malformed 200; a separate task landed a
  client-side hardening fix (commit 9645c12 in knowledge-rs) but the proxy
  root cause is tracked separately. Not a blocker; retry-once handles it.
- A clearly-labeled test-marker entry remains in the live KB from the MCP e2e;
  harmless (the KB is already hook-populated), left in place to avoid index/log
  inconsistency since no delete tool is exposed.
