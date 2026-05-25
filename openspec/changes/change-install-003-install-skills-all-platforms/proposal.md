# change-install-003-install-skills-all-platforms

**Phase**: machine-installation-2026-05-25  
**Status**: PENDING  
**Gaps closed**: G-SKILL-1, G-SKILL-2, G-SKILL-3, G-SKILL-4, G-INST-4

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
