---
id: learn-certify
title: /learn-certify
sidebar_label: learn-certify
---

# /learn-certify

Certification skill using Open Badges 3.0 (OB 3.0) and W3C Verifiable Credentials
(W3C VC) standards.

## What it does

When all three mastery conditions are met for a concept or skill cluster,
`learn-certify` generates a tamper-evident credential:

- **Open Badge 3.0** — JSON-LD credential with assertion, criteria, and evidence
- **W3C VC** — cryptographically signed credential using DID

## The credential includes

- Concept or skill achieved
- Date of mastery closure
- Mastery scores (grade, transfer, retention)
- Issuer DID (Prometheus operator)
- Learner DID (holder)

## Privacy

Credentials are generated and stored locally by default. Export is opt-in.
No credential content is forwarded to external services unless the learner
explicitly exports to a credential wallet.

## Usage

```
/learn-certify "Rust async and await"    # certify a completed learning arc
/learn-certify --export wallet           # export to W3C VC wallet
/learn-certify --list                    # show earned credentials
```
