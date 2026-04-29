# prometheus-skill-pack — OpenCode Plugin

Registers three tools and lifecycle hooks into OpenCode via the `@opencode-ai/plugin` API.

## Tools

| Tool | Description |
|------|-------------|
| `evolve` | Run an iterative evolution cycle (assess → analyze → plan → execute → reflect) |
| `gitops` | Bootstrap or transform GitOps pipelines (ArgoCD, Kustomize, CI/CD) |
| `kbd` | Drive the KBD process orchestrator (init, plan, execute, status, reflect) |

## Registration

Add this to your project's `opencode.json` (generated automatically by the installer):

```json
{ "plugin": ["./.opencode"] }
```

Or register manually:

```bash
npx tsx scripts/install-platforms.ts --platform opencode --scope project
```

## Testing Tools

Once registered, invoke tools from the OpenCode chat:

```
/evolve "my-feature" domain=software phase=assess
/gitops bootstrap
/kbd status
```

## Lifecycle Hooks

- **`shell.env`**: injects `PROMETHEUS_SKILL_PACK=1` into every shell command environment
- **`tool.execute.before`** / **`tool.execute.after`**: reserved stubs for write-guards and telemetry

## Development

```bash
cd .opencode
npm install
npx tsc --noEmit   # verify types
```
