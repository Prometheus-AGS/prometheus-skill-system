# Prometheus Feynman Learning Agent — Implementation Plan Addendum
# Marketplace, Master Certification, Creator Studio & Video Conferencing

## Document Control

| Field | Value |
|---|---|
| **Project** | Prometheus Feynman Learning Agent (PFLA) |
| **Document** | Implementation Plan Addendum: Marketplace, Master Certification, Creator Studio & Video Conferencing |
| **Version** | 1.0.0-draft |
| **Date** | 2026-07-01 |
| **Status** | Draft for Review |
| **Related Documents** | `prometheus-feynman-learning-agent-implementation-plan.md` (Base Implementation Plan), `prometheus-feynman-learning-agent-architecture-addendum.md` (Architecture Addendum), `prometheus-feynman-learning-agent-functional-spec-addendum.md` (Functional Spec Addendum) |

---

## 1. Implementation Philosophy

This addendum extends the base implementation plan with the **marketplace and coaching capabilities**. The philosophy is:

1. **Build the foundation first**: The base P0-P5 critical path (foundation, local-first data, Feynman loop, AI UI, MCP, LLM) must be solid before layering marketplace features. Coaches are useless without the learning loop.

2. **Launch personas before live video**: The digital persona (Creator Studio + Coach Catalog) is a software-only feature that can ship without video infrastructure. Live video is a heavy infrastructure investment that should come after persona validation.

3. **Master certification as a growth loop**: The certification pipeline turns the best students into coaches, creating organic growth. This is a mid-lifecycle feature, not a Day 1 feature.

4. **Video as a premium differentiator**: WebRTC conferencing is expensive and complex. It should be a premium feature (Pro tier + enterprise) that justifies higher pricing.

> **Reference Example**: The `jesus-twin` architecture is referenced throughout this plan **only as an exemplary implementation** of the persona creation methodology. Its build sequence (store → inference → core → API → admission → skills → CLI) is the template for the Creator Studio pipeline. The fine-tuning methodology (Unsloth LoRA → merge → voice fidelity test) is the template for coach voice conditioning. No religious product is proposed.

---

## 2. Phase Overview (Marketplace Addendum)

These phases run **after or alongside** the base P0-P10 phases. They can be interleaved once the critical path (P0-P5) is complete.

| Phase | Name | Duration | Base Dependency | Goal | Deliverable |
|---|---|---|---|---|---|
| **P11** | Coach Catalog & Discovery | 4 weeks | P5 (LLM Integration) | Searchable, filterable coach marketplace with quality metrics. | `GET /api/v2/coaches` working, React catalog UI, admin approval workflow. |
| **P12** | Creator Studio (Core) | 5 weeks | P11 (Coach Catalog) | Self-service portal for corpus upload, processing, and prompt-based persona creation. | Creator Studio web app with corpus upload, processing pipeline, prompt editor, test mode. |
| **P13** | Creator Studio (Advanced) | 4 weeks | P12 (Creator Studio Core) | LoRA fine-tuning, voice fidelity testing, quality gates, publishing workflow. | Fine-tuning pipeline, blind test automation, quality gate dashboard, publish flow. |
| **P14** | Master Certification Pipeline | 5 weeks | P2 (Feynman Loop), P8 (Karpathy Loop) | Three-pillar certification: mastery, pedagogy, LVS. Exam engine, badge minting, re-certification. | Certification progress tracking, simulated teaching, experiment framework, exam engine, badge system. |
| **P15** | Live Coaching (Text & Async) | 4 weeks | P14 (Master Certification) | Text-based coaching sessions, async video messaging, booking, calendar, payment. | Booking flow, text session UI, async video messaging, calendar integration, Stripe Connect. |
| **P16** | Video Conferencing (flint-realtime-fabric) | 6 weeks | P15 (Live Coaching) | WebRTC 1:1 and group video sessions with AI augmentation, recording, transcription. | SFU/MCU workers, signaling server, WebRTC client, recording pipeline, Whisper transcription. |
| **P17** | Revenue Share & Marketplace Engine | 4 weeks | P15 (Live Coaching) | Stripe Connect payouts, revenue analytics, fraud detection, dispute resolution. | Master earnings dashboard, platform revenue dashboard, auto-payout, tax forms. |
| **P18** | Mobile Video & Tauri Integration | 4 weeks | P16 (Video Conferencing), P7 (Tauri) | Native video on Tauri desktop + mobile, CallKit, push notifications, background audio. | Tauri video calls, iOS CallKit, Android full-screen notifications, mobile-optimized video UI. |
| **P19** | AI Augmentation in Live Sessions | 4 weeks | P16 (Video Conferencing) | Real-time ASR, AI whisper suggestions, shared persona mode, post-session analysis. | Whisper ASR integration, real-time suggestion sidebar, post-session Feynman analysis. |
| **P20** | Marketplace Launch & Scale | 4 weeks | P17 (Revenue Engine), P19 (AI Augmentation) | Performance optimization, security audit, load testing, marketing launch. | 99.9% uptime SLA, 1000 concurrent video sessions, public marketplace launch. |

**Total marketplace addendum**: ~40 weeks (can be parallelized with base P6-P10 after P5 is complete).

---

## 3. Phase Details

### Phase 11: Coach Catalog & Discovery (Weeks 22-25, after P5)

**Goal**: A searchable, filterable coach marketplace that students can browse and preview before committing to a Feynman loop.

#### P11.1 Database Schema & Migrations

1. Create migration `002_coach_catalog.sql`:
   - `coaches` table (see Architecture Addendum §5.1)
   - `coach_corpus_sources` table
   - `coach_ratings` table (student ratings with review text)
   - `coach_usage_stats` table (daily aggregated metrics: sessions, mastery rate, LVS delta)

2. Add to ElectricSQL sync shapes:
   - `coaches` (read-only, all active coaches)
   - `coach_ratings` (read-only, aggregated)
   - `coach_usage_stats` (read-only, daily)

#### P11.2 Coach API Endpoints

1. `GET /api/v2/coaches` — list with filtering, sorting, pagination
   - Query params: `subject`, `language`, `proficiency`, `min_rating`, `max_price`, `sort`, `page`, `per_page`
   - Sort options: `relevance`, `rating`, `mastery_rate`, `lvs_delta`, `newest`
   - Response: paginated list of coach cards with all metrics

2. `GET /api/v2/coaches/{id}` — detail page with full metrics, corpus sources, creator profile, reviews

3. `POST /api/v2/coaches/{id}/preview` — 3-turn sample interaction (no auth, no persistence)
   - Returns AG-UI event stream (same as Feynman loop)
   - Rate limited: 3 previews per IP per hour

4. `GET /api/v2/coaches/recommended` — personalized recommendations for authenticated user
   - Based on: active goals, mastered concepts, past ratings, learning style inference
   - Algorithm: collaborative filtering + content-based + recency boost

5. `POST /api/v2/coaches/{id}/favorite` — add/remove from favorites list

6. `GET /api/v2/coaches/trending` — computed trending list (refreshed every 6 hours via cron)

#### P11.3 Coach Quality Metrics Pipeline

1. **Nightly aggregation job** (Rust async job or cron trigger):
   - Query `coaching_sessions`, `grades`, `artifacts` for each coach
   - Compute: `mastery_rate`, `lvs_delta`, `coverage_score`, `average_rating`, `session_count`, `student_count`
   - Update `coaches` table with aggregated metrics
   - Invalidate cache (Redis) for trending and recommended lists

2. **Real-time event updates** (for active coaches):
   - On each Feynman loop completion with a coach: increment `session_count`
   - On each grade: update rolling `mastery_rate` average
   - On each rating: update rolling `average_rating`

#### P11.4 React UI: Coach Catalog

1. **Coach Catalog Screen** (`/coaches`):
   - Search bar with auto-complete (subjects, coach names)
   - Filter chips: Subject, Language, Proficiency, Price, Rating
   - Sort dropdown: Relevance, Rating, Mastery Rate, Newest
   - Grid of coach cards (responsive: 1 col mobile, 2 col tablet, 3 col desktop, 4 col wide)
   - Coach Card: avatar, name, subject tags, rating stars, price badge, "Preview" button, "Learn with Coach" button

