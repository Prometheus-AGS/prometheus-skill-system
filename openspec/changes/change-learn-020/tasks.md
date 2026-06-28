# Tasks — change-learn-020

- [ ] Create `skills/learn/` directory and write `skills/learn/README.md` with domain overview, skill list table (name, purpose, entry command), and a skill dependency diagram (text-based or Mermaid) showing the learn loop flow
- [ ] Add `learn` domain entry to `marketplace/marketplace.json` (domain name, description, skill count placeholder, tags: `["learning", "feynman", "spaced-repetition", "knowledge-base"]`)
- [ ] Add `learn` to the skill category list in `.claude-plugin/plugin.json` so the plugin marketplace surfaces learn-domain skills
- [ ] Update `scripts/validate-skills.js` (or the npm validate script config) to include `skills/learn/` in the `validate:strict` sweep — confirm `npm run validate:strict` exits 0 on the empty domain directory
- [ ] Add `learn` row to the skills table in the top-level `README.md` (columns: domain, description, example skills, install path)
