### 2026-10-04T23:40Z — kbd-analyze decisions (phase-team-learning-hardening)
- G2b: server-side query-embedding LRU (quick_cache, already in lockfile) + client lexical GET for user/global scopes. Rejected: concurrent client requests (serialising executor), multi-scope API (deferred). Provenance: research (server source inspected).
- G3a: real Cortex 2.0.3 against scratch CORTEX_DATA_DIR; BLOCKED when embedding model is not cached. Provenance: research.
- G1b: installer sets generate_memories=false + archives memory_summary.md; leftover MEMORY.md is inert (codex prompt-input probe). Provenance: research.
- G1c: ship procedure in delivery-cadence skill with profile inputs. Provenance: implicit (no contested choice).
- G3c: build rebase-regenerate.sh; npm-merge-driver reference only. Provenance: research.
No contested stack choice; no elicitation.
- 2026-10-04T23:55Z revision after adversarial review: G2b option C (lexical user/global) REJECTED — conflicts with G3b; A gated on a single-embedding latency measurement. G1b TOML: line-level edit + re-parse + backup (smol-toml/@iarna rejected: drop comments). G2a: shell wrapper + shared scratch-surreal lib.
