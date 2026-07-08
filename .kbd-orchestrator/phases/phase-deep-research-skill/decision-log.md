# Decision Log — phase-deep-research-skill

---

### 2026-07-08 — Stage execution model
**Options:** Sequential pipeline vs DAG with parallelism  
**Decision:** Sequential default; DAG declared via `skill.toml [features] threaded = true`  
**Provenance:** research (codebase patterns)  
**Rationale:** All multi-stage skills in pack execute sequentially. Parallel execution requires binary process management outside SKILL.md scope. Stages 7+8 could parallelize but correctness requires sequential default.

---

### 2026-07-08 — Sub-skill invocation scope
**Options:** Top-level slash commands vs parent-callable only  
**Decision:** Parent-callable only; sub-skill frontmatter `name:` prefixed `deep-research-stage-0N`  
**Provenance:** research (codebase patterns)  
**Rationale:** 10 top-level stage commands would pollute harness namespace. Prefix prevents collision if ever installed standalone. Parent SKILL.md is the single entry point.

---

### 2026-07-08 — Native binary scope
**Options:** Include `prometheus-research` binary scaffold in this phase vs defer  
**Decision:** Defer to `phase-prometheus-research-binary`  
**Provenance:** implicit (scope management)  
**Rationale:** Binary generation via native-agent is a multi-hour process with its own KBD lifecycle. SKILL.md pipeline is a complete, functional deliverable. Binary adds streaming/checkpointing (P3 features).

---

### 2026-07-08 — Model routing policy
**Options:** Frontier-only vs tiered (frontier/medium/small)  
**Decision:** Tiered — frontier for reasoning-heavy stages, medium for execution stages, small for mechanical stages  
**Provenance:** research (liter-llm-bridge patterns + master spec)  
**Rationale:** Consistent with pack-wide liter-llm patterns. Saves ~40-60% token cost vs frontier-only by routing search/retrieve/collect/cite/export to cheaper classes.

---

### 2026-07-08 — .research package format
**Options:** Custom format vs OKF v0.1 base with extensions  
**Decision:** OKF v0.1 base with Prometheus research extensions  
**Provenance:** implicit (follows CLAUDE.md §Karpathy LLM Wiki)  
**Rationale:** OKF already vendored at `shared/references/okf-v0.1.md`. Permissive consumption rule means extensions don't break OKF consumers. Aligns with Karpathy wiki pattern.
