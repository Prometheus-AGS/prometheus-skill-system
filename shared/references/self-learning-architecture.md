# Self-Learning Architecture

The Prometheus skill pack implements a **self-improving execution engine** based on
the Hermes/GEPA self-learning pattern. Skills are not static — they improve from
execution experience through a four-layer feedback loop.

## The Four Layers

```
┌─────────────────────────────────────────────────────────────────┐
│ Layer 4: DISTRIBUTION (prometheus-cli, cowork-compatible)       │
│ install/uninstall/audit/verify/search across 10 platforms       │
│ Skills.toml + Skills.lock · Cedar governance · marketplace      │
├─────────────────────────────────────────────────────────────────┤
│ Layer 3: OPTIMIZATION (dspy-rs)                                 │
│ Skill → Signature · Traces → Dataset · BootstrapFewShot/MIPRO  │
│ PMPO engine · Cedar Skill Mutation PEP gates all writes         │
├─────────────────────────────────────────────────────────────────┤
│ Layer 2: KNOWLEDGE (prometheus-knowledge)                       │
│ Traces → RawDoc → WikiEntry (Karpathy method)                   │
│ compile → lint → focus → fix · TF-IDF + Graph-RAG retrieval    │
├─────────────────────────────────────────────────────────────────┤
│ Layer 1: MEMORY (surreal-memory)                                │
│ Knowledge graph · Graph-RAG · TaskStreams · Scoped memory       │
│ Temporal history · Hybrid search · Cross-session persistence    │
└─────────────────────────────────────────────────────────────────┘
```

## The Self-Learning Loop

```
EXECUTE
  Agent runs skill → full execution trace captured
  → surreal-memory: create_entity(type: "execution_trace")
  → filesystem: .prometheus/traces/{skill}/{timestamp}.json

EVALUATE
  Grader scores trace (automated metrics + LLM-as-judge)
  Failure traces get reflection: "WHY did this fail?"
  → surreal-memory: add_observations(trace_entity, {score, reflection})

COMPILE (prometheus-knowledge Karpathy pipeline)
  Successful traces → WikiEntry (lesson learned)
  Failed traces + reflection → WikiEntry (anti-pattern)
  Lint + focus cycles maintain knowledge consistency
  → surreal-memory: create_entity(type: "lesson") + relations
  → filesystem: .prometheus/wiki/{topic}.md

OPTIMIZE (dspy-rs)
  Skill prompt → dspy-rs Signature
  Execution traces → Dataset of Examples
  BootstrapFewShot collects best demos
  MIPRO optimizes instruction
  → Cedar PEP gates mutation: check(agent, skill.mutate, skill, context)
  → skill/SKILL.md updated with optimized prompt

DISTRIBUTE
  Updated skill validated (agentskills.io)
  Skills.lock checksums updated
  Audit verifies no regressions
```

## Cedar Governance

The **Skill Mutation PEP** (prometheus-cedar crate) gates all write operations
against governed skill artifacts. Default-deny: mutations require explicit permit.

| Operation | When | Cedar Action |
|-----------|------|--------------|
| Prompt optimization | dspy-rs writes back to SKILL.md | `skill.mutate` |
| Skill generation | PMPO creates new skills from gap detection | `skill.generate` |
| Skill promotion | Staging → active | `skill.promote` |
| Trace capture | Execution data collection | `trace.capture` |

Policies are Cedar text files in `policies/skill-mutation.cedar`.
Entities define agent groups and skill domains.

### Environments

| Environment | Mutation Policy |
|-------------|----------------|
| `development` | All operations permitted |
| `staging` | Mutations require validation_passed; promotions require human_approved |
| `production` | Mutations forbidden by default; trace capture requires governance_consent |

## UAR Integration

The prometheus-learn crate is designed as a **UAR-embeddable library**, not a
standalone intelligence layer. The correct dependency direction:

```
UAR's PMPO Reflect phase → calls prometheus-learn directly
prometheus-learn is a library crate with no CLI dependencies
prometheus-cli is a thin wrapper for manual invocation
```

Key integration points in UAR:
- `NativeSkillRegistry.mutate_skill()` → gates through Cedar PEP
- `Orchestrator` tool loop → calls trace capture after skill execution
- `MemoryService` → surreal-memory graph for trace + lesson storage
- `SkillService` → mutation path where Cedar evaluation happens

## Cross-Platform Trace Capture

Trace capture is a **first-class protocol**, not a Claude Code-specific hook.
Each platform adapter implements the `TraceCapture` trait:

```rust
pub trait TraceCapture {
    fn platform(&self) -> AgentKind;
    fn is_available(&self) -> bool;
    fn capture_latest(&self) -> Result<Option<PlatformTrace>>;
    fn capture_since(&self, since: &str) -> Result<Vec<PlatformTrace>>;
}
```

Implementations exist for: Claude Code, OpenCode, Codex (file-based).
Additional platforms added by implementing the trait.

## Gap Detection and Skill Spawning

When the knowledge graph reveals recurring failure patterns:

1. **Detect**: semantic_search for failure patterns with >3 occurrences
2. **Specify**: dspy-rs PMPO engine produces skill spec from gap analysis
3. **Generate**: pmpo-skill-creator builds the skill with SKILL.md + agents + hooks
4. **Verify**: execution against original failure traces
5. **Promote**: Cedar PEP gates promotion (requires human_approved + test_pass_rate >= 0.95)

## Storage Locations

| Data | Location | Format |
|------|----------|--------|
| Execution traces | `.prometheus/traces/{skill}/` | JSON |
| Knowledge wiki | `.prometheus/wiki/` | Markdown with YAML frontmatter |
| TF-IDF index | `.prometheus/wiki/.index/` | Binary |
| Cedar policies | `policies/skill-mutation.cedar` | Cedar text |
| Cedar entities | `policies/entities.json` | Cedar JSON |
| Skills.lock | `Skills.lock` | TOML-like checksums |

## Crate Map

| Crate | Role | Embeds In |
|-------|------|-----------|
| `prometheus-agents` | Platform detection, install, trace protocol | CLI + UAR |
| `prometheus-learn` | Trace capture, grading, knowledge compilation, optimization | UAR (primary), CLI (wrapper) |
| `prometheus-cedar` | Cedar Skill Mutation PEP | UAR SkillService |
| `prometheus-cli` | Thin CLI binary — invokes the libraries | Standalone binary |
