# Decision Log — research-agent-hardening

---

### 2026-09-03 — Build-vs-adopt decisions (kbd-analyze)

| Decision | Verdict | Status | Provenance |
|---|---|---|---|
| D-01: Execution model | Harness-driven stages with a contract-enforcing driver first (option B, 21/25); daemon `--daemon-job` spawns a headless harness (option C, 21/25); daemon-native Rust pipeline rejected this phase (option A, 15/25). B and C compose, so the tie is not a contested choice | resolved | research (Feynman CLI prompt model, running `claude -p` process, CLAUDE.md codex exec note, daemon Cargo.toml) |
| D-02: Contract of record | OpenSpec capabilities `research-pipeline-execution` and `learn-model-coherence` authored at spec; `research-package-spec.md` normative; `research-manifest.schema.json` validated by jsonschema crate in Rust and python3 jsonschema with jq fallback in shell; ajv-cli rejected | resolved | research (33 existing OpenSpec capabilities, certification gate, registry checks) |
| D-03: Output root | `~/.prometheus/research/<job_id>/`; daemon constants and four hook defaults move; `RESEARCH_OUTPUT_DIR` override retained; `.research` suffix dropped | resolved | research (pack `~/.prometheus/` convention; grep of roots) |
| D-04: Slug and job id | Feynman slug rule via relocated `subject_to_slug`; package dir `<slug>-<yyyymmdd>-<4hex>`; daemon job id kept in manifest | resolved | research (content-grounding-kb.sh:35, learn-goal id shape) |
| D-05: Labels, sidecar, ledger, resume | Claim labels verified/unverified/blocked/inferred; package label derived; `<slug>.provenance.md` on every exit; plan.md ledger sections; `--resume` reads checkpoint | resolved | research (Feynman CLI AGENTS.md, deepresearch.md) |
| D-06: Agent duties | `tools:` frontmatter on four research agents; ordering and scale gate in the driver; adversarial-review adopted with a new `research` packet target | resolved | research (Feynman agent frontmatter, adversarial-review SKILL.md) |
| D-07: Learn artifact path | `artifacts/<concept-id>/<artifact-id>.json` | resolved | research (two of three readers key by concept) |
| D-08: Corpus schema | Extend `content-grounding-kb.sh` to emit `key_points[]` and `misconceptions[]`; consolidate to one copy with wrappers | resolved | research (script line 61, learn-grade Step 1) |
| D-09: learner-model RPCs | `add_gap`, `add_session`, `set_certified`; `certified_at` on ConceptState | resolved | research (main.rs dispatch, store.rs) |
| D-10: FSRS | adopt rs-fsrs 1.2.1 for scheduling; reference fsrs-rs 6.6.2 | resolved, dependency weight owed at spec | research (Tier 1, 2, 3); weight inferred, not built |
| D-11: Scoring and graph | Port renormalisation, sensitivity, claim ids, contradicts edges to python3; semantic contradictions via `kbd_complete`; G5 ordered last | resolved | research (Feynman rank and claims code, survey) |
| D-12: Timestamps | adopt chrono 0.4, same pin as learner-model | resolved | research (Cargo.toml comparison) |
| D-13: Manifest drift | No generator; template deleted, SKILL.md example and export-package.sh hand-synced, `check-research-package.sh` drift check in certification | resolved | adversarial review W2 |
| D-14: Legacy output root | 28 daemon-created dirs under `~/.research-jobs/` left in place, never read; status output names the path | resolved | adversarial review S2, ls on 2026-09-03 |
| D-15: Eval review ordering | Human review of ground truth before G2, re-baseline after | resolved | adversarial review W1, assessment Q5 |

Open: headless hook policy (spec), rs-fsrs weight (spec), cargo check precondition (plan task 1, all Rust adopt verdicts conditional on it), runtime position contention with the concurrent companion executor (operator).

### 2026-09-04 — Plan decisions (kbd-plan, after adversarial review)

