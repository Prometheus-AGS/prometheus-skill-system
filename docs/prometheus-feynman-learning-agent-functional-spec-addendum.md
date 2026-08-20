# Prometheus Feynman Learning Agent — Functional Specification Addendum
# Creator Studio, Master Certification, Live Coaching & Video Conferencing

## Document Control

| Field | Value |
|---|---|
| **Project** | Prometheus Feynman Learning Agent (PFLA) |
| **Document** | Functional Specification Addendum: Creator Studio, Master Certification, Live Coaching & Video Conferencing |
| **Version** | 1.0.0-draft |
| **Date** | 2026-07-01 |
| **Status** | Draft for Review |
| **Related Documents** | `prometheus-feynman-learning-agent-functional-spec.md` (Base Functional Spec), `prometheus-feynman-learning-agent-architecture-addendum.md` (Architecture Addendum) |

---

## 1. Purpose and Scope

This addendum extends the PFLA Functional Specification with the **learning marketplace** capabilities: the Creator Studio for building digital coaching personas, the Master Certification pipeline that transforms students into earning coaches, live coaching sessions with AI augmentation, and video conferencing via `flint-realtime-fabric`.

> **Reference Example**: The `jesus-twin` architecture is referenced throughout this document **only as an exemplary methodology** for creating grounded, voice-faithful digital personas. It demonstrates the RAG-first grounding pattern, coverage gate, multi-protocol surfaces, and fine-tuning pipeline that are the technical foundation for the Creator Studio. No religious product is proposed.

---

## 2. Feature Catalog Additions

### 2.1 Coach Catalog & Discovery (FEAT-012)

**Description**: A searchable, filterable marketplace of certified digital coaches that students can browse, rate, and engage with during their Feynman loops.

**User Stories**:
- US-012.1: As a student, I can search for a coach by subject, language, proficiency level, and teaching style so that I find a coach that matches my learning needs.
- US-012.2: As a student, I can see a coach's quality metrics (mastery rate, LVS delta, voice fidelity, coverage score, student ratings) so that I can make an informed choice.
- US-012.3: As a student, I can preview a coach's teaching style by asking a sample question before committing to a Feynman loop with them.
- US-012.4: As a student, I can save coaches to a favorites list so that I can quickly access them later.
- US-012.5: As a student, I can see which coaches are "trending" or "recommended" based on my learning history and goals.
- US-012.6: As a platform admin, I can review coach submissions, approve or reject them, and monitor quality metrics.
- US-012.7: As a coach creator, I can see my coach's usage stats, student ratings, and earnings (if monetized) on a dashboard.

**Functional Requirements**:

| ID | Requirement | Priority | Acceptance Criteria |
|---|---|---|---|
| FR-012.1 | Coach catalog supports semantic search by subject, language, proficiency level, and teaching style tags. | Must | Search returns ranked results in < 500ms. Results include relevance score. |
| FR-012.2 | Coach detail page displays all quality metrics: mastery rate, LVS delta, coverage score, voice fidelity score, student rating (1-5), citation density, and average recursion depth. | Must | Metrics are computed from real usage data, refreshed daily. |
| FR-012.3 | Coach preview mode allows a free, no-account sample interaction (3 turns max, no persistence). | Must | Preview does not require login. Sample interaction is discarded after session. |
| FR-012.4 | Coach catalog is filterable by: subject, language, proficiency level, price (free/paid), rating, availability for live sessions. | Must | Filters are combinable (AND logic). Active filters show result count. |
| FR-012.5 | Trending coaches are computed by: new student count (7d), mastery rate improvement (7d), and rating velocity (new ratings per week). | Should | Trending list refreshes every 6 hours. |
| FR-012.6 | Recommended coaches are personalized based on: student's active goals, mastered concepts, preferred learning style (inferred from loop history), and past coach ratings. | Should | Recommendation engine runs nightly. Top 3 recommendations are shown on student dashboard. |
| FR-012.7 | Coach approval workflow: submitted → pending review → approved / rejected → active / suspended. | Must | Admins review within 48 hours. Rejected coaches include feedback and can be resubmitted. |
| FR-012.8 | Coach suspension triggers: DMCA complaint (automatic, pending review), student rating < 2.0 for 10+ consecutive sessions (flagged for review), harmful content detection (automatic), corpus coverage score < 0.5 (flagged). | Must | Suspended coaches are immediately removed from catalog. Existing sessions can complete. |

**UI Surfaces** (A2UI):
- `CoachCatalogSurface`: Search bar + filter chips + sort dropdown + grid of coach cards.
- `CoachCardSurface`: Card with avatar, name, subject tags, rating stars, price badge, "Preview" and "Learn with Coach" buttons.
- `CoachDetailSurface`: Full-page with metrics dashboard, sample interaction preview, corpus sources list, creator profile, reviews carousel, "Start Feynman Loop" CTA.
- `CoachPreviewSurface`: Modal chat interface with 3-turn limit and "Sign up to continue" gate.
- `CoachCreatorDashboardSurface`: Analytics cards (usage, ratings, earnings), submission status, edit button, quality alerts.

---

### 2.2 Creator Studio (FEAT-013)

**Description**: A self-service portal where certified experts (and eventually certified masters) can upload a corpus, configure a teaching voice, test the coach, and publish it to the catalog.

