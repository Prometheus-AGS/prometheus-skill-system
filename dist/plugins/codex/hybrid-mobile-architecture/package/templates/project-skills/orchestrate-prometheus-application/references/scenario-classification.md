# Scenario classification and reference manifest

## Known scenarios

| Scenario | ID | Required recipe |
|---|---|---|
| Full hybrid application (all surfaces) | `scenario-full-hybrid` | `docs/prompting/scenarios/full-hybrid.md` |
| Flutter-only mobile with Rust FFI | `scenario-flutter-rust-ffi` | `docs/prompting/scenarios/flutter-rust-ffi.md` |
| Tauri local inference desktop | `scenario-tauri-local-inference` | `docs/prompting/scenarios/tauri-local-inference.md` and `docs/prompting/data/recipes/tauri-local-inference.json` |
| Full-stack automation product | `scenario-full-stack-automation` | `docs/prompting/scenarios/full-stack-automation.md` |
| Multi-tenant SaaS | `scenario-multi-tenant-saas` | `docs/prompting/scenarios/multi-tenant-saas.md` |
| Local desktop agent client | `scenario-local-agent-client` | `docs/prompting/scenarios/local-agent-client.md` |
| Business ideation studio | `scenario-ideation-studio` | `docs/prompting/scenarios/ideation-studio.md` |
| Native Rust agent | `scenario-native-rust-agent` | `docs/prompting/scenarios/native-rust-agent.md` |
| Source-built multi-cloud deployment | `scenario-multi-cloud-deployment` | `docs/prompting/scenarios/multi-cloud-deployment.md` |
| Branded documentation portal | `scenario-branded-docs-portal` | `docs/prompting/scenarios/branded-docs-portal.md` |

## Composite scenarios

If the request spans multiple known scenarios, classify each part and order them
by dependency:

1. Feynman/KBD discovery.
2. Architecture and domain contracts.
3. Shared Rust/core behavior.
4. Surface UI implementation.
5. Persistence/sync/model routing.
6. Deployment/catalog packaging.
7. Documentation/publication.
8. Verification, critic, and Karpathy retention.

## Manifest output

Every orchestration response must emit this manifest:

```text
Scenario IDs:
- <id>

Architecture references:
- AGENT_BASE_RULES.md
- references/arch-standard.md
- <surface references>

Recipe references:
- docs/prompting/scenarios/<recipe>.md
- docs/prompting/data/recipes/<recipe>.json

Harness references:
- docs/prompting/harnesses.md
- docs/prompting/harnesses/<harness>.md

Loop references:
- docs/prompting/loops/feynman-loop.md
- docs/prompting/loops/kbd-lifecycle.md
- docs/prompting/loops/producer-critic-autonomy.md
- docs/prompting/loops/karpathy-pmpo.md

Role/model references:
- docs/prompting/model-registry.yaml
- docs/prompting/model-routing.generated.md

Retention references:
- project Karpathy memory skill

Verification references:
- hybrid-runtime-verification
- scenario public-boundary prompt
```
