# Contributing to prometheus-skill-pack

Thank you for contributing to the Prometheus Skill Pack — a production-grade collection of AI agent skills for the Claude Code and multi-platform AI ecosystem.

## Prerequisites

| Tool    | Minimum version |
| ------- | --------------- |
| Node.js | 20              |
| npm     | 10              |
| Rust    | stable (latest) |
| Git     | 2.36+           |

## Setup

```bash
# Clone with all submodules
git clone --recurse-submodules https://github.com/Prometheus-AGS/prometheus-skill-pack.git
cd prometheus-skill-pack

# Install Node dependencies
npm install

# Install skills to Claude Code (local dev)
bash scripts/install-skills-flat.sh

# Verify the toolchain
bash shared/scripts/detect-toolchain.sh
```

## Creating a Skill

1. **Choose a domain**: `skills/{react,rust,ui-ux,devops,testing,documentation,learn,...}`

2. **Create the skill directory**:

   ```bash
   mkdir -p skills/<domain>/<skill-name>
   cp docs/SKILL_TEMPLATE.md skills/<domain>/<skill-name>/SKILL.md
   ```

3. **Edit the frontmatter** — required fields:

   ```yaml
   ---
   name: my-skill-name
   description: One-line description (1–1024 chars)
   license: MIT
   metadata:
     author: your-name
     version: '1.0.0'
     category: <domain>
     tags: [tag1, tag2]
   ---
   ```

4. **Write skill instructions** — keep the main file under 500 lines; move detail to `references/`.

5. **Validate**:

   ```bash
   npm run validate:strict skills/<domain>/<skill-name>
   ```

6. **Test locally**:
   ```bash
   npm run install:project
   # In Claude Code: /reload-plugins, then try /<skill-name>
   ```

## Developing forge-rs

The `tools/forge-rs/` directory contains the Rust code enrichment engine.

```bash
cd tools/forge-rs

# Build
cargo build --workspace

# Run tests
cargo test --workspace

# Check formatting and lints
cargo fmt --check --all
cargo clippy --all --all-features -- -D warnings
```

## PR Checklist

Before opening a pull request:

- [ ] All native skills validate strict: `npm run validate:strict`
- [ ] No SSH submodule URLs in `.gitmodules` (use HTTPS)
- [ ] No hardcoded credentials or API keys anywhere
- [ ] forge-rs tests pass: `cargo test --workspace` in `tools/forge-rs/`
- [ ] `package-lock.json` is committed and `npm ci` succeeds cleanly
- [ ] No files in `.prometheus/` are staged
- [ ] `SCRATCHPAD.md` is not staged (it is gitignored)
- [ ] `gitleaks` scan clean (runs automatically in CI)

## Code Style

- Skills: follow `docs/SKILL_TEMPLATE.md`; no Windows-style backslashes in paths
- Rust: `cargo fmt` enforced; no `unwrap()` in non-test code; `anyhow` for applications, `thiserror` for libraries
- TypeScript: `prettier` enforced via `npm run check-format`

## Submodule Policy

- All submodule URLs must use HTTPS (never SSH)
- New submodules must be reviewed and pinned to a specific SHA after initial integration
- Update the pin comment in `.gitmodules` when advancing a submodule

## Questions

Open a GitHub issue using one of the provided templates or reach out via the Prometheus AGS GitHub organization.
