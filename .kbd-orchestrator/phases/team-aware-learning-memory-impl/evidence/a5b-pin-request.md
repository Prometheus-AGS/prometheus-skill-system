# Operator pin request: prometheus-skills-mini → v1.10.0

**Change:** A5b (team-aware-learning-memory-impl). **Owner:** operator. `versions.toml` is operator-authored, and agents never write it.

## What to land, in one commit on mini `main`

`rules/test/versions-toml.test.mjs` requires every `[submodules]` gitlink in HEAD to equal its `versions.toml` value. The gitlink move and the `versions.toml` edit therefore have to be in the same commit.

### 1. Gitlinks

| Path | From (v1.9.0) | To (v1.10.0) |
|---|---|---|
| `tools/prometheus-knowledge` | `1bbaecc61391257e4bb69d67d5c3716aa8dece4d` | `1126c3043f8fdd0740e4b0265967c3031da28894` |
| `tools/surreal-memory-server` | `777cf7249d46ecfb9da2d38aefb1508d3b0311e7` | `0af8ae1f3c7486bf7128beff8ef83aaf1a832ea6` |

```bash
git -C tools/prometheus-knowledge fetch origin --tags && git -C tools/prometheus-knowledge checkout v1.10.0
git -C tools/surreal-memory-server fetch origin --tags && git -C tools/surreal-memory-server checkout v1.10.0
git add tools/prometheus-knowledge tools/surreal-memory-server
```

### 2. `versions.toml` diff

```diff
-"tools/prometheus-knowledge"    = "1bbaecc61391257e4bb69d67d5c3716aa8dece4d"   # prometheus-knowledge v1.9.0
+"tools/prometheus-knowledge"    = "1126c3043f8fdd0740e4b0265967c3031da28894"   # prometheus-knowledge v1.10.0
-"tools/surreal-memory-server"   = "777cf7249d46ecfb9da2d38aefb1508d3b0311e7"    # surreal-memory-server v1.9.0 (surrealdb =3.3.0, matching the image below)
+"tools/surreal-memory-server"   = "0af8ae1f3c7486bf7128beff8ef83aaf1a832ea6"    # surreal-memory-server v1.10.0 (surrealdb =3.3.0, matching the image below)
```

The SurrealDB image pin does not change: surreal-memory-server v1.10.0 still uses `surrealdb =3.3.0`.

### 3. Check

```bash
node --test rules/test/versions-toml.test.mjs
```

## What v1.10.0 brings to mini

- **pk:** `pk context` scores every entry (large knowledge bases are no longer truncated), and `pk ingest --type/--tag` and `pk context --tag` are new. The learning worker attributes records and commits prompt snapshots. Mini's `deliverToPk` invocation is unchanged. D1b will pass `--type/--tag` once this lands.
- **surreal-memory:** search responses are lean, the categories filter works, and there is a loopback-only `rekey_agent_id` operation.
