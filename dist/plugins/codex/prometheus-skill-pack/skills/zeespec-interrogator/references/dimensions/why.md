# Why Dimension — Motivation & Purpose

**Interrogative**: Why does this system or change exist?
**Core Concern**: Goals, rules, value, success criteria, constraints.
**Criticality**: HIGHEST — undefined motivation = undefined success criteria.
**Critical Threshold**: 70%

The Why dimension is interrogated first. If Why has coverage below 70%, the
interrogation is almost certain to produce a NO-GO regardless of other scores.
All other dimensions answer to Why — they explain how the motivation is realized.

---

## Q1. What is the primary problem this system or change solves?

**Why it matters**: Without a clear problem statement, every design decision
is unmoored. Features that don't address the problem cannot be identified as waste.

**Good answer example**: "The system solves the latency gap between a client
AI agent submitting a job and receiving a response, caused by synchronous HTTP
coupling. The problem is p99 latency > 2s under concurrent load."

**Implicit implication**: The system or change will be built toward an assumed
problem. That assumption may not match what stakeholders need.

---

## Q2. Who or what benefits, and how is that benefit measured?

**Why it matters**: Benefit without measurement cannot be validated. If there
is no measurement, there is no definition of done.

**Good answer example**: "Enterprise clients benefit through reduced API response
time. Measurement: p99 latency < 200ms for 20 concurrent sessions on an L4 GPU."

**Implicit implication**: Success will be declared by judgment rather than
measurement. The system may ship without ever knowing if it worked.

---

## Q3. What are the explicit goals and their priority order?

**Why it matters**: Multiple goals often conflict. Without priority, every
conflict is unresolvable and every tradeoff is arbitrary.

**Good answer example**: "1 (highest): sovereign inference on local hardware.
2: sub-200ms latency. 3: support for 20+ concurrent sessions."

**Implicit implication**: Goal conflicts will be resolved arbitrarily during
implementation. The result may satisfy no goal well.

---

## Q4. What are the regulatory, compliance, or legal constraints?

**Why it matters**: Compliance constraints are non-negotiable and often
invisible until enforcement. They constrain every other dimension.

**Good answer example**: "HIPAA if storing protected health information.
GDPR for EU users. SOC 2 Type II required for enterprise contracts."

**Implicit implication**: The system will be built assuming no compliance
requirements. If any apply, the system will require expensive remediation.

---

## Q5. What are the ethical or policy constraints?

**Why it matters**: Ethical constraints define what the system must not do.
They are harder to retrofit than technical constraints.

**Good answer example**: "No profiling of user behavior for ad targeting.
No model output caching that could surface other users' data."

**Implicit implication**: Ethical constraints will emerge from incidents
rather than design. Post-incident remediation costs more than pre-design clarity.

---

## Q6. What does success look like at 30 days, 90 days, and 1 year?

**Why it matters**: Success definitions change over time. Near-term and
long-term success often require different design decisions.

**Good answer example**: "30 days: CI/CD pipeline with one model serving
requests. 90 days: 10 enterprise clients in trial. 1 year: $2M ARR."

**Implicit implication**: Success will be defined retroactively, making it
impossible to design toward it or recognize when it has been achieved.

---

## Q7. What are the reasons this could fail, and which are acceptable?

**Why it matters**: Known failure modes that are acceptable define the
risk envelope. Unknown failure modes that are unacceptable cause crises.

**Good answer example**: "Acceptable: latency spike during model loading.
Unacceptable: data leakage between tenant sessions. Unknown: GPU OOM under load."

**Implicit implication**: The system will be designed without a failure
mode envelope. Unacceptable failures may not be architecturally prevented.

---

## Q8. What are the non-negotiable constraints that cannot be compromised?

**Why it matters**: Non-negotiables define the design space boundary.
Violating them invalidates the system regardless of other qualities.

**Good answer example**: "Inference must run on local hardware — no cloud
API calls for model execution. This is a sovereign computing requirement."

**Implicit implication**: Constraints will be treated as preferences.
The system may be built in ways that violate them without recognition.

---

## Q9. What value does this deliver relative to alternatives?

**Why it matters**: If the alternative is better, the system should not be built.
Comparative value justifies the investment.

**Good answer example**: "vs. managed cloud inference: sovereign control,
no per-token cost at scale, no data egress. vs. existing vLLM: TurboQuant
compression gives 9.8x KV cache reduction on consumer GPUs."

**Implicit implication**: The system will be built without validating that
it is the best solution, potentially displacing a superior alternative.

---

## Q10. What are the strategic dependencies — what must be true for this to matter?

**Why it matters**: A system that succeeds technically but fails strategically
is waste. Strategic dependencies are existential conditions.

**Good answer example**: "Must be true: enterprise clients value data sovereignty
enough to pay a premium. Must be true: Rust inference tooling matures to the
point where Candle-vLLM can serve Qwen-class models without gaps."

**Implicit implication**: The system will be built without validating its
strategic premise. It may succeed technically while failing in market relevance.
