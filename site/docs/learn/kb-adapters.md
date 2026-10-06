---
id: kb-adapters
title: KB Adapters
sidebar_label: KB Adapters
---

# KB Adapters

Register a named corpus with [learn-kb](/docs/learn/skills/learn-kb), then select that name in a learning goal:

```text
/learn-kb add --type local --name clinical-protocols --content-dir /path/to/protocols
/learn-goal "Understand our clinical protocols" --kb clinical-protocols
/learn-kb query --name clinical-protocols --subject "inclusion criteria" --top-k 5
```

The registry supports local, Dify, Palace and URL-registration procedures. URL registration is an explicit Firecrawl scrape followed by configured Palace ingestion; the grounding helper has only `dify:`, `palace:` and `local:` adapters.

Read [KB Adapter Guide](/docs/learn-internals/kb-adapter-guide) for helper arguments, configuration and failure behavior. There is no automatic waterfall between backends. Local file reads stay local, but configured Dify/Palace endpoints and any downstream model receiving extracted text may be remote. A helper's `privacy_mode` result does not enforce destination authorization or prevent later publication.

Use only authorized corpora and endpoints. Missing credentials, unreadable files or absent results must remain explicit failures; generated answers do not establish corpus coverage.
