# Prometheus Feynman Learning Agent — Architecture Addendum
# Coach Catalog, Creator Studio, Master Certification & Video Conferencing

## Document Control

| Field | Value |
|---|---|
| **Project** | Prometheus Feynman Learning Agent (PFLA) |
| **Document** | Architecture Addendum: Coach Catalog, Master Certification & Video Conferencing |
| **Version** | 1.0.0-draft |
| **Date** | 2026-07-01 |
| **Status** | Draft for Review |
| **Related Documents** | `prometheus-feynman-learning-agent-architecture.md` (Base Architecture) |

---

## 1. Executive Summary

This addendum extends the PFLA architecture with a **learning marketplace** that transforms the Feynman loop from a solo AI-tutor experience into a **multi-agent ecosystem** where:

1. **Certified Experts** use a **Creator Studio** to build digital coaching personas grounded in their expertise and corpus.
2. **Students** learn through the Feynman loop, optionally guided by a persona coach, and can earn **Master Certification** through the Karpathy Loop's continuous improvement metrics.
3. **Certified Masters** monetize their expertise by coaching other students via **AI-augmented coaching sessions** (persona + real master) and **live video conferencing**.
4. **The Platform** enables this marketplace with a revenue-share engine, WebRTC conferencing via `flint-realtime-fabric`, and an A2A agent registry.

> **Reference Example**: The `jesus-twin` architecture (see `/Users/gqadonis/Projects/bible/`) is cited throughout this document **only as an exemplary methodology** for creating grounded, voice-faithful digital personas. It demonstrates the RAG-first grounding pattern (retrieval owns truth, adapter owns voice, coverage gate refuses out-of-corpus), multi-protocol surfaces (AG-UI, A2A, MCP, OpenAI), and admission control — all of which are the technical foundation for the Creator Studio. No religious product is proposed; the pattern is universally applicable to any domain expert.

---

## 2. High-Level Architecture: The Learning Marketplace

```
┌─────────────────────────────────────────────────────────────────────────────────────────────┐
│                              PLATFORM OPERATOR (Prometheus AGS)                              │
│  ┌─────────────────────────────────────────────────────────────────────────────────────┐ │
│  │                        PFLA CENTRAL PLATFORM (Rust Axum)                              │ │
│  │  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐  ┌────────────────┐ │ │
│  │  │ Coach Catalog   │  │ Creator Studio  │  │ Master Cert     │  │ Revenue Engine │ │ │
│  │  │ (A2A Registry)  │  │ (Corpus+Train)  │  │ (Karpathy Loop) │  │ (Billing+Share)│ │ │
│  │  │                 │  │                 │  │                 │  │                │ │ │
│  │  │ - Agent Cards   │  │ - Upload corpus │  │ - LVS tracking  │  │ - Stripe       │ │ │
│  │  │ - Capability    │  │ - Fine-tune     │  │ - Exam engine   │  │ - Master wallet│ │ │
│  │  │   matching      │  │   voice LoRA    │  │ - Badge minting │  │ - Commission   │ │ │
│  │  │ - Ratings       │  │ - Test & deploy │  │ - Revocation    │  │ - Payouts      │ │ │
│  │  │ - Discovery     │  │ - Publish Card  │  │ - Re-certify    │  │ - Analytics    │ │ │
│  │  └─────────────────┘  └─────────────────┘  └─────────────────┘  └────────────────┘ │ │
│  │  ┌─────────────────────────────────────────────────────────────────────────────────┐ │ │
│  │  │                    Video Conferencing Engine (flint-realtime-fabric)               │ │ │
│  │  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐ │ │ │
│  │  │  │ Signaling   │  │   SFU/MCU   │  │  Recording  │  │   Session Orchestration │ │ │ │
│  │  │  │ Server      │  │   Bridge    │  │  Pipeline   │  │   (Persona + Human)     │ │ │ │
│  │  │  │ (WebSocket) │  │  (Selective │  │  (Async   │  │   - AI coach whispers   │ │ │ │
│  │  │  │             │  │  Forwarding)│  │   replay)  │  │   - Real-time grading   │ │ │ │
│  │  │  │             │  │             │  │            │  │   - Gap detection       │ │ │ │
│  │  │  └─────────────┘  └─────────────┘  └─────────────┘  └─────────────────────────┘ │ │ │
│  │  └─────────────────────────────────────────────────────────────────────────────────┘ │ │
│  └─────────────────────────────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────────────────────────────────┘
                                       │                              │
                    ┌──────────────────┼──────────────────┐           │
                    │                  │                  │           │
                    ▼                  ▼                  ▼           ▼
┌─────────────────────────────────────────────────────────────────────────────────────────────┐
│                              EXTERNAL SYSTEMS & AGENTS                                       │
│  ┌──────────────────────────┐  ┌──────────────────────────┐  ┌──────────────────────────┐ │
│  │   Expert Coach Agents   │  │   Master Human Coaches  │  │   Third-Party Services   │ │
│  │   (A2A / MCP)           │  │   (Live Video + AI      │  │   - Stripe Connect       │ │
│  │   - Persona #1: Feynman │  │    Augmentation)         │  │   - ElectricSQL Cloud   │ │
│  │   - Persona #2: Socrates│  │   - Real human video     │  │   - WebRTC CDN          │ │
│  │   - Persona #3: Custom  │  │   - AI persona assistant │  │   - LLM APIs            │ │
│  │   - Fine-tuned on       │  │   - Session notes by AI  │  │   - Supabase Auth       │ │
│  │     expert corpus        │  │   - Gap analysis post    │  │   - Object Storage      │ │
│  │   - RAG-grounded,        │  │     session              │  │                          │ │
│  │     coverage-gated       │  │   - Earnings dashboard   │  │                          │ │
│  │   - Self-hosted or       │  │                          │  │                          │ │
│  │     platform-hosted      │  │                          │  │                          │ │
│  └──────────────────────────┘  └──────────────────────────┘  └──────────────────────────┘ │
└─────────────────────────────────────────────────────────────────────────────────────────────┘
                                       │
                    ┌──────────────────┼──────────────────┐
                    │                  │                  │
                    ▼                  ▼                  ▼
┌─────────────────────────────────────────────────────────────────────────────────────────────┐
│                              CLIENT LAYER (Same as Base Arch)                              │
│  ┌──────────────────────────┐  ┌──────────────────────────┐  ┌──────────────────────────┐ │
│  │   Web Browser (React 19)  │  │   Tauri Desktop          │  │   Tauri Mobile           │ │
│  │   + Coach Marketplace UI   │  │   + Video conferencing   │  │   + Video conferencing   │ │
│  │   + Creator Studio UI      │  │   + Screen sharing       │  │   + Push notifications   │ │
│  │   + Master Dashboard       │  │   + Recording playback   │  │   + Offline study mode   │ │
│  └──────────────────────────┘  └──────────────────────────┘  └──────────────────────────┘ │
└─────────────────────────────────────────────────────────────────────────────────────────────┘
```

