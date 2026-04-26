# liter-llm-bridge — Install

You are running the **install** phase of the liter-llm-bridge skill. The goal is to get the `liter-llm` binary built and on PATH, with no other side effects.

## Model Selection

**Required model class: `small`**

This is mechanical install work — clone, build, verify. No reasoning required. Read `project.json → model_policy.phases.liter-bridge-install`. If the policy is absent, proceed; the host model is fine for this phase.

## Procedure

1. **Sanity check Rust toolchain** — run `bash scripts/detect-cargo.sh`. The skill pack precondition guarantees `cargo` is present, but verify before doing work. If absent, stop and instruct the user to install Rust via `rustup`.

2. **Clone or update the fork** — run `bash scripts/install-liter-llm.sh`. The script:
   - Clones `https://github.com/GQAdonis/liter-llm.git` to `~/.local/share/liter-llm/src` if absent.
   - Otherwise `git fetch && git pull --ff-only` to update.
   - Pins to the latest tagged release if `--release` flag is passed (default: track `main`).

3. **Build and install the binary** — the same script runs:

   ```
   cargo install --path crates/liter-llm-cli --locked --root ~/.local
   ```

   This puts `liter-llm` in `~/.local/bin/`. Verify the user's PATH includes it; if not, suggest they add it.

4. **Verify install** — run `liter-llm --version`. Expected output: a semver string. If the binary is not found or errors, do NOT proceed to `/liter-llm-bridge configure` — debug install first.

5. **Verify MCP transport works** — run `liter-llm mcp --transport stdio --help` (or equivalent help flag). The MCP subcommand must be available; otherwise the user has an old version.

## Output

Write a one-line summary to stdout:

```
INSTALL COMPLETE: liter-llm <version> at ~/.local/bin/liter-llm
Next: /liter-llm-bridge configure
```

If any step fails, output `INSTALL FAILED: <reason>` and stop. Do not partially configure.

## Idempotency

Running install on an already-installed system should:

- Update the source tree if newer commits exist
- Rebuild only if the source changed
- Skip the `cargo install` step if the binary is up-to-date (cargo handles this automatically with `--locked`)

Never delete or move existing config (`~/.config/liter-llm/config.toml`) during install — that is `configure`'s job.
