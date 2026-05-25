# Tasks: change-install-003-install-skills-all-platforms

- [ ] Check if `~/.config/zed/skills/` or `~/.config/zed/` exists to confirm zed config dir
- [ ] Edit `scripts/install-skills-flat.sh` — add `install_to_dir "zed" "$HOME/.config/zed/skills"` after the cline line
- [ ] Run `bash scripts/install-skills-flat.sh` from project root
- [ ] Verify `ls ~/.config/zed/skills/ | wc -l` → > 0
- [ ] Verify `ls ~/.opencode/skills/ | wc -l` → > 0
- [ ] Verify `ls ~/.cursor/skills/ | wc -l` → > 0
- [ ] Verify `ls ~/.codex/skills/ | wc -l` → > 0
- [ ] Verify `ls ~/.claude/skills/ | wc -l` → >= 394
