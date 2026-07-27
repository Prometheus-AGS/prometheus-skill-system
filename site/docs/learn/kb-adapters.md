---
id: kb-adapters
title: KB Adapters
sidebar_label: KB Adapters
---

# KB Adapters

The Learn domain supports custom knowledge bases via the `/learn-kb` skill. KB adapters
allow you to ground learning in your own proprietary content.

## Adding a KB

```bash
/learn-kb add dify:my-legal-kb
/learn-kb add local:/path/to/clinical-protocols
/learn-kb add palace:my-collection
/learn-kb add web:https://my-docs.example.com
```

## Adapter types and privacy guarantee

The four adapter prefixes (`dify:` / `palace:` / `local:` / `web:`), their
backends, and the `content-grounding-kb.sh` privacy enforcement are documented
once, canonically, in the
[KB Adapter Guide](/docs/learn-internals/kb-adapter-guide) — this page stays
the narrative entry point and does not duplicate that reference.

## Using a KB in a learning session

```bash
/learn-goal "I want to understand clinical trial protocols" --kb local:/protocols
```

The KB adapter is loaded alongside the standard learning arc. Concept gaps are
grounded in the KB content rather than the model's training data.

## Managing KBs

```bash
/learn-kb list                    # show registered KBs
/learn-kb remove dify:my-kb       # deregister a KB
/learn-kb status                  # check adapter health
```
