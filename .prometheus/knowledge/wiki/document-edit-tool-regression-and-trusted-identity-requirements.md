---
type: Reference
id: document-edit-tool-regression-and-trusted-identity-requirements
title: Document Edit Tool Regression and Trusted Identity Requirements
tags:
- document-editing
- tool-registry
- agent-runtime
- authenticated-identity
- docx-transforms
- release-verification
sources:
- ssr-workspace/document-edit-tool-regression/implementation-pending-2026-08-31
timestamp: 2026-08-31T12:20:41.009034+00:00
created_at: 2026-08-31T12:20:41.009034+00:00
updated_at: 2026-08-31T12:20:41.009034+00:00
revision: 0
content_hash: c8ce85584b2ecadaf263de8e0302423452e113d0e3c2e325f566cb615d9bac46
---

## Status

As of 2026-08-31, the `document-edit-tool-regression` corrective phase is **in progress** and is not deployed or accepted.

- Phase state: `3 changes / 8 tasks`
- Implemented: tasks 1–6
- Incomplete:
  - Task 7: review/CI
  - Task 8: paired deployment/acceptance
- Review round 2 was partial and blocked; it must not be cited as approval.

## Confirmed Regression

The breaking agent revision was `a8b91327` on 2026-08-30 15:04 CDT.

Root cause:

- A post-construction C29 tool-map overwrite replaced the final runtime tool registry.
- Document IDs were hidden from the model while old open-document schemas still required document IDs.
- The initialized production agent lacked required document tools:
  - outline
  - search
  - exact-read
- A production tool call supplied the literal value `context.document.documentId`, which was rejected.
- A constructor-only test mocked both `Agent` and the post-construction modifier, masking the final-runtime regression.

## Candidate Fix Direction

Candidate agent revision: `37fe9a12a586d3718ee5d2570e2e1109bf906ac2`

Selection rationale:

- Isolated from deployed `a8b91327`.
- Avoids coupling to newer mainline approval/database rollout.

Frontend staging:

- Frontend staged on deployed `403adea2`.
- Not pushed yet.

Release policy:

- Ship one combined release only.
- Do **not** ship a `basic-edit-only` release.

Required implementation properties:

- Restore one final tool registry.
- Bind open-document tools to authenticated server context.
- Reject explicit document identity mismatches.
- Retain original validation and concurrency guards:
  - schema validation
  - revision checks
  - hash checks
  - CAS guards

## Document Geometry and Save Semantics

Client geometry is observational only.

- Resolve physical-page targets against authorized saved bytes.
- Never equate page number with section index.
- A saved mutation is not sufficient proof of visual editing success.
- Acceptance must verify both persisted bytes and rendered visible result.

## Download Route and Ownership Corrections

Independent review found:

- A real-download-route fixture substitute.
- Account-generation ownership gaps.

Candidate corrections:

- Call the actual authenticated download `GET` route and storage helpers.
- Use fixtures only for Azure SDK I/O.
- Bind the following to normalized account generations:
  - pending downloads
  - commits
  - layout verification
  - save completion

Required invalidation behavior:

- Sequence `A → B → A` must invalidate stale `A1` and `B` callbacks.
- Dirty editor bytes must not be discarded during that invalidation.

Identity rule:

- Browser cookie sessions may have `userId = 0` and an email.
- That browser principal is **not** server authorization.

## Pending Release Verification

Release verification remains **pending**.

Required CI scenario:

- One top-level CI scenario.
- No retry.
- Must exercise the real producer-to-consumer chain:
  - renderer
  - authentication
  - final `Agent` registry
  - DOCX transforms
- No successful local functional run existed at the time of the source note.

Required production proof:

- Use real Kratos.
- Use a marked disposable copy.
- Perform exactly two edits.
- Verify actual visible spacing.
- Verify persisted replacement.
- Verify original hash remains unchanged.

Required before closure:

- Full independent source review.
- CI pass.
- Paired immutable rollout:
  1. agent first
  2. frontend second
- Authenticated acceptance.

Do **not** force success by changing:

- credentials
- business-document contents
- identity mappings
- PostgreSQL settings
- feature flags

## Permanent Engineering Lessons

- Review unchanged producers, not only changed consumers.
- Review post-construction modifiers that can alter runtime artifacts.
- Test final runtime artifacts, not constructor-only mocks.
- Test the actual authenticated user path.
- Validate saved bytes and rendered result.
- Do not treat pod readiness or HTTP 200 responses alone as editing acceptance.

# Citations

1. ssr-workspace/document-edit-tool-regression/implementation-pending-2026-08-31