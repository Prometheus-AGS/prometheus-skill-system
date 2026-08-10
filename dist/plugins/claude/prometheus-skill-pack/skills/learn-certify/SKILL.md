---
name: learn-certify
description: Certification endpoint for the Feynman learning loop. Runs prerequisite gates (feynman-artifacts, practice results, capstone), emits an Open Badges 3.0 / W3C Verifiable Credential self-issued JSON-LD signed with did-plc, and detects anomalous mastery trajectories via an integrity guardrail.
version: '1.0.0'
license: MIT
metadata:
  author: prometheus-skill-pack
  category: learn
  tags: [learn, certify, credential, badge, ob3, w3c-vc, mastery, did-plc]
---

# learn-certify

Certification endpoint for the Feynman learning loop. Issues an OB 3.0 / W3C
Verifiable Credential after verifying mastery gates. Works in two modes:
checkpoint (single concept) and final (full goal certification).

## When to invoke

```
/learn-certify <goal-id> [--checkpoint <concept-id>] [--final] [--issuer <endpoint-url>]
```

- `--checkpoint <concept-id>`: intermediate milestone check on a single concept
- `--final`: full certification for the entire goal (requires all mastery gates met)
- `--issuer <endpoint-url>`: forward the signed credential to a 1EdTech-compatible
  endpoint (OPTIONAL — not required for self-issued)

Exactly one of `--checkpoint` or `--final` must be provided.

## Two modes

### Checkpoint mode (`--checkpoint <concept-id>`)

Gate check for a single concept.

1. Load concept state from `~/.prometheus/learn/goals/<goal-id>/learner-model.json`
2. Verify all three mastery criteria met:
   - `learn-grade` passed: `overall_score >= 0.7` and no active misconceptions
   - Transfer problems solved: transfer scores in artifact `>= 0.7`
   - Retention passed: `retention_passed: true` in artifact
3. If any criterion fails: report which checks failed and stop. Do NOT issue.
4. Run the integrity guardrail (see below) against this concept's observations.
5. Emit a checkpoint badge (OB 3.0 assertion scoped to the concept).
6. Update learner-model: set `certified_at: <ISO datetime>` on the concept entry.
7. Write the checkpoint credential to
   `~/.prometheus/learn/goals/<goal-id>/checkpoints/<concept-id>-credential.json`.

Print:
```
Checkpoint certified: <concept-id>
Grade: <overall_score>  Transfer: <scores>  Retention: passed
Path: ~/.prometheus/learn/goals/<goal-id>/checkpoints/<concept-id>-credential.json
```

### Final certification mode (`--final`)

Full goal certification. Run prerequisite gates in order — stop and report if
any gate fails. Do NOT issue until all gates pass.

**Gate 1 — All concepts certified**
Every concept in `curriculum.json` must have `certified_at` set in the
learner-model. If any concept is missing, list the uncertified concepts and stop.

**Gate 2 — Practice breadth**
The learner-model observations must show at least 2 distinct `learn-practice`
sessions per concept. Count by unique `session_id` values tagged `type: practice`.

**Gate 3 — Retention breadth**
All concepts must have `retention_passed: true` in their artifact entries.

**Gate 4 — Capstone (conditional)**
If the goal's `curriculum.json` has `capstone_required: true`, a free-form
synthesis artifact must exist at
`~/.prometheus/learn/goals/<goal-id>/capstone.md`.
The capstone must reference at least three concept IDs from the curriculum.
Check for the file and the concept references. If absent or insufficient, stop.

When all gates pass:
1. Run the integrity guardrail across all concept observations.
2. Build the credential (see format below).
3. Write to `~/.prometheus/learn/goals/<goal-id>/credential.json`.
4. If `--issuer <url>` provided, note in the credential that forwarding is
   requested and print the endpoint URL. (Actual HTTP call is out of scope for
   self-issued mode — instruct the user to POST the file to the endpoint.)

Print:
```
Credential issued: <goal-id>
Subject: <subject> (<target_level>)
Concepts certified: N
Evidence entries: N
Path: ~/.prometheus/learn/goals/<goal-id>/credential.json
```

## OB 3.0 / W3C VC credential format

Emit valid JSON-LD. Populate every field from the learner-model and artifacts.

