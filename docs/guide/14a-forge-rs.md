# 14a · forge-rs & the Template Runtimes

Two separate systems share the word "forge". They are unrelated — different repos,
different engines, different jobs. The naming is genuinely confusing, so start here:

| | `forge` (forge-rs) | `template-forge` |
|---|---|---|
| Job | inject the right knowledge into an agent's context | render brand-themed HTML/SVG |
| Engine | Tera | MiniJinja |
| Writes | one Markdown document | HTML/SVG to stdout or a file |
| Lives in | `tools/forge-rs/` | `skills/imported/artifact-refiner/tools/template-forge-rs/` |

## Part 1 — `forge`: a context-enrichment engine

The most common misconception is that forge-rs is a scaffolding tool. **It is not.** It
never writes source files into your project. It reads an OpenSpec task, works out which
skills apply, renders their templates into **one Markdown document**, and hands that
document to the AI agent as reading material.

Templates here are *reference patterns injected into agent context*, not files emitted to
disk.

```
OpenSpec task folder
    │
    ▼ forge enrich <task-path>
    │
    ├── 1. Read tasks.md → detect language
    ├── 2. SkillRegistry.resolve(language, description)
    ├── 3. ConstitutionChecker → flag forbidden patterns
    ├── 4. KarpathyFocus → pk focus "<topic>" → prior learned context
    ├── 5. Tera template rendering
    └── 6. Write .forge/enriched/<task-id>.context.md
              ↓ the AI agent reads this and implements the code
```

It sits at **Layer 4** — the lowest level of the skill stack — between OpenSpec output and
the agent that writes the code.

### Commands

```console
$ forge --help
forge-rs: code enrichment for Prometheus AGS

Commands:
  enrich             Enrich an OpenSpec task with skills, constitution, and Karpathy context
  reflect            Process a completed iteration into the Karpathy learning loop
  drift              Report skill drift across recent iterations
  validate           Check a file against the active language constitution
  mcp                Start the forge MCP server
  init               Scaffold .forge/ in the current project
  status             Show forge configuration and service status
  skill              Manage forge skills
  constitution       Edit a language constitution
  package-librefang  Package an agent directory as a LibreFang WASM skill zip
```

`--project-root`, `--skills-root`, and `--pk-mcp-url` are global — they work on every
subcommand.

| Command | Signature |
|---|---|
| `enrich` | `forge enrich <TASK_PATH>` — folder or `tasks.md` |
| `reflect` | `forge reflect <ITERATION_ID>` |
| `validate` | `forge validate --language <LANG> <FILE>` — exits `1` on any `error`-severity hit |
| `mcp` | `forge mcp [--port 8943] [--bind 127.0.0.1]` |
| `init` | `forge init` — scaffolds `.forge/` |
| `package-librefang` | `forge package-librefang <AGENT_DIR> [--no-build] [-o OUT]` |

### Language constitutions

The genuinely useful, immediately usable feature. A constitution is a TOML file declaring
what your codebase considers correct, and `forge validate` enforces it:

```toml
language = "rust"

[standards]
web_framework  = "axum"
error_handling = "thiserror for library errors, anyhow for application errors"
state_sharing  = "Arc<RwLock<T>> default; parking_lot::Mutex for hot paths"

[[forbidden_patterns]]
pattern  = "unwrap()"
reason   = "Panics in production. Use ? or explicit error handling."
severity = "error"

[[forbidden_patterns]]
pattern  = "thread::sleep"
reason   = "Blocking sleep in async context. Use tokio::time::sleep."
severity = "error"

[[forbidden_patterns]]
pattern  = "std::sync::Mutex"
reason   = "Use parking_lot::Mutex for lower overhead and no poisoning."
severity = "warning"
```

Six ship out of the box: `rust`, `typescript`, `python`, `flutter`, `go`, `tauri`.
`forge constitution <language>` scaffolds and opens one in `$EDITOR`.

> Matching is **case-insensitive substring**, not AST or regex. `unwrap()` will match
> inside comments and string literals too. Severity is `error | warning | info`; only
> `error` fails the command.

### Skill manifests and triggers

A skill becomes visible to forge when it has a `skill.toml`:

