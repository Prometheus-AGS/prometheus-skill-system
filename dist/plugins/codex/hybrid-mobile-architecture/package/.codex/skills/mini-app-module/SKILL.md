---
name: mini-app-module
description: Add or review a self-contained mini-application module inside a governed web shell. Use for route modules, feature manifests, capability declarations, navigation registration, module policy, lazy loading, or independently testable applets.
---

# Mini-App Module

A mini-app is a versioned feature boundary, not an arbitrary component folder.

## Required manifest

Declare a stable module ID, version, routes, entity types, required
capabilities, policy actions, event contracts, and optional migrations. The
shell validates this manifest before mounting the module.

## Boundaries

- Presentation depends on domain interfaces.
- Data adapters implement domain interfaces and register with the shell.
- Cross-module communication uses typed events or shared entity references.
- A module cannot import another module's internal store or component tree.
- Server mutations require verified identity and policy enforcement.
- Agent triggers call the governed UAR gateway.
- Capability requests are deny-by-default.

## Done

Test manifest rejection, lazy-load failure, route isolation, entity cleanup,
permission denial, and upgrade compatibility. A module is complete only when it
can be disabled without breaking the host shell.
