# Where Dimension — Location & Network

**Interrogative**: Where does the system execute, store, and communicate?
**Core Concern**: Deployment topology, network boundaries, geographic constraints, latency paths.
**Criticality**: STANDARD — structural gaps tolerated if Why/Who/When are clear.
**Critical Threshold**: 50%

The Where dimension defines the physical and logical topology of the system.
Infrastructure decisions made without this clarity are expensive to undo:
region changes require data migration, network topology changes require
re-architecting service meshes, and sovereignty violations require system redesign.

---

## Q1. Where does the system execute — what hardware, region, or cloud?

**Why it matters**: Execution location determines latency, cost, compliance jurisdiction,
and sovereignty. Cloud vs. on-premise is a strategic, not just technical, decision.

**Good answer example**: "Primary inference: RTX 4070 Ti local machine (Prosper, TX).
Scale: GCP L4 (us-central1) on K3s. No execution outside US regions.
Sovereign requirement: no model execution on third-party managed infrastructure."

**Implicit implication**: Execution location will be wherever is most convenient
for the first deployment. Geographic sovereignty constraints will be violated
without recognition.

---

## Q2. Where is data stored, and what are the geographic boundaries?

**Why it matters**: Data residency is a compliance and sovereignty question.
Data stored in the wrong geography violates legal constraints and contractual commitments.

**Good answer example**: "SurrealDB: local NVMe for hot data, GCP Persistent Disk
for backup (us-central1 only). Model weights: local storage, never cloud-synced.
No data crosses US jurisdiction."

**Implicit implication**: Data will be stored wherever the implementation writes it.
Residency violations will be discovered during compliance audits.

---

## Q3. Where do system components communicate across network boundaries?

**Why it matters**: Cross-boundary communication defines the security perimeter.
Undefined perimeters produce undefined exposure.

**Good answer example**: "Internal: UAR → candle-vllm via localhost gRPC.
External: Claude API via HTTPS. No inbound public exposure of inference endpoints.
Envoy Gateway terminates TLS at the edge."

**Implicit implication**: Network boundaries will emerge from implementation choices.
Services will be exposed or isolated without deliberate design.

---

## Q4. What are the latency characteristics of the deployment topology?

**Why it matters**: Latency budgets must be allocated across network hops. If the
topology is undefined, latency budget allocation is impossible.

**Good answer example**: "Local machine → model: <5ms (shared memory). Local → GCP L4:
< 30ms (us-central1). GCP L4 → client: < 50ms. Total inference path budget: < 350ms."

**Implicit implication**: Latency will be whatever the topology produces. SLA
commitments may be unachievable with the deployed topology.

---

## Q5. What are the network security boundaries and trust zones?

**Why it matters**: Trust zones define which services can communicate with which,
under what authentication model. Undefined trust zones produce implicit full-mesh trust.

**Good answer example**: "Zone 0 (untrusted): public internet. Zone 1 (edge): Envoy Gateway.
Zone 2 (internal): UAR, inference, memory services. Zone 3 (control plane): ArgoCD,
K3s API. Zones communicate only with adjacent zones via defined protocols."

**Implicit implication**: All internal services will trust each other implicitly.
A compromised service in any zone will have access to all other zones.

---

## Q6. Where are secrets and credentials stored and accessed?

**Why it matters**: Secret management location determines the blast radius of a
compromise. Secrets in environment variables, code, or plaintext files are
accessible to any process on the host.

**Good answer example**: "API keys: GCP Secret Manager. Kubernetes secrets: SOPS-encrypted
in git, decrypted at deploy time. Local development: .env files not committed.
No secrets in Docker images or build artifacts."

**Implicit implication**: Secrets will be stored in the most convenient location.
Environment variable leakage, committed .env files, and image-embedded credentials
will occur.

---

## Q7. Where does the system fail over to if the primary location is unavailable?

**Why it matters**: Without a defined failover target, any outage is a full outage.
The failover path must be pre-tested or it is not a failover — it is an aspiration.

**Good answer example**: "Primary: local RTX 4070 Ti. Failover: GCP L4 (auto via
ArgoCD and K3s). Manual failover time: < 5 minutes. No automatic client rerouting —
ops action required."

**Implicit implication**: Failover will be improvised during an outage. Recovery
time will be undefined and longer than stakeholders expect.

---

## Q8. What geographic or jurisdictional constraints apply?

**Why it matters**: Jurisdictional constraints are hard constraints from law, contract,
or policy. They cannot be satisfied retroactively.

**Good answer example**: "US jurisdiction only. No data processing in EU without
GDPR compliance review. Enterprise clients in Texas: Texas TDPA may apply to
consumer data. ITAR does not apply."

**Implicit implication**: Jurisdictional constraints will be discovered when a
client in a restricted jurisdiction signs a contract. Compliance will require
system redesign.

---

## Q9. Where are logs, metrics, and traces collected and retained?

**Why it matters**: Observability infrastructure is infrastructure. Its location
determines who can access it, how long data is retained, and what compliance
obligations it creates.

**Good answer example**: "Structured logs: stdout → GCP Cloud Logging (30-day retention).
Metrics: Prometheus scrape → Grafana (local). Traces: Langfuse (self-hosted GCP).
No PII in any observability stream."

**Implicit implication**: Observability will be installed reactively. During an
incident, the required data may not exist or may be in an inaccessible location.

---

## Q10. Where are deployments initiated from, and who can initiate them?

**Why it matters**: Deployment origin determines the attack surface for supply chain
attacks and unauthorized deployments.

**Good answer example**: "GitHub Actions only. No manual kubectl apply to production.
ArgoCD syncs from main branch on merge. Local development: minikube or colima only.
No deployment from developer laptops to production."

**Implicit implication**: Deployments will be initiated from wherever is most
convenient. Unauthorized or accidental production deployments will occur.
