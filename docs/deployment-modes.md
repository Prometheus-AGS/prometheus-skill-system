# Deployment choices

Choose independent capabilities from the shipped source; these are not progressive superset tiers. A skill definition, configured endpoint, running service and accepted user workflow are different delivery states.

| Need | Source-owned path | Additional requirement |
|---|---|---|
| Skills and local workflows | `./install.sh --profile skills` | Git, compatible Node and a supported harness |
| Local KBD | `prometheus kbd` | Explicit project identity, signed local runtime |
| Searchable memory/knowledge | Full native service templates or an existing authorized endpoint | Actual namespace/database/model and endpoint configuration |
| Multi-model inference | Configured liter-llm or another supported provider | Credentials and demonstrated model route |
| Learning UI | Learner model plus surface bridge | Supported host surface and installed service |
| Bounded execution | Prometheus Exec | Supported backend, authorized bytes and separate artifact/service install |
| Cross-device replication | Optional Companion | Its own source, identity, installation and acceptance evidence |

The full installer supports `skills` and `full` profiles. Full service templates vary by platform; a Linux-capable binary does not imply a Linux service installer. Native service installation needs Bash 4 or newer. Full skills support Windows through Git Bash/WSL; mini provides a Node-only Windows path with a reduced optional Compose stack.

Use the [service operations guide](guide/26-service-operations.md) for exact ownership, endpoints and data boundaries and [installation](guide/19-installation.md) for actual entrypoints. The [integration contract](integration-contract.md) keeps Companion optional and separately owned. The pack does not automatically install a sync daemon, and its detector reports observations rather than proving an entire deployment mode usable.
