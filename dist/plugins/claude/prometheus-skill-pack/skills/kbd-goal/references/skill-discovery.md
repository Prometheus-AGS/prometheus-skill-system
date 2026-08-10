# Skill/MCP Discovery Reference

`kbd-goal-discover.sh` analyzes a goal description using keyword matching
against this table to recommend skills and MCP servers.

## Domain → Skills/MCPs Mapping

| Domain Keywords | Skills | MCPs |
|----------------|--------|------|
| go, golang, gopher | golang-patterns, golang-testing | context7 |
| rust, cargo, crate | rust-reviewer, rust-auditor | context7 |
| python, flask, django, fastapi, asyncio | python-reviewer | context7 |
| typescript, ts, react, next, nextjs, vue, svelte | typescript-reviewer | context7, shadcn |
| javascript, js, node, bun, deno | typescript-reviewer | context7 |
| swift, xcode, ios, macos, swiftui, uikit | - | context7 |
| kotlin, android, compose | - | context7 |
| database, sql, postgres, postgresql, sqlite, mysql | database-reviewer | supabase-mcp |
| api, rest, graphql, grpc, openapi | - | context7 |
| auth, authentication, oauth, jwt, session | security-reviewer | - |
| deploy, deployment, docker, k8s, kubernetes, ci, cd | devops-engineer | kubernetes |
| test, testing, tdd, bdd, e2e, playwright | tdd-guide, e2e-runner | - |
| security, vulnerability, xss, injection, csrf | security-reviewer | - |
| refactor, cleanup, dead code, unused | refactor-cleaner, code-simplifier | - |
| performance, perf, optimization, bottleneck, slow | performance-optimizer | - |
| documentation, docs, readme, changelog | doc-updater | - |
| ui, ux, design, css, tailwind, figma, storybook | ui-ux-designer | stitch, shadcn |
| llm, ai, rag, vector, embedding, prompt, agent | ai-engineer, prompt-engineer | surreal-memory |
| git, github, pr, branch, merge, commit | - | mcp__github |
| cli, command line, terminal, shell, bash | - | - |

## Advisory Only

Recommendations from `kbd-goal-discover.sh` are advisory. The user always
decides which skills to load. Output is printed once at goal start; not blocking.

## Adding New Entries

To add new domain mappings, edit this file. Format:
```
| domain keyword(s) | skill-name-1, skill-name-2 | mcp-server-1 |
```

The script reads this file and builds its keyword table dynamically.
Empty cells (`-`) mean no recommendation for that column.

## Example Discovery Output

For goal `"build a weekly standup generator CLI in Go"`:

```json
{
  "recommended_skills": ["golang-patterns", "golang-testing"],
  "recommended_mcps": ["context7"],
  "rationale": "Goal mentions Go; golang-patterns covers idioms; golang-testing for test patterns; context7 for Go stdlib docs."
}
```

For goal `"add authentication to the React dashboard using JWT"`:

```json
{
  "recommended_skills": ["typescript-reviewer", "security-reviewer"],
  "recommended_mcps": ["context7"],
  "rationale": "Goal mentions React (typescript-reviewer), authentication and JWT (security-reviewer); context7 for React docs."
}
```
