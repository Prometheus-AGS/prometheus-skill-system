---
id: learn-kb
title: /learn-kb
sidebar_label: learn-kb
---

# /learn-kb

KB registry and adapter management. Register custom knowledge bases that the
Learn domain can use to ground sessions in proprietary content.

See [KB Adapters](/docs/learn/kb-adapters) for the full adapter reference.

## Quick usage

```
/learn-kb add dify:my-legal-kb
/learn-kb add local:/path/to/protocols
/learn-kb list
/learn-kb remove dify:my-legal-kb
```

## Supported adapters

| Prefix | Backend |
|--------|---------|
| `dify:` | Dify knowledge base |
| `palace:` | surreal-memory palace RAG |
| `local:` | Filesystem markdown |
| `web:` | Firecrawl live fetch |

## Using with learn-goal

```
/learn-goal "clinical trial design" --kb local:/path/to/protocols
```
