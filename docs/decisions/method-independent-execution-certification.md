# Decision: certify evidence properties without mandating the production method

**Status:** accepted · 2026-08-05 · release 1.7.0

## Context

Making one execution tool mandatory for final certification would turn an evidence feature into agent-time mutation policing and restrict legitimate Bash, Python, Edit, or Write workflows. Conversely, accepting a green command result without independently checkable properties makes certification tool-dependent and easy to false-green.

## Decision

Certification requirements declare evidence properties, environment classification, and independent verification inputs. Any producer may satisfy them. `prometheus-exec` can produce a portable evidence index, but is never the only accepted production method. Unavailable environments are `pending_evidence`; unavailable judges are separately `pending_review`.

## Alternatives considered

- **Require `prometheus-exec` for every certified command:** uniform, but restricts creative tool use and confuses method with evidence.
- **Trust command exit status:** flexible, but cannot establish artifact identity, environment, or reviewer independence.
- **Collapse all pending states:** simpler dashboards, but hides the difference between missing runtime evidence and unavailable review.

## Consequences

Agents retain unrestricted ordinary tools. Final local certification resolves indexed files, hashes, signatures, receipts, artifacts, and environment records regardless of producer. Reports are more detailed because artifact, disposable runtime, installed host, remote, mobile, device, and judge dimensions cannot collapse into one percentage.

## Verification

Contract fixtures prove equivalent evidence properties are accepted across producer methods and that `pending_evidence`, `pending_review`, blocked, failed, and completed states remain distinct and deterministically serialized.
