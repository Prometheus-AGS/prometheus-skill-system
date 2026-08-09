# Who Dimension — People & Roles

**Interrogative**: Who is involved — as owners, operators, users, or subjects?
**Core Concern**: Access, ownership, responsibility, accountability, governance.
**Criticality**: HIGH — undefined ownership = undefined security and governance boundaries.
**Critical Threshold**: 65%

The Who dimension determines who can do what, who is responsible when things go wrong,
and who the system is built for. Undefined roles create security vulnerabilities,
accountability gaps, and systems that serve nobody in particular.

---

## Q1. Who are the end users, and what are their technical capabilities?

**Why it matters**: Interface complexity, documentation depth, and error message
quality all depend on the user's technical level. A system designed for engineers
will fail in the hands of domain experts who are not engineers.

**Good answer example**: "Primary: enterprise IT architects (technical). Secondary:
domain experts using KnowMe tools (non-technical). No end-user access to infrastructure."

**Implicit implication**: The system will be designed for an assumed user. That
user may not match the actual user base, producing friction or failure in production.

---

## Q2. Who owns the system — who is ultimately responsible for its behavior?

**Why it matters**: Ownership determines who makes final architectural decisions,
who is accountable to stakeholders, and who holds the keys.

**Good answer example**: "Travis James (CTO, Prometheus AGS) owns the system.
Randy Jesberg (CEO) is accountable to enterprise clients. No shared ownership."

**Implicit implication**: Ownership will be unclear. Decisions will be deferred
or duplicated. Accountability in incidents will be contested.

---

## Q3. Who operates the system day-to-day, and what are their responsibilities?

**Why it matters**: Operators need different capabilities than users or owners.
If operators are undefined, operational procedures cannot be designed.

**Good answer example**: "Travis James operates inference infrastructure.
No dedicated DevOps — operations is self-service via kubectl and ArgoCD."

**Implicit implication**: Operations responsibilities will be discovered through
incidents rather than defined in advance. On-call will be ambiguous.

---

## Q4. Who has administrative access, and how is that access governed?

**Why it matters**: Administrative access is the highest-risk access. Undefined
governance is a security and compliance failure waiting to happen.

**Good answer example**: "Travis James: cluster admin. Randy Jesberg: billing/enterprise
dashboards. No shared credentials. MFA required. Cedar policy enforces RBAC."

**Implicit implication**: Administrative access will be broader than necessary.
Blast radius of a compromise will be the full system.

---

## Q5. Who are the external stakeholders — clients, partners, regulators?

**Why it matters**: External stakeholders have expectations and contractual rights
that constrain the system even when they have no direct access to it.

**Good answer example**: "Citizens National Bank: pilot client with contractual
data sovereignty requirements. Anthropic: API dependency, usage policy applies."

**Implicit implication**: External stakeholder constraints will be discovered
after the system is built. Redesign will be required.

---

## Q6. Who is affected by the system's failures or degradation?

**Why it matters**: Impact scope determines SLA requirements, incident priority,
and the cost of downtime. An undefined impact scope produces an undefined SLA.

**Good answer example**: "L4 failure affects all active inference sessions.
KnowMe tool downtime affects direct consumers. Banking mesh failure affects
CNB pilot clients — high business impact."

**Implicit implication**: SLA design will not reflect actual impact scope.
The system may have a 99.0% uptime SLA when a 99.9% SLA is required.

---

## Q7. Who can grant or revoke access, and through what mechanism?

**Why it matters**: Access governance must be defined before access is granted.
Retroactive access control is expensive and politically difficult.

**Good answer example**: "Travis James via Cedar policy edits. Ory Kratos
for consumer identity. No self-service access escalation. Access revocation
must propagate within 60 seconds."

**Implicit implication**: Access will be granted informally. Revocation will
be incomplete. Terminated access may persist in active sessions.

---

## Q8. Who is responsible for data — collection, retention, and deletion?

**Why it matters**: Data responsibility is a legal and compliance question.
Undefined data ownership is a GDPR and HIPAA failure surface.

**Good answer example**: "Travis James: data controller for Prometheus Fabric.
Users: own their KnowMe conversation data. Retention: 90 days default,
configurable per client. Deletion: on request within 30 days."

**Implicit implication**: Data ownership will be undefined. Deletion requests
will have no guaranteed path to completion. Compliance will be improvised.

---

## Q9. Who are the adversaries — who might try to misuse or attack the system?

**Why it matters**: Threat modeling requires named adversary classes. Undefined
adversaries produce undefined defenses.

**Good answer example**: "External: credential stuffing against Ory Kratos.
Insider: agent prompt injection from malicious OpenSpec tasks. Tenant isolation
breach: one client's agent accessing another's data."

**Implicit implication**: Security design will address generic threats rather
than specific, likely threats. High-probability attacks may go undefended.

---

## Q10. Who validates and signs off on system changes before they reach production?

**Why it matters**: Undefined approval chains produce unauthorized deployments
or indefinitely blocked deployments.

**Good answer example**: "Travis James signs off on all infrastructure changes.
No changes to production Kubernetes without a GitHub Actions gate with passing
tests. No production deployments on Fridays."

**Implicit implication**: Changes will reach production through informal approval.
Unauthorized deployments may not be detected. Post-incident review will be unable
to trace the approval path.
