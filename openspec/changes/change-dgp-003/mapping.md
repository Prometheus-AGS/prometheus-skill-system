# change-dgp-003 — README & CLAUDE.md → site mapping (Goal 3 audit)

## README.md (512 lines) → site
| README section | Site home | Status |
|---|---|---|
| What it is / overview | /docs/guide/introduction (01) | represented |
| Install / platforms | /docs/guide/installation (19) + platform-support (17) | represented |
| Quick start / commands | /docs/guide/quick-start (00) + cli-and-scripts (16) | represented |
| Skills catalog summary | /docs/guide/skills-overview (08) + generated catalog (change-dgp-006) | represented |
| Marketplace / plugin install | /docs/guide/plugins-and-marketplace (18) | represented |
| Contributing / license | /docs/guide/contributing (21) | represented |
| Badges / repo housekeeping | out-of-site (repo-front-door only) | deliberate |
README gets a "Full documentation" link to the Pages URL once deployed (post-dgp-002 enablement).

## CLAUDE.md (1075 lines) → site
| CLAUDE.md section | Site home | Status |
|---|---|---|
| Project overview / architecture | /docs/guide/introduction + four-layer-pipeline (04) + mcp-substrate (05) | already mirrored (audited: guide chapters cover the same material in book form) |
| Essential commands (cowork/dsg/validate/build) | /docs/guide/cli-and-scripts (16) + tools-reference (13) | already mirrored |
| Learn domain | /docs/learn/* section | already mirrored |
| KBD lifecycle / progress signaling | /docs/guide/metaprompting-pmpo-kbd (02) — expanded by change-dgp-008 | partial → dgp-008 |
| Memory rules, hooks internals, BDD immutable-tests, Codex integration, agent-ops rules | out-of-site | deliberate: agent-operational policy, not user docs |
