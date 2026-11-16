#!/usr/bin/env node
// scan-secrets.mjs — refuse eval artifacts that contain credential-shaped text
// (VLBV 22 / AT 22; vlbv-r4-portability SAFE-2).
//
// Usage: node evals/vlbv-protocol/scan-secrets.mjs <path>... [--json]
// Exit 0 clean (or a path absent), 1 secret found, 2 usage.
// Findings name the file, line and pattern; the matched value is never printed.
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

export const PATTERNS = [
  ['private-key', /-----BEGIN (?:[A-Z]+ )?PRIVATE KEY-----/],
  ['github-token', /\b(?:ghp|gho|ghu|ghs|ghr)_[A-Za-z0-9]{30,}\b|\bgithub_pat_[A-Za-z0-9_]{40,}\b/],
  ['anthropic-key', /\bsk-ant-[A-Za-z0-9_-]{20,}/],
  ['openai-style-key', /\bsk-(?:proj-)?[A-Za-z0-9_-]{32,}\b/],
  ['slack-token', /\bxox[abprs]-[A-Za-z0-9-]{10,}/],
  ['aws-access-key', /\bAKIA[0-9A-Z]{16}\b/],
  ['json-token-field', /"(?:access_token|refresh_token|id_token|api_key|apiKey|client_secret)"\s*:\s*"[^"\s]{16,}"/],
  ['bearer-header', /\bBearer\s+[A-Za-z0-9._~+/-]{30,}=*/],
];
const MAX_BYTES = 8 * 1024 * 1024;

function files(target) {
  if (!fs.existsSync(target)) return [];
  const stat = fs.lstatSync(target);
  if (stat.isSymbolicLink()) return [];
  if (stat.isFile()) return [target];
  return fs.readdirSync(target).sort().flatMap(name => files(path.join(target, name)));
}

/**
 * @param {string[]} targets files or directories
 * @returns {{file: string, line: number, pattern: string}[]}
 */
export function scanPaths(targets) {
  const findings = [];
  for (const file of targets.flatMap(files)) {
    const stat = fs.statSync(file);
    if (stat.size > MAX_BYTES) {
      findings.push({ file, line: 0, pattern: 'oversize-unscanned' });
      continue;
    }
    const lines = fs.readFileSync(file, 'utf8').split('\n');
    lines.forEach((text, index) => {
      for (const [name, regex] of PATTERNS) {
        if (regex.test(text)) findings.push({ file, line: index + 1, pattern: name });
      }
    });
  }
  return findings;
}

function main(argv) {
  const json = argv.includes('--json');
  const targets = argv.filter(a => a !== '--json');
  if (!targets.length) {
    process.stderr.write('usage: scan-secrets.mjs <path>... [--json]\n');
    return 2;
  }
  const findings = scanPaths(targets.map(t => path.resolve(t)));
  if (json) process.stdout.write(`${JSON.stringify({ clean: findings.length === 0, findings }, null, 2)}\n`);
  else if (findings.length) {
    for (const f of findings) process.stdout.write(`SECRET ${f.pattern} ${f.file}:${f.line}\n`);
  } else process.stdout.write(`CLEAN ${targets.join(' ')}\n`);
  return findings.length ? 1 : 0;
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  process.exitCode = main(process.argv.slice(2));
}