2. **Coach Detail Screen** (`/coaches/{id}`):
   - Hero: large avatar, name, subjects, rating, price, "Start Feynman Loop" CTA
   - Metrics dashboard: mastery rate, LVS delta, coverage score, voice fidelity, citation density (radial charts or score bars)
   - About section: description, creator profile, corpus sources list
   - Reviews carousel: 5 most recent reviews with student name, rating, text, date
   - Preview modal: 3-turn chat interface with the coach

3. **Trending & Recommended Sections** (on student dashboard):
   - Horizontal scrollable carousels
   - "Trending this week" and "Recommended for you"

#### P11.5 Admin Approval Workflow

1. **Admin Dashboard** (`/admin/coaches`):
   - Table of pending coaches: name, creator, subject, submitted date, coverage score, auto-review status
   - "Approve" / "Reject" / "Request Changes" buttons
   - Rejection includes: reason text + specific remediation steps
   - Approved coaches go to catalog immediately

2. **Quality Alerts**:
   - Automated alerts for coaches with rating < 2.0 for 10+ sessions
   - Automated alerts for coaches with coverage score < 0.5
   - Automated suspension for DMCA complaints (pending review)

**P11 Exit Criteria**:
- Coach catalog API endpoints all functional with < 500ms response time.
- UI is responsive and accessible (WCAG 2.1 AA).
- Quality metrics are computed nightly and visible on detail pages.
- Preview mode works for 3 turns without authentication.
- Admin approval workflow is functional.
- 10+ seed coaches are loaded for testing (various subjects, free and paid).

---

### Phase 12: Creator Studio (Core) (Weeks 26-30, after P11)

**Goal**: Self-service portal for experts to upload corpus, configure a teaching persona, and test it.

#### P12.1 Corpus Upload & Processing Pipeline

1. **Upload Service**:
   - Direct upload to S3/R2 presigned URLs (frontend gets presigned URL, uploads directly to storage)
   - Supported formats: PDF, TXT, MD, EPUB, DOCX, MP3, MP4, WAV
   - Max file size: 100MB per file, 10GB total per coach
   - Virus scanning: ClamAV or Cloudflare R2 malware scanning

2. **Document Parser Workers** (Rust async workers, or Python Celery/Redis):
   - **PDF**: `pdfplumber` (text) + `Tesseract OCR` (scanned pages) + `PyMuPDF` (structure extraction)
   - **TXT/MD**: Native, preserve structure (headings, lists, code blocks)
   - **EPUB/DOCX**: `ebooklib` / `python-docx` → extract text + metadata
   - **MP3/MP4/WAV**: `Whisper` (OpenAI API or local `whisper.cpp`) → transcript + timestamps
   - Output: JSONL with fields: `doc_id`, `page_num`, `section_path`, `text`, `format`, `metadata`

3. **Chunking Worker**:
   - Semantic chunking: split by paragraph, preserve sentence boundaries
   - Configurable chunk size: 256-2048 tokens (default: 512)
   - Configurable overlap: 0-128 tokens (default: 64)
   - Output: chunks with `chunk_id`, `doc_id`, `page_num`, `section_path`, `text`, `start_char`, `end_char`

4. **Embedding Worker**:
   - Generate embeddings using platform's default model (Embedding Gemma or Qwen3-Embedding, 768d)
   - Batch size: 100 chunks per request
   - Output: `chunk_id` + `embedding` vector

5. **Indexing Worker**:
   - Insert into SurrealDB 3.1 or Postgres + pgvector
   - Create HNSW index on embeddings (if not exists)
   - Create BM25 full-text index on text fields
   - Build graph edges (if structural relationships are extracted, e.g., `section -> subsection`)

6. **Quality Gate Worker**:
   - Run coverage test: 50 auto-generated questions across the subject domain
   - Compute coverage score: % of questions answered within corpus
   - Output: coverage score + detailed report (which questions were refused, which were answered)

7. **Pipeline Orchestration**:
   - Each stage is a separate async worker queue (Redis/RabbitMQ/SQS)
   - Stage completion triggers the next stage
   - Failure at any stage: retry 3 times, then mark as failed with detailed error log
   - Real-time progress: WebSocket or SSE to frontend showing current stage, progress bar, ETA

#### P12.2 React UI: Creator Studio

1. **Dashboard** (`/creator-studio`):
   - Project cards: active projects (corpus uploaded, processing, testing, published)
   - "Create New Coach" button → wizard

2. **Wizard Step 1: Name & Subject** (`/creator-studio/new`):
   - Coach name, description, subject tags (auto-complete from platform taxonomy), proficiency levels, languages, modalities

3. **Wizard Step 2: Corpus Upload** (`/creator-studio/new/corpus`):
   - Drag-and-drop zone with file type icons
   - File list with: name, size, format, status (uploaded / processing / complete / error)
   - Total size indicator (10GB limit)
   - "Start Processing" button (enabled when all files uploaded)

4. **Processing Dashboard** (`/creator-studio/{project_id}/processing`):
   - Visual pipeline: Upload → Parse → Extract → Chunk → Embed → Index → Quality Gate
   - Each stage: icon, status (pending / running / complete / error), progress bar, duration
   - Error stage: expandable error details, "Retry" button, "Cancel" button
   - Real-time updates via WebSocket

5. **Wizard Step 3: Prompt Editor** (`/creator-studio/{project_id}/prompt`):
   - Text area for system prompt with syntax highlighting
   - Variable autocomplete: `{{subject}}`, `{{proficiency}}`, `{{corpus_citation}}`, `{{student_name}}`
   - Template sidebar: "Socratic Questioner", "Direct Explainer", "Visual Analogizer", "Storyteller"
   - "Test Prompt" button: generates a sample explanation for a test concept, shows output + latency + citation count

6. **Wizard Step 4: Test & Validate** (`/creator-studio/{project_id}/test`):
   - Split-pane: left = chat test interface; right = debug panel
   - Debug panel tabs: Retrieved Chunks, Citations, Coverage Score, Latency, Voice Fidelity Estimate
   - "Run Coverage Test" button: runs 50 auto-generated questions, shows pass/fail chart
   - "Run Voice Fidelity Test" button: queues blind peer review (if expert provided example exchanges)

7. **Wizard Step 5: Publish** (`/creator-studio/{project_id}/publish`):
   - Quality Gate summary: score cards with traffic lights
   - Pricing config: free / per-session / per-minute / included in subscription
   - Agent Card preview (editable JSON)
   - "Submit for Review" button (disabled if any gate fails)

#### P12.3 API Endpoints

1. `POST /api/v2/creator/projects` — create new coach project
2. `GET /api/v2/creator/projects/{id}` — get project status
3. `POST /api/v2/creator/projects/{id}/upload` — get presigned URL for file upload
4. `POST /api/v2/creator/projects/{id}/process` — start processing pipeline
5. `GET /api/v2/creator/projects/{id}/progress` — SSE stream of processing progress
6. `POST /api/v2/creator/projects/{id}/prompt` — save system prompt
7. `POST /api/v2/creator/projects/{id}/test` — run test query, return AG-UI stream + debug info
8. `POST /api/v2/creator/projects/{id}/coverage-test` — run coverage test, return results
9. `POST /api/v2/creator/projects/{id}/submit` — submit for admin review

#### P12.4 Storage & Backend

1. **S3/R2 bucket structure**:
   ```
   s3://pfla-creator-corpora/{creator_id}/{project_id}/
   ├── raw/
   │   ├── file1.pdf
   │   ├── file2.mp3
   │   └── ...
   ├── parsed/
   │   ├── file1.jsonl
   │   └── file2.jsonl
   ├── chunks/
   │   └── all_chunks.jsonl
   └── embeddings/
       └── all_embeddings.parquet
   ```

2. **Database tracking**:
   - `creator_projects` table: project metadata, status, processing stage, error log
   - `creator_files` table: file metadata, upload status, parsing status

