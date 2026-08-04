#!/usr/bin/env node

import fs from 'node:fs';
import path from 'node:path';

const args = process.argv.slice(2);
const rootIndex = args.indexOf('--root');
const repoRoot = path.resolve(rootIndex >= 0 ? args[rootIndex + 1] : process.cwd());
const workflowsRoot = path.join(repoRoot, '.github', 'workflows');
const allowed = new Set(['docs-pages.yml', 'docs-pages.yaml', 'docs-sync.yml', 'docs-sync.yaml']);
const failures = [];

const forbiddenCommands = [
  [/\bdocs:check\b/i, 'documentation validation'],
  [/\bnpm\s+(?:run\s+)?(?:test|lint|validate|check-format)\b/i, 'Node validation'],
  [/\b(?:pnpm|yarn|bun)\s+(?:run\s+)?(?:test|lint|validate|check)\b/i, 'JavaScript validation'],
  [/\bcargo\s+(?:test|check|clippy|fmt)\b/i, 'Rust validation'],
  [/\b(?:pytest|pyright|ruff\s+check|go\s+test|swift\s+test)\b/i, 'language validation'],
  [/\b(?:prometheus|pk|codex|cowork)\s+doctor\b/i, 'doctor execution'],
  [/\b(?:certif(?:y|ication)|secret[- ]scan|gitleaks|actionlint)\b/i, 'release validation'],
];

if (fs.existsSync(workflowsRoot)) {
  for (const name of fs.readdirSync(workflowsRoot).sort()) {
    if (!/\.ya?ml$/i.test(name)) continue;
    const file = path.join(workflowsRoot, name);
    const source = fs.readFileSync(file, 'utf8');
    const relative = path.relative(repoRoot, file);

    if (!allowed.has(name)) {
      failures.push(`${relative}: hosted workflow is not an authorized documentation workflow`);
      continue;
    }
    if (/^\s*pull_request\s*:/m.test(source)) {
      failures.push(`${relative}: pull_request triggers are forbidden`);
    }
    if (!/^\s*push\s*:/m.test(source) && !/^\s*workflow_dispatch\s*:/m.test(source)) {
      failures.push(`${relative}: workflow must be main-push or explicit deployment automation`);
    }
    if (/^\s*(?:test|tests|validate|validation|lint|doctor|certif(?:y|ication)|checks?)\s*:/mi.test(source)) {
      failures.push(`${relative}: validation-shaped job or step name is forbidden`);
    }
    for (const [pattern, label] of forbiddenCommands) {
      if (pattern.test(source)) failures.push(`${relative}: ${label} is forbidden on hosted runners`);
    }
    if (name.startsWith('docs-sync') && !/\bdocs:sync\b/.test(source)) {
      failures.push(`${relative}: docs sync workflow must invoke only the deterministic docs:sync entry point`);
    }
    if (name.startsWith('docs-pages') && !/\bbuild:deploy\b/.test(source)) {
      failures.push(`${relative}: Pages must use build:deploy so npm prebuild validation stays local`);
    }
  }
}

if (failures.length > 0) {
  console.error(failures.map(failure => `Workflow policy: ${failure}`).join('\n'));
  process.exit(1);
}

console.log('Workflow policy passed: hosted automation is documentation-only.');
