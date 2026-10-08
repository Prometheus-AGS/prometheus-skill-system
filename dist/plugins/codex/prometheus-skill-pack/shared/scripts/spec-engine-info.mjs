#!/usr/bin/env node
// shared/scripts/spec-engine-info.mjs
//
// Prints the spec-engine registry (config/spec-engines.json) as JSON, and
// optionally the engine detected for a repository root using the same
// heuristic as kbd-apply.sh's backend_detect:
//   - .kbd-orchestrator/project.json specBackend pin wins
//     ("openspec"|"speckit"|"native-kbd"; "auto"/absent falls through)
//   - openspec/ dir -> openspec
//   - .specify/ dir or specs/*/tasks.md -> speckit
//   - .kbd-orchestrator/changes/*/tasks.json|change.md -> native-kbd
//
// Usage: node shared/scripts/spec-engine-info.mjs [--root <dir>]
//   --root defaults to the current working directory, so running the script
//   from a repository root detects that repository's engine.
// Pure Node (node:fs, node:path); zero dependencies; works offline.

import { existsSync, readFileSync, readdirSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const scriptDir = dirname(fileURLToPath(import.meta.url));
const repoRoot = resolve(scriptDir, '..', '..');

function readRegistry(root) {
  const p = join(root, 'config', 'spec-engines.json');
  if (!existsSync(p)) {
    throw new Error(`spec-engine registry not found at ${p}`);
  }
  return JSON.parse(readFileSync(p, 'utf8'));
}

function hasEntry(root, rel) {
  return existsSync(join(root, rel));
}

// Mirror of kbd-apply.sh backend_detect (repo-wide branch + specBackend pin).
function detectEngine(root) {
  // Pin wins.
  const projectJson = join(root, '.kbd-orchestrator', 'project.json');
  if (existsSync(projectJson)) {
    try {
      const pinned = JSON.parse(readFileSync(projectJson, 'utf8')).specBackend;
      if (pinned === 'openspec' || pinned === 'speckit' || pinned === 'native-kbd') {
        return pinned;
      }
    } catch {
      /* unreadable project.json: fall through to auto-detection */
    }
  }
  if (hasEntry(root, 'openspec')) return 'openspec';
  const specsDir = join(root, 'specs');
  if (hasEntry(root, '.specify') || (existsSync(specsDir) && readdirSync(specsDir).some((e) => hasEntry(specsDir, join(e, 'tasks.md'))))) {
    return 'speckit';
  }
  const changesDir = join(root, '.kbd-orchestrator', 'changes');
  if (
    existsSync(changesDir) &&
    readdirSync(changesDir).some(
      (e) => hasEntry(changesDir, join(e, 'tasks.json')) || hasEntry(changesDir, join(e, 'change.md')),
    )
  ) {
    return 'native-kbd';
  }
  return null;
}

function main(argv) {
  let root = process.cwd();
  for (let i = 0; i < argv.length; i++) {
    if (argv[i] === '--root' && argv[i + 1]) {
      root = resolve(argv[++i]);
    }
  }
  const registry = readRegistry(repoRoot);
  const out = { default: registry.default, engines: registry.engines };
  if (root !== null) {
    out.detected = { root, engine: detectEngine(root) };
  }
  process.stdout.write(JSON.stringify(out, null, 2) + '\n');
}

main(process.argv.slice(2));
