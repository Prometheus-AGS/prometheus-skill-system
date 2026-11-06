# Analysis — issue-195-closeout

Mode: **stack specified** (existing Node/Bash/Rust/Markdown stack; no stack contest). Research was reuse-first: Tier 1 inspected this repository only; Tier 2 used Context7 for OpenCode configuration; Tiers 3–4 were not needed (no new dependency is proposed). Budget used: 7 Tier 1 queries, 2 Tier 2 queries, well under caps.

Inputs: `assessment.md` (revision 2428, adversarially reviewed PASS 0/2/1), `goals.md`, assess handoff.

## Recalled lessons that bear on these choices

- "A CRITICAL shipped despite all per-change gates passing … `install-skills-flat.sh` never applied the Codex memory setting" — surface checks (Goal D, PKG-001 rerun) must observe the installer-dispatched payload, not source text. Applied to OpenCode below: a run that loads `~/.claude` would test the wrong payload.
- "Use rebase-regenerate.sh for Generated-Only Rebase Conflicts" — Goal B regenerates `dist/`; plan for generated-only conflicts.
- "`kbd-apply mark-done` updates the task flag but does not synchronize the canonical KBD ledger" — task closure in Execute uses `begin-task`/`end-task` only.

## Knowledge gaps (from prior-context; offer `/learn-goal <topic>`, not run)

Same seven entries as assessment.md (memory-index ranking, Codex memory setting, unversioned cadence procedure, live `:23001` test publish, Cortex stub-only, SubagentStart watchdog skip, `mark-done` ledger drift). None affects these decisions.

## Build-vs-adopt decisions

| Gap | Decision | Candidate | Readiness effect |
|---|---|---|---|
| A — conflicts block | **Adopt** decision pattern + **build** status semantics | `kbd-decision-policy-ack` | Satisfies contract `functionalScope` for #195 acceptance 2; small helper change, no runtime/Rust change, no new event kind |
| B — single reference | **Adapt** existing reference | `method-context-reference` | Satisfies acceptance 5 drift risk; Markdown + one equality check in an existing validator; regenerate `dist/` |
| C — VER-001/VER-003 | **Adopt** existing driver skeleton | `kbd-194-196-session-driver` | Satisfies acceptance 4 and 6 evidence; needs operator scratch login (Gap E) at the boundary |
| D/E — OpenCode | **Adapt** with strict isolation, or report UNSUPPORTED | `opencode-headless` | Acceptance 5 surfaces; never blocks READY — an honest UNSUPPORTED is an allowed outcome |
| D/E — Codex | **Adopt** existing driver path (attempted only when its scratch home is logged in) | session driver | Same as above |
| G — cross-model review | **Adopt** per-run override; config fix left to owner | `liter-llm-env-override-judge` | Review evidence available today with label `unverified-producer-unknown` |

### A — acknowledgement design (recommended)

- Payload: `KBD-POLICY-ACK {"phaseId","path","sha256","issue","note"}` recorded with `prometheus kbd decision record`; replaced through `--supersedes`.
- `method-context.mjs`: an observed conflict is `acknowledged` only when a current, non-superseded ack for the same phase and path matches the file's current SHA-256. Any unacknowledged conflict sets `status: "CONFLICT"` (distinct from `BLOCKED`) and `--require-resolved` exits 2. A changed policy file re-raises the conflict automatically.
- Never edits `CLAUDE.md`/`AGENTS.md`. Control description stays honest: instruction-level authority; the decision actor is not recorded in canonical state, so "operator" is the existing trust model, not a new guarantee.
- Alternative rejected: a marker file — unsigned, not in the canonical journal, and invisible to recovery.

### B — shape (recommended)

Replace each 11-copy section with an identical ~4-line core (run the helper; selected-VLBV whole-phase boundary; surface conflicts and require acknowledgement; link to `references/method-context.md`) and add an equality check to an existing validator so drift fails locally. Rejected: a bare link (adherence risk) and an include system in the generator (new machinery for one section).

### C — scenario bodies

