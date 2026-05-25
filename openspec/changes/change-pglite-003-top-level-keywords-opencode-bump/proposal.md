# Change: pglite-003 — Top-Level Plugin Keywords + OpenCode Dependency Bump

**Phase**: pglite-certification-2026-05-25  
**Gaps closed**: G4 (pglite keyword discoverability), G5 (opencode plugin dep lag)  
**Priority**: Low  
**Effort**: 15 minutes

## Problem

1. **G4**: `.claude-plugin/plugin.json` `keywords` array does not include `"pglite"` or `"electricsql"`, making the pack undiscoverable in marketplace keyword searches for these terms despite the pack containing a full PGLite skill.

2. **G5**: `.opencode/package.json` pins `@opencode-ai/plugin` to `1.14.29` (and `@opencode-ai/sdk` to `^1.14.29`). v1.15.6 (released May 20, 2026) includes a plugin load error fix that prevents one bad plugin from breaking the rest. Bumping ensures the pack benefits from this stability improvement.

## Proposed Changes

### File 1: `.claude-plugin/plugin.json`

In the `keywords` array, add `"pglite"` and `"electricsql"` (after `"cargo"`):
```json
"keywords": [
  "react",
  "entity-management",
  "process-orchestration",
  "iterative-evolution",
  "bdd-testing",
  "pmpo",
  "gitops",
  "devops",
  "argocd",
  "kustomize",
  "surreal-memory",
  "rust",
  "auditor",
  "clippy",
  "cargo",
  "pglite",
  "electricsql"
]
```

### File 2: `.opencode/package.json`

Update versions:
```json
"dependencies": {
  "@opencode-ai/plugin": "^1.15.0",
  "@opencode-ai/sdk": "^1.15.0",
  "zod": "^3.23.0"
}
```

## Acceptance Criteria

- [ ] `"pglite"` present in top-level `plugin.json` keywords
- [ ] `"electricsql"` present in top-level `plugin.json` keywords
- [ ] `@opencode-ai/plugin` version is `^1.15.0` or higher in `.opencode/package.json`
- [ ] `@opencode-ai/sdk` version updated to match
- [ ] JSON valid in both files
- [ ] `npm run validate:strict` → 0 errors
