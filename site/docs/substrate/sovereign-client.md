---
id: sovereign-client
title: sovereign-client
---

# Sovereign client relocation

The `sovereign-client` SDK moved to optional Companion. This pack no longer contains `substrate/sovereign-client`; do not use that old path dependency or infer its API from historical examples.

The [integration contract](/docs/kbd/integration-contract) preserves the one-way extension boundary. See [service operations](/docs/guide/service-operations#optional-companion) for ownership and the [retained SDK route](/docs/sovereign-sync/rust-sdk) for relocation context. Companion publication remains a separate release step.