---

## 3. Core Concept: The Certified Digital Persona

### 3.1 What Is a Digital Persona?

A **digital persona** is an AI agent that:
1. **Retrieves from a curated corpus** (the expert's published works, lectures, notes, or a licensed corpus)
2. **Speaks with a conditioned voice** (fine-tuned or prompt-conditioned to match the expert's reasoning style, analogies, and pedagogical approach)
3. **Refuses out-of-corpus** (coverage gate prevents hallucination or answering outside the expert's domain)
4. **Cites every claim** (every explanation traces back to a specific source passage)
5. **Integrates with the Feynman loop** (generates explanations, grades learner responses, identifies gaps, and escalates to human master when needed)

### 3.2 The Jesus-Twin as Methodology Example

> **This is an architectural reference only, not a product feature.**

The `jesus-twin` project demonstrates the technical methodology for creating a grounded digital persona:

| Pattern | Jesus-Twin Implementation | Generalized to Creator Studio |
|---|---|---|
| **Corpus grounding** | Synoptic Gospels, WEB (public domain) + reasoning moves graph | Expert uploads PDFs, lectures, notes, books, licensed datasets |
| **Retrieval** | Hybrid: vector (HNSW) + BM25 + graph + RRF | Same stack, generalized to any corpus format |
| **Coverage gate** | `Coverage::NoCoverage` → refusal if query not in corpus | Configurable per coach; can be strict or lenient |
| **Voice fidelity** | Gemma 4 E4B fine-tuned with Unsloth LoRA, merged for inference | Creator Studio fine-tunes open-weight models (Gemma, Qwen, Llama) with expert's examples |
| **Multi-protocol** | AG-UI, A2A, MCP, OpenAI REST | Standard for all coaches in the catalog |
| **Admission control** | `prometheus-parking-lot` gatekeeper | Platform-wide admission control for all coaches |
| **Citation** | Every claim cites verse reference | Every claim cites source document + page/section |
| **State snapshot** | Mind-map of reasoning moves + parallels | Concept graph of the expert's knowledge map |

**The generalized principle**: Any expert with a substantial corpus and a distinctive pedagogical voice can be modeled as a retrieval-grounded, voice-conditioned agent. The `jesus-twin` is simply an early proof of concept that validates the full stack — from data pipeline to multi-protocol deployment.

### 3.3 Persona Lifecycle

```
┌──────────────────────────────────────────────────────────────────────────────┐
│                         CERTIFIED DIGITAL PERSONA LIFECYCLE                      │
│                                                                                │
│  EXPERT (Human)                                                                │
│    │                                                                           │
│    ▼                                                                           │
│  ┌──────────────────┐    ┌──────────────────┐    ┌──────────────────┐          │
│  │  1. UPLOAD       │ -> │  2. FINE-TUNE    │ -> │  3. TEST &       │          │
│  │     CORPUS       │    │     VOICE        │    │     VALIDATE       │          │
│  │                  │    │                  │    │                  │          │
│  │  - PDFs, books   │    │  - LoRA on       │    │  - Coverage gate  │          │
│  │  - Lecture       │    │    Gemma/Qwen/   │    │    test: refuses  │          │
│  │    transcripts   │    │    Llama         │    │    out-of-corpus? │          │
│  │  - Notes, papers │    │  - Prompt-only   │    │  - Voice fidelity:│          │
│  │  - Licensed      │    │    alternative   │    │    expert blind   │          │
│  │    datasets      │    │    (no fine-     │    │    test           │          │
│  │                  │    │    tune)         │    │  - Pedagogical    │          │
│  │                  │    │                  │    │    quality: 3     │          │
│  │                  │    │                  │    │    peers grade    │          │
│  └──────────────────┘    └──────────────────┘    └──────────────────┘          │
│    │                                                                           │
│    ▼                                                                           │
│  ┌──────────────────┐    ┌──────────────────┐    ┌──────────────────┐          │
│  │  4. PUBLISH       │ -> │  5. DEPLOY       │ -> │  6. MONITOR &      │          │
│  │     AGENT CARD    │    │     & SERVE      │    │     IMPROVE        │          │
│  │                  │    │                  │    │                  │          │
│  │  - A2A Agent Card  │    │  - Platform-hosted│    │  - Learner ratings │          │
│  │    with capabilities│   │    (default)     │    │  - Karpathy Loop   │          │
│  │  - Tags, subjects, │    │  - Self-hosted   │    │    on coach        │          │
│  │    proficiency     │    │    (Pro creators)│    │    effectiveness   │          │
│  │  - Pricing model   │    │  - Hybrid        │    │  - Corpus drift    │          │
│  │    (free/paid)     │    │    (edge + cloud)│    │    detection       │          │
│  └──────────────────┘    └──────────────────┘    └──────────────────┘          │
│                                                                                │
│  OUTPUT: A certified, grounded, voice-faithful digital coach available in the   │
│          PFLA Coach Catalog.                                                   │
└────────────────────────────────────────────────────────────────────────────────┘
```

---

## 4. Subsystem Deep Dives

### 4.1 Creator Studio Architecture

The Creator Studio is a web application (built on the same React 19 + Vite 7 + shadcn/ui stack) that allows certified experts to create and publish coaching personas.

#### 4.1.1 Corpus Ingestion Pipeline

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                          CORPUS INGESTION PIPELINE                             │
│                                                                                │
│  Expert Uploads    ┌──────────────┐    ┌──────────────┐    ┌──────────────┐ │
│  (PDF, TXT, MD,   │   Document   │ -> │   Text       │ -> │   Chunking   │ │
│   EPUB, MP3,      │   Parser     │    │   Extraction │    │   &          │ │
│   Video)          │              │    │   (OCR + ASR)│    │   Structuring│ │
│                   │              │    │              │    │              │ │
│                   │ - PDF: pdfplum│   │ - OCR: Tesseract│  │ - Semantic   │ │
│                   │ - MD: native  │   │ - ASR: Whisper  │  │   chunking   │ │
│                   │ - MP3: Whisper│   │ - Cleanup: LLM  │  │ - Hierarchy  │ │
│                   │ - Video: frame│   │   post-process  │  │   preservation│ │
│                   │   + audio     │   │              │    │ - Metadata   │ │
│                   │              │    │              │    │   extraction │ │
│                   └──────────────┘    └──────────────┘    └──────────────┘ │
│                                                                │              │
│                                                                ▼              │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐    ┌──────────────┐ │
│  │  Embedding   │ <- │   Vector     │ <- │   Knowledge  │ <- │   Quality    │ │
│  │  Generation  │    │   Index      │    │   Graph      │    │   Gate       │ │
│  │              │    │   (HNSW)     │    │   (Concepts, │    │              │ │
│  │ - Embedding  │    │              │    │    Moves,    │    │ - Duplicate  │ │
│  │   Gemma /    │    │              │    │    Parallels)│    │   detection  │ │
│  │   Qwen3      │    │              │    │              │    │ - Corruption │ │
│  │              │    │              │    │              │    │   detection  │ │
│  │              │    │              │    │              │    │ - Coverage   │ │
│  │              │    │              │    │              │    │   score      │ │
│  └──────────────┘    └──────────────┘    └──────────────┘    └──────────────┘ │
│                                                                                │
│  OUTPUT: Indexed, chunked, embedded, graphed corpus ready for retrieval.       │
└────────────────────────────────────────────────────────────────────────────────┘
```

**Storage Backend**: Each creator's corpus is stored in a dedicated namespace within the platform's SurrealDB 3.1 (or Postgres + pgvector) instance. The `Store` trait from the `jesus-twin` architecture generalizes to:

```rust
pub trait CoachStore: Send + Sync {
    async fn ingest(&self, documents: Vec<Document>) -> Result<CorpusId, StoreError>;
    async fn retrieve(&self, query: &str, limit: usize) -> Result<RetrievalSet, StoreError>;
    async fn embed(&self, text: &str) -> Result<Vec<f32>, StoreError>;
    async fn mindmap(&self, topic: &str) -> Result<GraphProjection, StoreError>;
    async fn coverage_score(&self, query: &str) -> Result<f32, StoreError>;
}
```

#### 4.1.2 Voice Fine-Tuning Pipeline

Two tracks are offered in the Creator Studio:

**Track A: Prompt-Only (Free, Fast, No GPU)**
- Expert writes a system prompt describing their pedagogical style, common analogies, tone, and reasoning approach.
- Platform injects this prompt into every generation request.
- No fine-tuning required; voice is purely prompt-conditioned.
- Limitation: Less distinctive voice; higher token cost per request.

**Track B: LoRA Fine-Tuning (Paid, Requires GPU, Authentic Voice)**
- Expert provides **example exchanges** (50-200 pairs of: user question + expert's answer).
- Platform uses **Unsloth** to fine-tune a LoRA adapter on Gemma 4 / Qwen3 / Llama 3.3.
- LoRA is merged into the base model for inference (following `jesus-twin` methodology).
- Thinking mode is OFF for voice fidelity (the model renders the expert's voice, not its reasoning chain).
- Output: a merged checkpoint deployed to the platform's inference workers (or downloadable for self-hosted creators).

**Quality Gate**: Before a fine-tuned coach can be published, it must pass a **blind test**:
- 3 certified peers in the same domain are shown 5 answers: 2 from the real expert, 2 from the coach, 1 from a generic LLM.
- If the peers cannot distinguish the coach from the real expert with > 70% accuracy, the coach passes voice fidelity.
- If the coach hallucinates or answers out-of-corpus, it fails and must be re-trained.

#### 4.1.3 Publishing & Agent Card

Once validated, the coach is published with an **A2A Agent Card**:

```json
{
  "name": "Dr. Feynman Physics Coach",
  "description": "A retrieval-grounded physics tutor based on Richard Feynman's lectures and pedagogical style. Answers only within the corpus of Feynman's published works and Caltech lecture recordings.",
  "version": "1.0.0",
  "capabilities": {
    "skills": [
      { "id": "explain", "description": "Explain a physics concept in Feynman's voice and analogies" },
      { "id": "grade", "description": "Grade a student's explanation against Feynman's reasoning" },
      { "id": "retrieve", "description": "Find relevant passages from the Feynman Lectures" },
      { "id": "mindmap", "description": "Generate a concept graph of a physics topic" }
    ],
    "subjects": ["physics", "quantum_mechanics", "electromagnetism", "thermodynamics"],
    "proficiency_levels": ["novice", "peer", "skeptic"],
    "languages": ["en"],
    "modalities": ["text", "diagram", "equation"]
  },
  "auth": {
    "type": "bearer",
    "token_url": "https://api.prometheus-ags.com/coaches/feynman/token"
  },
  "pricing": {
    "model": "per-session",
    "base_rate": 0.0,
    "currency": "USD",
    "notes": "Free when used within Feynman Loop. Live sessions with human master are billed separately."
  },
  "deployment": {
    "mode": "platform-hosted",
    "endpoint": "https://coaches.prometheus-ags.com/feynman/a2a",
    "protocols": ["a2a", "agui", "mcp"]
  },
  "creator": {
    "id": "uuid-of-caltech-or-estate",
    "name": "California Institute of Technology",
    "verified": true,
    "certification_date": "2026-06-01"
  },
  "corpus": {
    "sources": [
      { "title": "The Feynman Lectures on Physics", "license": "fair-use-educational" },
      { "title": "Caltech Lecture Recordings 1961-1963", "license": "cc-by-nc" }
    ],
    "coverage_score": 0.94,
    "last_updated": "2026-06-15"
  }
}
```

### 4.2 Coach Catalog & Discovery

The Coach Catalog is a **searchable, filterable marketplace** of certified digital coaches. It is exposed to the PFLA frontend via GraphQL and to external agents via A2A discovery.

#### 4.2.1 Discovery Mechanisms

| Mechanism | Endpoint | Use Case |
|---|---|---|
| **Semantic Search** | `POST /api/v1/coaches/search` | Student searches "quantum mechanics tutor with visual analogies" → returns ranked coaches by capability match |
| **Subject Browse** | `GET /api/v1/coaches/subjects/{subject}` | Browse all physics coaches, sorted by rating and LVS |
| **Feynman Loop Auto-Match** | Internal | When a student starts a Feynman loop on "quantum entanglement," the orchestrator queries the catalog for the best-matched coach |
| **A2A Agent Card Registry** | `GET /.well-known/agent.json` (per coach) | External agents discover and delegate to PFLA coaches |
| **Recommendation Engine** | `GET /api/v1/coaches/recommended` | Based on learner's past goals, mastered concepts, and learning style |

#### 4.2.2 Coach Quality Metrics

Each coach is scored on dimensions visible to students:

| Metric | Source | Description |
|---|---|---|
| **Mastery Rate** | Karpathy Loop | % of students who achieve mastery using this coach |
| **LVS Delta** | Karpathy Loop | Learning Velocity Score improvement vs. no coach / generic coach |
| **Coverage Score** | Coverage gate | % of queries answered vs. refused (higher = more comprehensive corpus) |
| **Voice Fidelity** | Blind peer test | 0-1 score from expert impersonation test |
| **Student Rating** | Post-session survey | 1-5 stars with qualitative feedback |
| **Retention Pass Rate** | Retention checks | % of students who pass retention checks after mastery |
| **Recursion Depth** | Loop analytics | Average depth needed to achieve mastery (lower = better explanations) |
| **Citations / Answer** | Citation analytics | Average citations per response (higher = more grounded) |

### 4.3 Master Certification Pipeline

The **Karpathy Loop** (continuous improvement engine) is not only for optimizing the platform's pedagogy — it is also the basis for **certifying students as Masters** who can then coach others.

#### 4.3.1 Certification Requirements

To become a **Certified Master** in a subject, a student must satisfy all three pillars:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                     MASTER CERTIFICATION: THREE PILLARS                        │
│                                                                                │
│  PILLAR 1: MASTERY DEMONSTRATION (Proven by Feynman Loop)                     │
│  ├── Complete Feynman loops on ALL concepts in the subject curriculum        │
│  ├── Achieve mastery (score ≥ 0.7, no misconceptions, transfer ≥ 0.7)       │
│  ├── Pass ALL retention checks (24h, 7d, 30d, 90d) at ≥ 0.8                 │
│  └── Minimum 50 concepts mastered (or full curriculum if smaller)             │
│                                                                                │
│  PILLAR 2: PEDAGOGICAL SKILL (Proven by Teaching Simulation)                  │
│  ├── Complete 10 peer-level Feynman loops where the student is the          │
│  │   TEACHER (explaining to a simulated novice)                              │
│  ├── Graded by the platform's AI grader on: clarity, analogy quality,       │
│  │   gap anticipation, misconception handling                                  │
│  ├── Minimum average score: 0.8 on pedagogical rubric                         │
│  └── Complete 3 skeptic-level explanations with > 0.7 score                  │
│                                                                                │
│  PILLAR 3: LEARNING VELOCITY (Proven by Karpathy Loop Analytics)              │
│  ├── LVS (Learning Velocity Score) in the top 20% of the subject cohort       │
│  ├── Demonstrate improvement trajectory: LVS increases over time              │
│  ├── Complete at least one pedagogical experiment (e.g., "teach with         │
│  │   visual analogies vs. verbal analogies") and measure outcome            │
│  └── Contribute at least one improvement to the platform's curriculum       │
│      (approved by the Karpathy Loop engine)                                   │
│                                                                                │
│  OUTPUT: Master Badge (NFT-style, verifiable), listing in Master Directory,   │
│          eligibility to coach, eligibility to create a digital persona.       │
└────────────────────────────────────────────────────────────────────────────────┘
```

#### 4.3.2 Certification Ceremony & Badge

Upon completion, the student receives:
1. **Master Badge**: A verifiable credential (W3C Verifiable Credential or blockchain-backed) containing the subject, date, mastery scores, and LVS.
2. **Master Profile**: A public profile in the Master Directory with their mastery tree, student testimonials, and availability for coaching.
3. **Persona Creation Rights**: Permission to use the Creator Studio to build a digital persona based on their own teaching examples.
4. **Revenue Share Eligibility**: Ability to set coaching rates and earn money from student sessions.

#### 4.3.3 Re-Certification

Masters must re-certify annually:
- Complete a **retention recertification exam** (20 randomly selected concepts from their mastered corpus, must score ≥ 0.8).
- Maintain a **minimum student satisfaction rating** of 4.0/5.0 if they have coached students.
- Participate in at least one **pedagogical experiment** per year.

Failure to re-certify results in badge suspension (grace period: 90 days). The master can re-earn by re-taking the certification exam.

### 4.4 Live Coaching & Video Conferencing Architecture

#### 4.4.1 The Coaching Session Model

A **coaching session** is a real-time interaction between a student and a master (human), optionally augmented by the master's digital persona. Three modes are supported:

| Mode | Description | Price | Use Case |
|---|---|---|---|
| **Persona-Only** | Student interacts with the digital persona via text/chat. No human master. | Free (included in Plus/Pro) or per-message | Self-directed learning, 24/7 availability, gap drilling |
| **Human + Persona** | Human master leads the session; the persona is a "whispering assistant" that suggests analogies, flags misconceptions, and retrieves citations in real-time. Only visible to the master. | Master's rate + platform fee | Deep tutoring, exam prep, concept clarification |
| **Human + Shared Persona** | Both student and master see the persona's contributions. The persona acts as a Socratic moderator, asking clarifying questions, suggesting exercises. | Master's rate + platform fee + AI premium | Collaborative learning, group sessions, advanced pedagogy |

#### 4.4.2 Video Conferencing Stack (flint-realtime-fabric)

The video conferencing layer is built on **flint-realtime-fabric**, the platform's real-time infrastructure:

```
┌─────────────────────────────────────────────────────────────────────────────────────────┐
│                      VIDEO CONFERENCING: flint-realtime-fabric                          │
│                                                                                          │
│  ┌─────────────────────┐    ┌─────────────────────┐    ┌─────────────────────┐        │
│  │   Signaling Server   │    │   SFU / MCU Bridge   │    │   Session Record   │        │
│  │   (WebSocket)        │    │   (Selective Fwd)   │    │   & Async Replay   │        │
│  │                      │    │                      │    │                      │        │
│  │  - WebRTC offer/     │    │  - Routes video      │    │  - Per-session      │        │
│  │    answer/ICE        │    │    streams to        │    │    recording        │        │
│  │  - Room management   │    │    participants      │    │  - AI transcription │        │
│  │  - Presence/typing   │    │  - Simulcast layers  │    │  - Feynman loop     │        │
│  │  - Session state     │    │  - Bandwidth adapt   │    │    post-analysis    │        │
│  │                      │    │  - TURN relay fallback│   │  - Retention clips  │        │
│  └─────────────────────┘    └─────────────────────┘    └─────────────────────┘        │
│           │                          │                          │                       │
│           └──────────────────────────┴──────────────────────────┘                       │
│                                       │                                                   │
│                                       ▼                                                   │
│  ┌─────────────────────────────────────────────────────────────────────────────────┐     │
│  │                         AI AUGMENTATION LAYER (Real-Time)                          │     │
│  │                                                                                   │     │
│  │  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐  ┌───────────────┐  │     │
│  │  │   ASR (Whisper) │  │   Persona       │  │   Gap Detection  │  │   Dashboard   │  │     │
│  │  │   Live Transcribe│  │   Whisper Engine │  │   (Real-Time)    │  │   Feed        │  │     │
│  │  │                  │  │                  │  │                  │  │               │  │     │
│  │  │  - Real-time     │  │  - Suggests      │  │  - Detects when  │  │  - Master sees│  │     │
│  │  │    speech-to-text│  │    analogies     │  │    student is    │  │    transcript │  │     │
│  │  │  - Feeds persona │  │  - Flags         │  │    confused      │  │  - AI suggests│  │     │
│  │  │    and gap det   │  │    misconceptions│  │  - Triggers      │  │  - Citations  │  │     │
│  │  │                  │  │  - Retrieves     │  │    persona       │  │    auto-retrieved│ │     │
│  │  │                  │  │    citations     │  │    intervention  │  │  - Student    │  │     │
│  │  │                  │  │                  │  │                  │  │    progress   │  │     │
│  │  └─────────────────┘  └─────────────────┘  └─────────────────┘  └───────────────┘  │     │
│  └─────────────────────────────────────────────────────────────────────────────────┘     │
│                                       │                                                   │
│  ┌────────────────────────────────────┴────────────────────────────────────────────┐     │
│  │                              CLIENT INTEGRATION                                   │     │
│  │                                                                                   │     │
│  │  Web: WebRTC via getUserMedia + RTCPeerConnection (no plugin needed)             │     │
│  │  Tauri Desktop: WebRTC in WebView (same as web) + native audio processing         │     │
│  │  Tauri Mobile: Native WebRTC + push notifications for session start              │     │
│  │                                                                                   │     │
│  └─────────────────────────────────────────────────────────────────────────────────┘     │
└─────────────────────────────────────────────────────────────────────────────────────────┘
```

**Key Technologies**:
- **Signaling**: WebSocket server (Rust, Axum, `tokio-tungstenite`) for offer/answer/ICE exchange.
- **Media Routing**: **Selective Forwarding Unit (SFU)** for 1:1 and small group sessions; **Multipoint Control Unit (MCU)** for large group sessions (composites video into a single stream). Built on `mediasoup` or `pion` (Rust bindings).
- **TURN Relay**: For NAT traversal behind firewalls. Can be self-hosted (`coturn`) or cloud (Twilio TURN, Cloudflare TURN).
- **Recording**: Server-side recording via `ffmpeg` gRPC workers. Each session is recorded as:
  - Raw video/audio files (for archival)
  - AI-transcribed transcript (Whisper API, local if available)
  - Post-session Feynman analysis: the AI analyzes the transcript to identify gaps in the student's understanding and generates a personalized re-study plan.

#### 4.4.3 Tauri Integration for Video

For Tauri desktop/mobile, the video stack is the same WebRTC as the browser (since Tauri uses the OS WebView). However, native capabilities are added:

| Feature | Web | Tauri Desktop | Tauri Mobile |
|---|---|---|---|
| Camera | getUserMedia | getUserMedia | getUserMedia |
| Microphone | getUserMedia | getUserMedia + native noise suppression | getUserMedia + native noise suppression |
| Screen sharing | getDisplayMedia | getDisplayMedia + Tauri `screen` API | Not available (iOS restriction) |
| Background audio | Not reliable | Tauri `audio` plugin (background processing) | iOS/Android background audio API |
| Push notifications | Web Push | System notifications | APNS / FCM |
| CallKit integration | N/A | N/A | iOS CallKit (incoming call UI) |
| PiP (Picture-in-Picture) | Web API | Native PiP window | Not available |

### 4.5 Revenue Share Engine

The revenue share engine handles all financial transactions between students, masters, and the platform.

#### 4.5.1 Transaction Flow

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           REVENUE SHARE ENGINE                               │
│                                                                                │
│  STUDENT pays $50 for a 60-minute session with Master Alice                 │
│    │                                                                           │
│    ▼                                                                           │
│  ┌─────────────────────────────────────────────────────────────────────────┐ │
│  │  STRIPE CONNECT (Platform + Express accounts)                             │ │
│  │                                                                           │ │
│  │  - Student charged: $50.00                                               │ │
│  │  - Stripe fee (2.9% + $0.30): -$1.75                                     │ │
│  │  - Net collected: $48.25                                                  │ │
│  │                                                                           │ │
│  │  PLATFORM SPLIT:                                                          │ │
│  │  - Master Alice (70%): $33.78  → Express account (payout in 2 days)      │ │
│  │  - Platform (30%): $14.47  → Platform account                             │ │
│  │                                                                           │ │
│  │  Note: Platform percentage decreases with master tier:                    │ │
│  │    - Beginner Master: 30% platform                                       │ │
│  │    - Verified Master: 25% platform                                       │ │
│  │    - Elite Master: 20% platform                                          │ │
│  │    - Celebrity/Institution: negotiable (15-20%)                          │ │
│  └─────────────────────────────────────────────────────────────────────────┘ │
│    │                                                                           │
│    ▼                                                                           │
│  MASTER DASHBOARD:                                                           │
│  - Earnings: $33.78 (session) + $120.00 (this week) + $450.00 (this month)    │
│  - Payout schedule: Weekly auto-payout to bank account                       │
│  - Tax form: 1099-K (US) or W-8BEN (international) auto-generated             │
│  - Analytics: sessions, ratings, no-shows, student mastery rates            │
│                                                                                │
│  PLATFORM REVENUE ALLOCATION:                                                 │
│  - Infrastructure (SFU, recording, LLM): 40% of platform share                │
│  - Payment to video CDN (Cloudflare Stream, Mux): 20%                         │
│  - Sales & marketing: 20%                                                     │
│  - Platform profit: 20%                                                       │
└────────────────────────────────────────────────────────────────────────────────┘
```

#### 4.5.2 Pricing Models (Master-Defined)

Masters can choose their pricing model within platform bounds:

| Model | Description | Platform Min | Platform Max | Example |
|---|---|---|---|---|
| **Per-session (flat)** | Fixed price per session (e.g., $50 for 60 min) | $10 | $500 | "$50 for a 60-minute physics tutoring session" |
| **Per-minute** | Metered by actual duration | $0.20/min | $5.00/min | "$1.00/minute, billed to the second" |
| **Package** | Pre-paid bundle of sessions | $40 for 5 sessions | $2000 for 50 sessions | "$200 for 5 sessions (20% discount)" |
| **Subscription** | Weekly/monthly unlimited access | $50/week | $500/month | "$199/month for unlimited physics coaching" |
| **Group rate** | Per-student rate for 2-10 students | $5/student | $100/student | "$20/student for a group of 5" |
| **Async only** | Video message exchange, no live session | $5/message | $50/message | "$15 per video question + written answer" |

**Platform fee**: In addition to the revenue share percentage, the platform charges a **per-session platform fee** ($1.00 per session for Beginner, $0.50 for Verified, $0.25 for Elite). This covers Stripe fees, infrastructure, and moderation.

#### 4.5.3 AI-Augmented Session Premium

When the digital persona is actively involved in the session (Modes B and C), an **AI premium** is added to the master's rate:

| Mode | AI Premium | Who Pays |
|---|---|---|
| **Persona-Only** (text) | $0 (included in Plus/Pro subscription) | Platform (subscription revenue) |
| **Human + Whisper** (AI visible to master only) | +$5/session | Student |
| **Human + Shared** (AI visible to both) | +$10/session | Student |

This premium is split: **50% to the master** (for providing the persona) and **50% to the platform** (for AI compute and infrastructure).

### 4.6 Karpathy Loop for Coach & Master Optimization

The Karpathy Loop is extended to optimize not just the platform's generic pedagogy, but also **individual coaches and masters**:

#### 4.6.1 Coach-Level Karpathy Loop

Each digital persona runs its own Karpathy Loop experiments:
- **Editable asset**: The persona's system prompt, fine-tuning examples, or LoRA weights.
- **Scalar metric**: The persona's **Mastery Rate** (percentage of students who achieve mastery with this coach).
- **Time-boxed cycle**: 7-day experiment windows on a subset of learners.

Experiments a coach might run autonomously:
- "Try more visual analogies vs. verbal analogies for thermodynamics"
- "Increase recursion depth to 4 vs. 3 for this concept"
- "Use Socratic questioning style vs. direct explanation style"
- "Add more citations vs. fewer citations per answer"

Winning experiments are **auto-committed to the coach's configuration** (with a human-in-the-loop gate for major changes). The coach improves its pedagogy automatically.

#### 4.6.2 Master-Level Karpathy Loop

For human masters, the Karpathy Loop provides **personalized coaching analytics**:
- **LVS trajectory**: Is the master's student LVS improving over time? If not, what pedagogical changes correlate with improvement?
- **Session analysis**: AI analysis of recorded sessions to identify missed teaching opportunities, unclear explanations, or student confusion signals.
- **Peer benchmark**: How does this master's student mastery rate compare to other masters in the same subject?
- **Experiment proposals**: "Your students struggle with recursion depth 2 on electromagnetism. Try this 3-minute video analogy — it improved Master Bob's student mastery by 18%."

---

## 5. Data Architecture Additions

### 5.1 New Database Schema

```sql
-- Coaches / Digital Personas
CREATE TABLE coaches (
    id UUID PRIMARY KEY,
    creator_id UUID NOT NULL REFERENCES masters(id), -- or external verified creator
    name TEXT NOT NULL,
    description TEXT,
    subject_tags TEXT[],
    proficiency_levels TEXT[], -- ['novice', 'peer', 'skeptic']
    languages TEXT[],
    modalities TEXT[], -- ['text', 'diagram', 'equation', 'video']
    deployment_mode TEXT DEFAULT 'platform-hosted', -- 'platform-hosted' | 'self-hosted' | 'hybrid'
    agent_card JSONB NOT NULL,
    status TEXT DEFAULT 'pending', -- 'pending' | 'testing' | 'active' | 'suspended' | 'retired'
    coverage_score FLOAT,
    voice_fidelity_score FLOAT,
    mastery_rate FLOAT,
    lvs_delta FLOAT,
    student_rating FLOAT,
    pricing_model TEXT, -- 'free' | 'per-session' | 'per-minute' | 'package' | 'subscription'
    base_rate_cents INTEGER DEFAULT 0, -- in cents, 0 = free
    ai_premium_cents INTEGER DEFAULT 0,
    created_at TIMESTAMPTZ DEFAULT now(),
    activated_at TIMESTAMPTZ,
    retired_at TIMESTAMPTZ
);

-- Coach corpus sources
CREATE TABLE coach_corpus_sources (
    id UUID PRIMARY KEY,
    coach_id UUID NOT NULL REFERENCES coaches(id),
    title TEXT NOT NULL,
    author TEXT,
    license_type TEXT NOT NULL, -- 'public-domain' | 'cc-by' | 'cc-by-nc' | 'fair-use' | 'licensed' | 'original'
    license_url TEXT,
    file_path TEXT, -- path to stored document
    file_size_bytes INTEGER,
    page_count INTEGER,
    chunk_count INTEGER,
    embedding_model TEXT,
    vector_index_status TEXT DEFAULT 'pending', -- 'pending' | 'indexing' | 'ready' | 'error'
    indexed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT now()
);

-- Masters (human coaches)
CREATE TABLE masters (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES learners(id) UNIQUE, -- must be a learner first
    display_name TEXT NOT NULL,
    bio TEXT,
    avatar_url TEXT,
    subjects TEXT[], -- subjects they are certified to teach
    certification_level TEXT DEFAULT 'beginner', -- 'beginner' | 'verified' | 'elite' | 'celebrity'
    certification_date TIMESTAMPTZ,
    re_certification_due TIMESTAMPTZ,
    lvs_percentile FLOAT, -- within their subject cohort
    total_sessions INTEGER DEFAULT 0,
    total_students INTEGER DEFAULT 0,
    average_rating FLOAT,
    no_show_rate FLOAT DEFAULT 0.0,
    is_available_for_booking BOOLEAN DEFAULT false,
    stripe_connect_account_id TEXT,
    payout_schedule TEXT DEFAULT 'weekly', -- 'daily' | 'weekly' | 'monthly'
    minimum_session_cents INTEGER DEFAULT 1000, -- $10.00
    maximum_session_cents INTEGER DEFAULT 50000, -- $500.00
    timezone TEXT DEFAULT 'UTC',
    created_at TIMESTAMPTZ DEFAULT now()
);

-- Master certification records
CREATE TABLE master_certifications (
    id UUID PRIMARY KEY,
    master_id UUID NOT NULL REFERENCES masters(id),
    subject TEXT NOT NULL,
    certification_level TEXT NOT NULL, -- 'beginner' | 'verified' | 'elite'
    pillar_1_mastery_score FLOAT NOT NULL, -- Pillar 1: Feynman loop mastery
    pillar_2_pedagogy_score FLOAT NOT NULL, -- Pillar 2: teaching simulation
    pillar_3_lvs_percentile FLOAT NOT NULL, -- Pillar 3: Karpathy Loop LVS
    exam_transcript JSONB, -- full exam record
    badge_id UUID, -- reference to verifiable credential
    issued_at TIMESTAMPTZ DEFAULT now(),
    expires_at TIMESTAMPTZ NOT NULL,
    revoked_at TIMESTAMPTZ,
    revocation_reason TEXT
);

-- Coaching sessions
CREATE TABLE coaching_sessions (
    id UUID PRIMARY KEY,
    student_id UUID NOT NULL REFERENCES learners(id),
    master_id UUID REFERENCES masters(id), -- null for persona-only sessions
    coach_id UUID REFERENCES coaches(id), -- null for human-only sessions
    mode TEXT NOT NULL, -- 'persona-only' | 'human-whisper' | 'human-shared' | 'human-only'
    status TEXT DEFAULT 'scheduled', -- 'scheduled' | 'in-progress' | 'completed' | 'cancelled' | 'no-show'
    scheduled_start TIMESTAMPTZ NOT NULL,
    scheduled_end TIMESTAMPTZ NOT NULL,
    actual_start TIMESTAMPTZ,
    actual_end TIMESTAMPTZ,
    duration_seconds INTEGER, -- actual duration
    student_paid_cents INTEGER NOT NULL, -- total amount charged to student
    platform_fee_cents INTEGER NOT NULL, -- $1.00 + Stripe fee
    master_earnings_cents INTEGER, -- null for persona-only
    ai_premium_cents INTEGER DEFAULT 0,
    recording_url TEXT, -- link to session recording
    transcript_url TEXT, -- link to AI transcript
    post_session_analysis JSONB, -- Feynman gap analysis from session transcript
    student_rating INTEGER, -- 1-5
    student_feedback TEXT,
    master_notes TEXT,
    cancellation_reason TEXT,
    cancelled_by TEXT, -- 'student' | 'master' | 'system'
    created_at TIMESTAMPTZ DEFAULT now()
);

-- Master availability (calendar slots)
CREATE TABLE master_availability (
    id UUID PRIMARY KEY,
    master_id UUID NOT NULL REFERENCES masters(id),
    start_time TIMESTAMPTZ NOT NULL,
    end_time TIMESTAMPTZ NOT NULL,
    is_booked BOOLEAN DEFAULT false,
    session_id UUID REFERENCES coaching_sessions(id),
    recurrence_rule TEXT, -- iCal RRULE for recurring slots
    created_at TIMESTAMPTZ DEFAULT now()
);

-- Revenue transactions
CREATE TABLE revenue_transactions (
    id UUID PRIMARY KEY,
    session_id UUID REFERENCES coaching_sessions(id),
    type TEXT NOT NULL, -- 'session' | 'subscription' | 'tip' | 'refund' | 'payout' | 'platform-fee'
    amount_cents INTEGER NOT NULL,
    currency TEXT DEFAULT 'USD',
    from_user_id UUID REFERENCES learners(id), -- student for session revenue
    to_user_id UUID REFERENCES masters(id), -- master for earnings
    stripe_payment_intent_id TEXT,
    stripe_transfer_id TEXT,
    stripe_payout_id TEXT,
    status TEXT DEFAULT 'pending', -- 'pending' | 'succeeded' | 'failed' | 'refunded'
    metadata JSONB,
    created_at TIMESTAMPTZ DEFAULT now()
);

-- Karpathy Loop experiments (coach-level)
CREATE TABLE coach_experiments (
    id UUID PRIMARY KEY,
    coach_id UUID NOT NULL REFERENCES coaches(id),
    hypothesis TEXT NOT NULL,
    parameter TEXT NOT NULL,
    control_value TEXT NOT NULL,
    experiment_value TEXT NOT NULL,
    cohort_size INTEGER NOT NULL,
    start_time TIMESTAMPTZ NOT NULL,
    end_time TIMESTAMPTZ,
    control_mastery_rate FLOAT,
    experiment_mastery_rate FLOAT,
    p_value FLOAT,
    status TEXT DEFAULT 'pending', -- 'pending' | 'running' | 'completed' | 'committed' | 'rejected'
    committed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT now()
);

-- Video conference rooms (flint-realtime-fabric)
CREATE TABLE video_rooms (
    id UUID PRIMARY KEY,
    session_id UUID NOT NULL REFERENCES coaching_sessions(id),
    room_token TEXT NOT NULL, -- JWT for room access
    sfu_worker_id TEXT, -- which SFU worker is routing this room
    recording_status TEXT DEFAULT 'not-started', -- 'not-started' | 'recording' | 'completed' | 'error'
    recording_url TEXT,
    max_participants INTEGER DEFAULT 2,
    created_at TIMESTAMPTZ DEFAULT now()
);
```

### 5.2 ElectricSQL Sync Shapes for Coaching

```typescript
// Additional shapes for the coaching marketplace
const coachingShapes = [
  {
    table: 'coaches',
    where: "status = 'active'",
    // Read-only sync for client-side catalog browsing
  },
  {
    table: 'master_certifications',
    where: "master_id = '${masterId}'",
    // Sync to master dashboard
  },
  {
    table: 'coaching_sessions',
    where: "student_id = '${learnerId}' OR master_id = '${masterId}'",
    // Sync to both student and master calendars
  },
  {
    table: 'master_availability',
    where: "master_id = '${masterId}'",
    // Sync to master's calendar for booking
  },
  {
    table: 'revenue_transactions',
    where: "from_user_id = '${learnerId}' OR to_user_id = '${masterId}'",
    // Sync to earnings dashboard
  },
];
```

---

## 6. Security & Privacy Considerations

### 6.1 Video Session Privacy

| Concern | Mitigation |
|---|---|
| **Unauthorized recording** | Server-side recording only; clients cannot download raw video. Watermarked with session ID. |
| **Recording consent** | Both parties must consent before recording. Consent is logged in the database. Student can opt out (master is notified). |
| **Recording retention** | Recordings stored for 90 days (configurable). After 90 days, only transcript and AI analysis are retained. Student can request deletion earlier. |
| **End-to-end encryption** | DTLS-SRTP for WebRTC media. Signaling is over WSS. No platform access to decrypted media during the call (except for server-side recording, which is a separate pipeline). |
| **AI transcription** | Transcription is processed server-side (Whisper). Transcript is accessible to both parties. Master can opt to have the AI assistant disabled. |
| **Child safety** | If student is under 18 (COPPA), video sessions require parental consent. AI persona cannot initiate video sessions with minors — only text. |

### 6.2 Content Moderation

| Layer | Mechanism |
|---|---|
| **Pre-session** | Both parties' profiles are verified. Master must have active certification. Student must have completed at least one Feynman loop (no cold video calls to strangers). |
| **During session** | AI moderation listens to transcript (async, not real-time to avoid latency). Flags: harassment, hate speech, sexual content, dangerous instructions. Flagged sessions trigger a review queue. |
| **Post-session** | Both parties rate each other. Low ratings trigger review. Student can report misconduct. Master can report inappropriate student behavior. |
| **Corpus moderation** | Creator Studio corpus is scanned for copyrighted material, hate speech, and harmful content before indexing. DMCA takedown process for reported violations. |

### 6.3 Financial Security

- **Stripe Connect**: All master payouts go through Stripe Connect Express accounts. Platform never holds master funds directly.
- **Escrow**: Student payment is authorized at booking time but not captured until the session is completed. If cancelled > 24h in advance, no charge. If cancelled < 24h, student pays 50% (master gets 50%, platform gets 50%). If no-show, full charge.
- **Dispute resolution**: Platform mediates disputes. Evidence includes: session recording (if consented), transcript, chat logs, ratings. Platform has final say but can escalate to Stripe dispute resolution.
- **Fraud detection**: AI monitoring detects suspicious patterns (e.g., master creating fake student accounts to inflate ratings). Banned accounts forfeit earnings.

---

## 7. Deployment Architecture for Video

### 7.1 SFU/MCU Scaling

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        VIDEO INFRASTRUCTURE (K8s)                              │
│                                                                                │
│  ┌──────────────────────────┐    ┌──────────────────────────┐                  │
│  │   Ingress Controller     │    │   Signaling Server       │                  │
│  │   (nginx / traefik)      │    │   (Axum + WebSocket)     │                  │
│  │   - WSS termination      │    │   - 3 replicas minimum   │                  │
│  │   - Sticky sessions        │    │   - Redis for presence   │                  │
│  │     (by room)            │    │   - Stateless            │                  │
│  └──────────────────────────┘    └──────────────────────────┘                  │
│              │                              │                                   │
│              ▼                              ▼                                   │
│  ┌─────────────────────────────────────────────────────────────────────────┐ │
│  │                    SFU Worker Pool (mediasoup / pion)                     │ │
│  │  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐       │ │
│  │  │ SFU-01  │  │ SFU-02  │  │ SFU-03  │  │ SFU-04  │  │ SFU-0N  │       │ │
│  │  │ (GPU)   │  │ (GPU)   │  │ (GPU)   │  │ (GPU)   │  │ (GPU)   │       │ │
│  │  │ - 50    │  │ - 50    │  │ - 50    │  │ - 50    │  │ - 50    │       │ │
│  │  │   rooms │  │   rooms │  │   rooms │  │   rooms │  │   rooms │       │ │
│  │  └─────────┘  └─────────┘  └─────────┘  └─────────┘  └─────────┘       │ │
│  │  HPA: 3-20 replicas based on CPU/GPU utilization                        │ │
│  └─────────────────────────────────────────────────────────────────────────┘ │
│              │                                                                  │
│              ▼                                                                  │
│  ┌─────────────────────────────────────────────────────────────────────────┐ │
│  │                    Recording Workers (CPU, burstable)                       │ │
│  │  - ffmpeg gRPC workers                                                    │ │
│  │  - Upload to S3 / R2 after session                                        │ │
│  │  - Trigger Whisper transcription job                                      │ │
│  └─────────────────────────────────────────────────────────────────────────┘ │
│              │                                                                  │
│              ▼                                                                  │
│  ┌─────────────────────────────────────────────────────────────────────────┐ │
│  │                    Object Storage (S3 / R2 / MinIO)                         │ │
│  │  - Session recordings (encrypted at rest)                                   │ │
│  │  - Transcripts                                                            │ │
│  │  - 90-day TTL with lifecycle policy                                       │ │
│  └─────────────────────────────────────────────────────────────────────────┘ │
└────────────────────────────────────────────────────────────────────────────────┘
```

**Capacity Planning**:
- Each SFU worker (GPU-enabled, e.g., NVIDIA T4) handles ~50 concurrent 1:1 video sessions or ~10 group sessions (5 participants each).
- Target: 1,000 concurrent sessions → 20 SFU workers + 3 signaling replicas + 5 recording workers.
- Cost: ~$2,000/month for SFU pool (spot instances) + $500/month for recording + $200/month for storage.

---

## 8. Integration with Base Architecture

This addendum is designed to layer on top of the base PFLA architecture without breaking existing flows. The integration points are:

| Base Component | Addendum Integration | How |
|---|---|---|
| **Axum Router** | Add new routes: `/coaches/*`, `/masters/*`, `/sessions/*`, `/video/*` | `Router::merge` in `pfla-api` |
| **AG-UI SSE** | Coach events emit `COACH_SUGGESTION`, `COACH_CITATION`, `GAP_DETECTED` | Extend `AgentEvent` enum |
| **A2A Surface** | Coach is an A2A agent; PFLA orchestrator delegates to it | `pfla-api/src/coach_delegation.rs` |
| **MCP Client** | Coach can be an MCP server or MCP tool | `pfla-mcp` connects to coach endpoints |
| **Feynman Loop** | Loop orchestrator queries Coach Catalog before generating explanation | `pfla-feynman/src/coach_resolver.rs` |
| **Karpathy Loop** | Extended to coach-level and master-level experiments | `fdb-reflection`新增 `CoachExperiment` and `MasterAnalytics` |
| **PGlite / ElectricSQL** | New sync shapes for coaching data | Add to `electric.ts` config |
| **Tauri** | Video calls via WebRTC in WebView; native audio via Tauri plugin | `tauri/src-tauri/src/video.rs` |
| **Stripe** | Stripe Connect for master payouts, Stripe Checkout for session booking | `pfla-api/src/billing/coaching.rs` |
| **Flint-Forge** | `fdb-realtime` extended for video signaling events | `flint-realtime-fabric` integration |

---

## 9. Open Questions

| ID | Question | Status |
|---|---|---|
| AQ-01 | Should the digital persona be a separate A2A agent or an MCP tool within the PFLA orchestrator? | A2A agent preferred for richer interaction; MCP tool for simpler integration. Hybrid: both. |
| AQ-02 | How do we handle copyright for expert corpora? | Creator must verify rights. Platform provides DMCA takedown. For public-domain works, auto-verify via Gutenberg/Internet Archive. |
| AQ-03 | What is the maximum corpus size per coach? | Platform-hosted: 10GB free, 100GB Pro creator. Self-hosted: unlimited. |
| AQ-04 | Can a student become a master without ever using a coach? | Yes — the certification is based on the Feynman loop + Karpathy Loop, not coach usage. Coaches accelerate but are not required. |
| AQ-05 | How do we prevent masters from gaming the rating system? | Multi-signal: ratings + LVS + retention pass rate + AI analysis of session transcripts. Fake accounts detected by graph analysis. |
| AQ-06 | What happens when the master and student are in different time zones? | Master sets availability in their timezone; platform converts to student's timezone. Booking system handles the math. |
| AQ-07 | Should video sessions be recorded by default? | No — opt-in. Both parties must consent. Default is transcript-only (no video). |
| AQ-08 | Can a master use their own self-hosted coach? | Yes — Pro creators can self-host and connect via A2A Agent Card. Platform still handles billing, ratings, and discovery. |
| AQ-09 | What is the liability if a coach gives harmful advice? | Platform Terms of Service: coaches are educational tools, not professional advice. For regulated fields (medical, legal), coaches must include disclaimers. Coverage gate prevents out-of-corpus answers that could be dangerous. |
| AQ-10 | How does the AI assistant work during a live video call without adding latency? | ASR runs on a separate thread; persona suggestions are delivered to the master's dashboard (not the student's video stream) so there's no audio delay for the student. |

---

*End of Architecture Addendum*
