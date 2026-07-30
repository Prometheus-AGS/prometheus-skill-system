# Decision: vendor `openai-proxy` as an optional submodule

**Status:** accepted · 2026-07-30 · `change-arc-009-openai-proxy-vendoring-decision`
**Phase:** adversarial-review-for-creation

## The problem

`openai-proxy` is what the `kbd-judge` role actually resolves to. Verified live
this session:

```
gateway = http://localhost:8181/v1
judge   = kbd-judge → openai-proxy → gpt-5.6-sol
```

It is also the **only** untracked link in that chain. Every other component —
`liter-llm`, `surreal-memory-server`, `prometheus-knowledge`, `cowork-skills`,
`disk-space-guardian` — is a submodule under `tools/`. `openai-proxy` is not:

```console
$ grep -c openai-proxy .gitmodules
0
$ grep -n openai-proxy scripts/install-binaries.sh
140:    # (routes via the local :8181 openai-proxy) unless the user already has one.
```

One comment. The installer never builds it, never installs it, and never checks
for it. The binary on this machine (`~/.local/bin/openai-proxy`, dated
2026-07-17) arrived by some path the repository does not record.

**Why that matters:** when the gateway is absent, adversarial review does not
fail — it *degrades*. The judge falls back to a harness-native model, the review
still returns `PASS`, and the findings artifact records
`isolation_mode: harness-native`. That is precisely the failure this phase exists
to eliminate: eight consecutive reviews that all passed because Claude was
grading Claude. A dependency whose absence silently weakens a safety gate is the
worst kind to leave untracked.

## Options considered

### A — Vendor as a required submodule

Add `tools/openai-proxy` and build it in `install-binaries.sh` like `liter-llm`.

**Rejected.** This session produced direct evidence against it. `install-binaries.sh`
runs under `set -euo pipefail` (line 10) and builds submodules with an unguarded
`cargo build`:

```bash
if [ -f "${REPO_ROOT}/tools/liter-llm/Cargo.toml" ]; then
    (cd "${REPO_ROOT}/tools/liter-llm" && cargo build --release -p liter-llm-cli ...)
```

When `tools/liter-llm` was pinned to a commit whose `Cargo.toml` hardcoded
`version = "1.9.3"` against a workspace that had moved to `1.11.0`,
`cargo metadata` exited 101, the installer aborted **mid-run**, and 7 of 14
binaries were left stale. Nothing about that failure was specific to liter-llm.
Adding a second required submodule doubles the surface for the same outage, and
would break installation for users who never invoke a judge.

### B — Leave it a sibling, add a doctor check only

Document the expectation, detect the gateway in `prometheus doctor`, install
nothing.

**Rejected as insufficient, though the doctor check is kept.** It fixes the
*visibility* half of the problem and none of the *availability* half. A new
machine following the documented install path still ends up with no judge, and
the first indication is a review that quietly self-grades. Detection without a
supported way to obtain the thing detected just relocates the surprise.

### C — Vendor as an optional submodule ✅ **chosen**

Track `tools/openai-proxy` so the source is pinned and reproducible, but make the
build **strictly non-fatal**: a missing or unbuildable proxy warns and the
installer continues.

This takes A's reproducibility and B's tolerance while avoiding the failure each
one has on its own. Concretely:

- `.gitmodules` gains `tools/openai-proxy` → `https://github.com/GQAdonis/openai-proxy.git`,
  pinned at `7833663`. HTTPS, matching all eight existing submodules; the
  repository is public (anonymous API fetch returns 200), so a plain `git clone
  --recursive` works without credentials.
- `install-binaries.sh` builds it inside a guarded block that cannot abort the run.
- `prometheus doctor` reports judge-gateway availability as its own check, so the
  degraded state is visible rather than inferred.

## Consequences

**Accepted costs.** One more submodule to keep current, and one more thing that
can be behind. `git submodule update --remote` covers it, and because the build
is optional, a stale or broken pin degrades the judge rather than the install.

**What this does not do.** Vendoring does not guarantee a judge is *running* —
the proxy still has to be started, and `prometheus doctor` still has to be read.
It removes the "where do I even get this?" gap, not the operational one.

**Explicitly not required.** Users who never run an adversarial review need
nothing from this. That is the whole point of optional: the cost of the gate
falls on those who use it.

## Verification

All four exercised on 2026-07-30, not merely asserted:

| Scenario | Result |
|---|---|
| Submodule absent | `skip openai-proxy (submodule not initialized…)`, installer **exit 0** |
| Submodule present, build fails | `⚠️ openai-proxy build failed … Install continues`, **exit 0** |
| Gateway reachable | `review Adversarial judge gateway ✅ Reachable at http://localhost:8181/v1` |
| Gateway stopped (`launchctl bootout`) | `⚠️ No judge gateway reachable … reviews will DEGRADE to a same-model self-review` |

The unbuildable case was produced by replacing the submodule's `Cargo.toml` with
one depending on a nonexistent crate; it was restored byte-exact afterwards
(`git status` clean, pinned at `7833663`).

`cargo test -p prometheus-cli --test doctor` → 6 passed.

### One bug this found

The first version of the guarded block still aborted `--dry-run`. Under
`set -e`, an assignment from a command substitution whose command fails is fatal,
and `find` on a `target/release` that was never built exits non-zero:

```bash
OAP_BIN=$(find .../target/release -maxdepth 1 -name openai-proxy -type f 2>/dev/null | head -1) || true
#                                                                                                  ^^^^^^^ required
```

The existing liter-llm block at line 131 has the same shape without the `|| true`
and survives only because its `target/` happens to exist. That is latent, not
fixed here.

## See also

- [`09a · Adversarial Review`](../guide/09a-adversarial-review.md) — the gate this protects
- [`09b · liter-llm`](../guide/09b-liter-llm.md) — the other half of model routing
- `skills/process/adversarial-review/references/model-configuration.md`
