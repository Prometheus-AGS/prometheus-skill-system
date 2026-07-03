# Deep Research + Feynman Learning Integration Patterns

## Research Report for the Prometheus Skill Pack

**Date:** 2026-07-03  
**Researcher:** Prometheus Research Specialist  
**Topic:** Integration patterns between the Deep Research pipeline and the Feynman Learning Loop ecosystem

---

## Executive Summary

This report investigates ten integration patterns between deep research systems and the Feynman Learning Loop (PMPO-based learning cycle) within the Prometheus ecosystem. The findings are grounded in evidence from educational psychology, information science, multi-agent AI research, and knowledge management systems. Key conclusions:

1. **Deep research is a natural learning primitive** — inquiry-based learning (IBL) frameworks explicitly treat research as a constructivist learning cycle.
2. **Research outputs can auto-generate curriculum DAGs** — concept extraction from research findings, coupled with topological sorting, produces learnable prerequisite networks.
3. **The Feynman Technique validates research through self-explanation** — meta-analytic effect sizes of g=0.55 for both self-explanation and retrieval practice provide strong empirical backing.
4. **The `.research` package maps directly to the Karpathy LLM Wiki** — both are structured, compounding knowledge artifacts; the Agent-Native Research Artifact (ARA) protocol provides a four-layer convergence model.
5. **Research methodology is trackable as a competency** — ACRL's six frames and information literacy surveys provide proven taxonomies for learner models.
6. **Breadth-first vs. depth-first research strategies** correspond to the Feynman loop's `horizontal_escalation` and `recursion_floor` patterns, respectively.
7. **"Learning by researching" and "researching by learning" are bidirectional** — practitioner research and self-study frameworks demonstrate this reciprocity in teacher education and professional development.

---

## Table of Contents