```toml
name        = "axum-patterns"
language    = "rust"
description = "Axum 0.8 router, middleware, extractor, and state injection patterns"

[[templates]]
path               = "router.rs.tera"
output_description = "Axum router scaffold with typed state injection"

[[triggers]]
type     = "AlwaysForLanguage"
language = "rust"

[[triggers]]
type     = "Keywords"
keywords = ["axum", "router", "middleware", "handler", "endpoint", "api"]

depends_on = ["rust/error-handling"]
```

Four trigger types exist; two work as documented:

| Trigger | Status |
|---|---|
| `AlwaysForLanguage` | ✅ works |
| `Keywords` | ✅ case-insensitive substring match on the task description |
| `PathGlob` | ⚠️ not real globbing — `contains(glob.trim_matches('*'))` |
| `DependsOnPackage` | ❌ hardcoded to `false`; never fires |

Only **10** `skill.toml` files exist repo-wide, so most `templates/` directories are
invisible to forge.

### Template variables

Exactly six, and this is the important limitation:

| Variable | Value |
|---|---|
| `{{ task_description }}` | full `tasks.md` content |
| `{{ task_id }}` | change ID |
| `{{ task_path }}` | path to the task folder |
| `{{ acceptance_criteria }}` | GIVEN/WHEN/THEN blocks, when present |
| `{{ constitution_summary }}` | active constitution |
| `{{ karpathy_focus }}` | `pk focus` output, when `pk` succeeded |

> **Templates that declare their own variables silently fail.** `router.rs.tera` expects
> `module_name`, `state_type`, `resource_name`, and `base_path` — none of which forge
> supplies. Tera errors on undefined variables, and the error is swallowed with only a
> `warn!` log, so the template is simply omitted from the context document. There is no
> `--var` mechanism in the shipped binary. If a skill's templates are not appearing in
> your enriched context, this is why.

Project-local overrides do work: a template at
`.forge/skills/<language>/<skill>/templates/<name>.tera` takes priority over the pack's.

### The drift loop

The genuinely novel idea. `forge reflect` measures how much of each skill's suggestion the
user actually kept, computes an `acceptance_rate`, and skills below **0.5** are sorted to
the *end* of the next enrichment — visible, but deprioritised. No applicable skill is ever
silently dropped.

> **The loop has no live data source today.** `forge reflect` on a task with no prior
> record fabricates an empty stub (hardcoded `Language::Rust`, empty skill list) and
> reports success anyway. Nothing in this repo writes real `IterationRecord`s —
> `agent_produced` and `user_accepted` are never populated. Treat drift as designed and
> wired, but not yet fed.

### The MCP server (`:8943`)

Runs under launchd as `ai.prometheus.forge-mcp`. Two routes only:

- `POST /mcp` — JSON-RPC 2.0, **auth required**
- `GET /health` — no auth

> The README describes `"transport": "sse"` and a `GET /events` stream. **Neither
> exists.** It is plain JSON-RPC over HTTP POST.

Authentication is a bearer token from `FORGE_MCP_TOKEN`, compared in constant time. If
unset or blank, a random UUID is minted per boot and printed to stderr — which is why
clients start 401ing after a restart unless the token is pinned. The launchd plist pins a
stable localhost-only dev token.

```bash
curl -s -X POST http://127.0.0.1:8943/mcp \
  -H "Authorization: Bearer $FORGE_MCP_TOKEN" \
  -H 'Content-Type: application/json' \
  -d '{"jsonrpc":"2.0","id":1,"method":"tools/list"}'
```

Four tools:

| Tool | Reality |
|---|---|
| `forge_enrich` | ✅ full pipeline; rejects `task_path` outside the project root |
| `forge_validate` | ✅ constitution check — but takes inline `content`, not a path |
| `forge_reflect` | ⚠️ works, but reports "Ingested to prometheus-knowledge" unconditionally; a failing `pk ingest` only logs a warning |
| `forge_drift` | ⚠️ does not report drift — only checks whether the directory exists; its `language` argument is never read |

### Documented but not implemented

Do not plan around these:

- **`forge template new\|render\|list\|validate\|edit`** — the entire subcommand tree in
  the README. `forge template --help` → `error: unrecognized subcommand 'template'`. The
  four files under `tools/forge-rs/templates/meta/` are unreachable dead code.
- **`optimize` / `generate` / `evolve`** — printed by `forge status` as `[EXPERIMENTAL]`.
  They are hardcoded `println!` strings; no such subcommands exist.
