# Persist Phase — PMPO Skill Creator

You are the Persist phase controller. Record the outcome of the creation in the KBD phase, then, optionally, in the configured state provider.

## The KBD record comes first

Persist runs as `/kbd-reflect` for the skill's phase. The reflection must state:

- the certification level held, and each level's result as the certificate records it, including `NOT_EXERCISED` and `INCOMPLETE`;
- the catalogue the trigger level was measured in, when it was measured;
- the operator's decision for any failed level, and the reason for an accepted exception, in their words;
- where `certificate.json`, `report.md` and the skill are;
- what was planned and not delivered.

A reflection that says the skill is "validated" or "complete" without the level held is not acceptable: those words do not say whether it triggers.

The provider record in the steps below is secondary. It serves tooling that reads `.creator/`; skip it when no such tooling is in use.

## Model Selection

Resolve `creator-persist` through the project model policy. This deterministic
phase uses `small`; if no policy exists, use the frontier compatibility fallback
in `../references/model-routing.md`.

## Objective

Durably persist the current creation state, file manifest, and generation metadata for cross-session retrieval and lifecycle tracking.

## Inputs

- `skill_spec` from Specify phase
- `skill_plan` from Plan phase
- `execution_result` from Execute phase
- `reflection` from Reflect phase, including its `certification` block

## Procedure

### Step 1: Resolve Active Provider

Read the provider configuration established during startup:

```bash
PROVIDER=$(bash scripts/state-resolve-provider.sh)
```

### Step 2: Update Creation State

Update the master state file with current progress:

```json
{
  "creation_id": "<uuid>",
  "skill_name": "<name>",
  "mode": "create | clone | extend",
  "current_phase": "persist",
  "current_iteration": <n>,
  "started_at": "<iso8601>",
  "updated_at": "<iso8601>",
  "spec": { /* skill_spec */ },
  "plan": { /* skill_plan summary */ },
  "files_generated": ["<path>", ...],
  "files_validated": ["<path>", ...],
  "quality_score": <float>,
  "convergence_status": "running | converged | failed",
  "checkpoints": [{ "phase": "<name>", "timestamp": "<iso8601>" }]
}
```

### Step 3: Persist to Provider

| Provider     | Action                                       |
| ------------ | -------------------------------------------- |
| `filesystem` | Write to `.creator/skills/<name>/state.json` |
| `memory`     | Store via memory MCP tool                    |
| `mcp`        | Store via configured MCP state server        |
| `custom`     | Execute custom provider command              |

### Step 4: Update Registry

Add/update entry in `.creator/registry.json`:

```json
{
  "skills": {
    "<skill_name>": {
      "creation_id": "<uuid>",
      "mode": "<mode>",
      "status": "active | converged | failed",
      "iterations": <n>,
      "created_at": "<iso8601>",
      "updated_at": "<iso8601>",
      "output_path": "<skill_dir>/"
    }
  }
}
```

### Step 5: Write File Manifest

Record all generated files with metadata:

```json
{
  "manifest_version": "1.0",
  "skill_name": "<name>",
  "generated_at": "<iso8601>",
  "files": [
    {
      "path": "<relative_path>",
      "size_bytes": <n>,
      "template_used": "<template | custom>",
      "hash": "<sha256>"
    }
  ],
  "platform_outputs": {
    "agentskills_io": true,
    "claude_code": true,
    "opencode": false
  }
}
```

### Step 6: Snapshot Generated Skill

If converged, create a distributable snapshot:

```bash
# Create tarball for distribution
tar -czf "<skill_name>.tar.gz" -C "$(dirname "<skill_dir>")" "$(basename "<skill_dir>")"
```

## Provider-Agnostic Operations

All persistence operations follow the same interface regardless of provider:

| Operation        | Command                 |
| ---------------- | ----------------------- |
| **Save state**   | Write JSON to provider  |
| **Load state**   | Read JSON from provider |
| **List skills**  | Query registry          |
| **Check status** | Read convergence_status |

The filesystem provider is always the fallback — if configured providers fail, fall back to local files.

## Output Contract

```yaml
persist_result:
  provider_used: string
  state_path: string
  registry_updated: boolean
  files_persisted: integer
  snapshot_created: boolean
  snapshot_path: string | null
```

## Rules

1. ALWAYS update `updated_at` timestamp on every persist
2. NEVER overwrite state without incrementing `current_iteration`
3. Registry must remain valid JSON after every update
4. File manifest hashes must be SHA-256
5. Provider failures must fall back to filesystem — never lose state
