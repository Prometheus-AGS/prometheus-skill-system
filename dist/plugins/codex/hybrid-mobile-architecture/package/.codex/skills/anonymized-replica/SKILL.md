---
name: anonymized-replica
description: Build or audit an anonymized, reproducible data replica for development, proof-of-concept, or certification. Use for POC datasets, production-shape fixtures, de-identification, referential integrity, synthetic data, or privacy-preserving test environments.
---

# Anonymized Replica

An anonymized replica preserves behavior and relationships, not identity.

## Contract

- Start from an explicit field-classification manifest.
- Drop fields that are unnecessary for the scenario.
- Replace direct identifiers with deterministic, scoped surrogates.
- Generalize or synthesize quasi-identifiers with re-identification risk.
- Preserve referential integrity and important statistical edge cases.
- Exclude credentials, tokens, raw documents, free-form notes, and transcripts
  unless a reviewed transformation exists.
- Record source schema digest, transformation version, and output digest.
- Make generation deterministic from a protected seed, never a production key.

Run privacy review, uniqueness checks, secret scanning, referential-integrity
tests, and scenario coverage before distribution. Never call a replica
anonymous solely because names and email addresses were removed.
