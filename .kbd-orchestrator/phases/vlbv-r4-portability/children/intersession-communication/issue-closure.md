# Issue disposition for same-machine KBD communication

Checked GitHub on 2026-11-24 at 09:51 UTC. Full had zero open issues, the Companion repository had zero, and Mini had three. The local Companion checkout had no configured Git remote. The organization’s repository of that name describes itself as the optional connected-client and peer coordination service. [Full issues](https://github.com/Prometheus-AGS/prometheus-skill-system/issues?q=is%3Aissue+is%3Aopen) · [Companion issues](https://github.com/Prometheus-AGS/prometheus-companion/issues?q=is%3Aissue+is%3Aopen) · [Mini issues](https://github.com/Prometheus-AGS/prometheus-skills-mini/issues?q=is%3Aissue+is%3Aopen)

## Failures tracked in the child phase

The nine scenario IDs are the acceptance record. LOCAL-001 through LOCAL-003 cover live two-way updates, local fallback and same-authority handoff. PEER-001 through PEER-004 cover journal compatibility, receiver-side proof, offline and denied delivery, and bounded catch-up. GREEN-001 and GREEN-002 own the nine Full doctor failures, the discovery warning and the strict OpenSpec inventory across Full and Companion. Record each result against its own scenario before requesting phase exit. The issue search supplies no additional open ticket for these failures. [Phase specification](spec.md) · [Scenario contract](adopted-contract.json)

## Previously discussed tickets

| Ticket | GitHub disposition | Consequence for this phase |
| --- | --- | --- |
| [Full #241](https://github.com/Prometheus-AGS/prometheus-skill-system/issues/241) | Completed on November 23 | Operator-decided phase exit and read-only probe behavior have their own implementation record. This child exercises related authority boundaries using its own fixtures. |
| [Full #247](https://github.com/Prometheus-AGS/prometheus-skill-system/issues/247) | Completed on November 23 | Optional guidance for bounded VLBV impact and evidence reuse is accounted for in that issue. |
| [Full #243](https://github.com/Prometheus-AGS/prometheus-skill-system/issues/243) and [Mini #92](https://github.com/Prometheus-AGS/prometheus-skills-mini/issues/92) | Both closed as not planned on November 24 | The owner deferred the Code Mode recipe. Resumption depends on demonstrated package-user demand and, for Mini, a separate distribution request. |

## Remaining Mini backlog

| Ticket | Work required under its own acceptance criteria |
| --- | --- |
| [Mini #62](https://github.com/Prometheus-AGS/prometheus-skills-mini/issues/62) | Deliver the M1–M6 parity corpus covering scope, freshness, diagnosis, budgets, installed harnesses and capability refusal. The issue also owns Windows qualification, token telemetry, serialized cross-pack fixtures and adapter and elicitation conformance. Local Full/Companion receipts cover none of those Mini entry points. |
| [Mini #63](https://github.com/Prometheus-AGS/prometheus-skills-mini/issues/63) | Account for the Windows Node 22/24 suite hang and the four Mini failures in distribution generation, secret scanning, version agreement and CLI-backed reconciliation. GREEN-001 and GREEN-002 exercise Full doctor and OpenSpec, so Mini still needs its own platform evidence. |
| [Mini #69](https://github.com/Prometheus-AGS/prometheus-skills-mini/issues/69) | Deliver the compact discovery and BDD workflow. Its seven acceptance cases cover calibrated selection, assertion traceability, author-only behavior, a real Node journey with a negative control, evidence gaps, handoff and recovery, and parity across source, generated and selected installed surfaces. LOCAL-003 checks task handoff only in the Full pack. |

At phase exit, record the actual local results and refresh the open-issue inventory. All three Mini tickets retain independent closure criteria. [Full PR #251](https://github.com/Prometheus-AGS/prometheus-skill-system/pull/251) was open at inspection and records earlier R4 task state. Its review and merge follow their own process.
