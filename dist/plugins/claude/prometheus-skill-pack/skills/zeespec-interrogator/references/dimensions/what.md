# What Dimension — Data & Entities

**Interrogative**: What does the system manage, store, or transform?
**Core Concern**: Data models, entities, schemas, inputs, outputs, artifacts.
**Criticality**: STANDARD — structural gaps tolerated if Why/Who/When are clear.
**Critical Threshold**: 50%

The What dimension defines the information substrate of the system. Without it,
storage schemas are improvised, APIs have undocumented contracts, and the system
accumulates implicit data models that diverge from reality over time.

---

## Q1. What are the primary entities the system manages?

**Why it matters**: Primary entities are the core domain objects. Every schema,
API, and database table derives from them. Undefined entities produce schema sprawl.

**Good answer example**: "Agent sessions, inference jobs, model weights, Cedar policies,
user identities, constraint manifests. Each is a first-class entity with an ID, lifecycle,
and ownership."

**Implicit implication**: Entities will be invented during implementation.
The data model will be inconsistent, with overlapping and duplicate concepts.

---

## Q2. What are the inputs the system accepts?

**Why it matters**: Inputs define the contract between the system and its callers.
Undefined inputs produce undefined validation, which means undefined failure modes.

**Good answer example**: "HTTP: OpenAI-compatible /v1/chat/completions JSON.
MCP: JSON-RPC 2.0 tool calls. Files: Markdown and plain text into pk-watcher inbox.
All inputs: UTF-8 encoded, max 128KB per request."

**Implicit implication**: Input validation will be implemented case-by-case.
Malformed inputs will produce undefined behavior or silent corruption.

---

## Q3. What are the outputs and artifacts the system produces?

**Why it matters**: Outputs define the system's value delivery. Undefined output
formats produce consumer integration failures.

**Good answer example**: "Streaming token output via SSE. Constraint manifests as JSON.
Wiki articles as Markdown with YAML frontmatter. All outputs: versioned, schema-validated."

**Implicit implication**: Output format will be defined by whatever the first
implementation produces. Consumers will reverse-engineer the format.

---

## Q4. What data must be persisted, and what can be ephemeral?

**Why it matters**: Persistence decisions determine durability requirements,
storage costs, and disaster recovery scope. Undefined persistence is undefined
data loss risk.

**Good answer example**: "Persistent: model weights, wiki articles, constraint manifests,
Cedar policies, user sessions. Ephemeral: KV cache, in-flight inference state,
HTTP connection state. Ephemeral loss is acceptable; persistent loss is not."

**Implicit implication**: All data may be treated as either persistent or ephemeral
depending on implementation convenience. Data loss will be unclassified.

---

## Q5. What are the data quality and integrity requirements?

**Why it matters**: Undefined quality requirements produce systems that silently
corrupt or lose data without detection.

**Good answer example**: "Wiki articles: must have valid YAML frontmatter or they
are rejected at ingest. Inference logs: append-only, no deletion. Cedar policies:
schema-validated before activation. No silent data corruption acceptable."

**Implicit implication**: Data quality will be checked informally. Corrupted data
will enter the system and be discovered through incorrect outputs or user reports.

---

## Q6. What schemas or standards govern the data model?

**Why it matters**: Schema governance prevents model drift. Without it, the data
model diverges between implementations, versions, and teams.

**Good answer example**: "OpenAI API compatibility for inference I/O. W3C Verifiable
Credentials for Kaia attestations. YAML with defined frontmatter schema for wiki articles.
JSON Schema draft-07 for all manifest contracts."

**Implicit implication**: Schemas will be informal and undocumented. API consumers
will receive different structures as implementations evolve.

---

## Q7. What is the data volume and growth rate?

**Why it matters**: Storage technology decisions (flat files, SurrealDB, object storage)
depend on volume and growth. Wrong choices become expensive to migrate.

**Good answer example**: "Wiki: 500–2,000 articles, slow growth (< 10/week). Inference
logs: ~50MB/day at full load. Model weights: 10–100GB fixed. KV cache: up to 24GB
VRAM, ephemeral."

**Implicit implication**: Storage will be sized for current state. Growth will
surprise operations. Migrations will occur under production pressure.

---

## Q8. What are the sensitivity and classification levels of the data?

**Why it matters**: Sensitivity classification determines encryption requirements,
access logging, and retention policy. Unclassified data is treated as insensitive.

**Good answer example**: "Sensitive: user conversation data (PII if any). Confidential:
enterprise client configurations and Cedar policies. Public: wiki articles, open-source
model weights. Internal: inference logs."

**Implicit implication**: All data will be treated with the same level of protection
(typically the lowest). Sensitive data will be under-protected.

---

## Q9. What are the migration and versioning requirements?

**Why it matters**: Systems that cannot migrate their data safely cannot evolve
their data model. Schema versioning is required for any non-trivial system.

**Good answer example**: "All schemas are versioned. Wiki articles include a schema
version in frontmatter. Migrations are backward-compatible for one major version.
No breaking schema changes without a migration script."

**Implicit implication**: Schema changes will break existing data. Migrations will
be written under pressure with no tested path.

---

## Q10. What data must never exist or be generated?

**Why it matters**: Explicit prohibition is more enforceable than implicit expectation.
Defining what must not be generated prevents sensitive data from appearing in logs,
caches, or outputs.

**Good answer example**: "No plaintext passwords in any log or output. No PII in
inference logs. No other tenants' data in any API response. No model weights
served over unauthenticated endpoints."

**Implicit implication**: Prohibited data will be defined after it appears
in production. Cleanup will be reactive and incomplete.
