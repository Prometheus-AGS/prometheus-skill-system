# Tasks: change-drs-009-commit-push

- [ ] Run git status — verify only expected files (skills/research/, README.md, marketplace/, docs/)
- [ ] Run git add skills/research/ README.md marketplace/marketplace.json docs/CONTRIBUTING.md
- [ ] Run git status — confirm staged files, no secrets or temp files
- [ ] Run git commit with conventional commits feat(research) message
- [ ] Run git push origin main
- [ ] Verify: git log --oneline -1 shows the commit on origin/main
