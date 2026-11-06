#!/usr/bin/env node
// Check source sections against the one marked core in the method reference.
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const orchestrator = path.join(root, 'skills/process/kbd-process-orchestrator');
const stages = ['assess', 'child-exit', 'execute', 'handoff', 'init', 'new-child', 'next-child', 'pause', 'plan', 'resume', 'spec'];
const reference = fs.readFileSync(path.join(orchestrator, 'references/method-context.md'), 'utf8');
const core = reference.match(/<!-- selected-method-core:start -->\n([\s\S]*?)<!-- selected-method-core:end -->/);
if (!core || reference.split('<!-- selected-method-core:start -->').length !== 2) {
  throw new Error('method-context.md must contain exactly one canonical core');
}
const failures = [];
for (const stage of stages) {
  const file = path.join(orchestrator, 'skills', `kbd-${stage}`, 'SKILL.md');
  const text = fs.readFileSync(file, 'utf8');
  const headings = text.match(/^## Selected-method recovery and authority$/gm) ?? [];
  const section = text.match(/^## Selected-method recovery and authority\n[\s\S]*?(?=^## |$(?![\s\S]))/m)?.[0];
  if (headings.length !== 1 || section !== core[1]) failures.push(`kbd-${stage}: missing, duplicated or different selected-method core`);
}
if (failures.length) {
  process.stderr.write(`${failures.join('\n')}\n`);
  process.exitCode = 1;
} else {
  process.stdout.write(`selected-method core: ${stages.length}/${stages.length} byte-identical\n`);
}