**User Stories**:
- US-013.1: As an expert, I can upload my corpus (PDFs, text files, Markdown, lecture transcripts, video/audio for ASR extraction) so that the coach is grounded in my knowledge. | Must | Supported formats: PDF, TXT, MD, EPUB, DOCX, MP3, MP4, WAV. Max 10GB per upload batch. |
- US-013.2: As an expert, I can see a processing dashboard showing the status of each uploaded document (uploaded → parsing → extracting → chunking → embedding → indexing → ready). | Must | Real-time progress updates via WebSocket. Error states show actionable retry/cancel options. |
- US-013.3: As an expert, I can write a system prompt that defines my teaching style, common analogies, tone, and pedagogical approach. | Must | Prompt editor with character count, template suggestions, and a "Test Prompt" button that generates a sample explanation. |
- US-013.4: As an expert, I can provide example exchanges (Q&A pairs) to fine-tune a LoRA adapter for voice fidelity. | Should | Minimum 20 examples for fine-tuning. Platform provides a template and guidance. |
- US-013.5: As an expert, I can test my coach before publishing by asking questions and seeing the retrieved passages, generated answers, and coverage gate behavior. | Must | Test mode shows: retrieved chunks, citations, coverage score, and a "Would this pass voice fidelity?" estimate. |
- US-013.6: As an expert, I can configure the coverage gate strictness (strict / moderate / lenient) and the subjects/topics the coach is certified to teach. | Must | Strict mode: refuses any question not in corpus. Moderate: allows general knowledge if flagged. Lenient: allows general knowledge with disclaimer. |
- US-013.7: As an expert, I can set my coach's pricing model (free, per-session, per-minute, or included in platform subscription) and availability for live sessions. | Must | Free coaches are available to all students. Paid coaches require Plus/Pro subscription to access. |
- US-013.8: As an expert, I can preview the "Agent Card" that will be published to the A2A registry before going live. | Should | Agent Card preview is editable JSON with validation. |
- US-013.9: As an expert, I can publish my coach to the catalog after passing the quality gate (coverage score ≥ 0.7, voice fidelity test, admin review). | Must | Publish button is disabled until all gates pass. Publishing triggers admin review queue. |
- US-013.10: As an expert, I can update my coach's corpus and re-index without unpublishing. | Should | Updates create a new version. Students can opt into the new version or stay on the old one (grace period: 30 days). |

**Functional Requirements**:

| ID | Requirement | Priority | Acceptance Criteria |
|---|---|---|---|
| FR-013.1 | Document parser supports: PDF (text + OCR for scanned), TXT, MD, EPUB, DOCX, MP3/MP4/WAV (Whisper ASR extraction). | Must | Parser extracts text, preserves hierarchy (headings, sections), and generates page/section metadata. |
| FR-013.2 | Processing pipeline stages are: Upload → Parse → Extract → Chunk → Embed → Index → Quality Gate. Each stage is independently retryable and observable. | Must | Pipeline is idempotent. Re-running "Embed" on already-embedded docs is a no-op. |
| FR-013.3 | Chunking strategy is semantic (paragraph-aware) with configurable chunk size (256-2048 tokens) and overlap (0-128 tokens). | Must | Chunks preserve sentence boundaries. Overlap prevents context loss at boundaries. |
| FR-013.4 | Embedding generation uses the platform's default embedding model (Embedding Gemma or Qwen3-Embedding, 768d). | Must | Embeddings are generated server-side (GPU). 100 pages processed in < 5 minutes. |
| FR-013.5 | System prompt editor supports variables: `{{subject}}`, `{{proficiency}}`, `{{corpus_citation}}`, `{{student_name}}`. | Should | Variables are interpolated at runtime. Invalid variables are flagged. |
| FR-013.6 | Voice fidelity test (blind peer review) requires 3 certified peers in the same domain. Test is automated: peers receive 5 answers via email/A2A, vote on which are human vs. AI. | Should | Minimum 2/3 peers must rate the coach as "indistinguishable from expert" for pass. |
| FR-013.7 | Coverage gate test: 50 diverse questions are auto-generated across the subject domain. The coach must answer ≥ 70% within corpus (strict), ≥ 50% (moderate), or any answer with disclaimer (lenient). | Must | Coverage test results are shown to the expert before publishing. |
| FR-013.8 | Quality gate summary: coverage score, voice fidelity score, citation density, average latency, and a "Go/No-Go" recommendation. | Must | All scores must be ≥ 0.7 for "Go." No-Go includes specific remediation steps. |
| FR-013.9 | Published coaches are versioned. Update creates v2. Students on v1 are notified and can migrate. v1 is deprecated after 30 days, then archived (read-only). | Should | Versioning is automatic. Migration is one-click for students. |
| FR-013.10 | Self-hosted coaches: experts can download the merged model + corpus index + Agent Card to run on their own infrastructure. Platform still handles discovery, ratings, and billing. | Should | Download package includes Docker Compose for local deployment. A2A endpoint must be publicly accessible. |

**UI Surfaces** (A2UI):
- `CreatorStudioDashboardSurface`: Project cards (active, pending, published), "Create New Coach" button, usage/earnings summary.
- `CorpusUploadSurface`: Drag-and-drop zone, file list with progress bars, format icons, total size, "Start Processing" button.
- `ProcessingPipelineSurface`: Visual pipeline with stage icons, status (pending/running/complete/error), progress bars, error details, retry buttons.
- `PromptEditorSurface`: Text area with syntax highlighting, variable autocomplete, template sidebar, character counter, "Test Prompt" button.
- `FineTuneSurface`: Example exchange table (add/edit/delete), LoRA status (not started / training / complete / failed), estimated cost, "Start Training" button.
- `CoachTestSurface`: Split-pane: left = chat test interface; right = debug panel (retrieved chunks, citations, coverage score, latency, voice fidelity estimate).
- `QualityGateSurface`: Score cards for each metric, traffic light status, remediation checklist, "Submit for Review" button.
- `PricingConfigSurface`: Pricing model selector, rate input, currency, availability toggle (accepting live sessions), calendar sync.

---

### 2.3 Master Certification Pipeline (FEAT-014)

**Description**: A rigorous, multi-pillar certification process that transforms students who have deeply mastered a subject into **Certified Masters** who can coach other students and earn money. The certification is backed by the Karpathy Loop's continuous improvement metrics.