1. [Deep Research as a Learning Primitive](#1-deep-research-as-a-learning-primitive)
2. [Research Findings → Curriculum Inputs](#2-research-findings--curriculum-inputs)
3. [Feynman Loop Validating Research](#3-feynman-loop-validating-research)
4. [`.research` Package → Karpathy Wiki](#4-research-package--karpathy-wiki)
5. [Learner Model Tracking Research Skills](#5-learner-model-tracking-research-skills)
6. [`learn-kb` Using Deep Research Outputs](#6-learn-kb-using-deep-research-outputs)
7. [`learn-survey` Using Research Findings](#7-learn-survey-using-research-findings)
8. [`learn-plan` Using Research Scope](#8-learn-plan-using-research-scope)
9. [Recursion and Escalation in Research](#9-recursion-and-escalation-in-research)
10. [Best Practices: Learning by Researching](#10-best-practices-learning-by-researching--researching-by-learning)
11. [Integration Architecture Recommendations](#11-integration-architecture-recommendations)
12. [References](#12-references)

---

## 1. Deep Research as a Learning Primitive

### 1.1 Theoretical Foundation: Inquiry-Based Learning

Inquiry-based learning (IBL) is a pedagogical approach that actively engages students in the process of scientific inquiry and knowledge construction. The core insight is that **research is not separate from learning — it is a form of learning**.

Ludwig Huber's widely-used definition states that inquiry-based teaching is distinguished by students "(co-)designing, experiencing and reflecting on the process of a research project aimed at gaining knowledge" — from question development through methods selection to results presentation. [^1]

The ACRL Framework for Information Literacy for Higher Education explicitly frames "Research as Inquiry" as one of its six core frames:

> "Research is iterative and depends upon asking increasingly complex or new questions whose answers in turn develop additional questions or lines of inquiry in any field." [^2]

This framework identifies knowledge practices including:
- Formulating questions based on information gaps
- Determining appropriate scope of investigation
- Breaking complex questions into simple ones
- Monitoring gathered information for gaps or weaknesses
- Synthesizing ideas from multiple sources
- Drawing reasonable conclusions based on analysis

These are **exactly the stages of the Deep Research pipeline** (Planner → Search → Retrieve → Collect → Verify → Resolve → Graph → Cite → Report → Export).

### 1.2 Research-First Learning as a Learning Primitive

The Feynman Learning Loop's `learn-survey` skill assesses prior knowledge before instruction. A "research-first" variant would **use the research process itself as the diagnostic** — the learner's ability to formulate questions, search for sources, and evaluate findings directly reveals their current knowledge state.

| Traditional Learning | Research-First Learning |
|---|---|
| Curriculum → Study → Assess | Question → Research → Explain → Gap → Re-research |
| Teacher provides content | Learner discovers content |
| Assessment is separate from learning | Research IS the assessment |
| Knowledge is received | Knowledge is constructed |

The UNESCO-recommended four-step inquiry process maps directly: [^3]
1. Set a challenge → **Planner** stage
2. Encourage active investigation → **Search + Retrieve + Collect**
3. Make generalizations → **Verify + Resolve + Graph**
4. Reflect → **Cite + Report + Export**

### 1.3 Evidence for Research as a Learning Strategy

A meta-analysis of 72 empirical studies on inquiry-based learning found that **guidance is pivotal** — the specificity of guidance matters less than its presence. [^4] This suggests that the Prometheus deep-research pipeline, with its structured 10-stage guidance, provides the scaffolding necessary for research to function as an effective learning primitive.

The constructivist foundation is clear: "Inquiry-based teaching belongs to the group of constructivist forms of teaching and learning" where "students acquire knowledge independently and thus construct it." [^1]

**Integration Pattern 1:** The Deep Research pipeline should be exposed as a **learning primitive** in the Feynman loop — not just a tool for producing reports, but as a `learn-research` skill that is itself a learnable competency.

---

## 2. Research Findings → Curriculum Inputs

### 2.1 From Research Output to Concept DAG

The Prometheus `learn-plan` skill already uses concept DAGs (directed acyclic graphs) stored in surreal-memory. The research pipeline can auto-generate these DAGs from its findings.

The SP-TeachLLM framework demonstrates this pattern: its Curriculum Decomposition Module (CDM) "combines LLM-driven semantic reasoning with pedagogical theory" to:
- Conduct fine-grained semantic analysis of learning objectives
- Apply Bloom's Taxonomy to organize objectives across cognitive levels
- **Construct a knowledge graph that captures inter-concept dependencies and prerequisite relations** [^5]

Formally, if O denotes the set of learning objectives, the decomposition function D(oi) produces sub-objectives {s1, s2, ..., sn}, and hierarchical relationships are represented as a DAG G=(V,E) where V denotes nodes and E denotes dependencies. [^5]

### 2.2 Curriculum Prerequisite Networks

Aldrich (2015) introduced the "Curriculum Prerequisite Network" (CPN) as a directed acyclic graph where courses are nodes and prerequisite relationships are edges. The network reveals: [^6]
- **High indegree courses** = "information sinks" requiring substantial prior knowledge
- **High outdegree courses** = "information sources" or hubs feeding downstream courses
- **High betweenness nodes** = bridges linking separate sub-graphs; these become bottlenecks

Topological sorting of the DAG provides a natural semester-by-semester sequence, and metrics like prerequisite depth, average chain length, and maximum chain length give quantitative estimates of minimum time to completion. [^7]

### 2.3 Auto-Extracting Curriculum from Research

The Deep Research pipeline can feed `learn-plan` via:

| Research Stage | Curriculum Output |
|---|---|
| Search → Retrieve | Raw concept list from sources |
| Collect → Verify | Filtered, validated concept set |
| Resolve → Graph | **Concept DAG with edges = prerequisite relations** |
| Cite → Report | Annotated curriculum with source provenance |
| Export | `curriculum.json` + `concept-DAG.json` |

NoteConnection (a knowledge DAG tool) demonstrates automatic "curriculum generation" through topological sort: [^8]
- **Domain Learning**: Master an entire concept cluster via topological sort
- **Diffusion Learning**: Find the most efficient path to a specific goal (shortest path + prerequisites)
- **Smart Strategies**: Choose "Foundational" (base-first) or "Core" (importance-first) sorting

### 2.4 Difficulty Estimation from Research Depth

SP-TeachLLM dynamically adjusts decomposition granularity: "beginners receive finer-grained content, while advanced learners engage with broader conceptual groupings." [^5] This maps to the Prometheus learner model's per-concept mastery tracking.

A data-driven framework for L2 instruction uses Item Response Theory (IRT) to estimate topic difficulty, then constructs topic-level knowledge graphs as DAGs to capture prerequisite relations. [^9] This demonstrates that **research depth (number of sources, complexity of synthesis) correlates with concept difficulty** — which `learn-plan` can use for time estimation.

**Integration Pattern 2:** The `Graph` stage of the Deep Research pipeline should emit a **validated concept DAG** that feeds directly into `learn-plan`'s curriculum engine. The `Export` stage should produce `curriculum.json` with topological ordering and difficulty estimates.

---

## 3. Feynman Loop Validating Research

### 3.1 The Feynman Technique as Epistemic Validation

The Feynman Technique (explain → identify gaps → fill gaps → simplify) is a specific application of **self-explanation** and **retrieval practice** — two of the most evidence-backed learning strategies.

A meta-analysis of 64 reports found that **self-explanation prompts improved learning with an overall effect size of g = 0.55**. A large meta-analysis of testing vs. restudying found a **median testing-effect size of g = 0.55 across 159 effect sizes from 61 studies**. [^10]

The Feynman Technique works because:
1. **Explaining is a learning strategy** — not just a check for understanding
2. **"Explain it to a beginner" forces clear definitions** — fewer hidden assumptions, fewer memorized phrases
3. **Voice-to-text captures hesitation** — revealing exactly where knowledge is fragile [^10]

### 3.2 Applying Feynman to Research Findings

The standard Feynman loop applies to research as follows:

| Feynman Step | Research Application |
|---|---|
| **Explain** | Summarize research findings in plain language as if teaching a novice |
| **Grade** | Evaluate explanation clarity, accuracy, and completeness |
| **Gap** | Identify concepts you cannot explain simply, logical leaps, unsupported claims |
| **Recurse** | Return to sources (or search for new ones) to fill identified gaps |

The Feynman Technique Tutor implementation explicitly identifies these gap indicators: [^11]
- Points where the explainer resorts to jargon or hand-waving ("it just works")
- Logical leaps — jumping from A to C without explaining B
- Vague or circular definitions
- Missing "why" — describing WHAT happens but not WHY
- Oversimplification that loses accuracy
- Contradictions or inconsistencies
- Areas skipped over or rushed through

### 3.3 Research-Specific Gap Analysis

When validating research findings, the Feynman loop should be extended with research-specific gap types:

| Gap Type | Indicator | Action |
|---|---|---|
| **Source gap** | Cannot explain which source supports a claim | Re-verify citation; search for alternative sources |
| **Methodology gap** | Cannot explain how a conclusion was reached | Re-read methods section; search for methodology critiques |
| **Synthesis gap** | Cannot explain how two sources connect | Build cross-reference; search for bridging literature |
| **Contradiction gap** | Cannot reconcile conflicting sources | Flag contradiction; search for resolution or meta-analysis |
| **Recency gap** | Cannot explain if findings are still current | Check publication dates; search for recent updates |

The Feynman Technique Tutor's "Teaching Test" provides five validation criteria directly applicable to research: [^11]
1. **The 5-Year-Old Test**: Can you explain in 2-3 sentences?
2. **The Follow-Up Barrage**: 5 rapid-fire "but why?" questions
3. **The Edge Case Test**: Unusual scenario application
4. **The Connection Test**: Connect to something already known
5. **The Misconception Test**: Explain why a common misconception is wrong

### 3.4 Feynman as a Quality Gate for Research

The Feynman loop should function as a **quality gate** between the Deep Research pipeline's `Report` and `Export` stages:

```
Report → Feynman Explain → Grade → Gap? → Re-research (back to Search)
                    ↓ No gaps
              Export → .research package
```

This creates a **research-quality loop** where the researcher's ability to explain findings determines whether the research is considered complete.

**Integration Pattern 3:** The Feynman loop (`feynman-loop` skill) should be callable as a **validation stage** after the Deep Research pipeline's `Report` stage. Any gaps identified trigger re-research (return to `Search` or `Retrieve`).

---

## 4. `.research` Package → Karpathy Wiki

### 4.1 The Karpathy LLM Wiki Pattern

Andrej Karpathy's LLM Wiki pattern (April 2026) defines a three-layer architecture: [^12]

1. **Raw sources** — immutable input documents (articles, papers, transcripts). The LLM reads but never modifies these.
2. **The wiki** — LLM-generated markdown files: summaries, entity pages, concept pages, comparisons, synthesis. The LLM owns this layer entirely.
3. **The schema** — `CLAUDE.md` or `AGENTS.md` defining wiki structure, conventions, and workflows.

The key insight: **"The wiki is a persistent, compounding artifact."** Cross-references are already there. Contradictions have already been flagged. Synthesis reflects everything read. [^12]

Karpathy's phrase: **"Obsidian is the IDE. The LLM is the programmer. The wiki is the codebase."**

### 4.2 The `.research` Package as Wiki Input

The Prometheus `.research` package (output of the 10-stage Deep Research pipeline) contains:
- Structured findings with citations
- Knowledge graph of concepts and relationships
- Verified claims with source provenance
- Contradictions flagged and resolved
- Synthesis across multiple sources

This maps directly to the wiki layer:

| `.research` Component | Wiki Destination |
|---|---|
| `report.md` | `wiki/analyses/<topic>.md` |
| `knowledge-graph.json` | `wiki/concepts/` + `wiki/entities/` + `[[wikilinks]]` |
| `citations.json` | `wiki/sources/` (source summary pages) |
| `contradictions.md` | `wiki/contradictions.md` (flagged conflicts) |
| `findings/` | `wiki/<topic>/findings.md` |

### 4.3 The Agent-Native Research Artifact (ARA) Protocol

A recent open-source protocol (March 2026) proposes the **Agent-Native Research Artifact (ARA)**, a file-system protocol organized across four interlocking layers: [^13]

1. **Cognitive Layer** (`/logic`) — structured scientific reasoning: problem.md, claims.md, experiments.md, related-work.md, solution/heuristics.md
2. **Physical Layer** (`/src`) — executable code kernel
3. **Exploration Graph** (`/trace`) — full branching research trajectory including dead ends
4. **Evidence Layer** (`/evidence`) — raw empirical results grounding every claim

This ARA protocol demonstrates that **the `.research` package can be more than a report** — it can be a machine-executable knowledge package that eliminates the "Storytelling Tax" (discarded dead ends) and "Engineering Tax" (missing implementation details). [^13]

On PaperBench and RE-Bench, ARA raised question-answering accuracy from 72.4% to 93.7% and reproduction success from 57.4% to 64.4%. [^13]

### 4.4 Feeding the Wiki

The integration flow should be:

```
Deep Research Pipeline → .research package → Compiler → Wiki pages
                                           ↓
                                    knowledge-forge / swarmvault
                                           ↓
                              index.md + log.md + concept pages
                                           ↓
                              Karpathy Loop (focus → reflect → ingest)
```

SwarmVault (an open-source implementation) provides the exact infrastructure: [^14]
- `raw/` → immutable copies of ingested material
- `wiki/` → generated markdown pages, graph reports, context packs
- `state/graph.json` → machine-readable knowledge graph
- `state/retrieval/` → local search index
- Contradiction detection with `lint --conflicts`
- Token-budgeted context packs for agent handoff

**Integration Pattern 4:** The `.research` package should be compiled into wiki pages via a **research-to-wiki compiler** skill. The knowledge graph from the `Graph` stage becomes the wiki's `[[wikilinks]]` structure. Contradictions from the `Resolve` stage are written to `wiki/contradictions.md`.

---

## 5. Learner Model Tracking Research Skills

### 5.1 Information Literacy as a Competency Framework

The ACRL Framework identifies six "frames" of information literacy, each with knowledge practices and dispositions: [^2]

1. **Authority Is Constructed and Contextual**
2. **Information Creation as a Process**
3. **Information Has Value**
4. **Research as Inquiry**
5. **Scholarship as Conversation**
6. **Searching as Strategic Exploration**

Each frame maps to trackable concepts in the Prometheus learner model:

| ACRL Frame | Prometheus Concept | Mastery Levels |
|---|---|---|
| Research as Inquiry | `research.question-formulation` | Novice → Strategic → Expert |
| Searching as Strategic Exploration | `research.search-strategy` | Single-source → Multi-source → Iterative |
| Authority Is Constructed | `research.source-evaluation` | Surface → CRAAP test → Epistemic analysis |
| Information Creation as a Process | `research.synthesis` | Summary → Integration → Novel synthesis |
| Scholarship as Conversation | `research.citation-practice` | Mention → Attribute → Position in discourse |

### 5.2 The IL-HUMASS Survey: Granular Skill Tracking

The IL-HUMASS survey provides a validated instrument for tracking 26 information literacy competencies across four categories: [^15]

| Category | Skills (examples) |
|---|---|
| **Searching** | Using printed/electronic sources, catalogues, search strategies, terminology |
| **Evaluation** | Assessing quality, recognizing author ideas, determining currency, knowing relevant authors |
| **Processing** | Systematizing information, abstracting, using reference managers, statistical programs |
| **Communication** | Public communication, writing, academic presentations, disseminating online |

These map directly to the Deep Research pipeline stages:
- Searching → `Search` + `Retrieve`
- Evaluation → `Verify` + `Resolve`
- Processing → `Collect` + `Graph`
- Communication → `Cite` + `Report` + `Export`

### 5.3 Research Competency in Professional Curricula

A systematic review of undergraduate and master's programs found that students who view research skills as valuable for their future careers "reap a range of benefits for their current education as well as their careers." [^16] The recommendation is to:
- Create learning outcomes targeting specific research skills
- Use backward design approach for course design
- Explicitly state targeted skills on syllabi
- Ensure constructive alignment between course components

This supports treating research methodology as a **first-class learnable competency** in the Prometheus learner model, not just a tool for producing outputs.

### 5.4 Cognitive Diagnostic Assessment for Research Skills

Cognitive Diagnostic Assessment (CDA) using the DINA (Deterministic Inputs, Noisy "And" gate) model can assess fine-grained mastery of research skills. [^17] The DINA model:
- Defines attributes (fine-grained skills) as latent binary variables
- Estimates probability of mastery for each attribute
- Can infer unobserved skill mastery from observed performance on related skills
- Provides interpretable mastery profiles (unlike black-box Deep Knowledge Tracing)

When combined with graph neural networks (GraphSAGE or GCN), the system can propagate knowledge state estimates from observed skills to unobserved, related skills based on graph distance. [^18]

**Integration Pattern 5:** The learner model should include a **"Research Literacy" competency cluster** with 20+ trackable concepts mapped to the ACRL/IL-HUMASS frameworks. Use Bayesian Knowledge Tracing (BKT) with graph propagation for mastery estimation.

---

## 6. `learn-kb` Using Deep Research Outputs

### 6.1 Research Packages as Knowledge Base Content

The `learn-kb` skill manages knowledge sources for Dify, palace, local, and URL sources. Deep research outputs are natural additions to this knowledge base.

Vectorize's Deep Research feature demonstrates the pattern: [^19]
- Analyzes data from the vector database (populated by a pipeline)
- Optionally enriches findings with web search data
- Generates structured reports following custom templates
- Bridges private knowledge with public information

The key benefit: "Uncover connections and patterns across your entire knowledge base that aren't apparent in isolated Q&A." [^19]

### 6.2 The KB Ingestion Pipeline

For Prometheus, the `learn-kb` ingestion flow for research outputs should be:

```
.research package
├── raw/                    → KB source (immutable, indexed)
├── report.md             → KB synthesis (chunked, embedded)
├── knowledge-graph.json  → KB graph structure (nodes + edges)
├── citations.json          → KB provenance links
└── contradictions.md     → KB conflict annotations
```

Each component serves a different KB function:
- **Raw sources** → Full-text search, re-extraction, re-synthesis
- **Report** → Primary retrieval target (pre-synthesized, citation-rich)
- **Knowledge graph** → GraphRAG traversal, structured querying
- **Citations** → Provenance tracking, source credibility scoring
- **Contradictions** → Uncertainty quantification, epistemic health

### 6.3 Structured Research Artifacts as Knowledge Atoms

The Agent-Native Research Artifact (ARA) protocol defines a **Claim** as "an atomic, verifiable statement extracted from a source" with confidence scores, provenance, and dependency chains. [^20] Quicky-Wiki (another implementation) extends this to:
- **Epistemic Events**: changes in belief (created, reinforced, challenged, weakened, superseded, resolved)
- **Knowledge Diff**: what's new, reinforced, challenged when ingesting a new source
- **Cascade**: when a foundational claim is challenged, confidence changes propagate through dependent claims [^20]

This is exactly the kind of structured knowledge that `learn-kb` should ingest and maintain.

### 6.4 The Compound Knowledge Model

SwarmVault's approach is most aligned with Prometheus: [^14]
- Every pipeline run, search, and query enriches the knowledge base
- Each subsequent interaction starts from a richer baseline
- Entity profiles evolve, clause patterns accumulate, cross-document relationships persist
- Full interaction chronicle is maintained

**Integration Pattern 6:** `learn-kb` should treat `.research` packages as **first-class knowledge sources** with structured ingestion: raw documents go to the source layer, synthesized reports to the retrieval layer, knowledge graphs to the graph layer, and contradictions to the epistemic health layer.

---

## 7. `learn-survey` Using Research Findings

### 7.1 Research-Based Diagnostic Assessment

Diagnostic assessment "assesses learners' prior knowledge" before instruction begins. [^21] The NEEDU report (South Africa) formalizes the assessment loop: [^22]

```
Diagnostic Assessment → Formative Assessment → Analysis → Feedback → Action → Summative Assessment
```

The `learn-survey` skill can use research findings to generate **research-based diagnostic questions**:

| Diagnostic Goal | Research-Based Question Type |
|---|---|
| Assess prior knowledge of a topic | "Explain [concept] as if to a beginner" (Feynman prompt) |
| Identify misconceptions | Present a common misconception; ask for critique |
| Gauge source evaluation skill | Present two conflicting sources; ask for resolution |
| Assess synthesis ability | Present three related findings; ask for integration |
| Measure inquiry skill | Open-ended: "What questions should we ask about [topic]?" |

### 7.2 Cognitive Diagnostic Assessment for Fine-Grained Placement

Cognitive Diagnostic Assessment (CDA) using the DINA model can get "the knowledge of students' mastery of fine-grained knowledge." [^17] A study on scientific explanation concepts used TIMSS test items coded against eight attributes to assess Grade 4 students. [^17]

For the Prometheus learner model, research-based diagnostic questions can be:
1. **Generated from the research knowledge graph** — each concept node becomes a potential diagnostic item
2. **Calibrated via IRT** — item difficulty parameters estimated from research complexity
3. **Attribute-tagged** — each question maps to specific research skills (questioning, searching, evaluating, synthesizing)

### 7.3 Research as the Diagnostic Instrument

The most powerful integration is to use the **research process itself as the diagnostic instrument**:

| Step | Diagnostic Signal | What It Measures |
|---|---|---|
| Formulate research question | Question quality, scope appropriateness | `research.question-formulation` |
| Search for sources | Source diversity, strategy variation | `research.search-strategy` |
| Evaluate sources | CRAAP criteria application | `research.source-evaluation` |
| Synthesize findings | Integration depth, coherence | `research.synthesis` |
| Identify gaps | Self-awareness of knowledge boundaries | `research.metacognition` |
| Explain findings | Feynman explanation quality | `research.communication` |

This is a **performance-based diagnostic** — far richer than multiple-choice questions.

**Integration Pattern 7:** `learn-survey` should be able to generate **diagnostic research tasks** from the knowledge graph. The learner's performance on a mini-research task (not a quiz) provides the diagnostic signal for curriculum placement.

---

## 8. `learn-plan` Using Research Scope

### 8.1 Research Depth → Concept Complexity → Time Estimates

The `learn-plan` skill's adaptive curriculum planner uses concept DAGs to estimate learning paths. Research scope provides the input for these complexity estimates.

Task complexity research shows that "as a task's complexity level increases, an individual needs to process more information, leading to a higher demand for attentional resources." [^23] Participants make larger time estimation errors during high-complexity tasks. [^23]

For curriculum planning, the following complexity indicators can be derived from research outputs:

| Research Indicator | Complexity Signal | Time Estimate Adjustment |
|---|---|---|
| Number of prerequisite concepts | Higher = more complex | +time per additional prerequisite |
| Max DAG depth | Deeper = more foundational | +time for depth-first traversal |
| Number of conflicting sources | More = more nuance needed | +time for resolution practice |
| Synthesis breadth | Wider = more integration | +time for cross-domain connections |
| Source recency span | Older = may need updating | +time for recency verification |

### 8.2 IRT-Based Difficulty Estimation

Item Response Theory (IRT) provides a principled framework for estimating concept difficulty. A three-parameter IRT model estimates: [^9]
- **Discrimination** (a): how well the item distinguishes between ability levels
- **Difficulty** (b): the ability level at which 50% of test-takers succeed
- **Guessing** (c): probability of success by random guessing

For curriculum planning, research-derived concepts can be:
1. **Rated for difficulty** by expert assessment or historical performance data
2. **Placed on an IRT scale** to estimate learner ability required
3. **Sequenced** using the DAG structure with difficulty as a constraint

### 8.3 Dynamic Granularity Adjustment

SP-TeachLLM demonstrates dynamic granularity: "Decomposition granularity is dynamically adjusted according to task complexity and learner proficiency." [^5] This maps directly to `learn-plan`:

- **Beginner learner + complex topic** → Fine-grained decomposition, longer path
- **Advanced learner + complex topic** → Broader conceptual groupings, shorter path
- **Beginner learner + simple topic** → Moderate granularity, standard path
- **Advanced learner + simple topic** → Minimal decomposition, rapid path

The research pipeline's `Graph` stage outputs a DAG annotated with complexity estimates, which `learn-plan` uses to adjust granularity dynamically.

### 8.4 Topological Complexity Metrics

From curriculum network analysis, the following metrics predict curriculum complexity: [^7]
- **Prerequisite depth**: longest path from any root to the concept
- **Average chain length**: mean path length through the DAG
- **Maximum chain length**: longest path in the entire graph
- **Betweenness centrality**: concepts that bridge otherwise separate domains (bottlenecks)
- **In-degree**: number of prerequisites (information sinks)

**Integration Pattern 8:** `learn-plan` should consume the **annotated concept DAG** from the Deep Research pipeline's `Graph` stage, using DAG depth, betweenness, and IRT difficulty estimates to generate learning path time estimates and dynamically adjust decomposition granularity.

---

## 9. Recursion and Escalation in Research

### 9.1 Depth-First vs. Breadth-First Research Strategies

The Feynman loop's `recursion_floor` and `horizontal_escalation` patterns have direct analogues in research strategy.

**Depth-First Search (DFS) / `recursion_floor`:**
- Follows one path as far as possible before backtracking
- Memory efficient: O(d) where d = depth
- Best for: exploring specific topics deeply, when the solution is likely deep in a branch
- Analogous to: drilling down on a single gap concept until mastery [^24]

**Breadth-First Search (BFS) / `horizontal_escalation`:**
- Explores all nodes at current depth before going deeper
- Memory intensive: O(b^d) where b = branching factor
- Best for: finding shortest path, discovering all perspectives, initial exploration
- Analogous to: surveying all sub-topics before diving into any one [^24]

### 9.2 Anthropic's Research Lead Pattern

Anthropic's multi-agent research pattern explicitly classifies queries: [^25]

> **Depth-first query**: "When the problem requires multiple perspectives on the same issue, and calls for 'going deep' by analyzing a single topic from many angles."

> **Breadth-first query**: "When the problem can be broken into distinct, independent sub-questions, and calls for 'going wide' by gathering information about each sub-question."

> **Straightforward query**: "When the problem is focused, well-defined, and can be effectively answered by a single focused investigation."

The pattern then delegates to sub-agents based on complexity:
- Simple: 1 subagent
- Standard: 2-3 subagents
- Medium: 3-5 subagents
- Complex: 5+ subagents [^25]

### 9.3 Pliny: Recursive Research with Configurable Depth

Pliny (an open-source recursive research agent) implements a three-step pipeline: [^26]
1. **Decompose** — break topic into N focused subtopics
2. **Fan-out** — research all subtopics in parallel
3. **Synthesize** — merge findings into a single report

Pliny's roadmap includes planned features directly relevant to Prometheus: [^26]
- **Recursive depth**: configurable recursion where sub-agents spawn sub-agents, with a critic loop identifying gaps and triggering re-exploration
- **Configurable branching factor**: sub-agents decide how many sub-subtopics to decompose into
- **Model-aware delegation**: fast models handle more breadth; slow models go deep on fewer topics
- **Shared memory across sub-agents**: findings from one inform others in real-time

### 9.4 Recursive Agent Optimization (RAO)

Recursive Agent Optimization (RAO) demonstrates that recursion depth scales with task difficulty: [^27]
- Easy problems stay shallow
- Hard problems unlock deeper trees
- Up to 2.5× wall-clock speedup on parallel tasks
- On sequentially dependent tasks (like multi-hop research), recursive agents still learn substantially faster

RAO results show that on tasks uniquely solved by the recursive agent, the average max depth was 4 — versus 2.9 for tasks solved by both single and recursive agents. [^27]

### 9.5 Combining BFS and DFS

Research shows that combining BFS and DFS "in a single algorithm combines the complementary strengths of both." [^28] For research, the hybrid strategy is:

1. **BFS phase**: Survey all sub-topics at depth 1 (horizontal escalation) to build a map of the territory
2. **DFS phase**: Drill into the most promising/important sub-topics (recursion floor) based on the survey
3. **Iterative deepening**: Return to BFS at deeper levels if the DFS reveals new branches

This mirrors the Prometheus Loops Architecture: [^29]
- **L0 harness micro-loop**: fast iteration on a single task (DFS on one concept)
- **L1 tactical KBD loop**: breadth across a module (BFS across sub-topics)
- **L2 strategic evolver loop**: depth-first on strategic gaps
- **L3 outer standing loop**: full curriculum breadth with periodic depth reviews

**Integration Pattern 9:** The Deep Research pipeline should support **query-type classification** (depth-first vs. breadth-first vs. straightforward) and dynamically select research strategy. The Feynman loop's `recursion_floor` should be configurable per research task, and `horizontal_escalation` should trigger parallel sub-agent research for breadth-first queries.

---

## 10. Best Practices: Learning by Researching / Researching by Learning

### 10.1 The Bidirectional Relationship

The relationship between learning and research is bidirectional. The Cambridge book *Learning to Research and Researching to Learn* (2020) frames this explicitly: [^30]

> "Being an educator involves continual reflection on practice to improve student learning and engagement." The book covers "all aspects of educational research, from how to conduct and engage with research, to how to collect, organise and analyse data."

The key insight: **research skills and domain knowledge develop simultaneously** through the same activity.

### 10.2 Practitioner Research and Self-Study

In teacher education, "researching their own practices can go beyond the resolution of concrete problems and overcome the classroom boundaries, giving voice to teachers and making them constructors of educational knowledge." [^31]

The arguments for teacher-as-researcher apply directly to Prometheus learners: [^31]
- The investigative dimension allows learners to become "constructors of professional knowledge and not only users of knowledge produced by others"
- It facilitates "development of questioning competencies" and "learning from their own teaching practice"

Loughran (2007) argues that enacting a pedagogy of teacher education requires: [^32]
> "a deep understanding of practice through researching practice. In order to develop such a deep understanding, it is important not to be constrained by a teacher educator's perspective but to actively seek to better understand the perspective of students of teaching."

This maps to the Feynman loop: **researching your own understanding (practice) is how you deepen it**.

### 10.3 Inquiry-Based Learning Best Practices

Evidence-based best practices for inquiry-based learning include: [^33]
- **Scaffolding**: provide guidance — the meta-analysis of 72 studies found guidance is pivotal regardless of specificity [^4]
- **Authentic problems**: real-world, ill-structured problems increase engagement and transfer
- **Collaborative inquiry**: peer-supported, teacher-facilitated environments for complex tasks
- **Iterative reflection**: in-process critique, peer feedback, and critical reflection on the inquiry process
- **Metacognitive regulation**: focus on thinking skills, develop a culture of inquiry, support inquiry discourse
- **Conceptual understanding**: provide information on the research topic and focus on conceptual understanding

### 10.4 The Four Levels of Inquiry

Inquiry-based learning operates on a spectrum from teacher-directed to student-directed: [^1]

| Level | Teacher Role | Student Role |
|---|---|---|
| **Structured Inquiry** | Presents question, prescribes process | Investigates through given process |
| **Guided Inquiry** | Presents question, provides feedback | Designs own process, synthesizes independently |
| **Open Inquiry** | Supports as guide | Generates questions, designs process, synthesizes |
| **Research-First Learning** | Provides schema and tools | Conducts full research, explains findings, identifies gaps |

Prometheus's deep-research + Feynman loop integration enables **Open Inquiry** and **Research-First Learning** at scale.

### 10.5 The Learning-Research Cycle

The optimal cycle integrates both directions:

```
┌─────────────────────────────────────────────────────────────────┐
│                     LEARNING-RESEARCH CYCLE                      │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│   LEARN → Research a topic using Deep Research pipeline         │
│     ↓                                                            │
│   EXPLAIN → Feynman loop: explain findings to novice            │
│     ↓                                                            │
│   GAP → Identify knowledge gaps in explanation                  │
│     ↓                                                            │
│   RE-RESEARCH → Return to Deep Research to fill gaps            │
│     ↓                                                            │
│   CURRICULUM → Extract concept DAG from new findings            │
│     ↓                                                            │
│   PLAN → learn-plan generates adaptive path through gaps        │
│     ↓                                                            │
│   PRACTICE → learn-practice targets identified weak spots       │
│     ↓                                                            │
│   RETAIN → learn-retain spaces repetitions across time          │
│     ↓                                                            │
│   CERTIFY → learn-certify validates mastery                     │
│     ↓                                                            │
│   WIKI → Research findings compiled into Karpathy Wiki        │
│     ↓                                                            │
│   (loop continues with new questions from the wiki)              │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

**Integration Pattern 10:** The Prometheus ecosystem should treat **learning and research as a single bidirectional cycle**, not separate activities. The Deep Research pipeline is the "explore" phase; the Feynman loop is the "consolidate" phase; the Karpathy Wiki is the "accumulate" phase; and the learner model tracks the "develop" phase.

---

## 11. Integration Architecture Recommendations

### 11.1 Proposed Integration Points

| Source Skill | Target Skill | Integration Mechanism | Data Flow |
|---|---|---|---|
| `deep-research` | `feynman-loop` | Post-report validation | `.research/report.md` → Feynman explain → gaps → re-research |
| `deep-research` | `learn-plan` | Concept DAG export | `knowledge-graph.json` → `curriculum.json` + `concept-DAG.json` |
| `deep-research` | `learn-kb` | Structured ingestion | `.research/` → KB source layer + graph layer |
| `deep-research` | `learn-survey` | Diagnostic generation | `knowledge-graph.json` → research-task diagnostics |
| `deep-research` | Karpathy Wiki | Wiki compiler | `.research/` → `wiki/` pages + `index.md` |
| `feynman-loop` | `deep-research` | Gap-triggered research | `gap-analysis.json` → new search queries |
| `learn-grade` | `deep-research` | Competency assessment | Mastery scores → research skill taxonomy |
| `learn-plan` | `deep-research` | Research scope input | Curriculum complexity → research breadth/depth config |

### 11.2 Recommended New Skill

**`learn-research`**: A dedicated skill that treats research methodology as a learnable competency. It should:
- Define a concept taxonomy of research skills (based on ACRL/IL-HUMASS)
- Provide diagnostic tasks (mini-research assignments) for assessment
- Scaffold the Deep Research pipeline with pedagogical guidance
- Track mastery via the learner model's Rust crate
- Integrate with the Feynman loop for validation of research explanations

### 11.3 Data Format Recommendations

The `.research` package should adopt the **Agent-Native Research Artifact (ARA)** four-layer structure: [^13]

```
.research/
├── logic/                    # Cognitive layer
│   ├── problem.md            # Observations, gaps, key insight
│   ├── claims.md             # Falsifiable claims with status
│   ├── experiments.md        # Verification plan
│   └── solution/             # Design decisions with rationale
├── src/                      # Physical layer (code, data, configs)
├── trace/                    # Exploration graph
│   ├── exploration_tree.yaml # Decision DAG with dead ends
│   └── sessions/             # Session logs with provenance
├── evidence/                 # Evidence layer
│   └── README.md             # Index of all empirical data
└── wiki/                     # Compiled wiki output
    ├── index.md              # Content catalog
    ├── log.md                # Chronological record
    ├── concepts/             # Concept pages
    ├── entities/             # Entity pages
    ├── sources/              # Source summaries
    └── contradictions.md     # Flagged conflicts
```

This format:
- Is machine-readable for agents
- Is human-readable for review
- Compounds over time (each research enriches the structure)
- Feeds directly into the Karpathy Wiki pattern
- Provides provenance for every claim
- Preserves negative knowledge (dead ends, rejected hypotheses)

---

## 12. References

[^1]: Ludwig Huber, cited in "Inquiry-based learning, research-based learning" (WWF Handout Workshop). https://www.phlu.ch/_Resources/Persistent/d/e/f/2/def2a1f42c6bdd8dc94033ef681fc2b1b8f375da/WWF_Handout%20Workshop.pdf

[^2]: ACRL Framework for Information Literacy for Higher Education. https://www.ala.org/acrl/standards/ilframework

[^3]: Australian Department of Education, "Inquiry-based learning." https://www.education.gov.au/australian-curriculum/national-stem-education-resources-toolkit/i-want-know-about-stem-education/what-works-best-when-teaching-stem/inquiry-based-learning

[^4]: Lazonder & Ruth (2016), meta-analysis of 72 empirical studies on inquiry-based learning guidance. Cited in "Technologies for Education: From Gamification to AI-enabled Learning." https://files.eric.ed.gov/fulltext/EJ1386129.pdf

[^5]: SP-TeachLLM: An LLM-Driven Framework for Personalized and Adaptive Programming Education (MDPI, 2025). https://www.mdpi.com/2078-2489/16/12/1045

[^6]: Aldrich (2015), Curriculum Prerequisite Network analysis. Cited in "Major curricula as structures for disciplinary acculturation." https://www.frontiersin.org/journals/education/articles/10.3389/feduc.2023.1176876/full

[^7]: "The curriculum prerequisite network: a tool for visualizing and analyzing academic curricula." https://koineu.com/en/posts/2014/08/2014-08-25-1408_5340/

[^8]: GitHub - Jacobinwwey/NoteConnection: Knowledge DAG visualization. https://github.com/Jacobinwwey/NoteConnection

[^9]: Katinskaia et al. (2025), "Topic dependencies and difficulty in L2 curriculum." ACL Anthology. https://aclanthology.org/people/anisia-katinskaia/

[^10]: "The Feynman Technique Backed by Research" (VoiceScriber, 2026). https://voicescriber.com/feynman-technique-learning-by-explaining

[^11]: Feynman Technique Tutor (FindSkill AI). https://findskill.ai/skills/education-learning/feynman-technique-tutor/

[^12]: Karpathy's LLM Wiki (Gist, April 2026). https://gist.github.com/karpathy/442a6bf555914893e9891c11519de94f

[^13]: Agent-Native Research Artifacts (OpenReview, 2026). https://openreview.net/pdf/0479e366b6de8abd5f8c484bf9ca5f88d7dc25eb.pdf

[^14]: GitHub - swarmclawai/swarmvault: Local-first LLM Wiki. https://github.com/swarmclawai/swarmvault

[^15]: Pinto et al. (2019), "Self-learning of Information Literacy Competencies." https://crl.acrl.org/index.php/crl/article/view/16945/19431

[^16]: Vieno et al. (2022), "Broadening the Definition of 'Research Skills'." https://pdfs.semanticscholar.org/c2fc/9008a4aab3079486169784eb1470586e869d.pdf

[^17]: Hu et al. (2021), "Research on data analysis knowledge assessment based on cognitive diagnostic assessment." https://link.springer.com/article/10.1007/s12144-021-01836-y

[^18]: "Personalized Learning Platform with AI-Driven Curriculum Adaptation" (2026). https://appdesign.intelligent-ps.store/blog/personalized-learning-platform-with-ai-driven-curriculum-adaptation-for-national-education-systems

[^19]: Vectorize, "Introducing Deep Research on Your Private Data" (2025). https://vectorize.io/blog/introducing-deep-research-on-your-private-data

[^20]: GitHub - anzal1/quicky-wiki: LLM-powered knowledge compiler. https://github.com/anzal1/quicky-wiki

[^21]: EBSCO, "Diagnostic Teaching and Testing." https://www.ebsco.com/research-starters/education/diagnostic-teaching-and-testing

[^22]: NEEDU, "Schools that Work II" (South Africa, 2018). https://www.education.gov.za/Portals/0/Documents/Reports/NEEDU%20School%20that%20work%20II%202018.pdf

[^23]: Li et al. (2021), "The Effect of Task Complexity on Time Estimation in the Virtual Reality Environment." https://www.mdpi.com/2076-3417/11/20/9779

[^24]: Codecademy, "Breadth-First Search vs Depth-First Search." https://www.codecademy.com/article/bfs-vs-dfs

[^25]: Anthropic Research multi-agent: research-lead-agent system prompt. https://www.nalandaprompts.com/prompts/2fa4e75b-6a64-45f8-851a-6821407496eb

[^26]: GitHub - kevinmichaelchen/pliny: Autonomous recursive research agent. https://github.com/kevinmichaelchen/pliny

[^27]: Recursive Agent Optimization (RAO) project page. https://apga.github.io/RAO/

[^28]: Zhou & Hansen, "Combining Breadth-First and Depth-First Strategies" (AAAI 2008). https://cdn.aaai.org/Workshops/2008/WS-08-10/WS08-10-024.pdf

[^29]: Prometheus ecosystem context: Four nested loop layers (L0-L3) with durable state on disk.

[^30]: Cambridge, *Learning to Research and Researching to Learn* (2020). https://www.cambridge.org/highereducation/books/learning-to-research-and-researching-to-learn/6D533E804ABF4294FAE3E0BEF27F9110

[^31]: "Transformative Orientation in Learning to Teach Physics and Chemistry." https://pdfs.semanticscholar.org/6bad/b07a4327546598c905ee926a627a2688efbb.pdf

[^32]: Loughran (2007), cited in teacher education pedagogy research. https://repository.tml.nul.ls/bitstreams/6f4b6f83-daaa-4cd1-b81e-4a4206df6fbd/download

[^33]: IBO, "Meanings and Practices of Inquiry-Based Teaching and Learning." https://ibo.org/globalassets/new-structure/research/pdfs/inquiry-based-teaching-and-learning-final-report.pdf

---

## Appendix A: Integration Summary Matrix

| # | Research Question | Key Finding | Recommended Integration |
|---|---|---|---|
| 1 | Deep research as learning primitive | IBL frameworks explicitly treat research as learning; ACRL "Research as Inquiry" frame | Expose `deep-research` as a learning primitive |
| 2 | Research → curriculum | Concept DAG extraction + topological sorting yields curriculum | `Graph` stage emits `curriculum.json` |
| 3 | Feynman validating research | Self-explanation (g=0.55) + retrieval practice (g=0.55) | Feynman loop as post-report quality gate |
| 4 | `.research` → Karpathy Wiki | Both are compounding artifacts; ARA protocol provides 4-layer model | Research-to-wiki compiler skill |
| 5 | Learner model tracking research | ACRL 6 frames + IL-HUMASS 26 skills provide taxonomy | Add "Research Literacy" competency cluster |
| 6 | `learn-kb` using research | Structured ingestion: raw + synthesis + graph + contradictions | Treat `.research` as first-class KB source |
| 7 | `learn-survey` using research | Research tasks are performance-based diagnostics | Generate diagnostic research tasks from knowledge graph |
| 8 | `learn-plan` using research | DAG depth + IRT difficulty + betweenness = complexity | Consume annotated DAG for time estimates |
| 9 | Recursion/escalation in research | Anthropic classifies queries as BFS/DFS; RAO scales depth with difficulty | Query-type classification + dynamic strategy selection |
| 10 | Learning by researching | Bidirectional cycle: learn → research → explain → gap → re-research | Unified cycle with all phases interconnected |

---

*Report generated: 2026-07-03 CDT*  
*For: Prometheus Skill Pack — deep-research skill enhancement*  
*Classification: Foundational research with evidence and citations*
