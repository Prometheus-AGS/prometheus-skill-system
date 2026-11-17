// Integration scenarios for the skill certifier's frontmatter reading, driven
// through its entry point:
//   node skills/testing/skill-certification/scripts/certify.mjs <dir> --out <dir> --skip-external
//
// Production failure caught: a description written as plain text continued on
// indented lines was read as a nested mapping, so a valid skill failed the
// specification level with a false `description` finding.
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..', '..');
const certify = path.join(repoRoot, 'skills', 'testing', 'skill-certification', 'scripts', 'certify.mjs');

const certifyWith = (frontmatter) => {
  const scratch = fs.mkdtempSync(path.join(os.tmpdir(), 'certify-frontmatter-'));
  try {
    const dir = path.join(scratch, 'sample-skill');
    fs.mkdirSync(dir);
    fs.writeFileSync(path.join(dir, 'SKILL.md'), ['---', ...frontmatter, '---', '', '# Sample', '', 'Do the thing.', ''].join('\n'));
    const out = path.join(scratch, 'out');
    const result = spawnSync(process.execPath, [certify, dir, '--out', out, '--skip-external'], { encoding: 'utf8', timeout: 60000 });
    assert.equal(result.status, 0, result.stderr);
    const certificate = JSON.parse(fs.readFileSync(path.join(out, 'certificate.json'), 'utf8'));
    return certificate.levels.find((entry) => entry.level === 'C0').findings.map((entry) => entry.check);
  } finally {
    fs.rmSync(scratch, { recursive: true, force: true });
  }
};

const REST = ['license: MIT', 'metadata:', "  version: '1.0.0'", "  tags: 'release, notes'"];

test('a description continued on indented lines is read as one scalar', () => {
  const checks = certifyWith(['name: sample-skill', 'description:', '  Formats a release note from a list of changes.', '  Use when asked to write release notes.', ...REST]);

  assert.deepEqual(checks, []);
});

test('an inline and a folded description still pass', () => {
  assert.deepEqual(certifyWith(['name: sample-skill', 'description: Formats a release note. Use when asked to write release notes.', ...REST]), []);
  assert.deepEqual(certifyWith(['name: sample-skill', 'description: >', '  Formats a release note.', '  Use when asked to write release notes.', ...REST]), []);
});

test('a description that really is a mapping is still reported', () => {
  const checks = certifyWith(['name: sample-skill', 'description:', '  short: Formats a release note.', '  long: Use when asked to write release notes.', ...REST]);

  assert.ok(checks.includes('description'), JSON.stringify(checks));
});

test('a list of tags under metadata is still reported as not a string', () => {
  const checks = certifyWith(['name: sample-skill', 'description: Formats a release note. Use when asked to write release notes.', 'license: MIT', 'metadata:', "  version: '1.0.0'", '  tags:', '    - release', '    - notes']);

  assert.ok(checks.includes('metadata-strings'), JSON.stringify(checks));
});
