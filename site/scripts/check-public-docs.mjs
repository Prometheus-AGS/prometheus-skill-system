#!/usr/bin/env node

import {readdir, readFile} from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import {fileURLToPath} from 'node:url';

const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const siteDir = path.resolve(scriptDir, '..');
const repoDir = path.resolve(siteDir, '..');
const roots = [path.join(siteDir, 'docs'), path.join(repoDir, 'docs', 'guide')];

const forbidden = [
  {
    label: 'machine-local home path',
    pattern: new RegExp(
      `${escapeRegExp(`${os.homedir()}${path.sep}`)}|/Users/[^/\\s]+/|/home/[^/\\s]+/`,
      'g',
    ),
  },
  {
    label: 'literal bearer credential',
    pattern: /Authorization:\s*Bearer\s+[A-Za-z0-9_-]{32,}/g,
  },
  {
    label: 'literal 32-byte private key field',
    pattern: /private[_-]?key\s*[:=]\s*["'][A-Za-z0-9+/_=-]{40,}["']/gi,
  },
  {
    label: 'raw private wiki path',
    pattern: /\.prometheus\/knowledge\/wiki\/[^\s)`]+/g,
  },
];

const findings = [];
for (const root of roots) {
  for (const file of await markdownFiles(root)) {
    const content = await readFile(file, 'utf8');
    const relative = path.relative(repoDir, file);
    for (const rule of forbidden) {
      rule.pattern.lastIndex = 0;
      for (const match of content.matchAll(rule.pattern)) {
        const line = content.slice(0, match.index).split('\n').length;
        findings.push(`${relative}:${line}: ${rule.label}`);
      }
    }
  }
}

if (findings.length > 0) {
  console.error('Public documentation sanitizer rejected the following content:');
  for (const finding of findings) console.error(`- ${finding}`);
  process.exit(1);
}

console.log('Public documentation sanitizer passed.');

async function markdownFiles(root) {
  const entries = await readdir(root, {withFileTypes: true});
  const files = [];
  for (const entry of entries) {
    const candidate = path.join(root, entry.name);
    if (entry.isDirectory()) files.push(...(await markdownFiles(candidate)));
    else if (entry.isFile() && /\.(md|mdx)$/.test(entry.name)) files.push(candidate);
  }
  return files;
}

function escapeRegExp(value) {
  return value.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
}
