# Decision Log — cowork-integration

---

### 2026-07-03 — All 6 build-vs-adopt decisions resolved (kbd-analyze)

| Decision | Verdict | Score Gap | Provenance |
|---|---|---|---|
| D-01: Fork strategy | Fork-and-extend | 90% vs 5% | research |
| D-02: Binary distribution | GitHub Releases + source fallback | 90% vs 10% | research (naming conflict + gitleaks precedent) |
| D-03: OpenCode registration | Direct JSON write | 85% vs 15% | research |
| D-04: Codex MCP config | TOML writer (Rust) | 90% vs 10% | research |
| D-05: cowork pack delegation | Shell-out to install-skills-flat.sh | 88% vs 12% | research |
| D-06: dsg integration | Graceful delegate stub | Clear | research |

No contested stack decisions (all score gaps > 15%). No escalation to pmpo-elicit required.

---

### 2026-07-03 — MMX CLI confirmed out of scope (kbd-analyze)

`mmx` is a standalone media-generation CLI (text/image/video/audio) with no plugin or skill architecture. The phase brief's reference to "MMX CLI support" maps to MiniMax Code IDE, which is already handled via the `minimax` agent entry. Documented and closed.

---

### 2026-07-03 — Kimi Desktop path confirmed (kbd-analyze)

Kimi Desktop uses a macOS Application Support path entirely separate from Kimi Code CLI:
`~/Library/Application Support/kimi-desktop/daimon-share/daimon/skills/`
This requires a new `kimi-desktop` agent entry with a macOS-only guard in `agents.rs`.

---

### 2026-07-03 — MiniMax Desktop confirmed shared path (kbd-analyze)

MiniMax Desktop Agent shares `~/.minimax/skills/` with the MiniMax Code IDE. No new install path is needed; only the detection logic in cowork's `minimax` agent entry needs to check for EITHER `~/.minimax/` OR `~/Library/Application Support/MiniMax Agent/`.

---

### 2026-07-03 — dsg parallel track approved (kbd-analyze)

disk-space-guardian will execute its 5 OpenSpec changes concurrently with cowork Wave 1, starting after cowork change-001 lands. The 17 GB build artifact accumulation in the prometheus-skill-pack tools directory makes this non-optional.
