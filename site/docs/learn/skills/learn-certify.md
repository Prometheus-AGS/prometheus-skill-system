---
id: learn-certify
title: /learn-certify
sidebar_label: learn-certify
---

# /learn-certify

Prepare a local, self-issued Open Badges / Verifiable Credential JSON-LD record from learning evidence. Choose exactly one mode:

```text
/learn-certify rust-basics --checkpoint borrow-checker
/learn-certify rust-basics --final
/learn-certify rust-basics --final --issuer https://issuer.example/credentials
```

Checkpoint mode checks the concept's explanation, transfer and retention artifacts, records certification through `learner-model`, and writes a checkpoint credential. Final mode additionally requires all concepts certified, two distinct practice sessions per concept, retention breadth and the capstone when the curriculum requires one. Missing state, artifacts or a failed gate prevents issuance under the skill procedure. An anomalous mastery trajectory adds an integrity note rather than blocking issuance.

Records live beneath `${PROMETHEUS_LEARN_HOME:-~/.prometheus/learn}/goals/<goal-id>/`: checkpoint files under `checkpoints/`, and the final record at `credential.json`.

This is a skill procedure, not a standalone cryptographic issuer or wallet client. Learner and issuer are the same identity; the skill does not generate a DID document. A populated JSON-LD file alone does not prove a valid signature, independent verification or educational accreditation. If `did.txt` is absent, the procedure uses a noted fallback identity. Inspect evidence and identity before sharing.

`--issuer` records a forwarding request and tells the user to POST the file. It does not send HTTP or export to a wallet. External publication requires authorization and a separate, verified issuer workflow.
