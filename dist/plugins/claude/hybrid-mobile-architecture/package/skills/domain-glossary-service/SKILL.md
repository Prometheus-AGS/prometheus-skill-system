---
name: domain-glossary-service
description: Implement or audit a versioned domain glossary used by applications and agents to normalize terms without leaking client-specific data into reusable packages. Use for canonical terms, synonyms, entity mappings, policy vocabulary, or terminology-driven retrieval.
---

# Domain Glossary Service

The glossary is a governed vocabulary contract, not a prompt fragment.

## Model

Each term has a stable ID, canonical label, definition, synonyms, domain,
locale, status, version, provenance, and optional entity or policy mappings.
Changes are append-only revisions with explicit deprecation and replacement.

## Boundaries

- Keep reusable schema and behavior generic.
- Store customer names, URLs, roles, and policy vocabulary in project overlays.
- Validate aliases for ambiguity and cycles.
- Resolve terms deterministically and expose the glossary revision to UAR.
- Authorize project glossary access by verified tenant.
- Cache only revisioned, non-sensitive projections.

Test ambiguous terms, deprecated aliases, locale fallback, revision pinning,
cross-tenant denial, cache invalidation, and agent runs resumed against a newer
glossary.