- **VER-001**: scratch project with completed production state; session invokes `value-scenario-verify` against the real repair-and-resume operation (IDR-002/003 path) as the runner; sensitivity challenge — a deliberately wrong expected value must yield FAIL, and a removed assertion must yield NOT_EXERCISED.
- **VER-003**: session drives the canonical closure writer through the closure adapter; assert the six claims (authored, measured, demonstrated, gaps, release eligibility, release authority) are reported separately, the typed disposition persists, and `eventCounts` shows no new phase, task or publication events.
- New: **CON-001**, **CON-002** (non-session: run the helper against a scratch project with the repo's real `CLAUDE.md` text), **REF-001** (non-session: equality across source, generated `dist/` and the installed generation).

### D/E — Codex path (existing, with one isolation gap)

Tier 1 evidence: `scripts/tests/lib/kbd-194-196-sessions.mjs` already drives Codex — login detection via `codex login status` under scratch `CODEX_HOME=~/.prometheus-scratch/kbd194/codex` (lines 39-41, 136) and `codex exec --json --skip-git-repo-check --ephemeral -s read-only` with tool calls read from JSONL events (lines 70-76). It keeps the real `HOME` (keychain login) and points memory URLs at a dead port. **Gap:** it never sets `PROMETHEUS_PLUGIN_ROOT`, which CLAUDE.md requires to be scratch when exercising hooks, because hook bootstrap activates bundles under `~/.prometheus/plugins/prometheus-skill-pack` and re-points installed skill symlinks. Required change: set `PROMETHEUS_PLUGIN_ROOT` (and the Claude path's equivalent) to a scratch root for every session harness. With that change, verdict stays **adopt**; without a scratch Codex login, Codex is NOT_EXERCISED.

### Generated surfaces that Goal B must regenerate and commit

`scripts/generate-skill-system-distribution.js` (`npm run build:distribution`; drift gate `npm run check:distribution`) writes tracked outputs under `dist/plugins/claude/**`, `dist/plugins/codex/**` (4,482 tracked files under `dist/`), `.claude-plugin/marketplace.json` and `.agents/plugins/marketplace.json`; `npm run generate:skills-index` refreshes `SKILLS.md`. Editing the 11 stage skills requires regenerating these in the same change and committing them; `check:distribution` must pass locally.

### D/E — OpenCode isolation requirements (from Tier 2 docs)

`OPENCODE_CONFIG_DIR` overrides but does not replace global config; auth lives in `~/.local/share/opencode/auth.json`; OpenCode reads `~/.claude` unless `OPENCODE_DISABLE_CLAUDE_CODE=1`. Required run environment: scratch `XDG_CONFIG_HOME` and `XDG_DATA_HOME`, `OPENCODE_DISABLE_CLAUDE_CODE=1`, `OPENCODE_DISABLE_AUTOUPDATE=1`, `--standalone`. Before any session, a preflight must confirm OpenCode 2.2.0 resolves config/data under the scratch roots on macOS; if not, report UNSUPPORTED.

## Open questions (carried to Spec/Plan; owner input)

1. Risk acceptance (n=1 samples; Codex/OpenCode NOT_EXERCISED acceptable?) and budgets.
2. Confirm `KBD-POLICY-ACK` as the acknowledgement mechanism.
3. Operator scratch logins at the boundary (Claude required; Codex/OpenCode optional).
4. Cross-model route: keep labelled override, or fix `~/.prometheus/kbd/models.toml` (owner-local, outside the repo; distinct `judge`/`critic` or a `backup` role, plus producer identity under `[role_identities]`). The durable fix is out of repo scope; the plan can only point to it.
5. Whether to also pass `CON-*` conflicts to `reanchor.sh` output as a blocking line (recommended: yes; it already prints `policyConflicts`).

## Adversarial review of this analysis

Judge glm-5.3 (per-run override; `unverified-producer-unknown`), verdict PASS, 0 CRITICAL / 2 WARNING / 1 SUGGESTION; anti-theater gate PASS. Both warnings rested on a packet file tree that omitted `dist/` and the Codex driver code; verified locally that `dist/` is tracked and a Codex driver exists, and incorporated the substantive asks: Codex evidence (which surfaced the `PROMETHEUS_PLUGIN_ROOT` isolation gap), the generated-surface list, and the `models.toml` location. Findings: `review/analyze/findings.json`.

## Recommendation

**READY to spec.** No new runtime code, dependency, event kind or schema is required. Spec can proceed now; the owner inputs above are needed before contract adoption (Plan), and the scratch logins only at the verification boundary.
