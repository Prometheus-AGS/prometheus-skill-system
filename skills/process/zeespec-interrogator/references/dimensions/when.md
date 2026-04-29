# When Dimension — Time & Events

**Interrogative**: When do things happen — what triggers, sequences, and schedules govern the system?
**Core Concern**: Events, triggers, ordering, timing, deadlines, lifecycle.
**Criticality**: HIGH — undefined triggers = undefined system behavior.
**Critical Threshold**: 60%

The When dimension defines the event fabric of the system. Without it, the system
has no defined lifecycle, race conditions go unrecognized, and timing-sensitive
requirements are discovered through failures rather than design.

---

## Q1. What triggers the system to start, stop, or change state?

**Why it matters**: Every system state transition must have a defined trigger.
Undefined triggers produce undefined behavior under edge conditions.

**Good answer example**: "Start: Kubernetes pod scheduling on ArgoCD sync.
State change: new inference job arrives in mailbox queue. Stop: graceful
shutdown on SIGTERM with 30s drain window."

**Implicit implication**: The system will start, stop, and change state in
undefined ways under load or failure. Debugging will require inference from logs.

---

## Q2. What is the expected request/event rate and how does it vary over time?

**Why it matters**: Systems designed for uniform load fail under bursty load.
Peak load assumptions drive capacity planning and queue depth decisions.

**Good answer example**: "Baseline: 2–5 concurrent inference sessions. Peak:
20 concurrent sessions at business day start. No scheduled batch — all event-driven."

**Implicit implication**: Capacity will be designed for an assumed load profile.
The system may queue indefinitely or shed load unexpectedly at real peak.

---

## Q3. What are the latency and throughput requirements?

**Why it matters**: Latency and throughput SLOs define the performance envelope.
Without them, performance is optimized toward unmeasured goals.

**Good answer example**: "First token latency: < 400ms at p95. Throughput:
>= 20 concurrent sessions on L4. End-to-end job latency: < 10s for 2048-token
completions."

**Implicit implication**: Performance will be measured against no defined baseline.
The system may ship with latency that is technically functional but commercially
unacceptable.

---

## Q4. What is the expected system lifetime and maintenance schedule?

**Why it matters**: Systems designed for short lifetimes accumulate technical debt
that becomes expensive when the system outlives its design. Maintenance schedules
determine when updates can be applied without user impact.

**Good answer example**: "5-year target lifetime. Maintenance window: Sundays
2–4am CT. Model updates: rolling, no maintenance window required."

**Implicit implication**: The system will be designed for its immediate needs.
Lifetime extension will require unplanned refactoring.

---

## Q5. What are the ordering and sequencing constraints between components or operations?

**Why it matters**: Undefined ordering is the root cause of race conditions, deadlocks,
and consistency violations. In distributed systems, ordering is always a design choice.

**Good answer example**: "Cedar policy must be loaded before any request is accepted.
TurboQuant compression runs before KV cache allocation. Mailbox delivery is FIFO
per session, concurrent across sessions."

**Implicit implication**: Ordering will be whatever the runtime produces.
Race conditions and consistency violations will be discovered through production failures.

---

## Q6. What are the timeout, retry, and deadline policies?

**Why it matters**: Undefined timeouts allow resource exhaustion. Undefined retries
produce either unnecessary failures or infinite loops under partial failures.

**Good answer example**: "Inference job timeout: 60s. HTTP client timeout: 10s.
Retry policy: 2 retries with exponential backoff (1s, 2s). No retry on 4xx.
Deadline propagation: parent timeout minus 100ms passed to child requests."

**Implicit implication**: Timeouts will be absent or arbitrary. The system will
hang under partial failures, exhaust connections, or retry in ways that amplify failures.

---

## Q7. What schedules govern background operations?

**Why it matters**: Background operations (garbage collection, cache eviction, health checks,
audit log rotation) compete with foreground work. Undefined schedules produce undefined
contention.

**Good answer example**: "KV cache eviction: triggered on >80% VRAM utilization.
prometheus-knowledge lint pass: every 6 hours. ArgoCD sync: every 5 minutes.
No scheduled jobs compete with inference during peak hours."

**Implicit implication**: Background operations will run at system-determined
times. They may compete with foreground work during peak hours.

---

## Q8. What are the data retention and expiry policies?

**Why it matters**: Data that is retained longer than needed creates compliance
risk. Data that expires too early creates operational gaps.

**Good answer example**: "Inference logs: 30-day retention, then deleted.
KnowMe conversation history: 90-day default, user-configurable. Model weights:
indefinite retention. Session tokens: 24h expiry."

**Implicit implication**: Data will be retained indefinitely or deleted
arbitrarily. Compliance audits will reveal uncontrolled retention.

---

## Q9. What events must be logged, and at what granularity?

**Why it matters**: Logging granularity is a performance cost paid at request time.
Undefined logging produces either overwhelming noise or empty audit trails.

**Good answer example**: "Log: every inference job start/complete with latency.
Log: every Cedar policy decision. Do not log: token-level model output.
Retention: 30 days. Structured JSON to stdout. No PII in logs."

**Implicit implication**: Logging will be added reactively after incidents.
The audit trail will have gaps precisely where incidents occurred.

---

## Q10. What happens at end-of-life — deprecation, migration, shutdown?

**Why it matters**: Systems without defined end-of-life procedures accumulate
zombie instances, orphaned data, and unmigrated clients.

**Good answer example**: "Deprecation notice: 90 days before shutdown.
Data export: available for 90 days post-shutdown. Replacement: must be
live and validated before primary system goes offline. No cold shutdowns."

**Implicit implication**: The system will be kept alive beyond its useful life
because shutdown is undefined. Or it will be shut down abruptly, stranding clients.
