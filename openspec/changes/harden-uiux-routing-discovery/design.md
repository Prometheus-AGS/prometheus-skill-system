## Context

See `proposal.md` for motivation. The injected template currently orders design skills but does not define target existence or optional-skill semantics. UAR demonstrates the failure: the future Presentation directory is absent while the incumbent A2UI page exists elsewhere.

## Goals / Non-Goals

**Goals:**

- Make target resolution an explicit precondition for bounded context loading.
- Turn missing optional skills into deterministic installed-capability fallbacks.
- Preserve managed-fence injection behavior.

**Non-Goals:**

- Vendor or install a third-party UX skill.
- Change UAR application code or its design direction.
- Teach the generic injector project-specific paths.

## Decisions

1. Put the contract in the reusable `template-uiux-routing.md`, not in UAR. Each project resolves its own existing target; the example may explain incumbent versus future path without hard-coding one product.
2. Replace the unconditional `frontend-design + ux-designer` step with a capability check and a named fallback to already required UI/UX Pro Max plus `frontend-design`. This preserves UX review intent without false provenance.
3. Update the roster to classify `ux-designer` as optional/community-provided rather than Anthropic-supplied unless a verifiable source is configured.

## Risks / Trade-offs

- [Agents may implement against the incumbent path instead of the planned destination] → State that the incumbent path is context authority, while the spec/plan still owns the destination.
- [Fallback reviews are not identical to a dedicated UX skill] → Require the fallback to be recorded so evidence remains honest.

## Migration Plan

Update template, roster, skill documentation, and managed-fence tests. Dry-run the injector against a temporary copy and resolve the UAR target read-only. Shared distribution refresh remains owned by the final reconciliation change.