**P12 Exit Criteria**:
- Creator can upload documents and see real-time processing progress.
- All processing stages (parse, extract, chunk, embed, index) complete successfully for PDF, TXT, MD.
- Prompt editor works with test generation and debug panel.
- Coverage test runs and returns a score.
- Quality gate blocks publish if score < 0.7.
- Admin review workflow receives submitted coaches.

---

### Phase 13: Creator Studio (Advanced) (Weeks 31-34, after P12)

**Goal**: LoRA fine-tuning, voice fidelity testing, and advanced publishing features.

#### P13.1 LoRA Fine-Tuning Pipeline

1. **Example Exchange Collection**:
   - UI for experts to add Q&A pairs: question text, expert's answer text, subject tag, proficiency level
   - Minimum: 20 pairs. Recommended: 50-200 pairs.
   - Import from: uploaded documents (auto-extract Q&A patterns), existing Feynman loop artifacts (if expert is a certified master)
   - Validation: check for duplicates, check for PII, check for copyrighted material

2. **Fine-Tuning Worker** (GPU-enabled, K8s job):
   - Framework: **Unsloth** (fast LoRA training, 2x faster, 70% less memory)
   - Base model: Gemma 4 E4B-it / Qwen3-72B / Llama 3.3 70B (configurable)
   - LoRA config: `r=64`, `lora_alpha=128`, `target_modules=["q_proj", "k_proj", "v_proj", "o_proj", "gate_proj", "up_proj", "down_proj"]`
   - Training: 3-5 epochs, learning rate `2e-4`, batch size 1 (gradient accumulation 4), max sequence length 4096
   - Output: LoRA adapter weights + training loss curve + validation loss
   - Estimated time: 30-120 minutes on A100 / H100 (depending on dataset size)
   - Cost: $5-20 per fine-tuning run (billed to expert or platform)

3. **Merge & Deploy**:
   - Merge LoRA into base model using `peft` merge utilities
   - Upload merged checkpoint to model storage (S3/R2)
   - Deploy to inference workers: load merged model, run warm-up inference
   - Update `coaches` table with `model_path` and `deployment_status`

4. **Alternative: Prompt-Only Track** (no fine-tuning):
   - Experts who don't want to fine-tune use the system prompt only
   - No GPU cost, faster to deploy
   - Trade-off: less distinctive voice, higher token cost per request
   - UI toggle: "Use Fine-Tuning" vs. "Use Prompt Only"

#### P13.2 Voice Fidelity Testing

1. **Automated Test Generation**:
   - Generate 5 test questions across the subject domain
   - Run each question through: the real expert (if available, via async message), the coach, and a generic LLM
   - Collect answers anonymously

2. **Blind Peer Review**:
   - Select 3 certified peers in the same domain (from the Master Directory)
   - Send each peer: 5 answers (shuffled, labeled A-E)
   - Peer task: "Which answers are from the real expert? Which from the AI? Rate each on a 1-5 scale for authenticity."
   - Platform collects responses, computes:
     - `voice_fidelity_score`: % of peers who correctly identified the coach as "AI but indistinguishable"
     - `authenticity_score`: average rating of coach answers vs. real expert answers
   - Pass threshold: ≥ 2/3 peers rate coach as "indistinguishable or very close" + authenticity score ≥ 4.0/5.0

3. **Self-Test Mode** (before peer review):
   - Expert can run a self-test: compare their own answer to the coach's answer side-by-side
   - Expert rates: "Exactly like me" / "Very similar" / "Somewhat similar" / "Different" / "Very different"
   - Self-test is advisory only; peer review is required for publish

#### P13.3 Advanced Publishing Features

1. **Versioning**:
   - Update corpus or prompt → creates new version (v2)
   - Students on v1 are notified: "This coach has been updated. Switch to v2?"
   - Grace period: 30 days. After grace period, v1 is archived (read-only, no new sessions).
   - Rollback: expert can revert to v1 if v2 has issues

2. **Self-Hosted Option** (Pro creators):
   - Expert downloads: merged model + corpus index + Docker Compose + Agent Card template
   - Expert runs on their own infrastructure (GPU server, cloud VM, local machine)
   - Expert provides public A2A endpoint in the Agent Card
   - Platform still handles: discovery, ratings, billing, student matching
   - Platform verifies self-hosted endpoint health via periodic ping

3. **Multi-Language Support**:
   - Expert can upload corpus in multiple languages
   - Expert can configure separate prompts per language
   - Expert can configure separate fine-tuning per language
   - Language is a filter in the Coach Catalog

**P13 Exit Criteria**:
- LoRA fine-tuning pipeline runs end-to-end: upload examples → train → merge → deploy → test.
- Voice fidelity blind test passes for at least 1 test coach.
- Versioning works: v1 → v2 update, student notification, grace period, archive.
- Self-hosted option works: download package, Docker Compose runs, A2A endpoint is reachable.
- Quality gates (coverage + voice fidelity) block publish if any gate fails.

---

### Phase 14: Master Certification Pipeline (Weeks 35-39, after P2 + P8)

**Goal**: Transform students who have deeply mastered a subject into Certified Masters who can coach others.

#### P14.1 Pillar 1: Mastery Verification (Already Exists)

Pillar 1 is automatically satisfied by the Feynman loop engine. No new code is needed — just a certification query:
- Check if all concepts in the subject curriculum have `status = 'mastered'`
- Check if all retention checks passed
- Compute `mastery_rate` and `retention_pass_rate`

UI: certification progress dashboard showing Pillar 1 as green (or progress bar to completion).

#### P14.2 Pillar 2: Pedagogical Skill Simulation

1. **Simulated Novice Agent**:
   - A specialized AI agent that acts as a "student" in a Feynman loop
   - The real student (certification candidate) explains a concept to the simulated novice
   - The simulated novice asks follow-up questions, gets confused, requests analogies, and pushes back on unclear explanations
   - The simulated novice adapts to the student's teaching style: if the student is too advanced, the novice asks "dumb" questions; if the student is unclear, the novice expresses confusion

2. **Grading Rubric** (5 criteria, 0-1 each):
   - **Clarity**: Is the explanation organized, structured, and easy to follow?
   - **Analogy Quality**: Are the analogies appropriate, accurate, and resonant?
   - **Gap Anticipation**: Does the student anticipate what the novice might not understand?
   - **Misconception Handling**: Does the student catch and correct the novice's misconceptions?
   - **Socratic Questioning**: Does the student ask clarifying questions that guide the novice to understanding?

3. **Session Flow**:
   - Student selects a concept from the mastered corpus
   - Simulated novice "arrives" with a pre-scripted confusion profile (e.g., "thinks mass and weight are the same")
   - Student explains (text or voice, recorded)
   - Simulated novice responds with questions/confusion
   - 5-10 turns per session
   - AI grader evaluates the entire session transcript against the rubric
   - Minimum 10 sessions required, average score ≥ 0.8

4. **React UI**: `SimulatedTeachingSurface`
   - Split-pane: left = chat interface (student explains, novice responds); right = real-time rubric with scores updating after each turn
   - "Finish Session" button → shows session summary + rubric scores + suggestions for improvement
   - "Next Session" button → queues next concept + novice profile

#### P14.3 Pillar 3: Learning Velocity & Experiment

1. **LVS Computation**:
   - Already computed by the Karpathy Loop engine (P8)
   - Certification requires: LVS in the top 20% of the subject cohort
   - Cohort: all learners who attempted ≥ 50% of the subject curriculum in the last 90 days
   - LVS formula: `(mastery_rate × 0.4) + (1/avg_recursion_depth × 0.3) + (retention_pass_rate × 0.3)`