- **`forge skill add`** — prints "skill registry pull not yet implemented".
- **`forge skill sync`** — a no-op that prints a message.
- **`forge drift --language`** — accepted, then discarded.
- **`Constitution.required_skills`** — parsed into the struct, read by nothing.

## Part 2 — `template-forge`: the branded artifact renderer

A different tool with a different job: **deterministic branded-artifact rendering**. A
brand TOML supplies colours and typography; a MiniJinja template consumes them.

```console
$ template-forge --help
Branded-artifact renderer for the artifact-refiner skill pack

Commands:
  version       Print the version string and exit
  list-engines  List all template engines compiled into this binary
  list-brands   List brands available in `<library>/brands/`
  render        Render a template for a brand
```

### Brands

```toml
[meta]
name = "KnowMe"
tagline = "Identity layer for human-AI collaboration"

[colors.light]
bg = "#F8FAFC"
surface = "#FFFFFF"
ink = "#0B0F14"
ember = "#E04E28"

[colors.dark]
bg = "#0B0F14"
surface = "#0F1620"
ink = "#E5EEF8"
ember = "#E04E28"

[typography]
display = "Space Grotesk"
ui      = "Inter"
body    = "Roboto"
mono    = "JetBrains Mono"
```

**Both light and dark palettes are required** — deliberately, so templates can reference
either without conditional logic. `bg`, `surface`, `ink`, and `ember` are mandatory in
each.

### Rendering

```bash
template-forge render \
  --brand knowme \
  --template logo-icon \
  --library        skills/imported/artifact-refiner/assets/library \
  --templates-base skills/imported/artifact-refiner/tools/template-forge-rs/templates
```

```svg
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 512 512" width="512" height="512">
  <rect width="512" height="512" fill="#E04E28" rx="64"/>
  <text x="50%" y="50%" font-family="Space Grotesk, system-ui, sans-serif"
        font-size="320" font-weight="700" fill="#ffffff"
        text-anchor="middle" dominant-baseline="central">K</text>
</svg>
```

`#E04E28` and `Space Grotesk` came straight from the brand TOML — that is the entire value
proposition in one output. Change the brand file, re-render, and every artifact follows.

Templates are addressed by stem (`.html` is appended) and the context root is `brand`, so
templates reference `{{ brand.meta.name }}`, `{{ brand.colors.dark.ember }}`. Four ship:
`brand-guide`, `logo-icon`, `moodboard`, `vite-shell-css`.

**Hot reload works** — templates load lazily via a path loader, so edits are picked up on
the next render without restarting.

> `--engine` implies a choice, but only `minijinja` is compiled in. Askama, Tera,
> Handlebars, and Mustache are named as follow-on phases in the source. The trait exists so
> they can be added without API churn.

### As an MCP server

`template-forge-mcp` speaks **stdio** (unlike forge-mcp's HTTP), exposing three tools:
`template_forge_list_engines`, `template_forge_list_brands`, `template_forge_render`.

```json
"template-forge": {
  "command": "template-forge-mcp",
  "args": [
    "--library", "${CLAUDE_PLUGIN_ROOT}/assets/library",
    "--templates-base", "${CLAUDE_PLUGIN_ROOT}/tools/template-forge-rs/templates"
  ]
}
```

### Where it is actually used

`scaffold-react-vite` shells out to it to generate brand-token CSS for a new Vite project,
and `rebrand-artifact` uses it to regenerate CSS variables after swapping a brand. That is
the concrete link between [artifact-refiner](11-artifact-refiner.md) and the template
runtime: refine an artifact, then scaffold it into a real project with your brand baked in.

## Choosing between them

- **Need an agent to write code that follows your conventions?** → `forge enrich`, backed
  by a constitution.
- **Need to check code against those conventions?** → `forge validate`.
- **Need branded HTML/SVG/CSS output?** → `template-forge render`.
- **Need to ship a generated agent as WASM?** → `forge package-librefang`.

## See also

- [11 · Artifact Refiner](11-artifact-refiner.md) — the main consumer of template-forge
- [12 · Agent Creator](12-native-agent-generator.md) — uses `forge package-librefang`
- [06 · Memory and Learning](06-memory-and-learning.md) — the Karpathy loop forge feeds