| Decision | Verdict | Status | Provenance |
|---|---|---|---|
| D-16: Round order | R1 {001, 007}, R2 002, R3 003, R4 {004, 005}, R5 {006, 010, 008 after operator gate}, R6 009, R7 011; cargo-gated changes 001, 004, 009 never overlap | resolved | plan |
| D-17: G5 deferral branch | 011's dependency on 010 is conditional; if 010 is deferred the deferral is recorded here, 011 proceeds, G5 is NOT MET, scoring section marked deferred to the named successor | resolved (branch not yet taken) | adversarial review W1 |
| D-18: Daemon hook policy | daemon fires no hooks; headless child runs the real driver, which fires the four deep-research hook scripts; KBD lifecycle hooks disabled in the child via `KBD_HOOKS_DISABLED=1`; asserted in job_execution.rs | resolved | adversarial review W3, analyze open question 1 |
| D-19: FSRS replacement optional | tasks 1 and 3 of change-rah-009 are optional; tied to assessment finding 26, not to any goal; phase certifies without them | resolved | adversarial review W4 |
| D-20: Hook file scope | only `skills/research/deep-research/hooks/*.sh` are edited; `hooks/hooks.json` and `hooks/codex-hooks.json` untouched; `validate:codex` added to 011 certification defensively | resolved | adversarial review W5 |
| D-21: Compile baseline artifact | `changes/change-rah-001-.../evidence/compile-baseline.txt` holds both cargo check outputs | resolved | adversarial review S1 |

### 2026-09-08 — Onyx-parity scope decision (operator, mid-execute)

| Decision | Verdict | Status | Provenance |
|---|---|---|---|
| D-22: Onyx deep-research parity scope | The Onyx-parity work is a **subsequent top-level phase**, not a child of this one and not an amendment to its plan. `research-agent-hardening` finishes as scoped (rah-011 tasks 3–4, then rah-007 tasks 2–3 and the eval re-baseline), closes, and the new phase is created from its reflection. Rationale: the five goals of this phase (execution, learn coherence, provenance, duties, scoring) are met or nearly so and none depend on threading; the report's own design *depends on* this phase's output (stage contracts, claim labels, checkpoint, verifier ordering, derived verification_status) and instructs "resist renumbering"; a child phase would hold this phase's certification open across a multi-crate refactor | resolved | operator decision 2026-09-08 (AskUserQuestion); `docs/deep-research/deep-research-skill-vs-onyx-report.md` §4, §8, §9 |
| D-23: Report's open verifications | The report's three unresolved items — liter-llm Rust API availability from UAR `Cargo.toml`/`versions.toml`, UAR-REM-002 status for the three correctness defects, and the nature of the nested `universal-agent-runtime/crates/prometheus-skill-system` checkout — are **assess-stage questions for the new phase**, verified by command before any change is specced. The report's Onyx benchmark standing (RACE ~54, "reported as #1 at points in 2026") is recorded as an **unverified** claim until `tests/bench/` actually runs; it is not carried into the plan as established fact | resolved | operator decision 2026-09-08; report §7 and the 2026-09-08 revision's own "Open verifications" |

**Successor phase seed** (for `/kbd-new-phase` after this phase closes; the
report's change list is input to assess, not a settled plan):

- Candidate name: `deep-research-onyx-parity`
- Candidate goals, from the report's gap table G1–G10 and its revised change list:
  - Thread execution: replace the sequential internals of stages 02–04 with a constrained director plus isolated research workers and a deterministic merge, leaving the stage contracts, numbers, and validators unchanged.
  - Lossless handoff: persist per-thread fact dossiers with inline content-addressed citations and claim quotes, so synthesis reads prose evidence rather than only compressed graph claims.
  - Multi-pass report: split stage 09 into outline, parallel section writers, deterministic assembly, and a coherence editor, bound by a claim-set invariant.
  - Budgets and failure semantics: cycle and wall-clock budgets at job, director, and thread level with force-complete paths, and a ledger row for every dispatch including failures.
  - Measurement: a benchmark harness reporting RACE, effective citations, citation accuracy, and verified-claim ratio, so "on par with Onyx" becomes falsifiable.
- Carried-forward defects found by this phase's rah-011 evidence run, both install-surface and out of rah-011's file scope:
  - D-A: `com.prometheus.research.plist` sets no `PATH`, so the launchd daemon cannot resolve any harness binary. Fix is a `__PROMETHEUS_PATH__` placeholder matching `shared/launchagents/*.plist` plus the matching substitution in `scripts/install-binaries.sh`.
  - D-B: the installed plugin generation ships a pre-phase stub driver, so the daemon's `~/.claude` driver candidate resolves to a script with none of the stage contract. Fix is a reinstall; `RESEARCH_DRIVER` is the documented interim override.
