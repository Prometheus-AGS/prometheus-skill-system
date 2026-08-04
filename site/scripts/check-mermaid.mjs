#!/usr/bin/env node

import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { JSDOM } from 'jsdom';

const dom = new JSDOM('<!doctype html><html><body></body></html>');
globalThis.window = dom.window;
globalThis.document = dom.window.document;
const { default: mermaid } = await import('mermaid');

const siteRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const repoRoot = path.resolve(siteRoot, '..');
const roots = [
  path.join(siteRoot, 'docs'),
  path.join(repoRoot, 'docs', 'guide'),
  path.join(repoRoot, 'docs', 'learn'),
];

const markdownFiles = directory => {
  const files = [];
  for (const entry of fs.readdirSync(directory, { withFileTypes: true })) {
    const candidate = path.join(directory, entry.name);
    if (entry.isDirectory()) files.push(...markdownFiles(candidate));
    else if (entry.isFile() && /\.mdx?$/.test(entry.name)) files.push(candidate);
  }
  return files;
};

const failures = [];
let diagramCount = 0;
let fileCount = 0;

for (const file of roots.flatMap(markdownFiles).sort()) {
  const source = fs.readFileSync(file, 'utf8');
  const fences = [...source.matchAll(/^```mermaid[ \t]*\r?\n([\s\S]*?)^```[ \t]*$/gm)];
  if (fences.length === 0) continue;
  fileCount += 1;

  for (const [index, fence] of fences.entries()) {
    diagramCount += 1;
    try {
      await mermaid.parse(fence[1]);
    } catch (error) {
      const message = error instanceof Error ? error.message.split('\n')[0] : String(error);
      failures.push(`${path.relative(repoRoot, file)} diagram ${index + 1}: ${message}`);
    }
  }
}

if (failures.length > 0) {
  console.error(failures.map(failure => `Mermaid check: ${failure}`).join('\n'));
  process.exit(1);
}

console.log(`Mermaid check passed: ${diagramCount} diagrams across ${fileCount} files.`);
