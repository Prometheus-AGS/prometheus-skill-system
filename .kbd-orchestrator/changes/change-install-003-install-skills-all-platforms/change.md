---
id: change-install-003-install-skills-all-platforms
title: Install skills all platforms
phase: machine-installation-2026-05-25
gaps: [G-SKILL-1, G-SKILL-2, G-SKILL-3, G-SKILL-4, G-INST-4]
agent: claude-code
status: done
scope:
  - scripts/install-skills-flat.sh
---

# change-install-003-install-skills-all-platforms — Install skills all platforms

## Summary

Add `zed` as a skill install target in `install-skills-flat.sh`, then run the script globally to install symlinks for all supported platforms.

## Files Modified

- `scripts/install-skills-flat.sh` — add zed target line

## Acceptance Criteria

- `ls ~/.config/zed/skills/ | wc -l` → > 0
- `ls ~/.opencode/skills/ | wc -l` → > 0
- `ls ~/.cursor/skills/ | wc -l` → > 0
- `ls ~/.codex/skills/ | wc -l` → > 0
- `ls ~/.claude/skills/ | wc -l` → >= 394

## Tasks

- [x] 1. Check if `~/.config/zed/skills/` or `~/.config/zed/` exists to confirm zed config dir
- [x] 2. Edit `scripts/install-skills-flat.sh` — add `install_to_dir "zed" "$HOME/.config/zed/skills"` after the cline line
- [x] 3. Run `bash scripts/install-skills-flat.sh` from project root
- [x] 4. Verify `ls ~/.config/zed/skills/ | wc -l` → > 0
- [x] 5. Verify `ls ~/.opencode/skills/ | wc -l` → > 0
- [x] 6. Verify `ls ~/.cursor/skills/ | wc -l` → > 0
- [x] 7. Verify `ls ~/.codex/skills/ | wc -l` → > 0
- [x] 8. Verify `ls ~/.claude/skills/ | wc -l` → >= 394
