# Submodule currency — surveyed and updated 2026-07-31

## Pack: all 9 already current
```
artifact-refiner  entity-management  sycophancy-correction  cowork-skills
disk-space-guardian  liter-llm  openai-proxy  prometheus-knowledge
surreal-memory-server        -> all behind=0 ahead=0
```

## UAR: four were stale, all fast-forward-verified before moving

| Submodule | Was | Now | Commits behind |
|---|---|---|---|
| crates/prometheus-skill-system | 8ddac9a (2026-06-01) | **e04bfa0** (2026-07-31) | **359** |
| vendor/git/liter-llm | 78b7496ca | **3545cf6a2** | 90 |
| models.dev | c36b8e94 | **03e217866** | 4664 |
| vendor/git/rust-mcp-filesystem | 21f5f68 | (already current) | 0 |

**The headline number:** UAR's view of the skill pack was **two months and
359 commits stale**, seeing 161 skills where the pack has 220. Nothing
detected this — which is requirement 5's whole problem, now measured.

liter-llm is now pinned to the SAME commit in both repos (3545cf6a2).

## Every move was ancestry-checked first
`git merge-base --is-ancestor HEAD origin/<branch>` returned true for each
before checkout, so no update dropped commits. This matters: an earlier
session in this stack committed a submodule pointer that was BEHIND the
recorded one, silently reverting fixes.

## Local edits found and PRESERVED, not discarded
Six checkouts carried uncommitted `AGENTS.md`/`CLAUDE.md` edits — a
"Phase-Gated Testing (MANDATORY)" policy — that a bare checkout would have
destroyed. All stashed with the message
`phase-gated-testing policy, pre-ff 2026-07-31` and captured as patches:
```
  nested-artifact-refiner.patch
  nested-prometheus-knowledge.patch
  nested-surreal-memory-server.patch
  nested-sycophancy-correction.patch
  uar-submodule-local-edits.patch
```

**That policy belongs in the pack, not in consumers' submodule checkouts.**
An edit there is invisible to the pack's git history and is destroyed by the
next submodule update — the same class of mistake as editing a plugin cache.
Recommend applying it to the pack's own AGENTS.md/CLAUDE.md instead.