**User Stories**:
- US-014.1: As a student, I can see my progress toward Master Certification for a subject, including which pillars are complete and which remain. | Must | Progress is a visual dashboard with three pillars, each with checklist items and progress bars. |
- US-014.2: As a student, I can initiate a certification exam after completing all curriculum concepts in a subject. | Must | Exam is available only when all concepts are mastered (Feynman loop score ≥ 0.7, no misconceptions, transfer ≥ 0.7, all retention checks passed). |
- US-014.3: As a student, I can complete Pillar 2 (Pedagogical Skill) by acting as a teacher in simulated Feynman loops, where I explain concepts to a simulated novice and am graded on my teaching quality. | Must | Simulated novice is an AI that asks follow-up questions, gets confused, and requires the student to adapt. Grading rubric: clarity, analogy quality, gap anticipation, misconception handling, Socratic questioning. |
- US-014.4: As a student, I can complete Pillar 3 (Learning Velocity) by running a pedagogical experiment and measuring its impact on my own or a peer's learning. | Must | Experiment is a 7-day A/B test on a single concept with two teaching approaches. Outcome is measured by LVS. |
- US-014.5: As a student, I receive a Master Badge upon certification that is a verifiable credential (W3C VC or blockchain-backed). | Must | Badge is downloadable as JSON-LD, shareable to LinkedIn, embeddable as an iframe. |
- US-014.6: As a certified master, I can see my Master Dashboard with earnings, upcoming sessions, student progress, and analytics. | Must | Dashboard is the primary screen after login for masters. |
- US-014.7: As a certified master, I can set my availability (calendar slots), pricing, and subjects I teach. | Must | Calendar is a drag-and-drop weekly view. Pricing is per-session or per-minute. Subjects are limited to certified subjects. |
- US-014.8: As a certified master, I must re-certify annually to maintain my status. | Must | Re-certification requires: retention exam (20 random concepts, ≥ 0.8), minimum student satisfaction ≥ 4.0/5.0 (if coached), and one new pedagogical experiment. |
- US-014.9: As a student, I can browse the Master Directory to find a human coach for live sessions, filtered by subject, rating, price, and availability. | Must | Master profiles show: certification badge, subjects, rating, price, availability calendar, student testimonials, and a "Book Session" button. |

**Functional Requirements**:

| ID | Requirement | Priority | Acceptance Criteria |
|---|---|---|---|
| FR-014.1 | Pillar 1 (Mastery) is automatically verified by the Feynman loop engine. All concepts in the subject curriculum must have `status = 'mastered'` and all retention checks passed. | Must | No manual intervention. System auto-detects eligibility. |
| FR-014.2 | Pillar 2 (Pedagogy) requires 10 simulated teaching sessions. Each session is graded by the AI grader on 5 criteria (0-1 each). Minimum average ≥ 0.8. | Must | Simulated novice adapts to the student's teaching style. If the student is too advanced, the novice asks "dumb" questions. If the student is unclear, the novice gets confused. |
| FR-014.3 | Pillar 3 (LVS) requires the student's LVS to be in the top 20% of the subject cohort. LVS is computed as: (mastery_rate × 0.4) + (1/avg_recursion_depth × 0.3) + (retention_pass_rate × 0.3). | Must | Cohort is all learners who have attempted ≥ 50% of the subject curriculum in the last 90 days. |
| FR-014.4 | Pillar 3 also requires one pedagogical experiment: 7-day A/B test with a control group (the student's own historical data or a peer). | Should | Experiment is guided by the Karpathy Loop engine: hypothesis generation, randomization, metric collection, statistical test (Mann-Whitney U). |
| FR-014.5 | Certification exam is a comprehensive assessment: 50 concepts randomly selected from the mastered corpus, including retention checks. | Must | Exam is timed (2 hours). Student must score ≥ 0.8 on each concept. 3 attempts allowed per year. |
| FR-014.6 | Master Badge is a W3C Verifiable Credential with: subject, certification date, expiration date, LVS, mastery scores, and a cryptographic signature. | Must | Badge is verifiable offline (signed JSON-LD). Can be uploaded to LinkedIn, Twitter, or personal website. |
| FR-014.7 | Master re-certification is required annually. Grace period: 90 days after expiration. During grace period, master can coach but cannot take new students. | Must | Re-certification notification is sent 30 days, 14 days, 7 days, and 1 day before expiration. |
| FR-014.8 | Master Directory is searchable by subject, certification level, rating, price, and availability. Masters can be sorted by "recommended" (platform algorithm) or "highest rated." | Must | Search returns in < 500ms. Masters with < 5 sessions are marked "New Master" with a boost in search. |
| FR-014.9 | Master profile includes: photo, bio, certification badges, subjects, pricing, availability calendar, ratings, testimonials, student count, and average student mastery rate. | Must | Profile is public (no login required to view). Student count and mastery rate are updated daily. |
| FR-014.10 | Master can set "persona-only" mode (no live sessions, only digital coach) if they prefer. | Should | Persona-only masters earn revenue from coach usage, not live sessions. |

**UI Surfaces** (A2UI):
- `CertificationProgressSurface`: Three-pillar dashboard with progress bars, checklist items, and "Start Exam" button (enabled when all prerequisites met).
- `SimulatedTeachingSurface`: Split-pane: left = student explains to simulated novice; right = real-time grading rubric with scores updating live. "Finish Session" button.
- `ExperimentSetupSurface`: Form with hypothesis, variable A, variable B, target concept, duration. "Run Experiment" button. Results dashboard after completion.
- `CertificationExamSurface`: Timer, progress bar (50 concepts), concept card with explanation input, submit button. Immediate feedback per concept.
- `MasterBadgeSurface`: Animated badge with share buttons (LinkedIn, Twitter, embed code). Badge details expandable.
- `MasterDashboardSurface`: Earnings card, upcoming sessions list, calendar widget, student progress chart, analytics tab, settings tab.
- `MasterProfileSurface`: Public profile with photo, bio, badges, subjects, pricing, calendar, reviews carousel, "Book Session" CTA.
- `MasterDirectorySurface`: Search + filter + sort + grid of master cards. Each card has photo, rating, price, subject tags, availability indicator, "View Profile" button.
- `AvailabilityCalendarSurface`: Weekly drag-and-drop calendar. Click to add slot, drag to adjust, click to delete. Time zone auto-detected. "Sync with Google Calendar" button.

---

### 2.4 Live Coaching Sessions (FEAT-015)

**Description**: Real-time video, audio, or text sessions between a student and a certified master, optionally augmented by the master's digital persona. The AI assistant provides real-time suggestions to the master (whisper mode) or participates visibly (shared mode).

**User Stories**:
- US-015.1: As a student, I can book a live session with a master by selecting an available time slot from their calendar and paying the session fee. | Must | Booking flow: select subject → browse masters → select master → view calendar → select slot → confirm → payment (Stripe Checkout) → confirmation email/notification. |
- US-015.2: As a student, I can join a live video session from my browser or Tauri app with one click. | Must | Join button is active 5 minutes before scheduled start. No additional software download required. |
- US-015.3: As a student, I can share my screen during a session so the master can see my work. | Must | Screen share is a button in the video call toolbar. Student can share a specific window or the full screen. |
- US-015.4: As a student, I can see the AI assistant's suggestions in real-time during the session (shared mode) or not see them (whisper mode, master-only). | Should | Shared mode requires both parties to consent. Default is whisper mode. |
- US-015.5: As a student, I can rate the master and leave feedback after the session. | Must | Rating is 1-5 stars + optional text. Rating is required to book future sessions. |
- US-015.6: As a master, I can start a session from my dashboard and see the student joined. | Must | Master receives a notification 5 minutes before the session. "Start Session" button initiates the video call. |
- US-015.7: As a master, I can see the AI assistant's real-time suggestions in a sidebar during the session (whisper mode). | Must | Suggestions include: retrieved citations, analogy suggestions, misconception flags, gap detection alerts, and Socratic question prompts. |
- US-015.8: As a master, I can see a transcript of the session in real-time with highlighted student misconceptions. | Must | Transcript is AI-generated (Whisper) and updated every 5 seconds. Misconceptions are flagged with confidence scores. |
- US-015.9: As a master, I can share a whiteboard or diagram with the student during the session. | Should | Whiteboard is a collaborative canvas (Excalidraw or similar). Drawing is synchronized in real-time. |
- US-015.10: As a master, I can record the session (with student consent) and receive an AI-generated post-session analysis with the student's knowledge gaps and recommended next steps. | Must | Recording is stored for 90 days. Post-session analysis is generated within 30 minutes of session end. |
- US-015.11: As a platform, I can enforce session quality by monitoring for no-shows, cancellations, and misconduct. | Must | No-show: student or master doesn't join within 15 minutes of start. Student no-show = full charge. Master no-show = refund + penalty. |

**Functional Requirements**:

| ID | Requirement | Priority | Acceptance Criteria |
|---|---|---|---|
| FR-015.1 | Booking flow supports: subject selection, master browsing, calendar slot selection, payment (Stripe Checkout), confirmation email, and calendar invite (.ics). | Must | Booking is confirmed only after payment succeeds. Student receives confirmation email with session link and calendar invite. |
| FR-015.2 | Video call uses WebRTC (getUserMedia + RTCPeerConnection) with fallback to TURN relay for NAT traversal. | Must | Video quality auto-adjusts based on bandwidth. Minimum viable quality: 320x240 @ 15fps. Target: 720p @ 30fps. |
| FR-015.3 | Session modes: 'text-only' (chat), 'audio-only' (voice), 'video' (full video). Student and master can switch modes during the session. | Must | Mode switch is instant (< 2s). Default is video. Text-only is available for low-bandwidth or privacy-conscious users. |
| FR-015.4 | AI whisper mode: real-time ASR (Whisper) feeds the master's digital persona. Persona generates suggestions every 10-15 seconds. Suggestions are displayed in a non-intrusive sidebar. | Should | Latency from speech to suggestion: < 20 seconds. Suggestions are contextual (relevant to the current topic). |
| FR-015.5 | AI shared mode: persona's suggestions are visible to both student and master in a shared chat panel. Both parties must consent. | Should | Shared mode is opt-in per session. Student can toggle it off during the session. |
| FR-015.6 | Post-session analysis: AI analyzes the transcript to identify: knowledge gaps, misconceptions, topics covered, concepts mastered, and recommended next steps. | Must | Analysis is generated within 30 minutes of session end. Analysis is sent to both student and master via email and in-app notification. |
| FR-015.7 | Post-session Feynman loop: the student can start a Feynman loop on a gap concept identified during the session. The loop is pre-populated with the session context. | Should | One-click "Start Loop on [Gap Concept]" from the post-session analysis. |
| FR-015.8 | Session recording: server-side recording via ffmpeg. Recording is encrypted at rest (AES-256). Access requires both parties' consent. | Must | Recording is opt-in. Consent is logged. Recording URL expires after 90 days. Student can request deletion earlier. |
| FR-015.9 | No-show policy: if student doesn't join within 15 minutes, session is marked "no-show student." Student is charged full amount. Master receives 50% of fee (platform keeps 50% as penalty processing fee). | Must | No-show is automatically detected by the signaling server. No manual intervention. |
| FR-015.10 | No-show policy: if master doesn't join within 15 minutes, session is marked "no-show master." Student is fully refunded. Master receives no payment. Master's no-show rate is tracked. | Must | Master with > 3 no-shows in 30 days is flagged for review. Master with > 5 no-shows is suspended. |
| FR-015.11 | Cancellation policy: > 24 hours before = full refund. < 24 hours = 50% refund (master gets 25%, platform keeps 25%). Session start time = scheduled start. | Must | Cancellation is one-click. Refund is processed automatically via Stripe. |
| FR-015.12 | Session quality: AI moderation flags sessions for: harassment, hate speech, sexual content, dangerous instructions. Flagged sessions are queued for human review. | Must | Moderation is post-session (transcript analysis), not real-time (to avoid latency). Flagged sessions are reviewed within 24 hours. |
| FR-015.13 | Group sessions: up to 5 students can join a session with one master. Pricing is per-student or flat-rate. | Should | Group sessions use MCU (multipoint control unit) for video compositing. Each student sees the master and up to 4 other students in a grid. |
| FR-015.14 | Async coaching: student sends a video message (up to 5 minutes). Master replies with a video message within 24 hours. | Should | Async messages are stored as video files + transcripts. Pricing is per-message. |

**UI Surfaces** (A2UI):
- `BookingFlowSurface`: Wizard with 4 steps: Subject → Master → Calendar → Payment. Progress bar at top.
- `VideoCallSurface`: Main video area (student/master), toolbar (mute, camera, screen share, chat, whiteboard, end call), sidebar (AI suggestions in whisper mode, transcript, chat panel).
- `AISidebarSurface`: Real-time suggestion cards with: citation, analogy suggestion, misconception flag, Socratic question. Each card has "Use This" and "Dismiss" buttons (master only).
- `TranscriptSurface`: Scrollable transcript with speaker labels, timestamps, and highlighted misconceptions. Searchable.
- `PostSessionAnalysisSurface`: Gap concept list, mastery summary, recommended next steps, "Start Feynman Loop" buttons, rating form.
- `SessionRecordingSurface`: Video player with transcript sync (click transcript to jump to timestamp), download button (if within 90 days), delete button.
- `GroupSessionSurface`: Grid layout (1 master + up to 4 students), "Raise Hand" button, "Mute All" button (master only), breakout rooms (future).
- `AsyncMessageSurface`: Video recorder (5-minute limit), send button, inbox with threaded replies, transcript preview.

---

### 2.5 Video Conferencing Infrastructure (FEAT-016)

**Description**: The underlying WebRTC infrastructure for live coaching sessions, built on `flint-realtime-fabric`. Includes signaling, SFU/MCU routing, recording, and Tauri integration.

**User Stories**:
- US-016.1: As a platform engineer, I can monitor the video infrastructure health (SFU worker load, connection count, bandwidth, error rates) in a dashboard. | Must | Dashboard shows real-time metrics, alerts, and auto-scaling events. |
- US-016.2: As a platform engineer, I can auto-scale SFU workers based on CPU/GPU utilization and connection count. | Must | HPA (horizontal pod autoscaler) scales 3-20 replicas. Scale-up: > 70% CPU or > 80% GPU. Scale-down: < 30% CPU for 10 minutes. |
- US-016.3: As a student on a mobile device, I can join a video session with acceptable quality even on a 3G connection. | Must | Video quality auto-downgrades: 720p (WiFi) → 480p (4G) → 240p (3G). Audio is always prioritized. |
- US-016.4: As a Tauri desktop user, I can use my native camera and microphone with noise suppression enabled. | Must | Tauri app uses getUserMedia in WebView (same as web). Native audio processing is a Tauri plugin. |
- US-016.5: As a platform, I can store session recordings securely with encryption at rest and access logging. | Must | Recordings are encrypted with AES-256-GCM. Access log includes: user ID, timestamp, action (view/download/delete), IP address. |

**Functional Requirements**:

| ID | Requirement | Priority | Acceptance Criteria |
|---|---|---|---|
| FR-016.1 | Signaling server uses WebSocket over WSS for offer/answer/ICE exchange. Supports 10,000 concurrent WebSocket connections per replica. | Must | Signaling is stateless. Room state is stored in Redis. 3 replicas minimum for HA. |
| FR-016.2 | SFU (Selective Forwarding Unit) routes video/audio streams between participants. Each SFU worker handles 50 concurrent 1:1 sessions or 10 group sessions (5 participants). | Must | SFU uses simulcast: each participant sends 3 quality layers (low/mid/high), SFU selects the best for each receiver based on bandwidth. |
| FR-016.3 | MCU (Multipoint Control Unit) composites video streams into a single layout for group sessions > 5 participants. | Should | MCU layout: speaker spotlight (largest tile) + grid of others. MCU is CPU-intensive; only used for large groups. |
| FR-016.4 | TURN relay server (coturn or Twilio TURN) for NAT traversal. TURN is used only when direct P2P fails (~15% of connections). | Must | TURN credentials are short-lived (1-hour TTL). Credentials are generated per session. |
| FR-016.5 | Recording pipeline: server-side ffmpeg captures each participant's stream + composite layout. Encoded as MP4 (H.264 + AAC). Stored in S3/R2 with AES-256 encryption. | Must | Recording starts within 5 seconds of session start. File size: ~50MB/hour for 720p. |
| FR-016.6 | Transcription pipeline: Whisper API (or local Whisper model) transcribes audio from recording. Transcript is stored as JSON with timestamps, speaker labels, and confidence scores. | Must | Transcription completes within 10 minutes of session end. Accuracy: > 90% for English. |
| FR-016.7 | Auto-scaling: K8s HPA scales SFU workers based on custom metrics (connection count, GPU utilization). Scale-up: < 2 minutes. Scale-down: < 5 minutes. | Must | Scale-up is triggered at 80% capacity. No session drops during scale-up (new sessions go to new workers). |
| FR-016.8 | Tauri desktop: video call uses WebRTC in WebView. Native audio processing (noise suppression, echo cancellation) is provided by the Tauri `audio` plugin. | Must | Audio quality is equivalent to web. Native audio processing is optional (enabled by default). |
| FR-016.9 | Flutter + Rust over FFI (mobile): video call uses WebRTC in a Flutter WebView / `flutter_inappwebview`. Push notifications for incoming calls (via APNS/FCM). Background audio continues when app is minimized. Tauri is **desktop only**; mobile is a separate Flutter shell over the same Rust substrate via `flutter_rust_bridge`. | Should | CallKit integration (iOS) shows incoming call UI. Android uses full-screen notification. `local_auth` gates the device identity. |
| FR-016.10 | Bandwidth estimation: WebRTC's built-in bandwidth estimation + custom heuristics. Video quality auto-adjusts based on measured bandwidth and packet loss. | Must | Bandwidth estimation updates every 2 seconds. Quality changes are smooth (no jarring drops). |
| FR-016.11 | Fallback: if WebRTC fails (firewall, browser incompatibility), fallback to a relayed audio-only call via WebSocket. | Must | Fallback is automatic. Student sees "Switching to audio-only mode" message. |
| FR-016.12 | Session security: room tokens are JWTs with 1-hour TTL. Room IDs are UUIDs (unguessable). No unauthenticated access. | Must | JWT is signed with platform's secret. Room token is invalidated when the session ends. |

---

### 2.6 Revenue & Marketplace (FEAT-017)

**Description**: The financial engine that handles all transactions between students, masters, and the platform, including session payments, subscriptions, revenue sharing, and payouts.

**User Stories**:
- US-017.1: As a student, I can pay for a coaching session using a credit card via Stripe Checkout. | Must | Payment is one-click for returning students (saved payment method). New students use Stripe Checkout. |
- US-017.2: As a student, I can see a clear breakdown of the session cost before booking: master fee + platform fee + AI premium (if applicable). | Must | Cost breakdown is shown on the booking confirmation page. Total is the final charge. |
- US-017.3: As a master, I can see my earnings dashboard with: total earnings, pending payouts, payout history, and projected earnings. | Must | Earnings are updated in real-time after each session. Payout history is exportable to CSV. |
- US-017.4: As a master, I can set my payout schedule (weekly, bi-weekly, monthly) and payout method (bank transfer, PayPal). | Must | Payout is processed via Stripe Connect. Minimum payout: $10. |
- US-017.5: As a platform admin, I can see a revenue dashboard with: total revenue, revenue by tier, revenue by subject, revenue by master, platform fees, and AI premium revenue. | Must | Dashboard is updated hourly. Data is exportable to CSV. |
- US-017.6: As a platform admin, I can adjust the platform revenue share percentage per master tier (Beginner: 30%, Verified: 25%, Elite: 20%). | Must | Changes apply to new sessions only. Existing sessions use the rate at booking time. |
- US-017.7: As a student, I can purchase a subscription package for unlimited sessions with a specific master (e.g., "$199/month for unlimited physics coaching with Master Alice"). | Should | Subscription is managed via Stripe Subscriptions. Student can cancel anytime. |

**Functional Requirements**:

| ID | Requirement | Priority | Acceptance Criteria |
|---|---|---|---|
| FR-017.1 | Session payment is processed via Stripe Checkout (one-time) or Stripe PaymentIntent (saved method). Currency: USD. | Must | Payment succeeds in < 10 seconds. Failed payments show retry option with error message. |
| FR-017.2 | Platform revenue share is: 30% (Beginner), 25% (Verified), 20% (Elite), negotiable (Celebrity/Institution). Platform fee per session: $1.00 (Beginner), $0.50 (Verified), $0.25 (Elite). | Must | Revenue share is configurable per master. Changes apply to new sessions. |
| FR-017.3 | Master earnings are calculated at session completion: (student_paid - platform_fee - stripe_fee) × master_share. | Must | Earnings are calculated immediately. Stripe transfer is initiated within 1 hour. |
| FR-017.4 | Payouts are processed via Stripe Connect Express. Minimum payout: $10. Default schedule: weekly. | Must | Payout arrives in master's bank account within 2-5 business days. |
| FR-017.5 | Refunds: > 24h cancellation = full refund. < 24h = 50% refund. No-show student = no refund. No-show master = full refund. | Must | Refunds are processed automatically. Refund appears in student's account within 5-10 business days. |
| FR-017.6 | Dispute resolution: platform mediates disputes. Evidence includes: session recording (if consented), transcript, chat logs, ratings. Platform decision is final but can be appealed to Stripe dispute resolution. | Must | Dispute is resolved within 7 days. Both parties are notified of the decision. |
| FR-017.7 | Tax handling: US masters receive 1099-K (if > $600/year). International masters receive W-8BEN. Tax forms are auto-generated and sent by January 31. | Must | Tax forms are generated from Stripe Connect data. Platform provides a tax dashboard for masters. |
| FR-017.8 | AI premium: when the digital persona is active in a session (whisper or shared mode), an additional fee is charged. Split: 50% to master, 50% to platform. | Must | AI premium is shown in cost breakdown before booking. |
| FR-017.9 | Fraud detection: AI monitors for suspicious patterns (e.g., master creating fake student accounts, circular booking). Banned accounts forfeit earnings. | Must | Fraud detection runs nightly. Suspicious accounts are flagged for manual review. |
| FR-017.10 | Revenue analytics: platform admin dashboard shows revenue by subject, by master tier, by session mode, by month, and by cohort. | Must | Dashboard is updated hourly. Data is exportable to CSV. |

---

## 3. Data Contracts & API Specifications

### 3.1 Coach Catalog API

#### GET /api/v2/coaches

**Query Parameters**:
- `subject` (string, optional): Filter by subject tag
- `language` (string, optional): Filter by language
- `proficiency` (string, optional): Filter by proficiency level
- `min_rating` (float, optional): Minimum student rating (1-5)
- `max_price` (integer, optional): Maximum price in cents
- `sort` (string, optional): `relevance` | `rating` | `mastery_rate` | `lvs_delta` | `newest`
- `page` (integer, default 1): Pagination
- `per_page` (integer, default 20): Results per page

**Response**:
```json
{
  "coaches": [
    {
      "id": "uuid",
      "name": "Dr. Feynman Physics Coach",
      "description": "A retrieval-grounded physics tutor based on Richard Feynman's lectures...",
      "subjects": ["physics", "quantum_mechanics"],
      "languages": ["en"],
      "proficiency_levels": ["novice", "peer", "skeptic"],
      "modalities": ["text", "diagram", "equation"],
      "rating": 4.8,
      "rating_count": 342,
      "mastery_rate": 0.78,
      "lvs_delta": 0.23,
      "coverage_score": 0.94,
      "voice_fidelity_score": 0.85,
      "base_rate_cents": 0,
      "ai_premium_cents": 0,
      "creator": {
        "id": "uuid",
        "name": "California Institute of Technology",
        "verified": true
      },
      "deployment": {
        "mode": "platform-hosted",
        "endpoint": "https://coaches.prometheus-ags.com/feynman/a2a"
      },
      "status": "active",
      "created_at": "2026-06-01T00:00:00Z"
    }
  ],
  "total": 156,
  "page": 1,
  "per_page": 20
}
```

#### POST /api/v2/coaches/{coach_id}/preview

**Request**:
```json
{
  "message": "Explain quantum entanglement to me like I'm a college student."
}
```

**Response** (streamed AG-UI events):
```json
{ "type": "TEXT_MESSAGE_CONTENT", "delta": "Quantum entanglement is..." }
{ "type": "CITATION", "ref": "Feynman Lectures, Vol. III, Chapter 18", "score": 0.95 }
{ "type": "RUN_FINISHED" }
```

### 3.2 Master Certification API

#### GET /api/v2/certification/{subject}/progress

**Response**:
```json
{
  "subject": "quantum_mechanics",
  "student_id": "uuid",
  "pillar_1": {
    "status": "complete",
    "concepts_total": 50,
    "concepts_mastered": 50,
    "retention_checks_passed": 50,
    "mastery_rate": 0.85,
    "completed_at": "2026-05-15T00:00:00Z"
  },
  "pillar_2": {
    "status": "in_progress",
    "sessions_required": 10,
    "sessions_completed": 7,
    "average_pedagogy_score": 0.82,
    "required_average": 0.80
  },
  "pillar_3": {
    "status": "pending",
    "lvs": 0.72,
    "cohort_percentile": 85,
    "required_percentile": 80,
    "experiments_required": 1,
    "experiments_completed": 0
  },
  "eligible_for_exam": false,
  "next_steps": [
    "Complete 3 more pedagogical sessions (Pillar 2)",
    "Run one pedagogical experiment (Pillar 3)"
  ]
}
```

#### POST /api/v2/certification/{subject}/exam

**Request**: Empty (exam is auto-generated based on mastered corpus)

**Response**:
```json
{
  "exam_id": "uuid",
  "subject": "quantum_mechanics",
  "duration_minutes": 120,
  "concepts_total": 50,
  "concepts": [
    {
      "concept_id": "uuid",
      "title": "Wave-Particle Duality",
      "question_type": "explain",
      "instructions": "Explain wave-particle duality in your own words, suitable for a first-year physics student."
    }
  ],
  "expires_at": "2026-07-01T14:00:00Z"
}
```

#### POST /api/v2/certification/{subject}/exam/{exam_id}/submit

**Request**:
```json
{
  "answers": [
    {
      "concept_id": "uuid",
      "explanation_text": "Wave-particle duality means that..."
    }
  ]
}
```

**Response**:
```json
{
  "exam_id": "uuid",
  "status": "graded",
  "overall_score": 0.83,
  "passed": true,
  "badge_id": "uuid",
  "badge_url": "https://credentials.prometheus-ags.com/badges/uuid",
  "issued_at": "2026-07-01T14:30:00Z",
  "expires_at": "2027-07-01T14:30:00Z"
}
```

### 3.3 Coaching Session API

#### POST /api/v2/sessions

**Request**:
```json
{
  "master_id": "uuid",
  "subject": "quantum_mechanics",
  "mode": "human-whisper",
  "scheduled_start": "2026-07-05T15:00:00Z",
  "scheduled_end": "2026-07-05T16:00:00Z",
  "student_paid_cents": 5000,
  "ai_premium_cents": 500
}
```

**Response**:
```json
{
  "session_id": "uuid",
  "status": "scheduled",
  "master_id": "uuid",
  "student_id": "uuid",
  "mode": "human-whisper",
  "scheduled_start": "2026-07-05T15:00:00Z",
  "scheduled_end": "2026-07-05T16:00:00Z",
  "student_paid_cents": 5000,
  "platform_fee_cents": 150,
  "master_earnings_cents": 3400,
  "ai_premium_cents": 500,
  "join_url": "https://meet.prometheus-ags.com/s/uuid",
  "calendar_ics_url": "https://api.prometheus-ags.com/sessions/uuid/calendar.ics"
}
```

#### GET /api/v2/sessions/{session_id}/video/token

**Response**:
```json
{
  "room_token": "eyJhbGciOiJIUzI1NiIs...",
  "room_url": "wss://video.prometheus-ags.com/signal",
  "sfu_url": "turn:video.prometheus-ags.com:3478",
  "turn_credentials": {
    "username": "uuid",
    "credential": "short-lived-password"
  },
  "expires_at": "2026-07-05T16:00:00Z"
}
```

### 3.4 Revenue API

#### GET /api/v2/masters/{master_id}/earnings

**Response**:
```json
{
  "master_id": "uuid",
  "currency": "USD",
  "total_earnings_cents": 125000,
  "pending_payout_cents": 45000,
  "this_week_cents": 12000,
  "this_month_cents": 45000,
  "lifetime_sessions": 89,
  "lifetime_students": 34,
  "average_session_earnings_cents": 1404,
  "payout_history": [
    {
      "payout_id": "uuid",
      "amount_cents": 30000,
      "status": "succeeded",
      "scheduled_at": "2026-06-28T00:00:00Z",
      "arrived_at": "2026-07-02T00:00:00Z"
    }
  ]
}
```

---

## 4. State Machines

### 4.1 Coaching Session State Machine

```
[Scheduled] --(student cancels > 24h)--> [Cancelled] --(full refund)--> [Refunded]
[Scheduled] --(student cancels < 24h)--> [Cancelled] --(50% refund)--> [Partially Refunded]
[Scheduled] --(master cancels)--> [Cancelled] --(full refund)--> [Refunded]
[Scheduled] --(5 min before start)--> [Ready] --(student joins)--> [Student Joined]
[Ready] --(master joins)--> [Master Joined] --(student already joined)--> [In Progress]
[Student Joined] --(master joins)--> [In Progress]
[Master Joined] --(student joins)--> [In Progress]
[In Progress] --(student or master ends)--> [Completed] --(AI analysis)--> [Analyzed]
[In Progress] --(student doesn't join in 15 min)--> [No-Show Student] --(full charge)--> [Charged]
[In Progress] --(master doesn't join in 15 min)--> [No-Show Master] --(full refund)--> [Refunded]
[In Progress] --(master no-show detected)--> [No-Show Master]
[In Progress] --(student leaves early)--> [Completed] --(billed for actual duration)--> [Charged]
[Completed] --(recording processed)--> [Recorded] --(transcription)--> [Transcribed] --(AI analysis)--> [Analyzed]
[Analyzed] --(student rates)--> [Rated] --(master rates)--> [Closed]
```

### 4.2 Coach Lifecycle State Machine

```
[Draft] --(expert submits)--> [Pending Review] --(admin approves)--> [Approved]
[Pending Review] --(admin rejects)--> [Rejected] --(expert resubmits)--> [Pending Review]
[Approved] --(expert publishes)--> [Active] --(expert updates)--> [Updating] --(re-index complete)--> [Active v2]
[Active] --(DMCA complaint)--> [Suspended] --(admin review clears)--> [Active]
[Active] --(quality drops below threshold)--> [Flagged] --(expert fixes)--> [Active]
[Active] --(expert retires)--> [Retired] --(grace period 30d)--> [Archived]
[Active v2] --(students migrate)--> [Active v2] --(grace period ends)--> [Active v2] (v1 archived)
```

### 4.3 Master Certification State Machine

```
[Student] --(completes Pillar 1)--> [Pillar 1 Complete]
[Pillar 1 Complete] --(completes Pillar 2)--> [Pillar 2 Complete]
[Pillar 2 Complete] --(completes Pillar 3)--> [Eligible for Exam]
[Eligible for Exam] --(passes exam ≥ 0.8)--> [Certified Master] --(badge issued)--> [Active Master]
[Eligible for Exam] --(fails exam)--> [Exam Failed] --(retry in 30 days)--> [Eligible for Exam]
[Active Master] --(re-certification due)--> [Re-Certification Due] --(passes)--> [Active Master]
[Re-Certification Due] --(fails)--> [Grace Period] --(90 days)--> [Suspended] --(re-earns)--> [Active Master]
[Grace Period] --(no action)--> [Suspended]
[Suspended] --(6 months inactive)--> [Revoked]
```

---

## 5. Error Handling & Edge Cases

| Scenario | Behavior | Fallback |
|---|---|---|
| Coach fails during a Feynman loop | Log error, fallback to generic platform AI tutor, notify coach creator | Student receives generic explanation. Creator is alerted. |
| Master drops mid-session (network) | AI persona continues the session in "shared mode" until master reconnects or session ends | Student is informed: "Master has disconnected. Continuing with AI assistant." |
| WebRTC fails entirely | Fallback to audio-only WebRTC, then to text-only chat via WebSocket | Session continues without video. No refund. |
| Payment fails during booking | Show error, allow retry with different card, hold slot for 5 minutes | Slot is released if payment fails after 5 minutes. |
| Master is overbooked (calendar bug) | Last booking wins, earlier bookings are cancelled with full refund + apology credit | Platform credit: $5 for inconvenience. |
| Recording fails | Session continues without recording. Post-session analysis is generated from transcript only (if ASR succeeded) or skipped. | Student and master are notified. No refund for recording failure. |
| AI analysis fails | Session is marked "Completed" but "Analysis Pending." Manual review queue. | Platform staff can generate analysis manually. |
| Student under 18 books a live session | Blocked. Live sessions require age verification (18+). Under-18 can use persona-only mode. | Error message: "Live coaching requires you to be 18+. You can use [Coach Name] in text mode." |
| Master's Stripe Connect account is suspended | Master cannot receive new bookings. Existing sessions proceed. Earnings held in escrow until resolved. | Master is notified to resolve Stripe issue. Platform support assists. |
| Refund processing fails (Stripe error) | Platform manually processes refund from platform reserve fund. | Platform absorbs the loss. Stripe issue is escalated. |
| Video quality is too poor (< 240p for > 60 seconds) | Auto-switch to audio-only. Student is notified. Session continues. | No refund — audio-only is a valid session mode. |
| AI assistant hallucinates during a live session | Master can override/ignore suggestions. "Flag Suggestion" button reports to platform. | Suggestion is logged for review. No impact on session. |
| Student and master are in different time zones | Platform converts all times to student's timezone for display. Master sees their own timezone. | Booking is stored in UTC. Conversion is client-side. |
| Group session has 1 participant (others no-show) | Session becomes 1:1 at the same rate. No refund for other students. | Master is notified. Session continues with remaining participants. |

---

## 6. Accessibility & Localization

| Requirement | Approach |
|---|---|
| **WCAG 2.1 AA** | All video call UI elements are keyboard-navigable. Captions are generated for all sessions (auto-transcription). |
| **Screen Reader Support** | Video call toolbar has ARIA labels. Chat messages are announced. AI suggestions are read aloud. |
| **Localization** | Coach catalog, Master Directory, and booking flow are localized in: English, Spanish, Mandarin, Japanese, German, French, Portuguese, Korean. |
| **RTL Support** | Booking calendar and coach catalog support RTL for Arabic and Hebrew. |
| **Reduced Motion** | Video call animations (join/leave transitions) respect `prefers-reduced-motion`. |
| **Color Contrast** | All video call UI elements pass WCAG AA contrast ratios. Dark mode is default. |
| **Captions** | Real-time captions (ASR) are available in all supported languages. Captions are on by default. |
| **Sign Language** | Future: AI-powered sign language interpretation (sign language avatar) for deaf students. |
| **Hearing Impaired** | Transcript is always available. Chat is primary communication mode for audio-only sessions. |
| **Vision Impaired** | High-contrast mode. Screen reader compatibility. Future: audio-only coaching with AI description of diagrams. |

---

*End of Functional Specification Addendum*
