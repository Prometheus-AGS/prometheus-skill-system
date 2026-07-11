---
type: Reference
id: google-okf-llm-wiki-standards
title: "Google OKF LLM Wiki Standards"
description: "Research on Google's Open Knowledge Format (OKF) v0.1, released 2026-06-12, and the emerging landscape of AI-native document standards."
tags:
- okf
- llm-wiki
- standards
- google
- knowledge-format
sources:
- manual-backfill
timestamp: 2026-07-10T19:49:34.767799+00:00
created_at: 2026-07-10T19:49:34.767799+00:00
updated_at: 2026-07-10T19:49:34.767799+00:00
revision: 0
---
# Google OKF LLM Wiki Standards

## Summary

Research on Google's Open Knowledge Format (OKF) v0.1, released 2026-06-12, and the emerging landscape of AI-native document standards.

## Key Findings

- **OKF v0.1**: Apache 2.0 licensed. Only required field: `type`. Extremely minimal base specification.
- **Karpathy LLM Wiki Pattern**: Formalized into OKF. Content-addressed, append-only, index + log structure.
- **A2UI Protocol**: Google's agent-to-UI specification for generative interfaces. Complements OKF for display.
- **Adoption for Prometheus**: `.research` package should use OKF as base format with extensions:
  - `research_id`: UUID for research session
  - `confidence`: Aggregate confidence score (0.0-1.0)
  - `verification_status`: enum (unverified, partial, verified, contradicted)
  - `research_stage`: enum (planning, searching, collecting, verifying, reporting, exporting)
  - `thread_provenance`: Array of thread IDs that contributed
- **Interoperability**: OKF ensures research outputs can be consumed by any OKF-compliant tool.

## Full Report

[Read the complete research report](/docs/deep-research/research/google-okf-llm-wiki-standards-report.md)

**Date:** 2026-07-03  
**Researcher:** Prometheus Research Agent  
**Lines:** ~460
