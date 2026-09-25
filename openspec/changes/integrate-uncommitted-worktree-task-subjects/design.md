# Design

## Context

See proposal.md. The detached source worktree is based on 1dc5a670; main is 10a41658. Main has newer slash/single-colon support that must survive the integration.

## Goals / Non-Goals

Preserve all three qualified forms and existing lookup behavior. Do not combine unrelated historical branch commits or main-checkout work.

## Decisions

Preserve the source patch on a named recovery branch, then port only the missing delimiter handling into main. Match slash first to preserve current precedence, then double colon before single colon. Keep canonical task lookup and bare-subject ambiguity handling unchanged. Exercise the production CLI against a real temporary signed runtime through the existing kbd integration target.

## Risks / Trade-offs

An old patch could replace newer parser behavior; a surgical integration retains the current implementation. Main contains unrelated dirty files; save a diff/hash baseline and commit only explicitly named paths.

## Migration Plan

No data migration. Revert only the integration commit to undo the addition; the original patch remains recoverable from its preservation branch.
