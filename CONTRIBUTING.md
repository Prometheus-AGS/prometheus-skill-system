# Contributing to Prometheus Skill System

Use the repository's [agent rules](AGENTS.md) and [canonical operating rules](CLAUDE.md) when preparing a contribution. Complete the production implementation for the active phase before authoring or running tests, checks, formatters or reviews. Acceptance requires local integration evidence through the production entry point and real collaborators.

## Prerequisites and setup

The root `package.json` requires Node.js 20.19.0 or later and pins Node 22.23.1 through Volta. Team helpers require Node 22 or later. Use the checked-in lockfile. Rust work follows the relevant workspace toolchain and dependency pins; load `prometheus-rust-workspace` for static guidance before Rust or Cargo work. Native service installation needs Bash 4 or later and has component-specific platform limits.

```bash
git clone --recurse-submodules https://github.com/Prometheus-AGS/prometheus-skill-system.git
cd prometheus-skill-system
npm ci
```

Dependency setup does not install skills into a user's harness directories or start services. Choose an installation scope deliberately using the [installation guide](docs/guide/19-installation.md). Use an isolated home for installer integration evidence; inherited `CODEX_HOME` takes precedence over a selected `--home` for Codex.

## Creating a skill

1. Choose the appropriate `skills/<domain>/<skill-name>` directory and use [the skill template](docs/SKILL_TEMPLATE.md).
2. Write the frontmatter and instructions against the actual helper interfaces. Keep the main instructions concise and move supporting detail into `references/`.
3. Make scripts under the skill's `scripts/` directory executable. Preserve protected BDD scenarios.
4. Finish all planned production changes in the phase before the final validation batch. At that boundary, strict skill validation is required for a new skill:

   ```bash
   npm run validate:strict skills/<domain>/<skill-name>
   ```

5. Exercise the real installed or packaged entry point with its actual collaborators. Structural validation, a helper-only result or a legacy unit suite does not prove that a harness can discover and invoke the skill.

See [skill authoring](docs/skill-authoring-guide.md), [plugin delivery](docs/guide/18-plugins-and-marketplace.md) and [local validation](site/docs/operations/local-validation-and-docs-automation.md).

## Rust changes

`tools/forge-rs/` contains the code enrichment engine. Other tools and substrate crates have their own workspaces and release identities. Read the affected workspace manifests and protected integration scenarios before implementing.

After all phase production is complete, choose the smallest integration target that exercises the changed production path, for example the command shape `cargo test -p <package> --test <integration-target>` within the affected workspace. Select real package and target names from that workspace; this is a command shape, not a ready-made gate. Do not use a workspace-wide unit-inclusive test command as the normal acceptance gate. Run required formatting, lint and broader integration checks only at the applicable final boundary. Only one Cargo/rustc process may build on the machine at a time.

## Preparing a pull request

- Record the exact local integration commands, environment scope and results after completing production work. Clearly separate source changes, generated artifacts, installed behavior and release acceptance.
- Run the relevant strict skill, distribution, documentation and protected-test integrity gates locally. Review requirements follow the completed-phase boundary.
- Preserve credentials, local runtime data and scratch files outside the commit. Review intentional project identity/configuration separately rather than excluding an entire project metadata directory by name.
- Commit dependency lockfiles when dependencies change. Keep submodule URLs HTTPS and pins deliberate; review every submodule update.
- Protected BDD changes require the repository owner's SSH-signed canonical approval manifest. Dependency and protected version changes require their applicable owner approval.
- Push after the applicable local gates pass. GitHub source hosting and review are supported; hosted test workflows and automatic secret-scanning claims are not release evidence. Run any required secret scan locally and record its result.

A passing local gate does not authorize merging or publishing. Follow the repository's local review receipt and owner merge requirements.

## Style and questions

Follow the existing skill, Rust and JavaScript conventions and the project UI protocol for instructional copy. Use the formatter configured by each workspace at the final boundary. Open a scoped issue in [Prometheus Skill System](https://github.com/Prometheus-AGS/prometheus-skill-system/issues) when a contract or ownership decision needs clarification.