2. **Pedagogical Experiment**:
   - Student proposes a hypothesis: "Teaching quantum mechanics with visual analogies (vs. verbal analogies) will improve mastery rate by 15%"
   - Platform guides the experiment setup:
     - Variable A: visual analogies (control: student's default style)
     - Variable B: verbal analogies
     - Target concept: selected from mastered corpus
     - Duration: 7 days
     - Cohort: the student recruits 2-5 peers (or platform matches peers)
   - Platform randomizes peers into A/B groups, tracks LVS for each group
   - After 7 days, platform computes statistical significance (Mann-Whitney U test)
   - Student writes a brief report: hypothesis, results, conclusion
   - Platform evaluates: was the experiment well-designed? Was the analysis correct? Does the conclusion follow?

3. **React UI**: `ExperimentSetupSurface` + `ExperimentResultsSurface`
   - Setup: form with hypothesis, variable A, variable B, concept, duration, peer recruitment
   - Progress: daily updates showing LVS for each group
   - Results: charts, statistical test results, p-value, conclusion editor

#### P14.4 Certification Exam Engine

1. **Exam Generation**:
   - Randomly select 50 concepts from the student's mastered corpus (weighted by recency and difficulty)
   - For each concept: generate an explanation prompt ("Explain [concept] to a [proficiency] student")
   - Some concepts are "retention checks" (re-testing previously mastered concepts)
   - Exam is timed: 2 hours (120 minutes = 2.4 minutes per concept, generous for text input)

2. **Exam Grading**:
   - AI grader (same as Feynman loop `learn-grade`) evaluates each explanation
   - Scoring: overall_score (0-1), misconceptions_absent (0-1), gaps (list), transfer_problems (2 problems)
   - Pass threshold: overall_score ≥ 0.8 on ALL 50 concepts, misconceptions_absent = 1.0 on ALL concepts, both transfer problems ≥ 0.7 on ALL concepts
   - Strict: one failure = exam fail

3. **Exam Attempts**:
   - 3 attempts per year
   - Between attempts: student must re-study failed concepts (Feynman loop on gaps)
   - Waiting period: 30 days between attempts

4. **Badge Minting**:
   - On pass: generate W3C Verifiable Credential (JSON-LD)
   - Badge contains: subject, certification date, expiration date, mastery scores, LVS, exam transcript hash
   - Cryptographic signature: platform's Ed25519 key
   - Badge URL: `https://credentials.prometheus-ags.com/badges/{badge_id}`
   - Badge is downloadable as JSON, embeddable as iframe, shareable to LinkedIn/Twitter

5. **React UI**: `CertificationExamSurface`
   - Timer (top, countdown)
   - Progress bar (50 concepts)
   - Concept card: title, prompt, text area, submit button
   - Immediate feedback: pass/fail per concept (after AI grading, ~30 seconds)
   - Final screen: overall result, pass/fail, badge preview (if pass), next steps (if fail)

#### P14.5 Re-Certification

1. **Annual Re-Certification Exam**:
   - 20 randomly selected concepts from the mastered corpus
   - Must score ≥ 0.8 on all concepts
   - One attempt per year (no retries for re-certification; if fail, enter grace period)

2. **Student Satisfaction Check** (if master has coached students):
   - Average rating ≥ 4.0/5.0 over the last 90 days
   - Minimum 5 ratings (if fewer than 5 ratings, this check is waived)

3. **Grace Period**:
   - 90 days after expiration
   - During grace period: master can continue coaching existing students but cannot take new students
   - Re-certification notification schedule: 30 days, 14 days, 7 days, 1 day before expiration

**P14 Exit Criteria**:
- Certification progress dashboard shows all three pillars with real data.
- Simulated novice teaching sessions work end-to-end: 10 sessions, rubric grading, average score.
- Pedagogical experiment framework works: setup, randomization, tracking, statistical test, report.
- Certification exam engine generates 50 concepts, grades them, produces pass/fail result.
- Badge is generated as a W3C Verifiable Credential with cryptographic signature.
- Re-certification workflow works: notification, exam, pass/fail, grace period.
- 5+ test students complete the full certification pipeline in a test environment.

---

### Phase 15: Live Coaching (Text & Async) (Weeks 40-43, after P14)

**Goal**: Text-based coaching sessions and async video messaging between students and certified masters.

#### P15.1 Booking Flow

1. **Student Booking**:
   - Student browses Master Directory (`/masters`)
   - Filters: subject, rating, price, availability, language
   - Selects master → views profile → clicks "Book Session"
   - Calendar widget shows available slots (master's timezone, converted to student's timezone)
   - Student selects slot → sees cost breakdown (master fee + platform fee + AI premium if applicable)
   - Stripe Checkout for payment (or saved payment method for returning students)
   - On success: confirmation email + calendar invite (.ics) + in-app notification

2. **Master Calendar**:
   - Master sets availability via drag-and-drop weekly calendar
   - "Sync with Google Calendar" button (OAuth2, read/write to a dedicated "PFLA Coaching" calendar)
   - "Block this slot" for personal time
   - "Recurring slots" (e.g., "Every Monday 3-5 PM")
   - Time zone auto-detected from browser, editable

3. **API Endpoints**:
   - `GET /api/v2/masters` — list with filters, sort, pagination
   - `GET /api/v2/masters/{id}` — detail profile
   - `GET /api/v2/masters/{id}/availability` — calendar slots for a date range
   - `POST /api/v2/sessions` — book a session ( Stripe Checkout redirect)
   - `POST /api/v2/sessions/{id}/cancel` — cancel with refund logic

#### P15.2 Text-Based Coaching Session

1. **Session Start**:
   - 5 minutes before: both parties receive notification (push, email, in-app)
   - "Join Session" button becomes active
   - Both join → session status changes to `in-progress`

2. **Text Chat Interface**:
   - Real-time chat (WebSocket)
   - Rich text: markdown, LaTeX equations, code blocks, image upload
   - AI assistant (if mode = human-whisper): suggestions in a sidebar visible only to master
   - AI assistant (if mode = human-shared): suggestions in a shared panel visible to both
   - "Share Whiteboard" button: opens collaborative canvas (Excalidraw integration)
   - "End Session" button: both parties can end

3. **Session End**:
   - On end: session status → `completed`
   - AI analysis queued (transcript analysis → gap identification → recommended next steps)
   - Rating prompt sent to both parties (1-5 stars + optional text)
   - Post-session analysis available within 30 minutes

#### P15.3 Async Video Messaging

1. **Student Sends Video Message**:
   - Student records video (up to 5 minutes) in-browser or Tauri app
   - Uploads to S3/R2 (presigned URL)
   - Transcription is auto-generated (Whisper)
   - Master receives notification: "New async message from [Student]"

2. **Master Replies**:
   - Master watches video, reads transcript
   - Master records reply video (up to 5 minutes)
   - Uploads, transcription auto-generated
   - Student receives notification

3. **Pricing**:
   - Per-message: $5-50 (master sets rate)
   - Or included in a subscription package
   - Platform fee: same % as live sessions

4. **React UI**: `AsyncMessageSurface`
   - Inbox: list of async conversations with unread count, last message preview, timestamp
   - Thread: video player (student's message) + transcript + reply video recorder + send button

#### P15.4 Payment & Payout (Basic)

1. **Stripe Connect Integration**:
   - Master onboarding: Stripe Connect Express account creation
   - KYC: identity verification, bank account, tax form (W-9 for US, W-8BEN for international)
   - Student payment: Stripe Checkout (one-time) or Stripe PaymentIntent (saved method)
   - Payout: Stripe Connect transfer to master's account

2. **Revenue Share** (Phase 17 will make this fully configurable):
   - Default: 70% master, 30% platform (Beginner)
   - Platform fee: $1.00 per session
   - Stripe fee: 2.9% + $0.30 (deducted from gross)

3. **React UI**: `MasterEarningsSurface`
   - Total earnings, pending payout, this week, this month
   - Payout history table
   - Settings: payout schedule, payout method, tax forms

**P15 Exit Criteria**:
- Booking flow works end-to-end: browse → select → pay → confirm → calendar invite.
- Text-based coaching session works: join, chat, AI suggestions, end, rating.
- Async video messaging works: record, upload, transcribe, notify, reply.
- Payment and payout work: Stripe Checkout, Connect transfer, earnings dashboard.
- Cancellation and refund work: > 24h full refund, < 24h 50% refund, no-show policy.
- 10+ test sessions run successfully between test accounts.

---

### Phase 16: Video Conferencing (flint-realtime-fabric) (Weeks 44-49, after P15)

**Goal**: WebRTC video conferencing for 1:1 and group coaching sessions, with recording, transcription, and AI augmentation.

#### P16.1 Signaling Server

1. **WebSocket Server** (Rust, Axum, `tokio-tungstenite`):
   - `wss://video.prometheus-ags.com/signal`
   - Endpoints:
     - `JOIN {room_token}` — authenticate and join room
     - `OFFER {sdp}` — send WebRTC offer
     - `ANSWER {sdp}` — send WebRTC answer
     - `ICE {candidate}` — send ICE candidate
     - `LEAVE` — leave room
     - `CHAT {message}` — in-call text chat
   - Room state stored in Redis (presence, participant list, roles)
   - Room token: JWT with 1-hour TTL, signed with platform secret
   - Rate limiting: 10 signaling messages per second per connection

2. **Scaling**:
   - 3 replicas minimum (HA)
   - Sticky sessions by room (ingress routes all messages for room X to the same signaling replica)
   - Redis pub/sub for cross-replica room state synchronization

#### P16.2 SFU / MCU Workers

1. **SFU (Selective Forwarding Unit)** — for 1:1 and small group (2-5 participants):
   - Technology: **mediasoup** (Node.js) or **pion** (Go) with Rust bindings
   - Each worker handles 50 concurrent 1:1 sessions or 10 group sessions (5 participants)
   - Simulcast: each participant sends 3 quality layers (low 240p, mid 480p, high 720p)
   - SFU selects the best layer for each receiver based on bandwidth estimation
   - Bandwidth estimation: WebRTC BWE + custom heuristics (packet loss, RTT, jitter)

2. **MCU (Multipoint Control Unit)** — for large group sessions (6+ participants):
   - Technology: **mediasoup** with composite router
   - Layout: speaker spotlight (largest tile) + grid of other participants
   - Composite video stream reduces bandwidth for receivers (single stream instead of N-1)
   - CPU-intensive: one MCU worker handles 5-10 group sessions

3. **Deployment**:
   - K8s Deployment with GPU nodes (NVIDIA T4 or better)
   - HPA: scale based on CPU/GPU utilization and connection count
   - Scale-up: < 2 minutes (pre-warm with spot instances)
   - Scale-down: < 5 minutes (graceful drain: no new sessions, wait for existing to complete)

4. **Cost Model**:
   - SFU: $0.50/hour per session (GPU + bandwidth)
   - MCU: $1.00/hour per session (GPU + CPU + bandwidth)
   - Platform absorbs cost for Free tier (no video). Plus/Pro tier includes video (cost included in subscription). Enterprise pays per minute.

#### P16.3 TURN Relay Server

1. **TURN Server** (coturn or Twilio TURN):
   - For NAT traversal when direct P2P fails (~15% of connections)
   - Short-lived credentials: 1-hour TTL, generated per session
   - Limit: 10 Mbps per relay connection

2. **Deployment**:
   - Self-hosted coturn on 2-3 global regions (US, EU, Asia)
   - Or: Twilio TURN (pay-as-you-go, $0.40/GB)
   - For production: Twilio TURN for reliability + self-hosted for cost optimization

#### P16.4 Recording Pipeline

1. **Server-Side Recording** (ffmpeg gRPC workers):
   - Each session is recorded by a dedicated ffmpeg worker
   - Recording starts within 5 seconds of session start
   - Input: SFU worker streams (via gRPC or RTP)
   - Output: MP4 (H.264 video + AAC audio), 720p, 30fps
   - File size: ~50MB/hour for 720p
   - Upload to S3/R2 after session ends (with AES-256 encryption)

2. **Composite Recording** (for group sessions):
   - Layout: grid of all participants
   - Same ffmpeg pipeline, but with multiple inputs composited
   - File size: ~100MB/hour for 5 participants

3. **Storage & Lifecycle**:
   - S3/R2 bucket: `s3://pfla-recordings/{session_id}/`
   - Files: `recording.mp4`, `participant_1.mp4`, `participant_2.mp4`, `composite.mp4`
   - TTL: 90 days. After 90 days, only transcript + AI analysis are retained.
   - Access: presigned URL with 1-hour TTL, logged in access log

#### P16.5 WebRTC Client (React)

1. **Video Call Component** (`VideoCallSurface`):
   - Main video area: student's video (self-view, small, draggable) + master's video (large, primary)
   - Toolbar: mute, camera, screen share, chat, whiteboard, end call, AI suggestions toggle
   - Sidebar (whisper mode): AI suggestions panel (master only)
   - Sidebar (shared mode): shared chat panel + AI suggestions panel (both visible)
   - Transcript panel: real-time ASR transcript with speaker labels

2. **Technical Implementation**:
   - `navigator.mediaDevices.getUserMedia()` for camera/microphone
   - `RTCPeerConnection` for WebRTC connection
   - `navigator.mediaDevices.getDisplayMedia()` for screen share
   - `ResizeObserver` for responsive video layout
   - Bandwidth estimation: monitor `RTCPeerConnection.getStats()` for bitrate, packet loss, RTT
   - Quality adaptation: reduce resolution/frame rate if bandwidth drops

3. **Tauri Integration**:
   - WebRTC runs in WebView (same as web)
   - Native audio: Tauri `audio` plugin for noise suppression, echo cancellation
   - Screen share: `getDisplayMedia()` works in WebView (macOS/Windows/Linux)
   - Push notifications: Tauri `notification` plugin for incoming call alerts

#### P16.6 Capacity Planning & Testing

1. **Load Testing**:
   - Target: 1,000 concurrent video sessions
   - SFU workers needed: 1,000 / 50 = 20 workers
   - Signaling server: 3 replicas (handles 10,000 WebSocket connections each)
   - TURN: 2-3 global instances
   - Recording workers: 50 workers (each handles 20 sessions)
   - Infrastructure cost: ~$2,000/month for SFU pool (spot instances) + $500/month for recording + $200/month for storage

2. **Performance Targets**:
   - Video latency: < 200ms (one-way, camera to screen)
   - Audio latency: < 150ms (one-way, microphone to speaker)
   - Packet loss: < 1%
   - Jitter: < 30ms
   - First video frame: < 3 seconds after joining

**P16 Exit Criteria**:
- 1:1 video session works end-to-end: join, video, audio, screen share, end.
- Group session with 5 participants works: grid layout, audio, video.
- Recording is captured and stored within 5 seconds of session end.
- Transcription is generated within 10 minutes of session end.
- SFU auto-scales from 3 to 20 workers under load.
- Video quality auto-adapts based on bandwidth.
- TURN relay works for connections behind NAT.
- 100 concurrent test sessions run for 1 hour without failures.

---

### Phase 17: Revenue Share & Marketplace Engine (Weeks 50-53, after P15)

**Goal**: Full financial engine for the marketplace: Stripe Connect payouts, revenue analytics, fraud detection, and dispute resolution.

#### P17.1 Stripe Connect Integration

1. **Master Onboarding**:
   - Master clicks "Become a Master" → redirected to Stripe Connect Express onboarding
   - Stripe handles: identity verification, bank account, KYC, tax form (W-9 / W-8BEN)
   - On success: master receives Stripe Connect account ID, stored in `masters.stripe_connect_account_id`
   - Platform receives webhook: `account.updated` with `charges_enabled: true`

2. **Session Payment**:
   - Student books session → Stripe Checkout or PaymentIntent
   - PaymentIntent captures at session start (or authorization at booking, capture at completion)
   - On session completion: Stripe transfer to master's Connect account (`transfer.create`)
   - Transfer amount: `(student_paid - platform_fee - stripe_fee) × master_share`

3. **Payout**:
   - Stripe Connect automatically pays out to master's bank account on schedule (daily/weekly/monthly)
   - Minimum payout: $10 (configurable)
   - Platform can trigger manual payout via `payout.create`

4. **Subscription Packages** (future):
   - Master can offer a subscription: "$199/month for unlimited sessions"
   - Stripe Subscription with Connect: `subscription.create` with `application_fee_percent`
   - Platform fee: 30% of subscription revenue (includes infrastructure cost)

#### P17.2 Revenue Analytics Dashboard

1. **Master Dashboard** (`/master/earnings`):
   - Total earnings (lifetime, this month, this week, today)
   - Pending payout
   - Payout history (table with date, amount, status, method)
   - Session breakdown: count, average earnings, average duration, average rating
   - Student breakdown: new vs. returning, top students by session count
   - Earnings chart: line chart over time, bar chart by subject

2. **Platform Admin Dashboard** (`/admin/revenue`):
   - Total revenue (all sessions, all subscriptions, all tips)
   - Revenue by tier: Free, Plus, Pro, Enterprise
   - Revenue by subject: Physics, Math, CS, etc.
   - Revenue by master tier: Beginner, Verified, Elite
   - Revenue by session mode: persona-only, human-whisper, human-shared, human-only
   - Platform fees: total, by master tier, by session type
   - AI premium revenue: total, by coach
   - Churned revenue: cancellations, no-shows, refunds
   - Monthly Recurring Revenue (MRR) and Annual Recurring Revenue (ARR)
   - Growth metrics: month-over-month revenue growth, new master signups, new student bookings

3. **API Endpoints**:
   - `GET /api/v2/masters/{id}/earnings` — master earnings summary
   - `GET /api/v2/masters/{id}/earnings/history` — payout history
   - `GET /api/v2/admin/revenue` — platform revenue summary (admin only)
   - `GET /api/v2/admin/revenue/breakdown` — revenue breakdown by dimension (admin only)

#### P17.3 Fraud Detection

1. **Automated Monitoring** (nightly job):
   - Pattern 1: Master creates fake student accounts and books sessions with themselves → circular booking detection (graph analysis: same IP, same device, same payment method, no retention improvement)
   - Pattern 2: Master inflates ratings by asking friends to book and rate 5 stars → rating velocity analysis (sudden spike in 5-star ratings from new accounts)
   - Pattern 3: Student books and cancels repeatedly to drain master's availability → cancellation rate analysis (> 50% cancellation rate flagged)
   - Pattern 4: Payment fraud → Stripe Radar integration (automatic fraud detection)

2. **Actions**:
   - Flag: account is flagged for manual review, master/student is notified
   - Suspend: account is suspended, pending sessions are cancelled, earnings are held in escrow
   - Ban: account is permanently banned, earnings are forfeited (if fraud is confirmed)
   - Report: suspicious activity is reported to Stripe for chargeback investigation

3. **Human Review Queue**:
   - Platform admin reviews flagged accounts
   - Evidence: session recordings, transcripts, chat logs, IP addresses, device fingerprints, payment history
   - Decision: clear flag, suspend, or ban
   - Appeal process: master/student can submit evidence for appeal

#### P17.4 Dispute Resolution

1. **Dispute Flow**:
   - Student or master opens a dispute (reason: "no-show", "poor quality", "fraud", "other")
   - Platform collects evidence: session recording (if consented), transcript, chat logs, ratings, payment history
   - Platform admin reviews evidence and makes a decision within 7 days
   - Decision: full refund to student, partial refund, no refund, or payout to master
   - Both parties are notified of the decision
   - Appeal: either party can appeal to Stripe dispute resolution within 14 days

2. **Automated Dispute Resolution** (for common cases):
   - No-show master: auto-refund to student (full)
   - No-show student: auto-charge student (full), no refund
   - Cancellation > 24h: auto-full refund
   - Cancellation < 24h: auto-50% refund
   - Rating < 2.0: manual review (not auto-refund)

#### P17.5 Tax Handling

1. **US Masters**:
   - Stripe 1099-K generation (if > $600/year)
   - Auto-sent by January 31
   - Master dashboard shows tax form download

2. **International Masters**:
   - W-8BEN collection during Stripe Connect onboarding
   - Platform withholds tax if required by tax treaty
   - Master dashboard shows tax form download

3. **Platform Tax**:
   - Sales tax: Stripe Tax integration for US states, EU VAT, etc.
   - Automatically calculated and added to student payment
   - Platform remits tax to authorities

**P17 Exit Criteria**:
- Stripe Connect onboarding works for 10+ test masters (US and international).
- Session payment, transfer, and payout work end-to-end.
- Master earnings dashboard shows real data.
- Platform revenue dashboard shows real data.
- Fraud detection flags 3+ test fraud patterns.
- Dispute resolution handles 3+ test cases with correct outcomes.
- Tax forms (1099-K, W-8BEN) are generated for test masters.

---

### Phase 18: Mobile Video & Tauri Integration (Weeks 54-57, after P16 + P7)

**Goal**: Native video call experience on Tauri desktop and mobile, with platform-specific optimizations.

#### P18.1 Tauri Desktop Video

1. **WebRTC in WebView**: Same as web (getUserMedia, RTCPeerConnection)
2. **Native Audio Plugin** (`tauri-plugin-audio`):
   - Noise suppression (RNNoise)
   - Echo cancellation (SpeexDSP)
   - Automatic gain control
   - Audio is processed before being sent to WebRTC
3. **Screen Share**: `getDisplayMedia()` works in WebView on macOS/Windows/Linux
4. **Picture-in-Picture**: Native PiP window for video call (Tauri `window` API)
5. **System Tray**: Tray icon shows call status (incoming, in-progress, missed). Click to bring window to front.

#### P18.2 Tauri Mobile Video (iOS)

1. **WebRTC in WKWebView**: `getUserMedia` is supported in WKWebView on iOS 14.3+
2. **CallKit Integration** (`tauri-plugin-callkit`):
   - Incoming call shows native iOS call UI (full screen, lock screen)
   - Caller ID: "PFLA Coaching — Master Alice"
   - Accept / Decline buttons
   - Call duration in status bar
   - Integration with iOS Do Not Disturb
3. **Push Notifications** (APNS):
   - Session reminder: 5 minutes before
   - Incoming call: real-time push
   - Missed call: notification with "Call Back" button
4. **Background Audio**: Tauri `audio` plugin keeps audio alive when app is backgrounded
5. **Safe Area**: Video UI respects iOS safe area insets (notch, home indicator)

#### P18.3 Tauri Mobile Video (Android)

1. **WebRTC in WebView**: `getUserMedia` works in Android WebView (Chrome 94+)
2. **Full-Screen Notification**: Incoming call shows full-screen notification with accept/decline
3. **FCM Push Notifications**: Firebase Cloud Messaging for session reminders and incoming calls
4. **Background Audio**: Tauri `audio` plugin keeps audio alive in background
5. **Doze Mode**: Whitelist app from Doze mode for reliable push notifications

#### P18.4 Mobile-Optimized Video UI

1. **Portrait Mode**: Vertical layout (mobile-friendly)
   - Top: master video (large, 60% of screen)
   - Bottom: self-video (small, picture-in-picture, draggable)
   - Toolbar: floating action buttons (mute, camera, end call, chat, AI toggle)
   - Swipe gestures: swipe up for chat, swipe down for AI suggestions, swipe left for transcript
2. **Landscape Mode**: Horizontal layout (for whiteboard/diagram sharing)
   - Left: master video (50%)
   - Right: whiteboard or shared screen (50%)
3. **Bandwidth Optimization**: Mobile uses 480p by default (lower bandwidth, lower battery)
   - Auto-switch to 240p on poor connection
   - Auto-switch to audio-only on very poor connection

**P18 Exit Criteria**:
- Tauri desktop video call works with native audio processing and screen share.
- iOS app shows CallKit incoming call UI, accepts call, video works.
- Android app shows full-screen notification, accepts call, video works.
- Mobile video UI is optimized for portrait and landscape modes.
- Push notifications work for session reminders and incoming calls.
- Background audio continues when app is minimized.
- 10+ test video calls on iOS and Android devices.

---

### Phase 19: AI Augmentation in Live Sessions (Weeks 58-61, after P16)

**Goal**: Real-time AI assistant during video sessions: ASR, whisper suggestions, shared persona mode, and post-session analysis.

#### P19.1 Real-Time ASR (Automatic Speech Recognition)

1. **Whisper Integration**:
   - Local Whisper (faster-whisper, Whisper.cpp) on GPU worker for real-time transcription
   - Or: OpenAI Whisper API (higher latency, lower cost per minute)
   - Pipeline: audio stream → VAD (Voice Activity Detection) → chunking (5-10 seconds) → Whisper → transcript
   - Latency target: < 5 seconds from speech to transcript
   - Transcript is sent to signaling server → broadcast to all clients

2. **Speaker Diarization**:
   - Identify who is speaking (student vs. master)
   - Use WebRTC SSRC (Synchronization Source) to separate audio streams
   - Or use simple heuristics: master audio is louder, or manual labels

3. **Transcript UI**:
   - Real-time transcript panel in the video call sidebar
   - Speaker labels: "Student: ..." / "Master: ..."
   - Timestamps: clickable to jump to that moment in the recording
   - Search: search transcript for keywords (e.g., "entanglement")

#### P19.2 AI Whisper Suggestions (Master-Only)

1. **Suggestion Pipeline**:
   - Every 10-15 seconds: send the last 30 seconds of transcript to the master's digital persona
   - Persona generates: suggested analogy, citation retrieval, misconception flag, Socratic question
   - Suggestions are displayed in a non-intrusive sidebar on the master's screen only
   - Master can click "Use This" to insert the suggestion into the chat, or "Dismiss" to hide it

2. **Latency Considerations**:
   - Suggestions are async: they don't block the video call
   - Master sees suggestions with a slight delay (10-15 seconds), but this is acceptable for tutoring
   - Persona inference is batched: process multiple transcripts in one request to reduce GPU overhead

3. **Suggestion Types**:
   - **Analogy**: "Try this analogy: 'Entanglement is like two coins that always land on opposite sides, even when flipped far apart.'"
   - **Citation**: "Relevant passage: 'The EPR paradox...' (Feynman Lectures, Vol. III, Ch. 18)."
   - **Misconception Flag**: "Student said 'mass and weight are the same.' This is a common misconception. Suggest correcting it."
   - **Socratic Question**: "Ask: 'What happens to the wave function when you measure one particle?'"
   - **Gap Alert**: "Student hasn't mentioned Bell's inequality. This is a key gap in their understanding."

#### P19.3 Shared Persona Mode (Both Parties See AI)

1. **Shared Chat Panel**:
   - AI persona messages appear in a shared chat panel visible to both student and master
   - Persona can: ask clarifying questions, suggest exercises, provide citations, summarize the discussion
   - Student can ask the persona questions directly: "@Coach, can you explain that again?"
   - Master can override the persona: "@Coach, hold on — let me address this first."

2. **Mode Toggle**:
   - Whisper mode: AI suggestions visible only to master (default)
   - Shared mode: AI visible to both parties (requires both to consent at session start)
   - Master can toggle during the session

#### P19.4 Post-Session Analysis

1. **Transcript Analysis** (AI pipeline, runs after session ends):
   - Input: full transcript + session metadata (subject, concepts discussed, goals)
   - AI analysis:
     - **Concepts Covered**: which curriculum concepts were discussed
     - **Knowledge Gaps**: which concepts were not fully understood (based on student questions, confusion signals)
     - **Misconceptions**: specific misconceptions detected
     - **Mastery Indicators**: which concepts the student seems to have mastered
     - **Recommended Next Steps**: Feynman loop concepts to re-study, practice problems, readings
   - Output: JSON report sent to both student and master via email and in-app notification

2. **Feynman Loop Trigger**:
   - One-click "Start Feynman Loop on [Gap Concept]" from the post-session analysis
   - Pre-populated with session context: "During your session with Master Alice, it was identified that you need to review Bell's inequality."
   - This creates a new Feynman loop artifact linked to the coaching session

3. **React UI**: `PostSessionAnalysisSurface`
   - Summary card: concepts covered, gaps identified, mastery indicators
   - Gap list: each gap has a "Start Loop" button, severity badge, and explanation
   - Recommended next steps: links to Feynman loops, practice problems, readings
   - Session recording: video player with transcript sync (click transcript to jump to timestamp)
   - Rating form: student rates master, master rates student (optional)

**P19 Exit Criteria**:
- Real-time ASR transcript is available within 5 seconds of speech with > 90% accuracy.
- AI whisper suggestions are generated every 10-15 seconds and displayed in master sidebar.
- Shared persona mode works: both parties see AI messages in shared chat panel.
- Post-session analysis is generated within 30 minutes of session end with accurate gap identification.
- Feynman loop can be triggered from post-session analysis with pre-populated context.
- 10+ test sessions with AI augmentation run successfully.

---

### Phase 20: Marketplace Launch & Scale (Weeks 62-65, after P17 + P19)

**Goal**: Launch the marketplace publicly, optimize performance, and scale to 1000+ concurrent video sessions.

#### P20.1 Performance Optimization

1. **Video Infrastructure**:
   - SFU worker optimization: reduce CPU usage by 20% via profiling (perf, flame graphs)
   - Bandwidth optimization: simulcast tuning, codec selection (H.264 vs. VP9), bitrate adaptation
   - Connection optimization: faster ICE gathering, reduced TURN relay usage

2. **Database**:
   - Add composite indexes for common queries: `coaches` by `subject` + `status`, `coaching_sessions` by `master_id` + `status`, `revenue_transactions` by `session_id` + `type`
   - Partition `coaching_sessions` by month (partition pruning for historical queries)
   - Cache hot data: coach ratings, trending lists, recommended lists (Redis, 1-hour TTL)

3. **API**:
   - GraphQL query optimization: limit depth, complexity analysis, persistent queries
   - REST endpoint caching: coach detail page cached for 5 minutes, catalog cached for 1 minute
   - CDN caching: static assets (coach avatars, master photos) cached at edge

4. **Frontend**:
   - Lazy loading: video call component is loaded only when joining a session
   - Code splitting: Creator Studio is a separate chunk, loaded only for creators
   - Bundle optimization: tree-shake unused shadcn/ui components, lazy-load recharts

#### P20.2 Security Audit

1. **Penetration Testing**:
   - Video session hijacking: attempt to join a room without a valid token
   - Recording theft: attempt to download recordings without authorization
   - Payment fraud: attempt to manipulate session pricing, bypass payment, or create fake masters
   - AI injection: attempt to send malicious A2UI messages during a video session

2. **Compliance**:
   - GDPR: data portability (student can export all their data), right to erasure (delete account + all recordings), data processing agreement
   - COPPA: no live video sessions for users under 13; persona-only mode for 13-17 with parental consent
   - FERPA: enterprise customers can sign a Business Associate Agreement (BAA) for educational records
   - PCI-DSS: Stripe handles all payment card data; platform never stores card numbers

3. **Privacy**:
   - Privacy policy: clear disclosure of data collection, recording, AI analysis, and third-party sharing
   - Consent management: granular consent for recording, AI analysis, data sharing with coaches
   - Data retention: recordings 90 days, transcripts 1 year, analytics 2 years, financial data 7 years (tax)

#### P20.3 Load Testing

1. **Video Load Test**:
   - Target: 1,000 concurrent video sessions
   - Method: automated test clients (WebRTC bots) that join sessions, send audio/video, and leave
   - Metrics: CPU usage per SFU worker, GPU usage, memory usage, bandwidth, latency, packet loss, session drop rate
   - Duration: 2 hours sustained load + 1 hour burst load (2,000 sessions)

2. **API Load Test**:
   - Target: 10,000 requests/second (coach catalog, booking, session management)
   - Method: `k6` or `locust` load testing
   - Metrics: p50/p95/p99 latency, error rate, database connection pool saturation

3. **Database Load Test**:
   - Target: 1,000 writes/second (session bookings, payments, ratings)
   - Method: `pgbench` or `sysbench`
   - Metrics: query latency, lock contention, replication lag

#### P20.4 Marketing Launch

1. **Launch Phases**:
   - **Week 1**: Creator Studio beta (invite-only for 50 experts)
   - **Week 2**: Master Certification beta (invite-only for 100 students)
   - **Week 3**: Live coaching beta (text-only, 50 masters + 200 students)
   - **Week 4**: Video conferencing beta (invite-only, 20 masters + 50 students)
   - **Week 6**: Public marketplace launch (all features, all tiers)

2. **Marketing Channels**:
   - **Product Hunt**: "Launch Day" for marketplace
   - **Hacker News**: "Show HN" post about the Creator Studio and Feynman loop
   - **Twitter/X**: Founder and team threads about the marketplace, master certification, and video coaching
   - **LinkedIn**: Posts targeting educators, tutors, and online course creators
   - **Reddit**: r/selfimprovement, r/learnprogramming, r/tutor, r/EdTech
   - **Newsletter**: Email to early access list (5,000+ subscribers)
   - **Influencer Partnerships**: Partner with 5-10 education YouTubers/podcasters to create coaches and promote the platform

3. **Launch Incentives**:
   - **For Experts**: First 100 experts to publish a coach get "Founding Creator" badge + 0% platform fee for 6 months
   - **For Students**: First 1,000 students to book a live session get 50% off
   - **For Masters**: First 100 certified masters get "Founding Master" badge + featured placement in Master Directory

**P20 Exit Criteria**:
- Platform can handle 1,000 concurrent video sessions with < 1% drop rate.
- API latency: p95 < 200ms for coach catalog, p95 < 500ms for booking.
- Security audit: no critical or high-severity vulnerabilities.
- Compliance: GDPR, COPPA, FERPA readiness confirmed by legal review.
- Privacy policy and consent management are live and functional.
- Marketing launch generates 1,000+ signups in the first week.
- 50+ coaches published, 20+ certified masters, 100+ live sessions completed in the first month.

---

## 4. Dependency Graph (Marketplace Addendum)

```
Base Critical Path: P0 → P1 → P2 → P5 → P10
                                      │
                                      ▼
                              ┌─────────────────┐
                              │  P11 (Coach     │
                              │   Catalog)      │
                              └─────────────────┘
                                      │
                    ┌─────────────────┼─────────────────┐
                    │                 │                 │
                    ▼                 ▼                 ▼
            ┌──────────────┐  ┌──────────────┐  ┌──────────────┐
            │ P12 (Creator │  │ P14 (Master  │  │ P15 (Live    │
            │   Studio     │  │   Cert)      │  │   Coaching)  │
            │   Core)      │  │              │  │              │
            └──────────────┘  └──────────────┘  └──────────────┘
                    │                 │                 │
                    ▼                 │                 │
            ┌──────────────┐        │                 │
            │ P13 (Creator │        │                 │
            │   Studio     │        │                 │
            │   Advanced)  │        │                 │
            └──────────────┘        │                 │
                    │                 │                 │
                    │                 │                 ▼
                    │                 │         ┌──────────────┐
                    │                 │         │ P17 (Revenue │
                    │                 │         │   Engine)    │
                    │                 │         └──────────────┘
                    │                 │                 │
                    │                 │                 ▼
                    │                 │         ┌──────────────┐
                    │                 │         │ P16 (Video   │
                    │                 │         │   Conf)      │
                    │                 │         └──────────────┘
                    │                 │                 │
                    │                 │                 ▼
                    │                 │         ┌──────────────┐
                    │                 │         │ P18 (Mobile  │
                    │                 │         │   Video)     │
                    │                 │         └──────────────┘
                    │                 │                 │
                    │                 │                 ▼
                    │                 │         ┌──────────────┐
                    │                 │         │ P19 (AI      │
                    │                 │         │   Augment)   │
                    │                 │         └──────────────┘
                    │                 │                 │
                    │                 │                 ▼
                    │                 │         ┌──────────────┐
                    │                 │         │ P20 (Launch  │
                    │                 │         │   & Scale)   │
                    │                 │         └──────────────┘
                    │                 │
                    │                 ▼
                    │         ┌─────────────────┐
                    │         │  P20 (Launch)   │
                    │         └─────────────────┘
                    │
                    ▼
            ┌─────────────────┐
            │  P20 (Launch)   │
            └─────────────────┘
```

**Critical Path for Marketplace MVP**: P11 → P12 → P15 → P17 = ~17 weeks after P5 completion.

**Critical Path for Full Video Coaching**: P11 → P12 → P15 → P16 → P17 → P19 → P20 = ~30 weeks after P5 completion.

**Parallelizable**:
- P14 (Master Certification) can run alongside P11-P13 (Creator Studio) — they share the Feynman loop and Karpathy Loop foundations.
- P18 (Mobile Video) can run alongside P19 (AI Augmentation) — they are independent of each other.

---

## 5. Team Composition & Responsibilities (Marketplace Addendum)

| Role | Count | Primary Responsibilities | Phases |
|---|---|---|---|
| **Rust Backend Engineer** | 2 | Coach catalog API, Creator Studio backend, video signaling, SFU/MCU, revenue engine, Stripe Connect | P11-P13, P15-P17, P20 |
| **Frontend Engineer** | 2 | Creator Studio UI, coach catalog, master dashboard, video call UI, mobile video UI | P11-P13, P15-P20 |
| **Video Engineer** | 1 | WebRTC, SFU/MCU, mediasoup/pion, recording, Tauri video, mobile video | P16-P19 |
| **AI / Prompt Engineer** | 1 (part-time) | Simulated novice agent, pedagogical rubric, voice fidelity test, post-session analysis, AI whisper suggestions | P13-P14, P19 |
| **DevOps / Platform** | 1 | Video infrastructure (K8s, GPU nodes), CDN, load testing, security audit, compliance | P16-P20 |
| **Designer** | 1 (part-time) | Creator Studio UX, video call UI, mobile video UI, marketing materials | P11-P20 |
| **Business / Operations** | 1 | Master onboarding, expert outreach, dispute resolution, tax compliance, marketing launch | P14-P20 |

**Total marketplace team**: 6-8 people (can be added to the base 4-5 person team, or hired as the base team scales).

---

## 6. Risk Mitigation (Marketplace Addendum)

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| **Video infrastructure costs spiral** | High | High | Start with text-only coaching. Video is Pro/Enterprise only. Use spot instances for SFU. Benchmark and optimize before scaling. |
| **LoRA fine-tuning is too expensive** | Medium | High | Offer prompt-only track as default. Fine-tuning is a paid upgrade. Use Unsloth for efficiency. Batch fine-tuning jobs. |
| **Master supply is low** | Medium | High | Launch with 10-20 seed masters (founding team, partner educators). Incentivize early masters with 0% fee for 6 months. Master certification is a growth loop. |
| **Student demand for live coaching is low** | Medium | Medium | Text and async coaching are lower friction. Video is a premium upsell. Monitor booking rates and adjust pricing. |
| **Fraud (fake masters, circular bookings)** | Medium | High | Automated fraud detection + manual review. Stripe Radar. Forfeit earnings for confirmed fraud. Identity verification for masters. |
| **Disputes drain support resources** | Medium | Medium | Automated dispute resolution for common cases (no-show, cancellation). Clear policies. Escalation only for complex cases. |
| **Video quality is poor on mobile** | Medium | Medium | Mobile uses 480p by default. Auto-switch to audio-only on poor connection. Extensive testing on real devices. |
| **Regulatory risk (tutoring licensing)** | Low | Medium | Platform is a marketplace, not a tutoring service. Masters are independent contractors. Terms of Service clarify this. Monitor regulations in target markets. |
| **Master churn (burnout, low earnings)** | Medium | High | Minimum earnings guarantee for first 10 sessions. Marketing support (featured placement). Community building (Discord, master meetups). |
| **Competitor builds similar marketplace** | Medium | Medium | Deep moat: Feynman loop + Karpathy Loop + persona grounding + master certification + continuous improvement. Network effects (masters bring students, students become masters). |

---

*End of Implementation Plan Addendum*
