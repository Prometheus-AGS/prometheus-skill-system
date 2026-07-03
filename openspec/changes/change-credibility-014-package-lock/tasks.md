# Tasks: change-credibility-014-package-lock

- [ ] Check `git check-ignore -v package-lock.json` — if excluded, add `!package-lock.json` to `.gitignore`
- [ ] Run `npm install` to regenerate a clean lockfile
- [ ] `git add package-lock.json`
- [ ] Check `site/package-lock.json` — commit it too if not already tracked
- [ ] Update `.github/workflows/validate.yml`: change `npm install` to `npm ci`
- [ ] Verify `npm ci` succeeds locally
- [ ] Verify `git status` shows `package-lock.json` as tracked
