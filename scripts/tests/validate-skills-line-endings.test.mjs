/**
 * The validator must not care how a skill was checked out.
 *
 * `validate-skills.js` anchored its frontmatter regex on `\n`, so a SKILL.md
 * checked out with CRLF began `---\r\n` and did not match. It reported
 * "SKILL.md must have YAML frontmatter" for a file that plainly has it.
 *
 * That was not an edge case. The submodules under `skills/imported/` set
 * core.autocrlf=true and ship no `.gitattributes`, and a submodule inherits
 * neither the parent repository's config nor its attributes -- so git stores LF
 * and materializes CRLF, and the validator was unusable against all 210 skills
 * they carry whenever the checkout happened on Windows.
 *
 * The same content is validated here in three line-ending forms. The verdict
 * has to be identical for all three, because the bytes git stores are.
 */

import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');
const results = [];
const check = (name, body) => {
  body();
  results.push(name);
};

const workspace = fs.mkdtempSync(path.join(os.tmpdir(), 'prometheus-line-endings-'));

const SKILL = [
  '---',
  'name: line-ending-fixture',
  'description: >',
  '  A folded description, so this also covers the block scalar path. Use when',
  '  verifying that a checkout convention cannot change a validation verdict.',
  'license: MIT',
  'metadata:',
  '  version: 1.0.0',
  '  tags: [fixture]',
  '---',
  '',
  '# Line Ending Fixture',
  '',
  'Body content so the validator does not reject an empty body.',
  '',
].join('\n');

/** Write the same skill with a given line ending, optionally with a BOM. */
function materialize(name, eol, bom = false) {
  const directory = path.join(workspace, name);
  fs.mkdirSync(directory, { recursive: true });
  const text = SKILL.split('\n').join(eol);
  fs.writeFileSync(path.join(directory, 'SKILL.md'), (bom ? '﻿' : '') + text, 'utf8');
  return directory;
}

function validate(directory) {
  const result = spawnSync(
    process.execPath,
    [path.join(repoRoot, 'scripts/validate-skills.js'), directory],
    { encoding: 'utf8', cwd: repoRoot }
  );
  const output = `${result.stdout ?? ''}${result.stderr ?? ''}`;
  return {
    status: result.status,
    missingFrontmatter: /must have YAML frontmatter/.test(output),
    errors: (output.match(/ERROR:/g) ?? []).length,
  };
}

const lf = validate(materialize('lf', '\n'));

check('a CRLF checkout validates exactly like an LF one', () => {
  const crlf = validate(materialize('crlf', '\r\n'));
  assert.equal(crlf.missingFrontmatter, false, 'CRLF frontmatter must be parsed');
  assert.equal(crlf.status, lf.status, 'CRLF and LF must reach the same verdict');
  assert.equal(crlf.errors, lf.errors);
});

check('a lone-CR checkout does not masquerade as valid', () => {
  // Classic Mac line endings are not a checkout convention any tool here
  // produces. What matters is that the file is REJECTED rather than silently
  // parsed as something else.
  const cr = validate(materialize('cr', '\r'));
  assert.equal(cr.missingFrontmatter, true, 'a lone-CR file has no parseable frontmatter');
});

check('a UTF-8 BOM does not hide the frontmatter', () => {
  const bom = validate(materialize('bom', '\r\n', true));
  assert.equal(bom.missingFrontmatter, false);
  assert.equal(bom.status, lf.status);
});

check('the real CRLF corpus validates on this host', () => {
  // The submodule skills, as this host actually checked them out. Before the
  // fix every one of these reported missing frontmatter.
  const roots = ['skills/imported/artifact-refiner/skills'];
  let checked = 0;
  for (const root of roots) {
    const absolute = path.join(repoRoot, root);
    if (!fs.existsSync(absolute)) continue;
    for (const name of fs.readdirSync(absolute).slice(0, 4)) {
      const skill = path.join(absolute, name);
      if (!fs.existsSync(path.join(skill, 'SKILL.md'))) continue;
      const verdict = validate(skill);
      assert.equal(
        verdict.missingFrontmatter,
        false,
        `${root}/${name} reported missing frontmatter`
      );
      checked += 1;
    }
  }
  assert.ok(checked > 0, 'expected at least one imported skill to check');
  results.push(`  (checked ${checked} imported skills as checked out)`);
});

fs.rmSync(workspace, { recursive: true, force: true });

process.stdout.write(`validate-skills line endings: ${results.length} checks passed\n`);
for (const name of results) process.stdout.write(`  ok   ${name}\n`);
