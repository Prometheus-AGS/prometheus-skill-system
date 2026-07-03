# Threaded/Concurrent Research with Per-Thread Context

## Summary

Research on implementing threaded and concurrent research with isolated per-thread context, leveraging surreal-memory for thread-safe storage.

## Key Findings

- **Thread Types**: Source threads, sub-question threads, verification threads, synthesis threads, drift threads.
- **Context Isolation**: Each thread gets its own surreal-memory namespace. No accidental cross-contamination.
- **Merge Stage**: Deterministic code (not LLMs) for deduplication, entity resolution, conflict detection. Critical for reproducibility.
- **Concurrency Control**: AIMD (Additive Increase/Multiplicative Decrease) backpressure + semantic caching.
- **surreal-memory Role**: Unified knowledge layer serving all threads simultaneously. Graph + vector + document + relational + time-travel in one store.
- **Scalability**: Thread pool size adjusts based on query complexity and system load.
- **Race Conditions**: Handled via surreal-Memory's ACID transactions and optimistic locking.

## Full Report

[Read the complete research report](/docs/deep-research/research/threaded_concurrent_research_per_thread_context.md)

**Date:** 2026-07-03  
**Researcher:** Prometheus Research Agent  
**Lines:** ~640