```json
{
  "@context": [
    "https://www.w3.org/2018/credentials/v1",
    "https://purl.imsglobal.org/spec/ob/v3p0/context-3.0.2.json"
  ],
  "id": "urn:uuid:<credential-id>",
  "type": ["VerifiableCredential", "OpenBadgeCredential"],
  "issuer": {
    "id": "did:plc:<learner-did>",
    "type": "Profile",
    "name": "Self-issued via Prometheus Skill Pack"
  },
  "issuanceDate": "<ISO datetime>",
  "credentialSubject": {
    "id": "did:plc:<learner-did>",
    "type": "AchievementSubject",
    "achievement": {
      "id": "urn:uuid:<achievement-id>",
      "type": "Achievement",
      "name": "<subject> Mastery — <target_level>",
      "description": "Demonstrated via Feynman explanation + transfer problems + retention checks",
      "criteria": {
        "narrative": "Passed learn-grade (>=0.7), solved transfer problems (>=0.7), retention check (>=0.6) for all concepts in the curriculum."
      }
    },
    "evidence": [
      {
        "id": "urn:uuid:<artifact-id>",
        "type": "Evidence",
        "name": "Feynman artifact: <concept-id>",
        "description": "Grade: <score>. Transfer: <scores>. Retention: passed."
      }
    ]
  }
}
```

**Field sources:**
- `<credential-id>`: generate a new UUID v4
- `<achievement-id>`: generate a new UUID v4
- `<learner-did>`: read from `~/.prometheus/learn/did.txt`; if absent, use
  `did:plc:self-issued-<goal-id>`
- `<subject>` and `<target_level>`: read from goal's `curriculum.json`
- `evidence` array: one entry per concept; populate from artifact files in
  `~/.prometheus/learn/goals/<goal-id>/artifacts/<concept-id>/`

## Integrity guardrail

Run before issuing any credential (checkpoint or final).

1. Load all observations from the learner-model for the relevant concepts.
2. For each concept, sort observations by `timestamp` ascending.
3. Compute mastery delta per consecutive observation pair:
   `Δmastery = obs[n].mastery_score - obs[n-1].mastery_score`
4. Flag as suspicious if any single step shows `Δmastery > 0.4`.
5. If flagged:
   - Add `"integrityNote"` field to the credential at the top level:
     ```json
     "integrityNote": "Anomalous mastery trajectory detected on concept <id>: delta=<value> between <ts1> and <ts2>. Credential issued with this note."
     ```
   - Do NOT block issuance. Report the anomaly and issue with the note.
   - Print a warning line before the normal completion output:
     ```
     WARNING: Anomalous mastery delta detected — see integrityNote in credential.
     ```

## Evidence binding

The credential's `evidence` array must be populated — one entry per concept.
Each entry must include:
- `artifact_id` (UUID from the artifact file)
- `concept_id`
- `overall_score` (from learn-grade result)
- `transfer_scores` (array from transfer problem results)
- `retention_passed` (boolean)

A credential with an empty `evidence` array is invalid and must not be issued.

## Self-issued note

The credential is self-issued: learner = issuer = subject via did-plc. This means:

- It cannot be independently verified without the learner's DID document.
- The trust model is: evidence is verifiable, the issuance process is documented.
- For external verification, use `--issuer <endpoint>` to note the forwarding
  target and instruct the user to POST the credential JSON to that URL.
- The DID document is not generated by this skill. If `~/.prometheus/learn/did.txt`
  is absent, the fallback DID is used and noted in the credential's `issuer.name`.

## Learner-model schema (relevant fields)

```json
{
  "goal_id": "<goal-id>",
  "concepts": {
    "<concept-id>": {
      "mastery_score": 0.85,
      "certified_at": null,
      "retention_passed": true,
      "observations": [
        {
          "timestamp": "<ISO>",
          "session_id": "<uuid>",
          "type": "practice",
          "mastery_score": 0.72
        }
      ]
    }
  }
}
```

## Error handling

| Condition | Action |
|---|---|
| `learner-model.json` absent | Abort: print path and instruct user to run `/learn-goal` |
| `curriculum.json` absent | Abort: print path and instruct user to run `/learn-plan` |
| Gate fails | Print which gate failed, what is missing, and which skill to run next |
| `evidence` array would be empty | Abort: print concept IDs missing artifacts |
| `did.txt` absent | Use fallback DID, note in credential, continue |

## File layout

```
~/.prometheus/learn/goals/<goal-id>/
├── learner-model.json          # source of truth for mastery state
├── curriculum.json             # concept list, capstone_required flag
├── credential.json             # final credential (written by --final)
├── capstone.md                 # free-form synthesis (if required)
├── did.txt                     # optional: learner DID
├── artifacts/
│   └── <concept-id>/
│       └── feynman-artifact.json
└── checkpoints/
    └── <concept-id>-credential.json
```
