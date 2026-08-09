# How Dimension — Function & Process

**Interrogative**: How does the system work — behavior, algorithms, protocols, implementation?
**Core Concern**: Mechanisms, APIs, protocols, algorithms, error handling, integration patterns.
**Criticality**: STANDARD — structural gaps tolerated if Why/Who/When are clear.
**Critical Threshold**: 50%

The How dimension is the closest to implementation, but it is still a specification
question. It defines the behavioral contract of the system — not the code, but the
agreed-upon behavior that the code must produce. Undefined How answers are where
implementation divergence and API contract violations originate.

---

## Q1. What protocols and APIs govern communication between components?

**Why it matters**: Protocol mismatches are some of the hardest bugs to debug.
Every inter-component boundary must have a defined protocol before implementation.

**Good answer example**: "UAR ↔ inference: OpenAI-compatible HTTP/1.1 REST.
MCP tools: JSON-RPC 2.0 over SSE (Axum). A2A: JSON-RPC 2.0 per spec.
AG-UI: SSE stream with agui.* event types. Internal: Rust function calls."

**Implicit implication**: Protocols will be chosen by the first person to implement
each boundary. Incompatibilities will be discovered during integration.

---

## Q2. What are the core algorithms or processing logic the system implements?

**Why it matters**: Algorithmic correctness requirements cannot be verified if the
algorithm is undefined. Implementation bugs will not be recognized as bugs.

**Good answer example**: "TurboQuant: ICLR 2026 FWHT-based 3-bit KV compression.
Parking-lot scheduler: priority queue with admission control. TF-IDF search:
BM25 variant with stemming. Cedar policy evaluation: Amazon Cedar engine."

**Implicit implication**: Algorithms will be chosen for implementation convenience.
The system may implement a functionally different algorithm than stakeholders expect.

---

## Q3. How does the system handle errors, degradation, and partial failures?

**Why it matters**: Error handling is a primary design surface. Undefined error
handling produces undefined user experience under failure conditions.

**Good answer example**: "Inference timeout: return 504 with retry-after header.
Model OOM: drain queue, log OOM, restart inference process. Partial failure:
return partial result with explicit incomplete flag. No silent failures."

**Implicit implication**: Error handling will be added reactively after errors
occur in production. Users will receive undefined behavior under failure.

---

## Q4. What are the authentication and authorization mechanisms?

**Why it matters**: AuthN/AuthZ are non-fungible requirements. Defining them here
prevents security gaps that are expensive to retrofit.

**Good answer example**: "Consumer: Supabase JWT via Ory Kratos. Enterprise: OIDC
with client certificates. Authorization: Cedar policy engine with RBAC. No
API key authentication for production endpoints."

**Implicit implication**: Authentication will be added when it is first needed.
Endpoints may be temporarily or permanently unauthenticated.

---

## Q5. How does the system scale — what is the scaling model?

**Why it matters**: Scaling model determines infrastructure provisioning. An
incorrectly assumed scaling model produces bottlenecks that require architectural change.

**Good answer example**: "Inference: single-model-per-GPU, vertical scaling only.
Memory server: horizontal (stateless MCP tools). UAR: horizontal, mailbox-per-agent.
No auto-scaling — manual scale events via ArgoCD."

**Implicit implication**: Scaling decisions will be made under load pressure.
The system will be scaled in ways that expose architectural bottlenecks.

---

## Q6. How does the system handle concurrent access and shared state?

**Why it matters**: Concurrency bugs are the hardest class of bugs to reproduce
and fix. Defining concurrency constraints before implementation prevents them.

**Good answer example**: "KV cache: Arc<RwLock<T>> for concurrent read, exclusive
write during eviction. Inference sessions: isolated per-session state, no sharing.
SurrealDB: optimistic concurrency with version tags. parking_lot over std::sync."

**Implicit implication**: Concurrency will be handled by whatever the implementation
produces. Race conditions and deadlocks will surface under production load.

---

## Q7. What are the deployment and release mechanisms?

**Why it matters**: Release mechanisms determine the risk and speed of change.
Undefined release mechanisms produce inconsistent deployment practices.

**Good answer example**: "Blue-green deployment via ArgoCD. Feature flags via
environment variables (no runtime toggle service). Canary: 10% traffic shift,
monitor 30 minutes, then full cut-over or rollback. Rollback: ArgoCD sync to prior tag."

**Implicit implication**: Deployments will use whatever mechanism is most expedient.
Rollback may require manual state recovery.

---

## Q8. How is the system monitored, alerted, and debugged?

**Why it matters**: Observability requirements drive implementation choices.
Undefined observability produces systems that cannot be diagnosed in production.

**Good answer example**: "Metrics: Prometheus pull every 15s. Alerts: Grafana for
p99 latency > 500ms and GPU utilization > 90%. Structured logs: JSON to stdout,
correlated by trace_id. Distributed tracing: Langfuse for LLM call spans."

**Implicit implication**: Monitoring will be added after the first incident
that required it. The first incident will be difficult or impossible to diagnose.

---

## Q9. What are the build, test, and quality gate requirements?

**Why it matters**: Quality gates define the conditions under which code is
allowed to reach production. Undefined quality gates produce undefined quality.

**Good answer example**: "Build: cargo build --release with zero warnings (-D warnings).
Tests: cargo test --workspace, >= 80% coverage. Linting: cargo clippy, zero warnings.
No PR merges without CI green. No production deploys without integration test pass."

**Implicit implication**: Quality gates will be informal. Code will reach
production without passing defined quality checks.

---

## Q10. How does the system integrate with external services and dependencies?

**Why it matters**: External integrations are the highest-risk surface. Undefined
integration contracts produce breakages when external services change.

**Good answer example**: "Anthropic API: OpenAI-compatible client via liter-llm.
Version pinned. Fallback: local model if API unreachable. SurrealDB: embedded
RocksDB in production (no remote dependency). GCP: Terraform-managed, pinned
provider versions."

**Implicit implication**: External integrations will be implemented without version
pinning or fallback. External service changes will break the system without warning.
